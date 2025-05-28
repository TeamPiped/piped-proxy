use crate::sabr_parser::SabrParser;
use crate::sabr_request::{
    create_buffered_range, create_format_id, BufferedRange, ClientInfo, FormatId,
    SabrRequestBuilder,
};
use actix_web::{HttpRequest, HttpResponse};
use base64::{engine::general_purpose, Engine as _};
use prost::Message;
use qstring::QString;
use reqwest::{Body, Client, Method, Request, Url};
use serde_json::Value;
use std::error::Error;
use std::str::FromStr;

#[derive(Debug)]
pub struct SabrRequestData {
    pub player_time_ms: i64,
    pub bandwidth_estimate: i64,
    pub client_viewport_width: i32,
    pub client_viewport_height: i32,
    pub playback_rate: f32,
    pub has_audio: bool,
    pub selected_audio_format_ids: Vec<FormatId>,
    pub selected_video_format_ids: Vec<FormatId>,
    pub buffered_ranges: Vec<BufferedRange>,
    pub video_playback_ustreamer_config: Option<Vec<u8>>,
    pub po_token: Option<Vec<u8>>,
    pub playback_cookie: Option<Vec<u8>>,
}

impl SabrRequestData {
    pub fn from_json_body(body: &str) -> Result<Self, Box<dyn Error>> {
        let json: Value = serde_json::from_str(body)?;

        let player_time_ms = json
            .get("playerTimeMs")
            .and_then(|v| v.as_f64())
            .map(|v| v as i64)
            .unwrap_or(0);

        let bandwidth_estimate = json
            .get("bandwidthEstimate")
            .and_then(|v| v.as_f64())
            .map(|v| v as i64)
            .unwrap_or(1000000); // Default 1Mbps

        let client_viewport_width = json
            .get("clientViewportWidth")
            .and_then(|v| v.as_f64())
            .map(|v| v as i32)
            .unwrap_or(1920);

        let client_viewport_height = json
            .get("clientViewportHeight")
            .and_then(|v| v.as_f64())
            .map(|v| v as i32)
            .unwrap_or(1080);

        let playback_rate = json
            .get("playbackRate")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(1.0);

        let has_audio = json
            .get("hasAudio")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // Parse audio format IDs
        let selected_audio_format_ids = json
            .get("selectedAudioFormatIds")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| parse_format_id_from_json(item))
                    .collect()
            })
            .unwrap_or_default();

        // Parse video format IDs
        let selected_video_format_ids = json
            .get("selectedVideoFormatIds")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| parse_format_id_from_json(item))
                    .collect()
            })
            .unwrap_or_default();

        // Parse buffered ranges
        let buffered_ranges = json
            .get("bufferedRanges")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| parse_buffered_range_from_json(item))
                    .collect()
            })
            .unwrap_or_default();

        // Parse base64 encoded fields
        let video_playback_ustreamer_config = json
            .get("videoPlaybackUstreamerConfig")
            .and_then(|v| v.as_str())
            .and_then(|s| general_purpose::STANDARD.decode(s).ok());

        let po_token = json
            .get("poToken")
            .and_then(|v| v.as_str())
            .and_then(|s| general_purpose::STANDARD.decode(s).ok());

        let playback_cookie = json
            .get("playbackCookie")
            .and_then(|v| v.as_str())
            .and_then(|s| general_purpose::STANDARD.decode(s).ok());

        Ok(SabrRequestData {
            player_time_ms,
            bandwidth_estimate,
            client_viewport_width,
            client_viewport_height,
            playback_rate,
            has_audio,
            selected_audio_format_ids,
            selected_video_format_ids,
            buffered_ranges,
            video_playback_ustreamer_config,
            po_token,
            playback_cookie,
        })
    }
}

fn parse_format_id_from_json(json: &Value) -> Option<FormatId> {
    let itag = json.get("itag")?.as_i64()? as i32;
    let last_modified = json.get("lastModified")?.as_u64()?;
    let xtags = json
        .get("xtags")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Some(create_format_id(itag, last_modified, xtags))
}

fn parse_buffered_range_from_json(json: &Value) -> Option<BufferedRange> {
    let format_id = json.get("formatId").and_then(parse_format_id_from_json)?;
    let start_time_ms = json.get("startTimeMs")?.as_i64()?;
    let duration_ms = json.get("durationMs")?.as_i64()?;
    let start_segment_index = json.get("startSegmentIndex")?.as_i64()? as i32;
    let end_segment_index = json.get("endSegmentIndex")?.as_i64()? as i32;

    Some(create_buffered_range(
        format_id,
        start_time_ms,
        duration_ms,
        start_segment_index,
        end_segment_index,
    ))
}

fn get_client_info_from_query(query: &QString) -> ClientInfo {
    // Extract client info from query parameters
    let client_name = query.get("c").and_then(|c| match c {
        "WEB" => Some(1),
        "ANDROID" => Some(3),
        "IOS" => Some(5),
        _ => Some(1), // Default to WEB
    });

    let client_version = query
        .get("cver")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "2.2040620.05.00".to_string());

    ClientInfo {
        device_make: None,
        device_model: None,
        client_name,
        client_version: Some(client_version),
        os_name: Some("Windows".to_string()),
        os_version: Some("10.0".to_string()),
        accept_language: None,
        accept_region: None,
        screen_width_points: None,
        screen_height_points: None,
        screen_width_inches: None,
        screen_height_inches: None,
        screen_pixel_density: None,
        client_form_factor: None,
        gmscore_version_code: None,
        window_width_points: None,
        window_height_points: None,
        android_sdk_version: None,
        screen_density_float: None,
        utc_offset_minutes: None,
        time_zone: None,
        chipset: None,
    }
}

pub async fn handle_sabr_request(
    req: HttpRequest,
    mut query: QString,
    host: String,
    client: &Client,
    request_body: Option<String>,
) -> Result<HttpResponse, Box<dyn Error>> {
    // Remove ump parameter before proxying by filtering it out
    let filtered_pairs: Vec<_> = query
        .into_pairs()
        .into_iter()
        .filter(|(key, _)| key != "sabr")
        .collect();
    query = QString::new(filtered_pairs);

    // Parse request body if provided
    let sabr_data = if let Some(body) = request_body {
        Some(SabrRequestData::from_json_body(&body)?)
    } else {
        None
    };

    // Build SABR request
    let mut sabr_builder = SabrRequestBuilder::new();

    // Set client info from query parameters
    let client_info = get_client_info_from_query(&query);
    sabr_builder = sabr_builder.with_client_info(client_info);

    if let Some(ref data) = sabr_data {
        sabr_builder = sabr_builder
            .with_player_time_ms(data.player_time_ms)
            .with_bandwidth_estimate(data.bandwidth_estimate)
            .with_viewport_size(data.client_viewport_width, data.client_viewport_height)
            .with_playback_rate(data.playback_rate)
            .with_enabled_track_types(if data.has_audio { 1 } else { 2 })
            .with_audio_formats(data.selected_audio_format_ids.clone())
            .with_video_formats(data.selected_video_format_ids.clone());

        // If no buffered ranges provided, create initial ones like in the working example
        let buffered_ranges = if data.buffered_ranges.is_empty() {
            let mut ranges = Vec::new();

            // Create buffered range for audio format (like the working example)
            if let Some(audio_format) = data.selected_audio_format_ids.first() {
                ranges.push(create_buffered_range(
                    audio_format.clone(),
                    0,     // start_time_ms
                    20000, // duration_ms (20 seconds like working example)
                    1,     // start_segment_index
                    2,     // end_segment_index
                ));
            }

            // Create buffered ranges for video format (like the working example)
            if let Some(video_format) = data.selected_video_format_ids.first() {
                ranges.push(create_buffered_range(
                    video_format.clone(),
                    0,     // start_time_ms
                    15021, // duration_ms (like working example)
                    1,     // start_segment_index
                    3,     // end_segment_index
                ));

                ranges.push(create_buffered_range(
                    video_format.clone(),
                    10014, // start_time_ms (like working example)
                    10014, // duration_ms (like working example)
                    3,     // start_segment_index
                    4,     // end_segment_index
                ));
            }

            ranges
        } else {
            data.buffered_ranges.clone()
        };

        sabr_builder = sabr_builder.with_buffered_ranges(buffered_ranges);

        if let Some(ref config) = data.video_playback_ustreamer_config {
            sabr_builder = sabr_builder.with_video_playback_ustreamer_config(config.clone());
        }

        if let Some(ref token) = data.po_token {
            sabr_builder = sabr_builder.with_po_token(token.clone());
        }

        if let Some(ref cookie) = data.playback_cookie {
            sabr_builder = sabr_builder.with_playback_cookie(cookie.clone());
        }
    }

    // Build the protobuf request
    let sabr_request = sabr_builder.build();
    let mut encoded_request = Vec::new();
    sabr_request.encode(&mut encoded_request)?;

    // Debug output
    eprintln!("SABR request structure:");
    eprintln!(
        "  client_abr_state: {:?}",
        sabr_request.client_abr_state.is_some()
    );
    eprintln!(
        "  selected_format_ids: {} items",
        sabr_request.selected_format_ids.len()
    );
    eprintln!(
        "  selected_audio_format_ids: {} items",
        sabr_request.selected_audio_format_ids.len()
    );
    eprintln!(
        "  selected_video_format_ids: {} items",
        sabr_request.selected_video_format_ids.len()
    );
    eprintln!(
        "  buffered_ranges: {} items",
        sabr_request.buffered_ranges.len()
    );
    eprintln!(
        "  video_playback_ustreamer_config: {} bytes",
        sabr_request
            .video_playback_ustreamer_config
            .as_ref()
            .map(|v| v.len())
            .unwrap_or(0)
    );
    eprintln!(
        "  streamer_context: {:?}",
        sabr_request.streamer_context.is_some()
    );
    if let Some(ref ctx) = sabr_request.streamer_context {
        eprintln!(
            "    po_token: {} bytes",
            ctx.po_token.as_ref().map(|v| v.len()).unwrap_or(0)
        );
        eprintln!(
            "    playback_cookie: {} bytes",
            ctx.playback_cookie.as_ref().map(|v| v.len()).unwrap_or(0)
        );
        eprintln!("    client_info: {:?}", ctx.client_info.is_some());
    }
    eprintln!("  field1000: {} items", sabr_request.field1000.len());

    // Print first 100 bytes of the protobuf for debugging
    eprintln!(
        "First 100 bytes of protobuf: {:?}",
        &encoded_request[..std::cmp::min(100, encoded_request.len())]
    );

    // Save protobuf to file for debugging
    std::fs::write("my_request_proto.bin", &encoded_request).unwrap_or_else(|e| {
        eprintln!("Failed to write protobuf to file: {}", e);
    });

    // Create the URL for the SABR request
    let qs = {
        let collected = query
            .into_pairs()
            .into_iter()
            .filter(|(key, _)| !matches!(key.as_str(), "host" | "rewrite" | "qhash"))
            .collect::<Vec<_>>();
        eprintln!("Filtered query parameters: {:?}", collected);
        QString::new(collected)
    };

    let mut url = Url::parse(&format!("https://{}{}", host, req.path()))?;
    url.set_query(Some(qs.to_string().as_str()));

    // Debug output
    eprintln!("SABR request URL: {}", url);
    eprintln!("SABR request body length: {}", encoded_request.len());

    // Create POST request with protobuf body
    let mut request = Request::new(Method::POST, url);
    request.body_mut().replace(Body::from(encoded_request));

    // Add Content-Type header for protobuf
    request
        .headers_mut()
        .insert("Content-Type", "application/x-protobuf".parse().unwrap());

    // Execute the request
    let resp = client.execute(request).await?;
    let status = resp.status();

    if !status.is_success() {
        // Get the response body for debugging
        let error_body = resp
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error body".to_string());
        eprintln!("SABR request failed with status: {}", status);
        eprintln!("Response body: {}", error_body);
        return Err(format!(
            "SABR request failed with status: {} - Body: {}",
            status, error_body
        )
        .into());
    }

    // Parse the SABR response
    let response_bytes = resp.bytes().await?;
    let mut parser = SabrParser::new();
    let sabr_response = parser.parse_response(&response_bytes)?;

    // Build the response
    let mut response_builder = HttpResponse::Ok();

    // Add CORS headers
    response_builder
        .append_header(("Access-Control-Allow-Origin", "*"))
        .append_header(("Access-Control-Allow-Headers", "*"))
        .append_header(("Access-Control-Allow-Methods", "*"))
        .append_header(("Access-Control-Max-Age", "1728000"));

    // Add playback cookie to response headers if available
    if let Some(cookie) = parser.get_playback_cookie() {
        let mut encoded_cookie = Vec::new();
        cookie.encode(&mut encoded_cookie)?;
        let encoded_cookie_b64 = general_purpose::STANDARD.encode(encoded_cookie);
        response_builder.append_header(("X-Playback-Cookie", encoded_cookie_b64));
    }

    // Add format-specific content ranges
    let mut audio_ranges = Vec::new();
    let mut video_ranges = Vec::new();

    for format in &sabr_response.initialized_formats {
        let is_audio = format
            .mime_type
            .as_ref()
            .map(|mime| mime.starts_with("audio/"))
            .unwrap_or(false);

        for chunk in &format.media_chunks {
            let range = format!("bytes=0-{}", chunk.len() - 1);
            if is_audio {
                audio_ranges.push(range);
            } else {
                video_ranges.push(range);
            }
        }
    }

    if !audio_ranges.is_empty() {
        response_builder.append_header(("X-Audio-Content-Ranges", audio_ranges.join(",")));
    }

    if !video_ranges.is_empty() {
        response_builder.append_header(("X-Video-Content-Ranges", video_ranges.join(",")));
    }

    // Combine all media chunks into a single response
    let mut combined_data = Vec::new();
    for format in &sabr_response.initialized_formats {
        for chunk in &format.media_chunks {
            combined_data.extend_from_slice(chunk);
        }
    }

    Ok(response_builder.body(combined_data))
}

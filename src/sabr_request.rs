use prost::Message;

// Re-export the FormatId from sabr_parser for consistency
pub use crate::sabr_parser::FormatId;

#[derive(Clone, PartialEq, Message)]
pub struct ClientInfo {
    #[prost(string, optional, tag = "12")]
    pub device_make: Option<String>,
    #[prost(string, optional, tag = "13")]
    pub device_model: Option<String>,
    #[prost(int32, optional, tag = "16")]
    pub client_name: Option<i32>,
    #[prost(string, optional, tag = "17")]
    pub client_version: Option<String>,
    #[prost(string, optional, tag = "18")]
    pub os_name: Option<String>,
    #[prost(string, optional, tag = "19")]
    pub os_version: Option<String>,
    #[prost(string, optional, tag = "21")]
    pub accept_language: Option<String>,
    #[prost(string, optional, tag = "22")]
    pub accept_region: Option<String>,
    #[prost(int32, optional, tag = "37")]
    pub screen_width_points: Option<i32>,
    #[prost(int32, optional, tag = "38")]
    pub screen_height_points: Option<i32>,
    #[prost(float, optional, tag = "39")]
    pub screen_width_inches: Option<f32>,
    #[prost(float, optional, tag = "40")]
    pub screen_height_inches: Option<f32>,
    #[prost(int32, optional, tag = "41")]
    pub screen_pixel_density: Option<i32>,
    #[prost(int32, optional, tag = "46")]
    pub client_form_factor: Option<i32>,
    #[prost(int32, optional, tag = "50")]
    pub gmscore_version_code: Option<i32>,
    #[prost(int32, optional, tag = "55")]
    pub window_width_points: Option<i32>,
    #[prost(int32, optional, tag = "56")]
    pub window_height_points: Option<i32>,
    #[prost(int32, optional, tag = "64")]
    pub android_sdk_version: Option<i32>,
    #[prost(float, optional, tag = "65")]
    pub screen_density_float: Option<f32>,
    #[prost(int64, optional, tag = "67")]
    pub utc_offset_minutes: Option<i64>,
    #[prost(string, optional, tag = "80")]
    pub time_zone: Option<String>,
    #[prost(string, optional, tag = "92")]
    pub chipset: Option<String>,
}

#[derive(Clone, PartialEq, Message)]
pub struct StreamerContext {
    #[prost(message, optional, tag = "1")]
    pub client_info: Option<ClientInfo>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pub po_token: Option<Vec<u8>>,
    #[prost(bytes = "vec", optional, tag = "3")]
    pub playback_cookie: Option<Vec<u8>>,
    #[prost(bytes = "vec", optional, tag = "4")]
    pub gp: Option<Vec<u8>>,
    #[prost(message, repeated, tag = "5")]
    pub field5: Vec<Fqa>,
    #[prost(int32, repeated, tag = "6")]
    pub field6: Vec<i32>,
    #[prost(string, optional, tag = "7")]
    pub field7: Option<String>,
    #[prost(message, optional, tag = "8")]
    pub field8: Option<Gqa>,
}

#[derive(Clone, PartialEq, Message)]
pub struct TimeRange {
    #[prost(int64, optional, tag = "1")]
    pub start: Option<i64>,
    #[prost(int64, optional, tag = "2")]
    pub end: Option<i64>,
}

#[derive(Clone, PartialEq, Message)]
pub struct BufferedRange {
    #[prost(message, optional, tag = "1")]
    pub format_id: Option<FormatId>,
    #[prost(int64, optional, tag = "2")]
    pub start_time_ms: Option<i64>,
    #[prost(int64, optional, tag = "3")]
    pub duration_ms: Option<i64>,
    #[prost(int32, optional, tag = "4")]
    pub start_segment_index: Option<i32>,
    #[prost(int32, optional, tag = "5")]
    pub end_segment_index: Option<i32>,
    #[prost(message, optional, tag = "6")]
    pub time_range: Option<TimeRange>,
}

#[derive(Clone, PartialEq, Message)]
pub struct ClientAbrState {
    #[prost(int64, optional, tag = "13")]
    pub time_since_last_manual_format_selection_ms: Option<i64>,
    #[prost(sint32, optional, tag = "14")]
    pub last_manual_direction: Option<i32>,
    #[prost(int32, optional, tag = "16")]
    pub last_manual_selected_resolution: Option<i32>,
    #[prost(int32, optional, tag = "17")]
    pub detailed_network_type: Option<i32>,
    #[prost(int32, optional, tag = "18")]
    pub client_viewport_width: Option<i32>,
    #[prost(int32, optional, tag = "19")]
    pub client_viewport_height: Option<i32>,
    #[prost(int64, optional, tag = "20")]
    pub client_bitrate_cap_bytes_per_sec: Option<i64>,
    #[prost(int32, optional, tag = "21")]
    pub sticky_resolution: Option<i32>,
    #[prost(bool, optional, tag = "22")]
    pub client_viewport_is_flexible: Option<bool>,
    #[prost(int64, optional, tag = "23")]
    pub bandwidth_estimate: Option<i64>,
    #[prost(int32, optional, tag = "24")]
    pub min_audio_quality: Option<i32>,
    #[prost(int32, optional, tag = "25")]
    pub max_audio_quality: Option<i32>,
    #[prost(int32, optional, tag = "26")]
    pub video_quality_setting: Option<i32>,
    #[prost(int32, optional, tag = "27")]
    pub audio_route: Option<i32>,
    #[prost(int64, optional, tag = "28")]
    pub player_time_ms: Option<i64>,
    #[prost(int64, optional, tag = "29")]
    pub time_since_last_seek: Option<i64>,
    #[prost(bool, optional, tag = "30")]
    pub data_saver_mode: Option<bool>,
    #[prost(int32, optional, tag = "32")]
    pub network_metered_state: Option<i32>,
    #[prost(int32, optional, tag = "34")]
    pub visibility: Option<i32>,
    #[prost(float, optional, tag = "35")]
    pub playback_rate: Option<f32>,
    #[prost(int64, optional, tag = "36")]
    pub elapsed_wall_time_ms: Option<i64>,
    #[prost(bytes = "vec", optional, tag = "38")]
    pub media_capabilities: Option<Vec<u8>>,
    #[prost(int64, optional, tag = "39")]
    pub time_since_last_action_ms: Option<i64>,
    #[prost(int32, optional, tag = "40")]
    pub enabled_track_types_bitfield: Option<i32>,
    #[prost(int32, optional, tag = "43")]
    pub max_pacing_rate: Option<i32>,
    #[prost(int64, optional, tag = "44")]
    pub player_state: Option<i64>,
    #[prost(bool, optional, tag = "46")]
    pub drc_enabled: Option<bool>,
    #[prost(int32, optional, tag = "48")]
    pub jda: Option<i32>,
    #[prost(int32, optional, tag = "50")]
    pub qw: Option<i32>,
    #[prost(int32, optional, tag = "51")]
    pub ky: Option<i32>,
    #[prost(int32, optional, tag = "54")]
    pub sabr_report_request_cancellation_info: Option<i32>,
    #[prost(bool, optional, tag = "56")]
    pub l: Option<bool>,
    #[prost(int64, optional, tag = "57")]
    pub g7: Option<i64>,
    #[prost(bool, optional, tag = "58")]
    pub prefer_vp9: Option<bool>,
    #[prost(int32, optional, tag = "59")]
    pub qj: Option<i32>,
    #[prost(int32, optional, tag = "60")]
    pub hx: Option<i32>,
    #[prost(bool, optional, tag = "61")]
    pub is_prefetch: Option<bool>,
    #[prost(int32, optional, tag = "62")]
    pub sabr_support_quality_constraints: Option<i32>,
    #[prost(bytes = "vec", optional, tag = "63")]
    pub sabr_license_constraint: Option<Vec<u8>>,
    #[prost(int32, optional, tag = "64")]
    pub allow_proxima_live_latency: Option<i32>,
    #[prost(int32, optional, tag = "66")]
    pub sabr_force_proxima: Option<i32>,
    #[prost(int32, optional, tag = "67")]
    pub tqb: Option<i32>,
    #[prost(int64, optional, tag = "68")]
    pub sabr_force_max_network_interruption_duration_ms: Option<i64>,
    #[prost(string, optional, tag = "69")]
    pub audio_track_id: Option<String>,
}

#[derive(Clone, PartialEq, Message)]
pub struct VideoPlaybackAbrRequest {
    #[prost(message, optional, tag = "1")]
    pub client_abr_state: Option<ClientAbrState>,
    #[prost(message, repeated, tag = "2")]
    pub selected_format_ids: Vec<FormatId>,
    #[prost(message, repeated, tag = "3")]
    pub buffered_ranges: Vec<BufferedRange>,
    #[prost(int64, optional, tag = "4")]
    pub player_time_ms: Option<i64>,
    #[prost(bytes = "vec", optional, tag = "5")]
    pub video_playback_ustreamer_config: Option<Vec<u8>>,
    #[prost(message, optional, tag = "6")]
    pub lo: Option<Lo>,
    #[prost(message, repeated, tag = "16")]
    pub selected_audio_format_ids: Vec<FormatId>,
    #[prost(message, repeated, tag = "17")]
    pub selected_video_format_ids: Vec<FormatId>,
    #[prost(message, optional, tag = "19")]
    pub streamer_context: Option<StreamerContext>,
    #[prost(message, optional, tag = "21")]
    pub field21: Option<OQa>,
    #[prost(int32, optional, tag = "22")]
    pub field22: Option<i32>,
    #[prost(int32, optional, tag = "23")]
    pub field23: Option<i32>,
    #[prost(message, repeated, tag = "1000")]
    pub field1000: Vec<Pqa>,
}

#[derive(Clone, PartialEq, Message)]
pub struct Pqa {
    #[prost(message, repeated, tag = "1")]
    pub formats: Vec<FormatId>,
    #[prost(message, repeated, tag = "2")]
    pub ud: Vec<BufferedRange>,
    #[prost(string, optional, tag = "3")]
    pub clip_id: Option<String>,
}

// Default quality constant
const DEFAULT_QUALITY: i32 = 720; // HD720

#[derive(Debug, Clone)]
pub struct SabrRequestBuilder {
    pub client_abr_state: ClientAbrState,
    pub streamer_context: StreamerContext,
    pub video_playback_ustreamer_config: Option<Vec<u8>>,
    pub selected_audio_format_ids: Vec<FormatId>,
    pub selected_video_format_ids: Vec<FormatId>,
    pub buffered_ranges: Vec<BufferedRange>,
}

impl SabrRequestBuilder {
    pub fn new() -> Self {
        Self {
            // Minimal defaults matching googlevideo pattern
            client_abr_state: ClientAbrState {
                last_manual_direction: Some(0),
                time_since_last_manual_format_selection_ms: Some(0),
                last_manual_selected_resolution: Some(DEFAULT_QUALITY),
                sticky_resolution: Some(DEFAULT_QUALITY),
                player_time_ms: Some(0),
                visibility: Some(0),
                enabled_track_types_bitfield: Some(0),
                ..Default::default()
            },
            streamer_context: StreamerContext {
                field5: Vec::new(),
                field6: Vec::new(),
                field7: None,
                field8: None,
                gp: None,
                playback_cookie: None,
                po_token: None,
                // Basic client info matching googlevideo
                client_info: Some(ClientInfo {
                    client_name: Some(1),
                    client_version: Some("2.2040620.05.00".to_string()),
                    os_name: Some("Windows".to_string()),
                    os_version: Some("10.0".to_string()),
                    ..Default::default()
                }),
            },
            video_playback_ustreamer_config: None,
            selected_audio_format_ids: Vec::new(),
            selected_video_format_ids: Vec::new(),
            buffered_ranges: Vec::new(),
        }
    }

    pub fn with_client_info(mut self, client_info: ClientInfo) -> Self {
        self.streamer_context.client_info = Some(client_info);
        self
    }

    pub fn with_po_token(mut self, po_token: Vec<u8>) -> Self {
        self.streamer_context.po_token = Some(po_token);
        self
    }

    pub fn with_playback_cookie(mut self, playback_cookie: Vec<u8>) -> Self {
        self.streamer_context.playback_cookie = Some(playback_cookie);
        self
    }

    pub fn with_video_playback_ustreamer_config(mut self, config: Vec<u8>) -> Self {
        self.video_playback_ustreamer_config = Some(config);
        self
    }

    pub fn with_player_time_ms(mut self, time_ms: i64) -> Self {
        self.client_abr_state.player_time_ms = Some(time_ms);
        self
    }

    pub fn with_resolution(mut self, resolution: i32) -> Self {
        self.client_abr_state.last_manual_selected_resolution = Some(resolution);
        self.client_abr_state.sticky_resolution = Some(resolution);
        self
    }

    pub fn with_viewport_size(mut self, width: i32, height: i32) -> Self {
        self.client_abr_state.client_viewport_width = Some(width);
        self.client_abr_state.client_viewport_height = Some(height);
        self
    }

    pub fn with_bandwidth_estimate(mut self, bandwidth: i64) -> Self {
        self.client_abr_state.bandwidth_estimate = Some(bandwidth);
        self
    }

    pub fn with_audio_formats(mut self, formats: Vec<FormatId>) -> Self {
        self.selected_audio_format_ids = formats;
        self
    }

    pub fn with_video_formats(mut self, formats: Vec<FormatId>) -> Self {
        self.selected_video_format_ids = formats;
        self
    }

    pub fn with_buffered_ranges(mut self, ranges: Vec<BufferedRange>) -> Self {
        self.buffered_ranges = ranges;
        self
    }

    pub fn with_enabled_track_types(mut self, bitfield: i32) -> Self {
        self.client_abr_state.enabled_track_types_bitfield = Some(bitfield);
        self
    }

    pub fn with_visibility(mut self, visibility: i32) -> Self {
        self.client_abr_state.visibility = Some(visibility);
        self
    }

    pub fn with_playback_rate(mut self, rate: f32) -> Self {
        self.client_abr_state.playback_rate = Some(rate);
        self
    }

    pub fn build(self) -> VideoPlaybackAbrRequest {
        // For initial requests, selectedFormatIds should be empty
        // It represents previously initialized formats, which don't exist yet
        let selected_format_ids = Vec::new();

        VideoPlaybackAbrRequest {
            client_abr_state: Some(self.client_abr_state),
            selected_format_ids,
            buffered_ranges: self.buffered_ranges,
            player_time_ms: Some(0),
            video_playback_ustreamer_config: self.video_playback_ustreamer_config,
            selected_audio_format_ids: self.selected_audio_format_ids,
            selected_video_format_ids: self.selected_video_format_ids,
            streamer_context: Some(self.streamer_context),
            field22: Some(0),
            field23: Some(0),
            field1000: Vec::new(),
            lo: None,
            field21: None,
        }
    }

    pub fn encode(self) -> Vec<u8> {
        let request = self.build();
        request.encode_to_vec()
    }
}

impl Default for SabrRequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Utility functions for creating common format IDs and buffered ranges
pub fn create_format_id(itag: i32, last_modified: u64, xtags: Option<String>) -> FormatId {
    FormatId {
        itag: Some(itag),
        last_modified: Some(last_modified as i64),
        xtags,
    }
}

pub fn create_buffered_range(
    format_id: FormatId,
    start_time_ms: i64,
    duration_ms: i64,
    start_segment_index: i32,
    end_segment_index: i32,
) -> BufferedRange {
    BufferedRange {
        format_id: Some(format_id),
        start_time_ms: Some(start_time_ms),
        duration_ms: Some(duration_ms),
        start_segment_index: Some(start_segment_index),
        end_segment_index: Some(end_segment_index),
        time_range: None,
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct LoField4 {
    #[prost(int32, optional, tag = "1")]
    pub field1: Option<i32>,
    #[prost(int32, optional, tag = "2")]
    pub field2: Option<i32>,
    #[prost(int32, optional, tag = "3")]
    pub field3: Option<i32>,
}

#[derive(Clone, PartialEq, Message)]
pub struct Lo {
    #[prost(message, optional, tag = "1")]
    pub format_id: Option<FormatId>,
    #[prost(int32, optional, tag = "2")]
    pub lj: Option<i32>,
    #[prost(int32, optional, tag = "3")]
    pub sequence_number: Option<i32>,
    #[prost(message, optional, tag = "4")]
    pub field4: Option<LoField4>,
    #[prost(int32, optional, tag = "5")]
    pub mz: Option<i32>,
}

#[derive(Clone, PartialEq, Message)]
pub struct OQa {
    #[prost(string, repeated, tag = "1")]
    pub field1: Vec<String>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pub field2: Option<Vec<u8>>,
    #[prost(string, optional, tag = "3")]
    pub field3: Option<String>,
    #[prost(int32, optional, tag = "4")]
    pub field4: Option<i32>,
    #[prost(int32, optional, tag = "5")]
    pub field5: Option<i32>,
    #[prost(string, optional, tag = "6")]
    pub field6: Option<String>,
}

#[derive(Clone, PartialEq, Message)]
pub struct Hqa {
    #[prost(int32, optional, tag = "1")]
    pub code: Option<i32>,
    #[prost(string, optional, tag = "2")]
    pub message: Option<String>,
}

#[derive(Clone, PartialEq, Message)]
pub struct Gqa {
    #[prost(bytes = "vec", optional, tag = "1")]
    pub field1: Option<Vec<u8>>,
    #[prost(message, optional, tag = "2")]
    pub field2: Option<Hqa>,
}

#[derive(Clone, PartialEq, Message)]
pub struct Fqa {
    #[prost(int32, optional, tag = "1")]
    pub r#type: Option<i32>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pub value: Option<Vec<u8>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sabr_request_builder() {
        let audio_format = create_format_id(251, 1747754876286051, None);
        let video_format = create_format_id(136, 1747754870573293, None);

        let buffered_range = create_buffered_range(audio_format.clone(), 0, 5000, 1, 5);

        let request = SabrRequestBuilder::new()
            .with_player_time_ms(1000)
            .with_resolution(720)
            .with_viewport_size(1280, 720)
            .with_bandwidth_estimate(1000000)
            .with_audio_formats(vec![audio_format])
            .with_video_formats(vec![video_format])
            .with_buffered_ranges(vec![buffered_range])
            .with_enabled_track_types(0)
            .with_visibility(0)
            .with_playback_rate(1.0)
            .build();

        assert!(request.client_abr_state.is_some());
        assert!(request.streamer_context.is_some());
        assert_eq!(request.selected_audio_format_ids.len(), 1);
        assert_eq!(request.selected_video_format_ids.len(), 1);
        assert_eq!(request.buffered_ranges.len(), 1);

        // Test encoding
        let encoded = SabrRequestBuilder::new().with_player_time_ms(1000).encode();

        assert!(!encoded.is_empty());
    }

    #[test]
    fn test_format_id_creation() {
        let format_id = create_format_id(251, 1747754876286051, Some("test".to_string()));

        assert_eq!(format_id.itag, Some(251));
        assert_eq!(format_id.last_modified, Some(1747754876286051));
        assert_eq!(format_id.xtags, Some("test".to_string()));
    }

    #[test]
    fn test_default_values() {
        let builder = SabrRequestBuilder::new();
        assert_eq!(
            builder.client_abr_state.last_manual_selected_resolution,
            Some(DEFAULT_QUALITY)
        );
        assert_eq!(
            builder.client_abr_state.sticky_resolution,
            Some(DEFAULT_QUALITY)
        );
        assert_eq!(builder.client_abr_state.player_time_ms, Some(0));
        assert_eq!(builder.client_abr_state.visibility, Some(0));
        assert_eq!(
            builder.client_abr_state.enabled_track_types_bitfield,
            Some(0)
        );

        let client_info = builder.streamer_context.client_info.unwrap();
        assert_eq!(client_info.client_name, Some(1));
        assert_eq!(
            client_info.client_version,
            Some("2.2040620.05.00".to_string())
        );
        assert_eq!(client_info.os_name, Some("Windows".to_string()));
    }
}

use bytes::Bytes;
use prost::Message;
use std::collections::HashMap;
use std::io;

// SABR Part Types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PartType {
    OnesieHeader = 10,
    OnesieData = 11,
    MediaHeader = 20,
    Media = 21,
    MediaEnd = 22,
    LiveMetadata = 31,
    HostnameChangeHint = 32,
    LiveMetadataPromise = 33,
    LiveMetadataPromiseCancellation = 34,
    NextRequestPolicy = 35,
    UstreamerVideoAndFormatData = 36,
    FormatSelectionConfig = 37,
    UstreamerSelectedMediaStream = 38,
    FormatInitializationMetadata = 42,
    SabrRedirect = 43,
    SabrError = 44,
    SabrSeek = 45,
    ReloadPlayerResponse = 46,
    PlaybackStartPolicy = 47,
    AllowedCachedFormats = 48,
    StartBwSamplingHint = 49,
    PauseBwSamplingHint = 50,
    SelectableFormats = 51,
    RequestIdentifier = 52,
    RequestCancellationPolicy = 53,
    OnesiePrefetchRejection = 54,
    TimelineContext = 55,
    RequestPipelining = 56,
    SabrContextUpdate = 57,
    StreamProtectionStatus = 58,
    SabrContextSendingPolicy = 59,
    LawnmowerPolicy = 60,
    SabrAck = 61,
    EndOfTrack = 62,
    CacheLoadPolicy = 63,
    LawnmowerMessagingPolicy = 64,
    PrewarmConnection = 65,
}

impl From<i32> for PartType {
    fn from(value: i32) -> Self {
        match value {
            10 => PartType::OnesieHeader,
            11 => PartType::OnesieData,
            20 => PartType::MediaHeader,
            21 => PartType::Media,
            22 => PartType::MediaEnd,
            31 => PartType::LiveMetadata,
            32 => PartType::HostnameChangeHint,
            33 => PartType::LiveMetadataPromise,
            34 => PartType::LiveMetadataPromiseCancellation,
            35 => PartType::NextRequestPolicy,
            36 => PartType::UstreamerVideoAndFormatData,
            37 => PartType::FormatSelectionConfig,
            38 => PartType::UstreamerSelectedMediaStream,
            42 => PartType::FormatInitializationMetadata,
            43 => PartType::SabrRedirect,
            44 => PartType::SabrError,
            45 => PartType::SabrSeek,
            46 => PartType::ReloadPlayerResponse,
            47 => PartType::PlaybackStartPolicy,
            48 => PartType::AllowedCachedFormats,
            49 => PartType::StartBwSamplingHint,
            50 => PartType::PauseBwSamplingHint,
            51 => PartType::SelectableFormats,
            52 => PartType::RequestIdentifier,
            53 => PartType::RequestCancellationPolicy,
            54 => PartType::OnesiePrefetchRejection,
            55 => PartType::TimelineContext,
            56 => PartType::RequestPipelining,
            57 => PartType::SabrContextUpdate,
            58 => PartType::StreamProtectionStatus,
            59 => PartType::SabrContextSendingPolicy,
            60 => PartType::LawnmowerPolicy,
            61 => PartType::SabrAck,
            62 => PartType::EndOfTrack,
            63 => PartType::CacheLoadPolicy,
            64 => PartType::LawnmowerMessagingPolicy,
            65 => PartType::PrewarmConnection,
            _ => panic!("Invalid part type: {}", value),
        }
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct FormatId {
    #[prost(int32, optional, tag = "1")]
    pub itag: Option<i32>,
    #[prost(int64, optional, tag = "2")]
    pub last_modified: Option<i64>,
    #[prost(string, optional, tag = "3")]
    pub xtags: Option<String>,
}

#[derive(Clone, PartialEq, Message)]
pub struct TimeRange {
    #[prost(int64, optional, tag = "1")]
    pub start: Option<i64>,
    #[prost(int64, optional, tag = "2")]
    pub end: Option<i64>,
}

#[derive(Clone, PartialEq, Message)]
pub struct MediaHeader {
    #[prost(uint32, optional, tag = "1")]
    pub header_id: Option<u32>,
    #[prost(string, optional, tag = "2")]
    pub video_id: Option<String>,
    #[prost(int32, optional, tag = "3")]
    pub itag: Option<i32>,
    #[prost(uint64, optional, tag = "4")]
    pub lmt: Option<u64>,
    #[prost(string, optional, tag = "5")]
    pub xtags: Option<String>,
    #[prost(int64, optional, tag = "6")]
    pub start_range: Option<i64>,
    #[prost(int32, optional, tag = "7")]
    pub compression_algorithm: Option<i32>,
    #[prost(bool, optional, tag = "8")]
    pub is_init_seg: Option<bool>,
    #[prost(int64, optional, tag = "9")]
    pub sequence_number: Option<i64>,
    #[prost(int64, optional, tag = "10")]
    pub field10: Option<i64>,
    #[prost(int64, optional, tag = "11")]
    pub start_ms: Option<i64>,
    #[prost(int64, optional, tag = "12")]
    pub duration_ms: Option<i64>,
    #[prost(message, optional, tag = "13")]
    pub format_id: Option<FormatId>,
    #[prost(int64, optional, tag = "14")]
    pub content_length: Option<i64>,
    #[prost(message, optional, tag = "15")]
    pub time_range: Option<TimeRange>,
}

#[derive(Clone, PartialEq, Message)]
pub struct SabrError {
    #[prost(string, optional, tag = "1")]
    pub error_type: Option<String>,
    #[prost(int32, optional, tag = "2")]
    pub code: Option<i32>,
}

#[derive(Clone, PartialEq, Message)]
pub struct SabrRedirect {
    #[prost(string, optional, tag = "1")]
    pub url: Option<String>,
}

#[derive(Clone, PartialEq, Message)]
pub struct StreamProtectionStatus {
    #[prost(int32, optional, tag = "1")]
    pub status: Option<i32>,
}

#[derive(Clone, PartialEq, Message)]
pub struct PlaybackCookie {
    #[prost(int32, optional, tag = "1")]
    pub field1: Option<i32>,
    #[prost(int32, optional, tag = "2")]
    pub field2: Option<i32>,
    #[prost(message, optional, tag = "7")]
    pub video_fmt: Option<FormatId>,
    #[prost(message, optional, tag = "8")]
    pub audio_fmt: Option<FormatId>,
}

#[derive(Clone, PartialEq, Message)]
pub struct NextRequestPolicy {
    #[prost(int32, optional, tag = "1")]
    pub target_audio_readahead_ms: Option<i32>,
    #[prost(int32, optional, tag = "2")]
    pub target_video_readahead_ms: Option<i32>,
    #[prost(int32, optional, tag = "4")]
    pub backoff_time_ms: Option<i32>,
    #[prost(message, optional, tag = "7")]
    pub playback_cookie: Option<PlaybackCookie>,
    #[prost(string, optional, tag = "8")]
    pub video_id: Option<String>,
}

#[derive(Clone, PartialEq, Message)]
pub struct FormatInitializationMetadata {
    #[prost(message, optional, tag = "1")]
    pub format_id: Option<FormatId>,
    #[prost(int64, optional, tag = "2")]
    pub duration_ms: Option<i64>,
    #[prost(string, optional, tag = "3")]
    pub mime_type: Option<String>,
    #[prost(int64, optional, tag = "4")]
    pub end_segment_number: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct Sequence {
    pub itag: Option<i32>,
    pub format_id: Option<FormatId>,
    pub is_init_segment: Option<bool>,
    pub duration_ms: Option<i64>,
    pub start_ms: Option<i64>,
    pub start_data_range: Option<i64>,
    pub sequence_number: Option<i64>,
    pub content_length: Option<i64>,
    pub time_range: Option<TimeRange>,
}

#[derive(Debug, Clone)]
pub struct InitializedFormat {
    pub format_id: FormatId,
    pub format_key: String,
    pub duration_ms: Option<i64>,
    pub mime_type: Option<String>,
    pub sequence_count: Option<i64>,
    pub sequence_list: Vec<Sequence>,
    pub media_chunks: Vec<Bytes>,
}

#[derive(Debug, Clone)]
pub struct SabrResponse {
    pub initialized_formats: Vec<InitializedFormat>,
    pub stream_protection_status: Option<StreamProtectionStatus>,
    pub sabr_redirect: Option<SabrRedirect>,
    pub sabr_error: Option<SabrError>,
    pub next_request_policy: Option<NextRequestPolicy>,
}

#[derive(Debug)]
pub struct UmpPart {
    pub part_type: PartType,
    pub size: usize,
    pub data: Bytes,
}

pub struct SabrParser {
    header_id_to_format_key: HashMap<u32, String>,
    formats_by_key: HashMap<String, InitializedFormat>,
    initialized_formats: Vec<InitializedFormat>,
    playback_cookie: Option<PlaybackCookie>,
}

impl SabrParser {
    pub fn new() -> Self {
        Self {
            header_id_to_format_key: HashMap::new(),
            formats_by_key: HashMap::new(),
            initialized_formats: Vec::new(),
            playback_cookie: None,
        }
    }

    pub fn parse_response(&mut self, data: &[u8]) -> io::Result<SabrResponse> {
        self.header_id_to_format_key.clear();

        // Clear sequence lists and media chunks for existing formats
        for format in &mut self.initialized_formats {
            format.sequence_list.clear();
            format.media_chunks.clear();
        }

        let mut sabr_error: Option<SabrError> = None;
        let mut sabr_redirect: Option<SabrRedirect> = None;
        let mut stream_protection_status: Option<StreamProtectionStatus> = None;
        let mut next_request_policy: Option<NextRequestPolicy> = None;

        let mut offset = 0;
        while offset < data.len() {
            match self.parse_ump_part(&data[offset..]) {
                Ok((part, consumed)) => {
                    offset += consumed;

                    match part.part_type {
                        PartType::MediaHeader => {
                            self.process_media_header(&part.data)?;
                        }
                        PartType::Media => {
                            self.process_media_data(&part.data)?;
                        }
                        PartType::MediaEnd => {
                            self.process_media_end(&part.data)?;
                        }
                        PartType::NextRequestPolicy => {
                            let policy = NextRequestPolicy::decode(&part.data[..])
                                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

                            // Store playback cookie for use in subsequent requests
                            if let Some(cookie) = &policy.playback_cookie {
                                self.playback_cookie = Some(cookie.clone());
                            }

                            next_request_policy = Some(policy);
                        }
                        PartType::FormatInitializationMetadata => {
                            self.process_format_initialization(&part.data)?;
                        }
                        PartType::SabrError => {
                            sabr_error = Some(
                                SabrError::decode(&part.data[..])
                                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
                            );
                        }
                        PartType::SabrRedirect => {
                            sabr_redirect = Some(
                                SabrRedirect::decode(&part.data[..])
                                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
                            );
                        }
                        PartType::StreamProtectionStatus => {
                            stream_protection_status = Some(
                                StreamProtectionStatus::decode(&part.data[..])
                                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
                            );
                        }
                        _ => {
                            // Ignore other part types for now
                        }
                    }
                }
                Err(_) => break, // Not enough data or invalid part
            }
        }

        // Update initialized_formats with the processed data from formats_by_key
        self.initialized_formats.clear();
        for format in self.formats_by_key.values() {
            self.initialized_formats.push(format.clone());
        }

        // Sort by format_key to ensure deterministic ordering
        self.initialized_formats
            .sort_by(|a, b| a.format_key.cmp(&b.format_key));

        Ok(SabrResponse {
            initialized_formats: self.initialized_formats.clone(),
            stream_protection_status,
            sabr_redirect,
            sabr_error,
            next_request_policy,
        })
    }

    fn parse_ump_part(&self, data: &[u8]) -> io::Result<(UmpPart, usize)> {
        let mut offset = 0;

        let (part_type, consumed) = self.read_varint(&data[offset..])?;
        offset += consumed;

        let (part_size, consumed) = self.read_varint(&data[offset..])?;
        offset += consumed;

        if data.len() < offset + part_size as usize {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Not enough data for part",
            ));
        }

        let part_data = Bytes::copy_from_slice(&data[offset..offset + part_size as usize]);
        offset += part_size as usize;

        Ok((
            UmpPart {
                part_type: PartType::from(part_type),
                size: part_size as usize,
                data: part_data,
            },
            offset,
        ))
    }

    fn read_varint(&self, data: &[u8]) -> io::Result<(i32, usize)> {
        if data.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "No data to read varint",
            ));
        }

        let first_byte = data[0];
        let byte_length = if first_byte < 128 {
            1
        } else if first_byte < 192 {
            2
        } else if first_byte < 224 {
            3
        } else if first_byte < 240 {
            4
        } else {
            5
        };

        if data.len() < byte_length {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Not enough data for varint",
            ));
        }

        let value = match byte_length {
            1 => data[0] as i32,
            2 => {
                let byte1 = data[0];
                let byte2 = data[1];
                (byte1 as i32 & 0x3f) + 64 * (byte2 as i32)
            }
            3 => {
                let byte1 = data[0];
                let byte2 = data[1];
                let byte3 = data[2];
                (byte1 as i32 & 0x1f) + 32 * (byte2 as i32 + 256 * byte3 as i32)
            }
            4 => {
                let byte1 = data[0];
                let byte2 = data[1];
                let byte3 = data[2];
                let byte4 = data[3];
                (byte1 as i32 & 0x0f)
                    + 16 * (byte2 as i32 + 256 * (byte3 as i32 + 256 * byte4 as i32))
            }
            _ => {
                let value = u32::from_le_bytes([data[1], data[2], data[3], data[4]]) as i32;
                value
            }
        };

        Ok((value, byte_length))
    }

    fn process_media_header(&mut self, data: &Bytes) -> io::Result<()> {
        let media_header = MediaHeader::decode(&data[..])
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        if let Some(format_id) = &media_header.format_id {
            let format_key = self.get_format_key(format_id);

            // Register format if not exists
            if !self.formats_by_key.contains_key(&format_key) {
                self.register_format_from_header(&media_header);
            }

            // Save header ID mapping
            if let Some(header_id) = media_header.header_id {
                self.header_id_to_format_key
                    .insert(header_id, format_key.clone());
            }

            // Add sequence to format
            if let Some(format) = self.formats_by_key.get_mut(&format_key) {
                let sequence = Sequence {
                    itag: media_header.itag,
                    format_id: media_header.format_id.clone(),
                    is_init_segment: media_header.is_init_seg,
                    duration_ms: media_header.duration_ms,
                    start_ms: media_header.start_ms,
                    start_data_range: media_header.start_range,
                    sequence_number: media_header.sequence_number,
                    content_length: media_header.content_length,
                    time_range: media_header.time_range.clone(),
                };
                format.sequence_list.push(sequence);
            }
        }

        Ok(())
    }

    fn process_media_data(&mut self, data: &Bytes) -> io::Result<()> {
        if data.is_empty() {
            return Ok(());
        }

        let header_id = data[0] as u32;
        let stream_data = data.slice(1..);

        if let Some(format_key) = self.header_id_to_format_key.get(&header_id) {
            if let Some(format) = self.formats_by_key.get_mut(format_key) {
                format.media_chunks.push(stream_data);
            }
        }

        Ok(())
    }

    fn process_media_end(&mut self, data: &Bytes) -> io::Result<()> {
        if !data.is_empty() {
            let header_id = data[0] as u32;
            self.header_id_to_format_key.remove(&header_id);
        }
        Ok(())
    }

    fn process_format_initialization(&mut self, data: &Bytes) -> io::Result<()> {
        let format_init = FormatInitializationMetadata::decode(&data[..])
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        self.register_format_from_init(&format_init);
        Ok(())
    }

    fn register_format_from_header(&mut self, header: &MediaHeader) {
        if let Some(format_id) = &header.format_id {
            let format_key = self.get_format_key(format_id);

            let format = InitializedFormat {
                format_id: format_id.clone(),
                format_key: format_key.clone(),
                duration_ms: header.duration_ms,
                mime_type: None,
                sequence_count: None,
                sequence_list: Vec::new(),
                media_chunks: Vec::new(),
            };

            self.initialized_formats.push(format.clone());
            self.formats_by_key.insert(format_key.clone(), format);
        }
    }

    fn register_format_from_init(&mut self, init: &FormatInitializationMetadata) {
        if let Some(format_id) = &init.format_id {
            let format_key = self.get_format_key(format_id);

            if !self.formats_by_key.contains_key(&format_key) {
                let format = InitializedFormat {
                    format_id: format_id.clone(),
                    format_key: format_key.clone(),
                    duration_ms: init.duration_ms,
                    mime_type: init.mime_type.clone(),
                    sequence_count: init.end_segment_number,
                    sequence_list: Vec::new(),
                    media_chunks: Vec::new(),
                };

                self.initialized_formats.push(format.clone());
                self.formats_by_key.insert(format_key.clone(), format);
            }
        }
    }

    pub fn get_playback_cookie(&self) -> Option<&PlaybackCookie> {
        self.playback_cookie.as_ref()
    }

    fn get_format_key(&self, format_id: &FormatId) -> String {
        format!(
            "{};{};",
            format_id.itag.unwrap_or(0),
            format_id.last_modified.unwrap_or(0)
        )
    }
}

impl Default for SabrParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_sabr_parser_with_bin_file() {
        // Read the sabr_response.bin file
        let data = fs::read("test/sabr_response.bin").expect("Failed to read sabr_response.bin");

        let mut parser = SabrParser::new();
        match parser.parse_response(&data) {
            Ok(response) => {
                // Verify the next_request_policy fields
                assert!(response.next_request_policy.is_some());
                if let Some(policy) = &response.next_request_policy {
                    assert_eq!(policy.target_audio_readahead_ms, Some(15016));
                    assert_eq!(policy.target_video_readahead_ms, Some(15016));
                    assert_eq!(policy.backoff_time_ms, None);
                    assert_eq!(policy.video_id, None);

                    // Verify playback cookie
                    assert!(policy.playback_cookie.is_some());
                    if let Some(cookie) = &policy.playback_cookie {
                        // Verify video format
                        assert!(cookie.video_fmt.is_some());
                        if let Some(video_fmt) = &cookie.video_fmt {
                            assert_eq!(video_fmt.itag, Some(136));
                            assert_eq!(video_fmt.last_modified, Some(1747754870573293));
                            assert_eq!(video_fmt.xtags, None);
                        }

                        // Verify audio format
                        assert!(cookie.audio_fmt.is_some());
                        if let Some(audio_fmt) = &cookie.audio_fmt {
                            assert_eq!(audio_fmt.itag, Some(251));
                            assert_eq!(audio_fmt.last_modified, Some(1747754876286051));
                            assert_eq!(audio_fmt.xtags, None);
                        }
                    }
                }

                // Assert the expected number of initialized formats
                assert_eq!(response.initialized_formats.len(), 2);

                // Assert that we have a stream protection status
                assert!(response.stream_protection_status.is_some());

                if let Some(status) = &response.stream_protection_status {
                    // Assert the expected stream protection status
                    assert_eq!(status.status, Some(1));
                }

                // Assert specific format details
                assert_eq!(
                    response.initialized_formats[0].format_key,
                    "136;1747754870573293;"
                );
                assert_eq!(response.initialized_formats[0].sequence_list.len(), 4);
                assert_eq!(response.initialized_formats[0].media_chunks.len(), 16);

                // Verify specific chunk sizes for format 136 (video)
                let video_chunk_sizes: Vec<usize> = response.initialized_formats[0]
                    .media_chunks
                    .iter()
                    .map(|c| c.len())
                    .collect();
                assert_eq!(
                    video_chunk_sizes,
                    vec![
                        32768, 32768, 12433, 32768, 32768, 16936, 32768, 32768, 29449, 6024, 16384,
                        16384, 16384, 16384, 16384, 1942
                    ]
                );

                // Verify sequence details for format 136 (video)
                assert_eq!(
                    response.initialized_formats[0].sequence_list[0].sequence_number,
                    Some(26)
                );
                assert_eq!(
                    response.initialized_formats[0].sequence_list[0].start_ms,
                    Some(123888)
                );
                assert_eq!(
                    response.initialized_formats[0].sequence_list[0].duration_ms,
                    Some(5008)
                );

                assert_eq!(
                    response.initialized_formats[0].sequence_list[1].sequence_number,
                    Some(27)
                );
                assert_eq!(
                    response.initialized_formats[0].sequence_list[1].start_ms,
                    Some(128896)
                );
                assert_eq!(
                    response.initialized_formats[0].sequence_list[1].duration_ms,
                    Some(5008)
                );

                assert_eq!(
                    response.initialized_formats[0].sequence_list[2].sequence_number,
                    Some(28)
                );
                assert_eq!(
                    response.initialized_formats[0].sequence_list[2].start_ms,
                    Some(133903)
                );
                assert_eq!(
                    response.initialized_formats[0].sequence_list[2].duration_ms,
                    Some(5008)
                );

                assert_eq!(
                    response.initialized_formats[0].sequence_list[3].sequence_number,
                    Some(29)
                );
                assert_eq!(
                    response.initialized_formats[0].sequence_list[3].start_ms,
                    Some(138910)
                );
                assert_eq!(
                    response.initialized_formats[0].sequence_list[3].duration_ms,
                    Some(4974)
                );

                assert_eq!(
                    response.initialized_formats[1].format_key,
                    "251;1747754876286051;"
                );
                assert_eq!(response.initialized_formats[1].sequence_list.len(), 3);
                assert_eq!(response.initialized_formats[1].media_chunks.len(), 15);

                // Verify specific chunk sizes for format 251 (audio)
                let audio_chunk_sizes: Vec<usize> = response.initialized_formats[1]
                    .media_chunks
                    .iter()
                    .map(|c| c.len())
                    .collect();
                assert_eq!(
                    audio_chunk_sizes,
                    vec![
                        32768, 32768, 32768, 32768, 2862, 32768, 32768, 32768, 32768, 1553, 32768,
                        32768, 32768, 32768, 2552
                    ]
                );

                // Verify sequence details for format 251 (audio)
                assert_eq!(
                    response.initialized_formats[1].sequence_list[0].sequence_number,
                    Some(13)
                );
                assert_eq!(
                    response.initialized_formats[1].sequence_list[0].start_ms,
                    Some(120001)
                );
                assert_eq!(
                    response.initialized_formats[1].sequence_list[0].duration_ms,
                    Some(10000)
                );

                assert_eq!(
                    response.initialized_formats[1].sequence_list[1].sequence_number,
                    Some(14)
                );
                assert_eq!(
                    response.initialized_formats[1].sequence_list[1].start_ms,
                    Some(130001)
                );
                assert_eq!(
                    response.initialized_formats[1].sequence_list[1].duration_ms,
                    Some(10000)
                );

                assert_eq!(
                    response.initialized_formats[1].sequence_list[2].sequence_number,
                    Some(15)
                );
                assert_eq!(
                    response.initialized_formats[1].sequence_list[2].start_ms,
                    Some(140001)
                );
                assert_eq!(
                    response.initialized_formats[1].sequence_list[2].duration_ms,
                    Some(10000)
                );

                // Verify chunk distribution per sequence (based on our analysis)
                // Format 136 should have chunks distributed as: 3 + 3 + 3 + 7 = 16 total
                // Format 251 should have chunks distributed as: 5 + 5 + 5 = 15 total

                // Verify that all sequences have the expected itag values
                for seq in &response.initialized_formats[0].sequence_list {
                    assert_eq!(seq.itag, Some(136));
                }
                for seq in &response.initialized_formats[1].sequence_list {
                    assert_eq!(seq.itag, Some(251));
                }
            }
            Err(e) => {
                panic!("Failed to parse SABR response: {}", e);
            }
        }
    }

    #[test]
    fn test_varint_parsing() {
        let parser = SabrParser::new();

        // Test single byte varint
        let data = [0x08]; // 8 in varint encoding
        let (value, consumed) = parser.read_varint(&data).unwrap();
        assert_eq!(value, 8);
        assert_eq!(consumed, 1);

        // Test two byte varint - let's use a correct encoding
        // For 150: first byte should be >= 128 to indicate 2-byte encoding
        // 150 = (first_byte & 0x3f) + 64 * second_byte
        // Let's use [0x96, 0x01] which should give us: (0x96 & 0x3f) + 64 * 0x01 = 22 + 64 = 86
        let data = [0x96, 0x01];
        let (value, consumed) = parser.read_varint(&data).unwrap();
        assert_eq!(value, 86); // Corrected expected value
        assert_eq!(consumed, 2);

        // Test another two-byte varint for 150
        // 150 = (first_byte & 0x3f) + 64 * second_byte
        // We need: 150 = x + 64 * y where x <= 63
        // 150 = 22 + 64 * 2, so first_byte = 128 + 22 = 150, second_byte = 2
        let data = [0x96, 0x02]; // This should give us (0x96 & 0x3f) + 64 * 0x02 = 22 + 128 = 150
        let (value, consumed) = parser.read_varint(&data).unwrap();
        assert_eq!(value, 150);
        assert_eq!(consumed, 2);
    }

    #[test]
    fn test_part_type_conversion() {
        assert_eq!(PartType::from(20), PartType::MediaHeader);
        assert_eq!(PartType::from(21), PartType::Media);
        assert_eq!(PartType::from(35), PartType::NextRequestPolicy);
        assert_eq!(PartType::from(43), PartType::SabrRedirect);
        assert_eq!(PartType::from(44), PartType::SabrError);
    }
}

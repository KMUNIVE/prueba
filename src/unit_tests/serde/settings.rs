use crate::types::profile::{FrameRateMatchingStrategy, Settings};
use chrono::{TimeZone, Utc};
use serde_test::{assert_de_tokens, assert_tokens, Token};
use url::Url;

#[test]
fn settings() {
    assert_tokens(
        &Settings {
            interface_language: "interface_language".to_owned(),
            streaming_server_url: Url::parse("https://streaming_server_url").unwrap(),
            player_type: Some("player".to_owned()),
            binge_watching: true,
            play_in_background: true,
            hardware_decoding: true,
            frame_rate_matching_strategy: FrameRateMatchingStrategy::FrameRateAndResolution,
            next_video_notification_duration: 30,
            audio_passthrough: true,
            audio_language: "audio_language".to_owned(),
            secondary_audio_language: Some("secondary_audio_language".to_owned()),
            subtitles_language: "subtitles_language".to_owned(),
            secondary_subtitles_language: Some("secondary_subtitles_language".to_owned()),
            subtitles_size: 1,
            subtitles_font: "subtitles_font".to_owned(),
            subtitles_bold: true,
            subtitles_offset: 1,
            subtitles_text_color: "subtitles_text_color".to_owned(),
            subtitles_background_color: "subtitles_background_color".to_owned(),
            subtitles_outline_color: "subtitles_outline_color".to_owned(),
            esc_exists_fullscreen: true,
            seek_time_duration: 1,
            seek_shift_time_duration: 2,
            streaming_server_warning_dismissed: Some(
                Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap(),
            ),
        },
        &[
            Token::Struct {
                name: "Settings",
                len: 24,
            },
            Token::Str("interfaceLanguage"),
            Token::Str("interface_language"),
            Token::Str("streamingServerUrl"),
            Token::Str("https://streaming_server_url/"),
            Token::Str("playerType"),
            Token::Some,
            Token::Str("player"),
            Token::Str("bingeWatching"),
            Token::Bool(true),
            Token::Str("playInBackground"),
            Token::Bool(true),
            Token::Str("hardwareDecoding"),
            Token::Bool(true),
            Token::Str("frameRateMatchingStrategy"),
            Token::UnitVariant {
                name: "FrameRateMatchingStrategy",
                variant: "FrameRateAndResolution",
            },
            Token::Str("nextVideoNotificationDuration"),
            Token::U32(30),
            Token::Str("audioPassthrough"),
            Token::Bool(true),
            Token::Str("audioLanguage"),
            Token::Str("audio_language"),
            Token::Str("secondaryAudioLanguage"),
            Token::Some,
            Token::Str("secondary_audio_language"),
            Token::Str("subtitlesLanguage"),
            Token::Str("subtitles_language"),
            Token::Str("secondarySubtitlesLanguage"),
            Token::Some,
            Token::Str("secondary_subtitles_language"),
            Token::Str("subtitlesSize"),
            Token::U8(1),
            Token::Str("subtitlesFont"),
            Token::Str("subtitles_font"),
            Token::Str("subtitlesBold"),
            Token::Bool(true),
            Token::Str("subtitlesOffset"),
            Token::U8(1),
            Token::Str("subtitlesTextColor"),
            Token::Str("subtitles_text_color"),
            Token::Str("subtitlesBackgroundColor"),
            Token::Str("subtitles_background_color"),
            Token::Str("subtitlesOutlineColor"),
            Token::Str("subtitles_outline_color"),
            Token::Str("escExistsFullscreen"),
            Token::Bool(true),
            Token::Str("seekTimeDuration"),
            Token::U32(1),
            Token::Str("seekShiftTimeDuration"),
            Token::U32(2),
            Token::Str("streamingServerWarningDismissed"),
            Token::Some,
            Token::Str("2021-01-01T00:00:00Z"),
            Token::StructEnd,
        ],
    );
}

#[test]
fn settings_de() {
    assert_de_tokens(
        &Settings::default(),
        &[
            Token::Struct {
                name: "Settings",
                len: 19,
            },
            Token::Str("interfaceLanguage"),
            Token::Str("eng"),
            Token::Str("streamingServerUrl"),
            Token::Str("http://127.0.0.1:11470/"),
            Token::Str("playerType"),
            Token::None,
            Token::Str("bingeWatching"),
            Token::Bool(true),
            Token::Str("playInBackground"),
            Token::Bool(true),
            Token::Str("hardwareDecoding"),
            Token::Bool(true),
            Token::Str("frameRateMatchingStrategy"),
            Token::UnitVariant {
                name: "FrameRateMatchingStrategy",
                variant: "FrameRateOnly",
            },
            Token::Str("nextVideoNotificationDuration"),
            Token::U32(35000),
            Token::Str("audioPassthrough"),
            Token::Bool(false),
            Token::Str("audioLanguage"),
            Token::Str("eng"),
            Token::Str("subtitlesLanguage"),
            Token::Str("eng"),
            Token::Str("subtitlesSize"),
            Token::U8(100),
            Token::Str("subtitlesFont"),
            Token::Str("Roboto"),
            Token::Str("subtitlesBold"),
            Token::Bool(false),
            Token::Str("subtitlesOffset"),
            Token::U8(5),
            Token::Str("subtitlesTextColor"),
            Token::Str("#FFFFFFFF"),
            Token::Str("subtitlesBackgroundColor"),
            Token::Str("#00000000"),
            Token::Str("subtitlesOutlineColor"),
            Token::Str("#000000"),
            Token::Str("escExistsFullscreen"),
            Token::Bool(true),
            Token::Str("seekTimeDuration"),
            Token::U32(20000),
            Token::Str("seekShiftTimeDuration"),
            Token::U32(10000),
            Token::Str("streamingServerWarningDismissed"),
            Token::None,
            Token::StructEnd,
        ],
    );
}

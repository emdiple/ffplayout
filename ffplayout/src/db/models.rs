use regex::Regex;
use serde::{
    de::{self, Visitor},
    Deserialize, Serialize,
};

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct User {
    #[sqlx(default)]
    #[serde(skip_deserializing)]
    pub id: i32,
    #[sqlx(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mail: Option<String>,
    pub username: String,
    #[sqlx(default)]
    #[serde(skip_serializing, default = "empty_string")]
    pub password: String,
    #[sqlx(default)]
    #[serde(skip_serializing)]
    pub role_id: Option<i32>,
    #[sqlx(default)]
    #[serde(skip_serializing)]
    pub channel_id: Option<i32>,
    #[sqlx(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

fn empty_string() -> String {
    "".to_string()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoginUser {
    pub id: i32,
    pub username: String,
}

impl LoginUser {
    pub fn new(id: i32, username: String) -> Self {
        Self { id, username }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, sqlx::FromRow)]
pub struct TextPreset {
    #[sqlx(default)]
    #[serde(skip_deserializing)]
    pub id: i32,
    pub channel_id: i32,
    pub name: String,
    pub text: String,
    pub x: String,
    pub y: String,
    #[serde(deserialize_with = "deserialize_number_or_string")]
    pub fontsize: String,
    #[serde(deserialize_with = "deserialize_number_or_string")]
    pub line_spacing: String,
    pub fontcolor: String,
    pub r#box: String,
    pub boxcolor: String,
    #[serde(deserialize_with = "deserialize_number_or_string")]
    pub boxborderw: String,
    #[serde(deserialize_with = "deserialize_number_or_string")]
    pub alpha: String,
}

/// Deserialize number or string
pub fn deserialize_number_or_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct StringOrNumberVisitor;

    impl<'de> Visitor<'de> for StringOrNumberVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or a number")
        }

        fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
            let re = Regex::new(r"0,([0-9]+)").unwrap();
            let clean_string = re.replace_all(value, "0.$1").to_string();
            Ok(clean_string)
        }

        fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
            Ok(value.to_string())
        }

        fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
            Ok(value.to_string())
        }

        fn visit_f64<E: de::Error>(self, value: f64) -> Result<Self::Value, E> {
            Ok(value.to_string())
        }
    }

    deserializer.deserialize_any(StringOrNumberVisitor)
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, sqlx::FromRow)]
pub struct Channel {
    #[serde(skip_deserializing)]
    pub id: i32,
    pub name: String,
    pub preview_url: String,
    pub extra_extensions: String,
    pub active: bool,
    pub current_date: Option<String>,
    pub time_shift: f64,

    #[sqlx(default)]
    #[serde(default)]
    pub utc_offset: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Configuration {
    pub id: i32,
    pub channel_id: i32,
    pub general_help: String,
    pub stop_threshold: f64,

    pub mail_help: String,
    pub subject: String,
    pub smtp_server: String,
    pub starttls: bool,
    pub sender_addr: String,
    pub sender_pass: String,
    pub recipient: String,
    pub mail_level: String,
    pub interval: i64,

    pub logging_help: String,
    pub ffmpeg_level: String,
    pub ingest_level: String,
    #[serde(default)]
    pub detect_silence: bool,
    #[serde(default)]
    pub ignore_lines: String,

    pub processing_help: String,
    pub processing_mode: String,
    #[serde(default)]
    pub audio_only: bool,
    #[serde(default = "default_track_index")]
    pub audio_track_index: i32,
    #[serde(default)]
    pub copy_audio: bool,
    #[serde(default)]
    pub copy_video: bool,
    pub width: i64,
    pub height: i64,
    pub aspect: f64,
    pub fps: f64,
    pub add_logo: bool,
    pub logo: String,
    pub logo_scale: String,
    pub logo_opacity: f32,
    pub logo_position: String,
    #[serde(default = "default_tracks")]
    pub audio_tracks: i32,
    #[serde(default = "default_channels")]
    pub audio_channels: u8,
    pub volume: f64,
    #[serde(default)]
    pub decoder_filter: String,

    pub ingest_help: String,
    pub ingest_enable: bool,
    pub ingest_param: String,
    #[serde(default)]
    pub ingest_filter: String,

    pub playlist_help: String,
    pub playlist_path: String,
    pub day_start: String,
    pub length: String,
    pub infinit: bool,

    pub storage_help: String,
    pub storage_path: String,

    #[serde(alias = "filler_clip")]
    pub filler: String,
    pub extensions: String,
    pub shuffle: bool,

    pub text_help: String,
    pub add_text: bool,

    pub fontfile: String,
    pub text_from_filename: bool,
    pub style: String,
    pub regex: String,

    pub task_help: String,
    pub task_enable: bool,
    pub task_path: String,

    pub output_help: String,
    pub output_mode: String,
    pub output_param: String,
}

fn default_track_index() -> i32 {
    -1
}

fn default_tracks() -> i32 {
    1
}

fn default_channels() -> u8 {
    2
}

#[derive(Clone, Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct AdvancedConfiguration {
    pub id: i32,
    pub channel_id: i32,
    pub decoder_input_param: Option<String>,
    pub decoder_output_param: Option<String>,
    pub encoder_input_param: Option<String>,
    pub ingest_input_param: Option<String>,
    pub deinterlace: Option<String>,
    pub pad_scale_w: Option<String>,
    pub pad_scale_h: Option<String>,
    pub pad_video: Option<String>,
    pub fps: Option<String>,
    pub scale: Option<String>,
    pub set_dar: Option<String>,
    pub fade_in: Option<String>,
    pub fade_out: Option<String>,
    pub overlay_logo_scale: Option<String>,
    pub overlay_logo_fade_in: Option<String>,
    pub overlay_logo_fade_out: Option<String>,
    pub overlay_logo: Option<String>,
    pub tpad: Option<String>,
    pub drawtext_from_file: Option<String>,
    pub drawtext_from_zmq: Option<String>,
    pub aevalsrc: Option<String>,
    pub afade_in: Option<String>,
    pub afade_out: Option<String>,
    pub apad: Option<String>,
    pub volume: Option<String>,
    pub split: Option<String>,
}

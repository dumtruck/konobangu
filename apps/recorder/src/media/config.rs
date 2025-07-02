use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, TS)]
#[ts(rename_all = "camelCase")]
pub enum AutoOptimizeImageFormat {
    #[serde(rename = "image/webp")]
    Webp,
    #[serde(rename = "image/avif")]
    Avif,
    #[serde(rename = "image/jxl")]
    Jxl,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, TS, PartialEq)]
#[ts(rename_all = "camelCase")]
pub struct EncodeWebpOptions {
    pub quality: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, TS, PartialEq)]
#[ts(rename_all = "camelCase")]
pub struct EncodeAvifOptions {
    pub quality: Option<u8>,
    pub speed: Option<u8>,
    pub threads: Option<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, TS, PartialEq)]
#[ts(rename_all = "camelCase")]
pub struct EncodeJxlOptions {
    pub quality: Option<f32>,
    pub speed: Option<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS, PartialEq)]
#[ts(tag = "mimeType")]
#[serde(tag = "mime_type")]
pub enum EncodeImageOptions {
    #[serde(rename = "image/webp")]
    Webp(EncodeWebpOptions),
    #[serde(rename = "image/avif")]
    Avif(EncodeAvifOptions),
    #[serde(rename = "image/jxl")]
    Jxl(EncodeJxlOptions),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MediaConfig {
    #[serde(default = "default_webp_quality")]
    pub webp_quality: f32,
    #[serde(default = "default_avif_quality")]
    pub avif_quality: u8,
    #[serde(default = "default_avif_speed")]
    pub avif_speed: u8,
    #[serde(default = "default_avif_threads")]
    pub avif_threads: u8,
    #[serde(default = "default_jxl_quality")]
    pub jxl_quality: f32,
    #[serde(default = "default_jxl_speed")]
    pub jxl_speed: u8,
    #[serde(default = "default_auto_optimize_formats")]
    pub auto_optimize_formats: Vec<AutoOptimizeImageFormat>,
}

impl Default for MediaConfig {
    fn default() -> Self {
        Self {
            webp_quality: default_webp_quality(),
            avif_quality: default_avif_quality(),
            avif_speed: default_avif_speed(),
            avif_threads: default_avif_threads(),
            jxl_quality: default_jxl_quality(),
            jxl_speed: default_jxl_speed(),
            auto_optimize_formats: default_auto_optimize_formats(),
        }
    }
}

fn default_webp_quality() -> f32 {
    80.0
}

fn default_avif_quality() -> u8 {
    80
}

fn default_avif_speed() -> u8 {
    6
}

fn default_avif_threads() -> u8 {
    1
}

fn default_jxl_quality() -> f32 {
    80.0
}

fn default_jxl_speed() -> u8 {
    7
}

fn default_auto_optimize_formats() -> Vec<AutoOptimizeImageFormat> {
    vec![
        AutoOptimizeImageFormat::Webp,
        // AutoOptimizeImageFormat::Avif,   // TOO SLOW */
        #[cfg(feature = "jxl")]
        AutoOptimizeImageFormat::Jxl,
    ]
}

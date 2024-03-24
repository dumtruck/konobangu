use lazy_static::lazy_static;
use oxilangtag::LanguageTag;
use serde::{Deserialize, Serialize};

use crate::parsers::errors::ParseError;

lazy_static! {
    static ref LANGTAG_ADHOC_ALIAS_PAIRS: Vec<(&'static str, &'static str)> = {
        vec![
            ("tc", "zh-TW"),
            ("zh-tw", "zh-TW"),
            ("cht", "zh-TW"),
            ("繁", "zh-TW"),
            ("sc", "zh-CN"),
            ("chs", "zh-CN"),
            ("简", "zh-CN"),
            ("zh-cn", "zh-CN"),
            ("eng", "en"),
            ("英", "en"),
            ("jp", "ja-JP"),
            ("jpn", "ja-JP"),
            ("日", "ja"),
        ]
    };
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LanguagePresetName {
    #[serde(rename = "zh-TW")]
    ZhCN,
    #[serde(rename = "zh-CN")]
    ZhTW,
    #[serde(rename = "zh")]
    Zh,
    #[serde(rename = "en")]
    En,
    #[serde(rename = "ja")]
    Ja,
}

#[derive(Debug, Clone)]
pub struct LanguagePreset {
    name: LanguagePresetName,
    tag: LanguageTag<String>,
}

impl LanguagePreset {
    pub fn parse<S: AsRef<str>>(s: S) -> Result<Self, ParseError> {
        let s = s.as_ref();
        let s_lower = s.to_lowercase();
        let mut s_rc = s;
        for (alias, v) in LANGTAG_ADHOC_ALIAS_PAIRS.iter() {
            if s_lower.contains(alias) {
                s_rc = v;
                break;
            }
        }
        let lang_tag = LanguageTag::parse(s_rc.to_string())?;

        let primary = lang_tag.primary_language();
        let region = lang_tag.region();

        let kind = match (primary, region) {
            ("zh", Some("TW")) => LanguagePresetName::ZhTW,
            ("zh", Some("CN")) => LanguagePresetName::ZhCN,
            ("zh", _) => LanguagePresetName::Zh,
            ("en", _) => LanguagePresetName::En,
            ("ja", _) => LanguagePresetName::Ja,
            _ => Err(ParseError::UnsupportedLanguagePreset(s_rc.to_string()))?,
        };

        Ok(Self {
            name: kind,
            tag: lang_tag,
        })
    }

    pub fn name(&self) -> &LanguagePresetName {
        &self.name
    }

    pub fn name_str(&self) -> &str {
        &self.name.as_ref()
    }

    pub fn tag(&self) -> &LanguageTag<String> {
        &self.tag
    }

    pub fn tag_str(&self) -> &str {
        &self.tag.as_str()
    }
}

impl AsRef<str> for LanguagePresetName {
    fn as_ref(&self) -> &str {
        match self {
            Self::ZhTW => "zh-TW",
            Self::ZhCN => "zh-CN",
            Self::Zh => "zh",
            Self::En => "en",
            Self::Ja => "ja",
        }
    }
}

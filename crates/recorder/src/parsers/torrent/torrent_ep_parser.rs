use eyre::OptionExt;
use fancy_regex::Regex as FancyRegex;
use lazy_static::lazy_static;
use quirks_path::Path;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::parsers::defs::SUBTITLE_LANG;

lazy_static! {
    static ref TORRENT_EP_PARSE_RULES: Vec<FancyRegex> = {
        vec![
            FancyRegex::new(
                r"(.*) - (\d{1,4}(?!\d|p)|\d{1,4}\.\d{1,2}(?!\d|p))(?:v\d{1,2})?(?: )?(?:END)?(.*)",
            )
            .unwrap(),
            FancyRegex::new(
                r"(.*)[\[\ E](\d{1,4}|\d{1,4}\.\d{1,2})(?:v\d{1,2})?(?: )?(?:END)?[\]\ ](.*)",
            )
            .unwrap(),
            FancyRegex::new(r"(.*)\[(?:第)?(\d*\.*\d*)[话集話](?:END)?\](.*)").unwrap(),
            FancyRegex::new(r"(.*)第?(\d*\.*\d*)[话話集](?:END)?(.*)").unwrap(),
            FancyRegex::new(r"(.*)(?:S\d{2})?EP?(\d+)(.*)").unwrap(),
        ]
    };
    static ref GET_FANSUB_SPLIT_RE: Regex = Regex::new(r"[\[\]()【】（）]").unwrap();
    static ref GET_FANSUB_FULL_MATCH_RE: Regex = Regex::new(r"^\d+$").unwrap();
    static ref GET_SEASON_AND_TITLE_SUB_RE: Regex = Regex::new(r"([Ss]|Season )\d{1,3}").unwrap();
    static ref GET_SEASON_AND_TITLE_FIND_RE: Regex =
        Regex::new(r"([Ss]|Season )(\d{1,3})").unwrap();
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TorrentEpisodeMediaMeta {
    pub fansub: Option<String>,
    pub title: String,
    pub season: i32,
    pub episode_index: i32,
    pub extname: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TorrentEpisodeSubtitleMeta {
    pub media: TorrentEpisodeMediaMeta,
    pub lang: Option<String>,
}

fn get_fansub(group_and_title: &str) -> (Option<&str>, &str) {
    let n = GET_FANSUB_SPLIT_RE
        .split(group_and_title)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();

    match (n.get(0), n.get(1)) {
        (None, None) => (None, ""),
        (Some(n0), None) => (None, *n0),
        (Some(n0), Some(n1)) => {
            if GET_FANSUB_FULL_MATCH_RE.is_match(*n1) {
                (None, group_and_title)
            } else {
                (Some(*n0), *n1)
            }
        }
        _ => unreachable!("vec contains n1 must contains n0"),
    }
}

fn get_season_and_title(season_and_title: &str) -> (String, i32) {
    let replaced_title = GET_SEASON_AND_TITLE_SUB_RE.replace_all(season_and_title, "");
    let title = replaced_title.trim().to_string();

    let season = GET_SEASON_AND_TITLE_FIND_RE
        .captures(season_and_title)
        .map(|m| {
            m.get(2)
                .unwrap_or_else(|| unreachable!("season regex should have 2 groups"))
                .as_str()
                .parse::<i32>()
                .unwrap_or_else(|_| unreachable!("season should be a number"))
        })
        .unwrap_or(1);

    (title, season)
}

fn get_subtitle_lang(media_name: &str) -> Option<&str> {
    let media_name_lower = media_name.to_lowercase();
    for (lang, lang_aliases) in SUBTITLE_LANG.iter() {
        if lang_aliases
            .iter()
            .any(|alias| media_name_lower.contains(alias))
        {
            return Some(lang);
        }
    }
    return None;
}

pub fn parse_episode_media_meta_from_torrent(
    torrent_path: &Path,
    torrent_name: Option<&str>,
    season: Option<i32>,
) -> eyre::Result<TorrentEpisodeMediaMeta> {
    let media_name = torrent_path
        .file_name()
        .ok_or_else(|| eyre::eyre!("failed to get file name of {}", torrent_path))?;
    let mut match_obj = None;
    for rule in TORRENT_EP_PARSE_RULES.iter() {
        match_obj = if let Some(torrent_name) = torrent_name.as_ref() {
            rule.captures(torrent_name)?
        } else {
            rule.captures(media_name)?
        };
        if match_obj.is_some() {
            break;
        }
    }
    if let Some(match_obj) = match_obj {
        let group_season_and_title = match_obj
            .get(1)
            .ok_or_else(|| eyre::eyre!("should have 1 group"))?
            .as_str();
        let (fansub, season_and_title) = get_fansub(group_season_and_title);
        let (title, season) = if let Some(season) = season {
            let (title, _) = get_season_and_title(season_and_title);
            (title, season)
        } else {
            get_season_and_title(season_and_title)
        };
        let episode_index = match_obj
            .get(2)
            .ok_or_eyre("should have 2 group")?
            .as_str()
            .parse::<i32>()
            .unwrap_or(1);
        let extname = torrent_path
            .extension()
            .map(|e| format!(".{}", e))
            .unwrap_or_default();
        Ok(TorrentEpisodeMediaMeta {
            fansub: fansub.map(|s| s.to_string()),
            title,
            season,
            episode_index,
            extname,
        })
    } else {
        Err(eyre::eyre!(
            "failed to parse episode media meta from torrent_path='{}' torrent_name='{:?}'",
            torrent_path,
            torrent_name
        ))
    }
}

pub fn parse_episode_subtitle_meta_from_torrent(
    torrent_path: &Path,
    torrent_name: Option<&str>,
    season: Option<i32>,
) -> eyre::Result<TorrentEpisodeSubtitleMeta> {
    let media_meta = parse_episode_media_meta_from_torrent(torrent_path, torrent_name, season)?;
    let media_name = torrent_path
        .file_name()
        .ok_or_else(|| eyre::eyre!("failed to get file name of {}", torrent_path))?;

    let lang = get_subtitle_lang(media_name);

    Ok(TorrentEpisodeSubtitleMeta {
        media: media_meta,
        lang: lang.map(|s| s.to_string()),
    })
}

#[cfg(test)]
mod tests {
    use quirks_path::Path;

    use super::{
        parse_episode_media_meta_from_torrent, parse_episode_subtitle_meta_from_torrent,
        TorrentEpisodeMediaMeta, TorrentEpisodeSubtitleMeta,
    };

    #[test]
    fn test_lilith_raws_media() {
        test_torrent_ep_parser(
            r#"[Lilith-Raws] Boku no Kokoro no Yabai Yatsu - 01 [Baha][WEB-DL][1080p][AVC AAC][CHT][MP4].mp4"#,
            r#"{"fansub": "Lilith-Raws", "title": "Boku no Kokoro no Yabai Yatsu", "season": 1, "episode_index": 1, "extname": ".mp4"}"#,
        );
    }

    #[test]
    fn test_sakurato_media() {
        test_torrent_ep_parser(
            r#"[Sakurato] Tonikaku Kawaii S2 [03][AVC-8bit 1080p AAC][CHS].mp4"#,
            r#"{"fansub": "Sakurato", "title": "Tonikaku Kawaii", "season": 2, "episode_index": 3, "extname": ".mp4"}"#,
        )
    }

    #[test]
    fn test_lolihouse_media() {
        test_torrent_ep_parser(
            r#"[SweetSub&LoliHouse] Heavenly Delusion - 08 [WebRip 1080p HEVC-10bit AAC ASSx2].mkv"#,
            r#"{"fansub": "SweetSub&LoliHouse", "title": "Heavenly Delusion", "season": 1, "episode_index": 8, "extname": ".mkv"}"#,
        )
    }

    #[test]
    fn test_sbsub_media() {
        test_torrent_ep_parser(
            r#"[SBSUB][CONAN][1082][V2][1080P][AVC_AAC][CHS_JP](C1E4E331).mp4"#,
            r#"{"fansub": "SBSUB", "title": "CONAN", "season": 1, "episode_index": 1082, "extname": ".mp4"}"#,
        )
    }

    #[test]
    fn test_non_fansub_media() {
        test_torrent_ep_parser(
            r#"海盗战记 (2019) S04E11.mp4"#,
            r#"{"title": "海盗战记 (2019)", "season": 4, "episode_index": 11, "extname": ".mp4"}"#,
        )
    }

    #[test]
    fn test_non_fansub_media_with_dirname() {
        test_torrent_ep_parser(
            r#"海盗战记/海盗战记 S01E01.mp4"#,
            r#"{"title": "海盗战记", "season": 1, "episode_index": 1, "extname": ".mp4"}"#,
        );
    }

    #[test]
    fn test_non_fansub_tc_subtitle() {
        test_torrent_ep_parser(
            r#"海盗战记 S01E08.zh-tw.ass"#,
            r#"{"media": { "title": "海盗战记", "season": 1, "episode_index": 8, "extname": ".ass" }, "lang": "zh-tw"}"#,
        );
    }

    #[test]
    fn test_non_fansub_sc_subtitle() {
        test_torrent_ep_parser(
            r#"海盗战记 S01E01.SC.srt"#,
            r#"{ "media": { "title": "海盗战记", "season": 1, "episode_index": 1, "extname": ".srt" }, "lang": "zh" }"#,
        )
    }

    #[test]
    fn test_non_fansub_media_with_season_zero() {
        test_torrent_ep_parser(
            r#"水星的魔女(2022) S00E19.mp4"#,
            r#"{"fansub": null,"title": "水星的魔女(2022)","season": 0,"episode_index": 19,"extname": ".mp4"}"#,
        )
    }

    #[test]
    fn test_shimian_fansub_media() {
        test_torrent_ep_parser(
            r#"【失眠搬运组】放学后失眠的你-Kimi wa Houkago Insomnia - 06 [bilibili - 1080p AVC1 CHS-JP].mp4"#,
            r#"{"fansub": "失眠搬运组","title": "放学后失眠的你-Kimi wa Houkago Insomnia","season": 1,"episode_index": 6,"extname": ".mp4"}"#,
        )
    }

    pub fn test_torrent_ep_parser(raw_name: &str, expected: &str) {
        let extname = Path::new(raw_name)
            .extension()
            .map(|e| format!(".{}", e))
            .unwrap_or_default()
            .to_lowercase();

        if extname == ".srt" || extname == ".ass" {
            let expected: Option<TorrentEpisodeSubtitleMeta> = serde_json::from_str(expected).ok();
            let found_raw =
                parse_episode_subtitle_meta_from_torrent(Path::new(raw_name), None, None);
            let found = found_raw.as_ref().ok().map(|s| s.clone());

            if expected != found {
                if found_raw.is_ok() {
                    println!(
                        "expected {} and found {} are not equal",
                        serde_json::to_string_pretty(&expected).unwrap(),
                        serde_json::to_string_pretty(&found).unwrap()
                    )
                } else {
                    println!(
                        "expected {} and found {:#?} are not equal",
                        serde_json::to_string_pretty(&expected).unwrap(),
                        found_raw
                    )
                }
            }
            assert_eq!(expected, found);
        } else {
            let expected: Option<TorrentEpisodeMediaMeta> = serde_json::from_str(expected).ok();
            let found_raw = parse_episode_media_meta_from_torrent(Path::new(raw_name), None, None);
            let found = found_raw.as_ref().ok().map(|s| s.clone());

            if expected != found {
                if found_raw.is_ok() {
                    println!(
                        "expected {} and found {} are not equal",
                        serde_json::to_string_pretty(&expected).unwrap(),
                        serde_json::to_string_pretty(&found).unwrap()
                    )
                } else {
                    println!(
                        "expected {} and found {:#?} are not equal",
                        serde_json::to_string_pretty(&expected).unwrap(),
                        found_raw
                    )
                }
            }
            assert_eq!(expected, found);
        }
    }
}

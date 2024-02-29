use std::borrow::Cow;

use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

use super::defs::{DIGIT_1PLUS_REG, ZH_NUM_MAP, ZH_NUM_RE};

lazy_static! {
    static ref TITLE_RE: Regex = Regex::new(
        r#"(.*|\[.*])( -? \d+|\[\d+]|\[\d+.?[vV]\d]|第\d+[话話集]|\[第?\d+[话話集]]|\[\d+.?END]|[Ee][Pp]?\d+)(.*)"#
    ).unwrap();
    static ref RESOLUTION_RE: Regex = Regex::new(r"1080|720|2160|4K|2K").unwrap();
    static ref SOURCE_RE: Regex = Regex::new(r"B-Global|[Bb]aha|[Bb]ilibili|AT-X|Web|WebRip").unwrap();
    static ref SUB_RE: Regex = Regex::new(r"[简繁日字幕]|CH|BIG5|GB").unwrap();
    static ref PREFIX_RE: Regex =
        Regex::new(r"[^\w\s\p{Unified_Ideograph}\p{scx=Han}\p{scx=Hira}\p{scx=Kana}-]").unwrap();
    static ref EN_BRACKET_SPLIT_RE: Regex = Regex::new(r"[\[\]]").unwrap();
    static ref MAIN_TITLE_PREFIX_PROCESS_RE1: Regex = Regex::new(r"新番|月?番").unwrap();
    static ref MAIN_TITLE_PREFIX_PROCESS_RE2: Regex = Regex::new(r"[港澳台]{1,3}地区").unwrap();
    static ref SEASON_EXTRACT_SEASON_ALL_RE: Regex = Regex::new(r"S\d{1,2}|Season \d{1,2}|[第].[季期]|1st|2nd|3rd|\d{1,2}th").unwrap();
    static ref SEASON_EXTRACT_SEASON_EN_PREFIX_RE: Regex = Regex::new(r"Season|S").unwrap();
    static ref SEASON_EXTRACT_SEASON_EN_NTH_RE: Regex = Regex::new(r"1st|2nd|3rd|\d{1,2}th").unwrap();
    static ref SEASON_EXTRACT_SEASON_ZH_PREFIX_RE: Regex = Regex::new(r"[第 ].*[季期(部分)]|部分").unwrap();
    static ref SEASON_EXTRACT_SEASON_ZH_PREFIX_SUB_RE: Regex = Regex::new(r"[第季期 ]").unwrap();
    static ref NAME_EXTRACT_REMOVE_RE: Regex = Regex::new(r"[(（]仅限[港澳台]{1,3}地区[）)]").unwrap();
    static ref NAME_EXTRACT_SPLIT_RE: Regex = Regex::new(r"/|\s{2}|-\s{2}").unwrap();
    static ref NAME_JP_TEST: Regex = Regex::new(r"[\p{scx=Hira}\p{scx=Kana}]{2,}").unwrap();
    static ref NAME_ZH_TEST: Regex = Regex::new(r"[\p{scx=Han}]{2,}").unwrap();
    static ref NAME_EN_TEST: Regex = Regex::new(r"[a-zA-Z]{3,}").unwrap();
    static ref TAGS_EXTRACT_SPLIT_RE: Regex = Regex::new(r"[\[\]()（）]").unwrap();
    static ref CLEAR_SUB_RE: Regex = Regex::new(r"_MP4|_MKV").unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RawEpisodeMeta {
    name_en: Option<String>,
    name_en_no_season: Option<String>,
    name_jp: Option<String>,
    name_jp_no_season: Option<String>,
    name_zh: Option<String>,
    name_zh_no_season: Option<String>,
    season: i32,
    season_raw: Option<String>,
    episode_index: i32,
    sub: Option<String>,
    source: Option<String>,
    fansub: Option<String>,
    resolution: Option<String>,
}

fn extract_fansub(raw_name: &str) -> Option<&str> {
    let mut groups = EN_BRACKET_SPLIT_RE.splitn(raw_name, 3);
    groups.nth(1)
}

fn replace_ch_bracket_to_en(raw_name: &str) -> String {
    raw_name.replace('【', "[").replace('】', "]")
}

fn title_body_prefix_process(title_body: &str, fansub: Option<&str>) -> eyre::Result<String> {
    let raw_without_fansub = if let Some(fansub) = fansub {
        let fan_sub_re = Regex::new(&format!(".{fansub}."))?;
        fan_sub_re.replace_all(title_body, "")
    } else {
        Cow::Borrowed(title_body)
    };
    let raw_with_prefix_replaced = PREFIX_RE.replace_all(&raw_without_fansub, "/");
    let mut arg_group = raw_with_prefix_replaced
        .split('/')
        .map(|s| s.trim())
        .collect::<Vec<_>>();

    if arg_group.len() == 1 {
        arg_group = arg_group.first_mut().unwrap().split(' ').collect();
    }
    let mut raw = raw_without_fansub.to_string();
    for arg in arg_group.iter() {
        if (arg_group.len() <= 5 && MAIN_TITLE_PREFIX_PROCESS_RE1.is_match(arg))
            || (MAIN_TITLE_PREFIX_PROCESS_RE2.is_match(arg))
        {
            let sub = Regex::new(&format!(".{arg}."))?;
            raw = sub.replace_all(&raw, "").to_string();
        }
    }
    Ok(raw.to_string())
}

fn extract_season_from_title_body(title_body: &str) -> (String, Option<String>, i32) {
    let name_and_season = EN_BRACKET_SPLIT_RE.replace_all(title_body, " ");
    let seasons = SEASON_EXTRACT_SEASON_ALL_RE
        .find(&name_and_season)
        .into_iter()
        .map(|s| s.as_str())
        .collect_vec();

    if seasons.is_empty() {
        return (title_body.to_string(), None, 1);
    }

    let mut season = 1;
    let mut season_raw = None;
    let name = SEASON_EXTRACT_SEASON_ALL_RE.replace_all(&name_and_season, "");

    for s in seasons {
        season_raw = Some(s);
        if let Some(m) = SEASON_EXTRACT_SEASON_EN_PREFIX_RE.find(s) {
            if let Ok(s) = SEASON_EXTRACT_SEASON_ALL_RE
                .replace_all(m.as_str(), "")
                .parse::<i32>()
            {
                season = s;
                break;
            }
        }
        if let Some(m) = SEASON_EXTRACT_SEASON_EN_NTH_RE.find(s) {
            if let Some(s) = DIGIT_1PLUS_REG
                .find(m.as_str())
                .and_then(|s| s.as_str().parse::<i32>().ok())
            {
                season = s;
                break;
            }
        }
        if let Some(m) = SEASON_EXTRACT_SEASON_ZH_PREFIX_RE.find(s) {
            if let Ok(s) = SEASON_EXTRACT_SEASON_ZH_PREFIX_SUB_RE
                .replace(m.as_str(), "")
                .parse::<i32>()
            {
                season = s;
                break;
            }
            if let Some(m) = ZH_NUM_RE.find(m.as_str()) {
                season = ZH_NUM_MAP[m.as_str()];
                break;
            }
        }
    }

    (name.to_string(), season_raw.map(|s| s.to_string()), season)
}

fn extract_name_from_title_body_name_section(
    title_body_name_section: &str,
) -> (Option<String>, Option<String>, Option<String>) {
    let mut name_en = None;
    let mut name_zh = None;
    let mut name_jp = None;
    let replaced = NAME_EXTRACT_REMOVE_RE.replace_all(title_body_name_section, "");
    let trimed = replaced.trim();
    let mut split = NAME_EXTRACT_SPLIT_RE
        .split(trimed)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect_vec();
    if split.len() == 1 {
        let mut split_space = split[0].split(' ').collect_vec();
        let mut search_indices = vec![0];
        if split_space.len() > 1 {
            search_indices.push(search_indices.len() - 1);
        }
        for i in search_indices {
            if NAME_ZH_TEST.is_match(split_space[i]) {
                let chs = split_space[i];
                split_space.remove(i);
                split = vec![chs.to_string(), split_space.join(" ")];
                break;
            }
        }
    }
    for item in split {
        if NAME_JP_TEST.is_match(&item) && name_jp.is_none() {
            name_jp = Some(item);
        } else if NAME_ZH_TEST.is_match(&item) && name_zh.is_none() {
            name_zh = Some(item);
        } else if NAME_EN_TEST.is_match(&item) && name_en.is_none() {
            name_en = Some(item);
        }
    }
    (name_en, name_zh, name_jp)
}

fn extract_episode_index_from_title_episode(title_episode: &str) -> Option<i32> {
    DIGIT_1PLUS_REG
        .find(title_episode)?
        .as_str()
        .parse::<i32>()
        .ok()
}

fn clear_sub(sub: Option<String>) -> Option<String> {
    sub.map(|s| CLEAR_SUB_RE.replace_all(&s, "").to_string())
}

fn extract_tags_from_title_extra(
    title_extra: &str,
) -> (Option<String>, Option<String>, Option<String>) {
    let replaced = TAGS_EXTRACT_SPLIT_RE.replace_all(title_extra, " ");
    let elements = replaced
        .split(' ')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());

    let mut sub = None;
    let mut resolution = None;
    let mut source = None;
    for element in elements {
        if SUB_RE.is_match(element) {
            sub = Some(element.to_string())
        } else if RESOLUTION_RE.is_match(element) {
            resolution = Some(element.to_string())
        } else if SOURCE_RE.is_match(element) {
            source = Some(element.to_string())
        }
    }
    (clear_sub(sub), resolution, source)
}

pub fn parse_episode_meta_from_raw_name(s: &str) -> eyre::Result<RawEpisodeMeta> {
    let raw_title = s.trim();
    let raw_title_without_ch_brackets = replace_ch_bracket_to_en(raw_title);
    let fansub = extract_fansub(&raw_title_without_ch_brackets);
    if let Some(title_re_match_obj) = TITLE_RE.captures(&raw_title_without_ch_brackets) {
        let title_body = title_re_match_obj
            .get(1)
            .map(|s| s.as_str().trim())
            .unwrap_or_else(|| unreachable!("TITLE_RE has at least 3 capture groups"));
        let title_episode = title_re_match_obj
            .get(2)
            .map(|s| s.as_str().trim())
            .unwrap_or_else(|| unreachable!("TITLE_RE has at least 3 capture groups"));
        let title_extra = title_re_match_obj
            .get(3)
            .map(|s| s.as_str().trim())
            .unwrap_or_else(|| unreachable!("TITLE_RE has at least 3 capture groups"));
        let title_body = title_body_prefix_process(title_body, fansub)?;
        let (name_without_season, season_raw, season) = extract_season_from_title_body(&title_body);
        let (name_en, name_zh, name_jp) = extract_name_from_title_body_name_section(&title_body);
        let (name_en_no_season, name_zh_no_season, name_jp_no_season) =
            extract_name_from_title_body_name_section(&name_without_season);
        let episode_index = extract_episode_index_from_title_episode(title_episode).unwrap_or(0);
        let (sub, resolution, source) = extract_tags_from_title_extra(title_extra);
        Ok(RawEpisodeMeta {
            name_en,
            name_en_no_season,
            name_jp,
            name_jp_no_season,
            name_zh,
            name_zh_no_season,
            season,
            season_raw,
            episode_index,
            sub,
            source,
            fansub: fansub.map(|s| s.to_string()),
            resolution,
        })
    } else {
        Err(eyre::eyre!("Can not parse episode meta from raw filename"))
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_episode_meta_from_raw_name, RawEpisodeMeta};

    struct TestCase {
        source: &'static str,
        expected: &'static str,
    }

    #[test]
    fn test_parse_episode_meta_from_raw_name() {
        let test_cases = vec![
            TestCase {
                // ep+version case
                source: r#"[LoliHouse] 因为不是真正的伙伴而被逐出勇者队伍，流落到边境展开慢活人生 2nd / Shin no Nakama 2nd - 08v2 [WebRip 1080p HEVC-10bit AAC][简繁内封字幕]"#,
                expected: r#"{
                    "name_en": "Shin no Nakama 2nd",
                    "name_en_no_season": "Shin no Nakama",
                    "name_zh": "因为不是真正的伙伴而被逐出勇者队伍，流落到边境展开慢活人生 2nd",
                    "name_zh_no_season": "因为不是真正的伙伴而被逐出勇者队伍，流落到边境展开慢活人生",
                    "season": 2,
                    "season_raw": "2nd",
                    "episode_index": 8,
                    "sub": "简繁内封字幕",
                    "source": "WebRip",
                    "fansub": "LoliHouse",
                    "resolution": "1080p"
                  }"#,
            },
            TestCase {
                // pure english title case
                source: r"[动漫国字幕组&LoliHouse] THE MARGINAL SERVICE - 08 [WebRip 1080p HEVC-10bit AAC][简繁内封字幕]",
                expected: r#"{
                "name_en": "THE MARGINAL SERVICE",
                "name_en_no_season": "THE MARGINAL SERVICE",
                "season": 1,
                "episode_index": 8,
                "sub": "简繁内封字幕",
                "source": "WebRip",
                "fansub": "动漫国字幕组&LoliHouse",
                "resolution": "1080p"
              }"#,
            },
            TestCase {
                // two zh titles case
                source: r#"[LoliHouse] 事与愿违的不死冒险者 / 非自愿的不死冒险者 / Nozomanu Fushi no Boukensha - 01 [WebRip 1080p HEVC-10bit AAC][简繁内封字幕]"#,
                expected: r#"{
                    "name_en": "Nozomanu Fushi no Boukensha",
                    "name_en_no_season": "Nozomanu Fushi no Boukensha",
                    "name_zh": "事与愿违的不死冒险者",
                    "name_zh_no_season": "事与愿违的不死冒险者",
                    "season": 1,
                    "season_raw": null,
                    "episode_index": 1,
                    "sub": "简繁内封字幕",
                    "source": "WebRip",
                    "fansub": "LoliHouse",
                    "resolution": "1080p"
                  }"#,
            },
            TestCase {
                // en+zh+jp case
                source: r#"[喵萌奶茶屋&LoliHouse] 碰之道 / ぽんのみち / Pon no Michi - 07 [WebRip 1080p HEVC-10bit AAC][简繁日内封字幕]"#,
                expected: r#"{
                    "name_en": "Pon no Michi",
                    "name_jp": "ぽんのみち",
                    "name_zh": "碰之道",
                    "name_en_no_season": "Pon no Michi",
                    "name_jp_no_season": "ぽんのみち",
                    "name_zh_no_season": "碰之道",
                    "season": 1,
                    "season_raw": null,
                    "episode_index": 7,
                    "sub": "简繁日内封字幕",
                    "source": "WebRip",
                    "fansub": "喵萌奶茶屋&LoliHouse",
                    "resolution": "1080p"
                }"#,
            },
            TestCase {
                // season nth case
                source: r#"[ANi] Yowai Character Tomozakikun /  弱角友崎同学 2nd STAGE - 09 [1080P][Baha][WEB-DL][AAC AVC][CHT][MP4]"#,
                expected: r#"{
                    "name_en": "Yowai Character Tomozakikun",
                    "name_en_no_season": "Yowai Character Tomozakikun",
                    "name_zh": "弱角友崎同学 2nd STAGE",
                    "name_zh_no_season": "弱角友崎同学",
                    "season": 2,
                    "season_raw": "2nd",
                    "episode_index": 9,
                    "sub": "CHT",
                    "source": "Baha",
                    "fansub": "ANi",
                    "resolution": "1080P"
                }"#,
            },
            TestCase {
                // season en + season zh case
                source: r#"[豌豆字幕组&LoliHouse] 王者天下 第五季 / Kingdom S5 - 07 [WebRip 1080p HEVC-10bit AAC][简繁外挂字幕]"#,
                expected: r#"{
                    "name_en": "Kingdom S5",
                    "name_en_no_season": "Kingdom",
                    "name_zh": "王者天下 第五季",
                    "name_zh_no_season": "王者天下",
                    "season": 5,
                    "season_raw": "第五季",
                    "episode_index": 7,
                    "sub": "简繁外挂字幕",
                    "source": "WebRip",
                    "fansub": "豌豆字幕组&LoliHouse",
                    "resolution": "1080p"
                }"#,
            },
        ];

        for case in test_cases {
            let expected: Option<RawEpisodeMeta> = serde_json::from_str(case.expected).unwrap();
            let found = parse_episode_meta_from_raw_name(case.source).ok();

            if expected != found {
                println!(
                    "expected {} and found {} are not equal",
                    serde_json::to_string_pretty(&expected).unwrap(),
                    serde_json::to_string_pretty(&found).unwrap()
                )
            }
            assert_eq!(expected, found);
        }
    }
}

/**
 * @TODO: rewrite with nom
 */
use std::borrow::Cow;

use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use snafu::whatever;

use crate::{
    errors::RecorderResult,
    extract::defs::{DIGIT_1PLUS_REG, ZH_NUM_MAP, ZH_NUM_RE},
};

const NAME_EXTRACT_REPLACE_ADHOC1_REPLACED: &str = "$1/$2";

lazy_static! {
    static ref TITLE_RE: Regex = Regex::new(
        r#"(.*|\[.*])( -? \d+|\[\d+]|\[\d+.?[vV]\d]|第\d+[话話集]|\[第?\d+[话話集]]|\[\d+.?END]|[Ee][Pp]?\d+|\[\s*\d+\s*[\-\~]\s*\d+\s*\p{scx=Han}*[话話集]\s*])(.*)"#
    ).unwrap();
    static ref EP_COLLECTION_RE:Regex = Regex::new(r#"\[?\s*\d+\s*[\-\~]\s*\d+\s*\p{scx=Han}*合?[话話集]\s*]?"#).unwrap();
    static ref MOVIE_TITLE_RE:Regex = Regex::new(r#"(.*|\[.*])(剧场版|[Mm]ovie|电影)(.*?)$"#).unwrap();
    static ref RESOLUTION_RE: Regex = Regex::new(r"1080|720|2160|4K|2K").unwrap();
    static ref SOURCE_L1_RE: Regex = Regex::new(r"B-Global|[Bb]aha|[Bb]ilibili|AT-X|W[Ee][Bb][Rr][Ii][Pp]|Sentai|B[Dd][Rr][Ii][Pp]|UHD[Rr][Ii][Pp]|NETFLIX").unwrap();
    static ref SOURCE_L2_RE: Regex = Regex::new(r"AMZ|CR|W[Ee][Bb]|B[Dd]").unwrap();
    static ref SUB_RE: Regex = Regex::new(r"[简繁日字幕]|CH|BIG5|GB").unwrap();
    static ref PREFIX_RE: Regex =
        Regex::new(r"[^\w\s\p{Unified_Ideograph}\p{scx=Han}\p{scx=Hira}\p{scx=Kana}-]").unwrap();
    static ref EN_BRACKET_SPLIT_RE: Regex = Regex::new(r"[\[\]]").unwrap();
    static ref MOVIE_SEASON_EXTRACT_RE: Regex = Regex::new(r"剧场版|Movie|电影").unwrap();
    static ref MAIN_TITLE_PREFIX_PROCESS_RE1: Regex = Regex::new(r"新番|月?番").unwrap();
    static ref MAIN_TITLE_PREFIX_PROCESS_RE2: Regex = Regex::new(r"[港澳台]{1,3}地区").unwrap();
    static ref MAIN_TITLE_PRE_PROCESS_BACKETS_RE: Regex = Regex::new(r"\[.+\]").unwrap();
    static ref MAIN_TITLE_PRE_PROCESS_BACKETS_RE_SUB1: Regex = Regex::new(r"^.*?\[").unwrap();
    static ref SEASON_EXTRACT_SEASON_ALL_RE: Regex = Regex::new(r"S\d{1,2}|Season \d{1,2}|[第].[季期]|1st|2nd|3rd|\d{1,2}th").unwrap();
    static ref SEASON_EXTRACT_SEASON_EN_PREFIX_RE: Regex = Regex::new(r"Season|S").unwrap();
    static ref SEASON_EXTRACT_SEASON_EN_NTH_RE: Regex = Regex::new(r"1st|2nd|3rd|\d{1,2}th").unwrap();
    static ref SEASON_EXTRACT_SEASON_ZH_PREFIX_RE: Regex = Regex::new(r"[第 ].*[季期(部分)]|部分").unwrap();
    static ref SEASON_EXTRACT_SEASON_ZH_PREFIX_SUB_RE: Regex = Regex::new(r"[第季期 ]").unwrap();
    static ref NAME_EXTRACT_REMOVE_RE: Regex = Regex::new(r"[(（]仅限[港澳台]{1,3}地区[）)]").unwrap();
    static ref NAME_EXTRACT_SPLIT_RE: Regex = Regex::new(r"/|\s{2}|-\s{2}|\]\[").unwrap();
    static ref NAME_EXTRACT_REPLACE_ADHOC1_RE: Regex = Regex::new(r"([\p{scx=Han}\s\(\)]{5,})_([a-zA-Z]{2,})").unwrap();
    static ref NAME_JP_TEST: Regex = Regex::new(r"[\p{scx=Hira}\p{scx=Kana}]{2,}").unwrap();
    static ref NAME_ZH_TEST: Regex = Regex::new(r"[\p{scx=Han}]{2,}").unwrap();
    static ref NAME_EN_TEST: Regex = Regex::new(r"[a-zA-Z]{3,}").unwrap();
    static ref TAGS_EXTRACT_SPLIT_RE: Regex = Regex::new(r"[\[\]()（）_]").unwrap();
    static ref CLEAR_SUB_RE: Regex = Regex::new(r"_MP4|_MKV").unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RawEpisodeMeta {
    pub name_en: Option<String>,
    pub name_en_no_season: Option<String>,
    pub name_jp: Option<String>,
    pub name_jp_no_season: Option<String>,
    pub name_zh: Option<String>,
    pub name_zh_no_season: Option<String>,
    pub season: i32,
    pub season_raw: Option<String>,
    pub episode_index: i32,
    pub subtitle: Option<String>,
    pub source: Option<String>,
    pub fansub: Option<String>,
    pub resolution: Option<String>,
}

fn extract_fansub(raw_name: &str) -> Option<&str> {
    let mut groups = EN_BRACKET_SPLIT_RE.splitn(raw_name, 3);
    groups.nth(1)
}

fn replace_ch_bracket_to_en(raw_name: &str) -> String {
    raw_name.replace('【', "[").replace('】', "]")
}

fn title_body_pre_process(title_body: &str, fansub: Option<&str>) -> RecorderResult<String> {
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
        .filter(|s| !s.is_empty())
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
    if let Some(m) = MAIN_TITLE_PRE_PROCESS_BACKETS_RE.find(&raw)
        && m.len() as f32 > (raw.len() as f32) * 0.5
    {
        let mut raw1 = MAIN_TITLE_PRE_PROCESS_BACKETS_RE_SUB1
            .replace(&raw, "")
            .chars()
            .collect_vec();
        while let Some(ch) = raw1.pop() {
            if ch == ']' {
                break;
            }
        }
        raw = raw1.into_iter().collect();
    }
    Ok(raw.to_string())
}

pub fn extract_season_from_title_body(title_body: &str) -> (String, Option<String>, i32) {
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
        if let Some(m) = SEASON_EXTRACT_SEASON_EN_PREFIX_RE.find(s)
            && let Ok(s) = SEASON_EXTRACT_SEASON_ALL_RE
                .replace_all(m.as_str(), "")
                .parse::<i32>()
        {
            season = s;
            break;
        }
        if let Some(m) = SEASON_EXTRACT_SEASON_EN_NTH_RE.find(s)
            && let Some(s) = DIGIT_1PLUS_REG
                .find(m.as_str())
                .and_then(|s| s.as_str().parse::<i32>().ok())
        {
            season = s;
            break;
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
    let replaced1 = NAME_EXTRACT_REMOVE_RE.replace_all(title_body_name_section, "");
    let replaced2 = NAME_EXTRACT_REPLACE_ADHOC1_RE
        .replace_all(&replaced1, NAME_EXTRACT_REPLACE_ADHOC1_REPLACED);
    let trimmed = replaced2.trim();
    let mut split = NAME_EXTRACT_SPLIT_RE
        .split(trimmed)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect_vec();
    if split.len() == 1 {
        let mut split_space = split[0].split(' ').collect_vec();
        let mut search_indices = vec![0];
        if split_space.len() > 1 {
            search_indices.push(split_space.len() - 1);
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
        .filter(|s| !s.is_empty())
        .collect_vec();

    let mut sub = None;
    let mut resolution = None;
    let mut source = None;
    for element in elements.iter() {
        if SUB_RE.is_match(element) {
            sub = Some(element.to_string())
        } else if RESOLUTION_RE.is_match(element) {
            resolution = Some(element.to_string())
        } else if SOURCE_L1_RE.is_match(element) {
            source = Some(element.to_string())
        }
    }
    if source.is_none() {
        for element in elements {
            if SOURCE_L2_RE.is_match(element) {
                source = Some(element.to_string())
            }
        }
    }
    (clear_sub(sub), resolution, source)
}

pub fn check_is_movie(title: &str) -> bool {
    MOVIE_TITLE_RE.is_match(title)
}

pub fn parse_episode_meta_from_raw_name(s: &str) -> RecorderResult<RawEpisodeMeta> {
    let raw_title = s.trim();
    let raw_title_without_ch_brackets = replace_ch_bracket_to_en(raw_title);
    let fansub = extract_fansub(&raw_title_without_ch_brackets);
    let movie_capture = check_is_movie(&raw_title_without_ch_brackets);
    if let Some(title_re_match_obj) = MOVIE_TITLE_RE
        .captures(&raw_title_without_ch_brackets)
        .or(TITLE_RE.captures(&raw_title_without_ch_brackets))
    {
        let mut title_body = title_re_match_obj
            .get(1)
            .map(|s| s.as_str().trim())
            .unwrap_or_else(|| unreachable!("TITLE_RE has at least 3 capture groups"))
            .to_string();
        let mut title_episode = title_re_match_obj
            .get(2)
            .map(|s| s.as_str().trim())
            .unwrap_or_else(|| unreachable!("TITLE_RE has at least 3 capture groups"));
        let title_extra = title_re_match_obj
            .get(3)
            .map(|s| s.as_str().trim())
            .unwrap_or_else(|| unreachable!("TITLE_RE has at least 3 capture groups"));

        if movie_capture {
            title_body += title_episode;
            title_episode = "";
        } else if EP_COLLECTION_RE.is_match(title_episode) {
            title_episode = "";
        }

        let title_body = title_body_pre_process(&title_body, fansub)?;
        let (name_without_season, season_raw, season) = extract_season_from_title_body(&title_body);
        let (name_en, name_zh, name_jp) = extract_name_from_title_body_name_section(&title_body);
        let (name_en_no_season, name_zh_no_season, name_jp_no_season) =
            extract_name_from_title_body_name_section(&name_without_season);
        let episode_index = extract_episode_index_from_title_episode(title_episode).unwrap_or(1);
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
            subtitle: sub,
            source,
            fansub: fansub.map(|s| s.to_string()),
            resolution,
        })
    } else {
        whatever!("Can not parse episode meta from raw filename {}", raw_title)
    }
}

#[cfg(test)]
mod tests {

    use super::{RawEpisodeMeta, parse_episode_meta_from_raw_name};

    fn test_raw_ep_parser_case(raw_name: &str, expected: &str) {
        let expected: Option<RawEpisodeMeta> = serde_json::from_str(expected).unwrap_or_default();
        let found = parse_episode_meta_from_raw_name(raw_name).ok();

        if expected != found {
            println!(
                "expected {} and found {} are not equal",
                serde_json::to_string_pretty(&expected).unwrap(),
                serde_json::to_string_pretty(&found).unwrap()
            )
        }
        assert_eq!(expected, found);
    }

    #[test]
    fn test_parse_ep_with_all_parts_wrapped() {
        test_raw_ep_parser_case(
            r#"[新Sub][1月新番][我心里危险的东西 第二季][05][HEVC][10Bit][1080P][简日双语][招募翻译]"#,
            r#"{
                  "name_zh": "我心里危险的东西",
                  "name_zh_no_season": "我心里危险的东西",
                  "season": 2,
                  "season_raw": "第二季",
                  "episode_index": 5,
                  "subtitle": "简日双语",
                  "source": null,
                  "fansub": "新Sub",
                  "resolution": "1080P"
                }"#,
        )
    }

    #[test]
    fn test_parse_ep_with_title_wrapped_by_one_square_bracket_and_season_prefix() {
        test_raw_ep_parser_case(
            r#"【喵萌奶茶屋】★01月新番★[我内心的糟糕念头 / Boku no Kokoro no Yabai Yatsu][18][1080p][简日双语][招募翻译]"#,
            r#"{
                  "name_en": "Boku no Kokoro no Yabai Yatsu",
                  "name_en_no_season": "Boku no Kokoro no Yabai Yatsu",
                  "name_zh": "我内心的糟糕念头",
                  "name_zh_no_season": "我内心的糟糕念头",
                  "season": 1,
                  "season_raw": null,
                  "episode_index": 18,
                  "subtitle": "简日双语",
                  "source": null,
                  "fansub": "喵萌奶茶屋",
                  "resolution": "1080p"
                }"#,
        );
    }

    #[test]
    fn test_parse_ep_with_ep_and_version() {
        test_raw_ep_parser_case(
            r#"[LoliHouse] 因为不是真正的伙伴而被逐出勇者队伍，流落到边境展开慢活人生 2nd / Shin no Nakama 2nd - 08v2 [WebRip 1080p HEVC-10bit AAC][简繁内封字幕]"#,
            r#"{
                    "name_en": "Shin no Nakama 2nd",
                    "name_en_no_season": "Shin no Nakama",
                    "name_zh": "因为不是真正的伙伴而被逐出勇者队伍，流落到边境展开慢活人生 2nd",
                    "name_zh_no_season": "因为不是真正的伙伴而被逐出勇者队伍，流落到边境展开慢活人生",
                    "season": 2,
                    "season_raw": "2nd",
                    "episode_index": 8,
                    "subtitle": "简繁内封字幕",
                    "source": "WebRip",
                    "fansub": "LoliHouse",
                    "resolution": "1080p"
                  }"#,
        )
    }

    #[test]
    fn test_parse_ep_with_en_title_only() {
        test_raw_ep_parser_case(
            r"[动漫国字幕组&LoliHouse] THE MARGINAL SERVICE - 08 [WebRip 1080p HEVC-10bit AAC][简繁内封字幕]",
            r#"{
                "name_en": "THE MARGINAL SERVICE",
                "name_en_no_season": "THE MARGINAL SERVICE",
                "season": 1,
                "episode_index": 8,
                "subtitle": "简繁内封字幕",
                "source": "WebRip",
                "fansub": "动漫国字幕组&LoliHouse",
                "resolution": "1080p"
              }"#,
        )
    }

    #[test]
    fn test_parse_ep_with_two_zh_title() {
        test_raw_ep_parser_case(
            r#"[LoliHouse] 事与愿违的不死冒险者 / 非自愿的不死冒险者 / Nozomanu Fushi no Boukensha - 01 [WebRip 1080p HEVC-10bit AAC][简繁内封字幕]"#,
            r#"{
                    "name_en": "Nozomanu Fushi no Boukensha",
                    "name_en_no_season": "Nozomanu Fushi no Boukensha",
                    "name_zh": "事与愿违的不死冒险者",
                    "name_zh_no_season": "事与愿违的不死冒险者",
                    "season": 1,
                    "season_raw": null,
                    "episode_index": 1,
                    "subtitle": "简繁内封字幕",
                    "source": "WebRip",
                    "fansub": "LoliHouse",
                    "resolution": "1080p"
                  }"#,
        )
    }

    #[test]
    fn test_parse_ep_with_en_zh_jp_titles() {
        test_raw_ep_parser_case(
            r#"[喵萌奶茶屋&LoliHouse] 碰之道 / ぽんのみち / Pon no Michi - 07 [WebRip 1080p HEVC-10bit AAC][简繁日内封字幕]"#,
            r#"{
                    "name_en": "Pon no Michi",
                    "name_jp": "ぽんのみち",
                    "name_zh": "碰之道",
                    "name_en_no_season": "Pon no Michi",
                    "name_jp_no_season": "ぽんのみち",
                    "name_zh_no_season": "碰之道",
                    "season": 1,
                    "season_raw": null,
                    "episode_index": 7,
                    "subtitle": "简繁日内封字幕",
                    "source": "WebRip",
                    "fansub": "喵萌奶茶屋&LoliHouse",
                    "resolution": "1080p"
                }"#,
        )
    }

    #[test]
    fn test_parse_ep_with_nth_season() {
        test_raw_ep_parser_case(
            r#"[ANi] Yowai Character Tomozakikun /  弱角友崎同学 2nd STAGE - 09 [1080P][Baha][WEB-DL][AAC AVC][CHT][MP4]"#,
            r#"{
                    "name_en": "Yowai Character Tomozakikun",
                    "name_en_no_season": "Yowai Character Tomozakikun",
                    "name_zh": "弱角友崎同学 2nd STAGE",
                    "name_zh_no_season": "弱角友崎同学",
                    "season": 2,
                    "season_raw": "2nd",
                    "episode_index": 9,
                    "subtitle": "CHT",
                    "source": "Baha",
                    "fansub": "ANi",
                    "resolution": "1080P"
                }"#,
        )
    }

    #[test]
    fn test_parse_ep_with_season_en_and_season_zh() {
        test_raw_ep_parser_case(
            r#"[豌豆字幕组&LoliHouse] 王者天下 第五季 / Kingdom S5 - 07 [WebRip 1080p HEVC-10bit AAC][简繁外挂字幕]"#,
            r#"{
                    "name_en": "Kingdom S5",
                    "name_en_no_season": "Kingdom",
                    "name_zh": "王者天下 第五季",
                    "name_zh_no_season": "王者天下",
                    "season": 5,
                    "season_raw": "第五季",
                    "episode_index": 7,
                    "subtitle": "简繁外挂字幕",
                    "source": "WebRip",
                    "fansub": "豌豆字幕组&LoliHouse",
                    "resolution": "1080p"
                }"#,
        )
    }

    #[test]
    fn test_parse_ep_with_airota_fansub_style_case1() {
        test_raw_ep_parser_case(
            r#"【千夏字幕组】【爱丽丝与特蕾丝的虚幻工厂_Alice to Therese no Maboroshi Koujou】[剧场版][WebRip_1080p_HEVC][简繁内封][招募新人]"#,
            r#"{
                  "name_en": "Alice to Therese no Maboroshi Koujou",
                  "name_en_no_season": "Alice to Therese no Maboroshi Koujou",
                  "name_zh": "爱丽丝与特蕾丝的虚幻工厂",
                  "name_zh_no_season": "爱丽丝与特蕾丝的虚幻工厂",
                  "season": 1,
                  "episode_index": 1,
                  "subtitle": "简繁内封",
                  "source": "WebRip",
                  "fansub": "千夏字幕组",
                  "resolution": "1080p"
                }"#,
        )
    }

    #[test]
    fn test_parse_ep_with_airota_fansub_style_case2() {
        test_raw_ep_parser_case(
            r#"[千夏字幕组&喵萌奶茶屋][电影 轻旅轻营 (摇曳露营) _Yuru Camp Movie][剧场版][UHDRip_2160p_HEVC][繁体][千夏15周年]"#,
            r#"{
                      "name_en": "Yuru Camp Movie",
                      "name_en_no_season": "Yuru Camp Movie",
                      "name_zh": "电影 轻旅轻营 (摇曳露营)",
                      "name_zh_no_season": "电影 轻旅轻营 (摇曳露营)",
                      "season": 1,
                      "episode_index": 1,
                      "subtitle": "繁体",
                      "source": "UHDRip",
                      "fansub": "千夏字幕组&喵萌奶茶屋",
                      "resolution": "2160p"
                }"#,
        )
    }

    #[test]
    fn test_parse_ep_with_large_episode_style() {
        test_raw_ep_parser_case(
            r#"[梦蓝字幕组]New Doraemon 哆啦A梦新番[747][2023.02.25][AVC][1080P][GB_JP][MP4]"#,
            r#"{
                      "name_en": "New Doraemon",
                      "name_en_no_season": "New Doraemon",
                      "name_zh": "哆啦A梦新番",
                      "name_zh_no_season": "哆啦A梦新番",
                      "season": 1,
                      "episode_index": 747,
                      "subtitle": "GB",
                      "fansub": "梦蓝字幕组",
                      "resolution": "1080P"
                    }"#,
        )
    }

    #[test]
    fn test_parse_ep_with_many_square_brackets_split_title() {
        test_raw_ep_parser_case(
            r#"【MCE汉化组】[剧场版-摇曳露营][Yuru Camp][Movie][简日双语][1080P][x264 AAC]"#,
            r#"{
                  "name_en": "Yuru Camp",
                  "name_en_no_season": "Yuru Camp",
                  "name_zh": "剧场版-摇曳露营",
                  "name_zh_no_season": "剧场版-摇曳露营",
                  "season": 1,
                  "episode_index": 1,
                  "subtitle": "简日双语",
                  "fansub": "MCE汉化组",
                  "resolution": "1080P"
                }"#,
        )
    }

    #[test]
    fn test_parse_ep_with_implicit_lang_title_sep() {
        test_raw_ep_parser_case(
            r#"[织梦字幕组][尼尔：机械纪元 NieR Automata Ver1.1a][02集][1080P][AVC][简日双语]"#,
            r#"{
                      "name_en": "NieR Automata Ver1.1a",
                      "name_en_no_season": "NieR Automata Ver1.1a",
                      "name_zh": "尼尔：机械纪元",
                      "name_zh_no_season": "尼尔：机械纪元",
                      "season": 1,
                      "episode_index": 2,
                      "subtitle": "简日双语",
                      "fansub": "织梦字幕组",
                      "resolution": "1080P"
                    }"#,
        )
    }

    #[test]
    fn test_parse_ep_with_square_brackets_wrapped_and_space_split() {
        test_raw_ep_parser_case(
            r#"[天月搬运组][迷宫饭 Delicious in Dungeon][03][日语中字][MKV][1080P][NETFLIX][高画质版]"#,
            r#"
                {
                  "name_en": "Delicious in Dungeon",
                  "name_en_no_season": "Delicious in Dungeon",
                  "name_zh": "迷宫饭",
                  "name_zh_no_season": "迷宫饭",
                  "season": 1,
                  "episode_index": 3,
                  "subtitle": "日语中字",
                  "source": "NETFLIX",
                  "fansub": "天月搬运组",
                  "resolution": "1080P"
                }
                "#,
        )
    }

    #[test]
    fn test_parse_ep_with_start_with_brackets_wrapped_season_info_prefix() {
        test_raw_ep_parser_case(
            r#"[爱恋字幕社][1月新番][迷宫饭][Dungeon Meshi][01][1080P][MP4][简日双语] "#,
            r#"{
                  "name_en": "Dungeon Meshi",
                  "name_en_no_season": "Dungeon Meshi",
                  "name_zh": "迷宫饭",
                  "name_zh_no_season": "迷宫饭",
                  "season": 1,
                  "episode_index": 1,
                  "subtitle": "简日双语",
                  "fansub": "爱恋字幕社",
                  "resolution": "1080P"
                }"#,
        )
    }

    #[test]
    fn test_parse_ep_with_small_no_title_extra_brackets_case() {
        test_raw_ep_parser_case(
            r#"[ANi] Mahou Shoujo ni Akogarete / 梦想成为魔法少女 [年龄限制版] - 09 [1080P][Baha][WEB-DL][AAC AVC][CHT][MP4]"#,
            r#"{
                  "name_en": "Mahou Shoujo ni Akogarete",
                  "name_en_no_season": "Mahou Shoujo ni Akogarete",
                  "name_zh": "梦想成为魔法少女 [年龄限制版]",
                  "name_zh_no_season": "梦想成为魔法少女 [年龄限制版]",
                  "season": 1,
                  "episode_index": 9,
                  "subtitle": "CHT",
                  "source": "Baha",
                  "fansub": "ANi",
                  "resolution": "1080P"
                }"#,
        )
    }

    #[test]
    fn test_parse_ep_title_leading_space_style() {
        test_raw_ep_parser_case(
            r#"[ANi]  16bit 的感动 ANOTHER LAYER - 01 [1080P][Baha][WEB-DL][AAC AVC][CHT][MP4]"#,
            r#"{
                      "name_zh": "16bit 的感动 ANOTHER LAYER",
                      "name_zh_no_season": "16bit 的感动 ANOTHER LAYER",
                      "season": 1,
                      "season_raw": null,
                      "episode_index": 1,
                      "subtitle": "CHT",
                      "source": "Baha",
                      "fansub": "ANi",
                      "resolution": "1080P"
                    }"#,
        )
    }

    #[test]
    fn test_parse_ep_title_leading_month_and_wrapped_brackets_style() {
        test_raw_ep_parser_case(
            r#"【喵萌奶茶屋】★07月新番★[银砂糖师与黑妖精 ~ Sugar Apple Fairy Tale ~][13][1080p][简日双语][招募翻译]"#,
            r#"{
                          "name_en": "~ Sugar Apple Fairy Tale ~",
                          "name_en_no_season": "~ Sugar Apple Fairy Tale ~",
                          "name_zh": "银砂糖师与黑妖精",
                          "name_zh_no_season": "银砂糖师与黑妖精",
                          "season": 1,
                          "episode_index": 13,
                          "subtitle": "简日双语",
                          "fansub": "喵萌奶茶屋",
                          "resolution": "1080p"
                        }"#,
        )
    }

    #[test]
    fn test_parse_ep_title_leading_month_style() {
        test_raw_ep_parser_case(
            r#"【极影字幕社】★4月新番 天国大魔境 Tengoku Daimakyou 第05话 GB 720P MP4（字幕社招人内详）"#,
            r#"{
                      "name_en": "Tengoku Daimakyou",
                      "name_en_no_season": "Tengoku Daimakyou",
                      "name_zh": "天国大魔境",
                      "name_zh_no_season": "天国大魔境",
                      "season": 1,
                      "episode_index": 5,
                      "subtitle": "字幕社招人内详",
                      "source": null,
                      "fansub": "极影字幕社",
                      "resolution": "720P"
                    }"#,
        )
    }

    #[test]
    fn test_parse_ep_tokusatsu_style() {
        test_raw_ep_parser_case(
            r#"[MagicStar] 假面骑士Geats / 仮面ライダーギーツ EP33 [WEBDL] [1080p] [TTFC]【生】"#,
            r#"{
              "name_jp": "仮面ライダーギーツ",
              "name_jp_no_season": "仮面ライダーギーツ",
              "name_zh": "假面骑士Geats",
              "name_zh_no_season": "假面骑士Geats",
              "season": 1,
              "episode_index": 33,
              "source": "WEBDL",
              "fansub": "MagicStar",
              "resolution": "1080p"
            }"#,
        )
    }

    #[test]
    fn test_parse_ep_with_multi_lang_zh_title() {
        test_raw_ep_parser_case(
            r#"[百冬练习组&LoliHouse] BanG Dream! 少女乐团派对！☆PICO FEVER！ / Garupa Pico: Fever! - 26 [WebRip 1080p HEVC-10bit AAC][简繁内封字幕][END] [101.69 MB]"#,
            r#"{
                      "name_en": "Garupa Pico: Fever!",
                      "name_en_no_season": "Garupa Pico: Fever!",
                      "name_zh": "BanG Dream! 少女乐团派对！☆PICO FEVER！",
                      "name_zh_no_season": "BanG Dream! 少女乐团派对！☆PICO FEVER！",
                      "season": 1,
                      "episode_index": 26,
                      "subtitle": "简繁内封字幕",
                      "source": "WebRip",
                      "fansub": "百冬练习组&LoliHouse",
                      "resolution": "1080p"
                    }"#,
        )
    }

    #[test]
    fn test_ep_collections() {
        test_raw_ep_parser_case(
            r#"[奶²&LoliHouse] 蘑菇狗 / Kinokoinu: Mushroom Pup [01-12 精校合集][WebRip 1080p HEVC-10bit AAC][简日内封字幕]"#,
            r#"{
                "name_en": "Kinokoinu: Mushroom Pup",
                "name_en_no_season": "Kinokoinu: Mushroom Pup",
                "name_zh": "蘑菇狗",
                "name_zh_no_season": "蘑菇狗",
                "season": 1,
                "episode_index": 1,
                "subtitle": "简日内封字幕",
                "source": "WebRip",
                "fansub": "奶²&LoliHouse",
                "resolution": "1080p",
                 "name": " 蘑菇狗 / Kinokoinu: Mushroom Pup [01-12 精校合集]"
            }"#,
        );

        test_raw_ep_parser_case(
            r#"[LoliHouse] 叹气的亡灵想隐退 / Nageki no Bourei wa Intai shitai [01-13 合集][WebRip 1080p HEVC-10bit AAC][简繁内封字幕][Fin]"#,
            r#"{
                "name_en": "Nageki no Bourei wa Intai shitai",
                "name_en_no_season": "Nageki no Bourei wa Intai shitai",
                "name_jp": null,
                "name_jp_no_season": null,
                "name_zh": "叹气的亡灵想隐退",
                "name_zh_no_season": "叹气的亡灵想隐退",
                "season": 1,
                "season_raw": null,
                "episode_index": 1,
                "subtitle": "简繁内封字幕",
                "source": "WebRip",
                "fansub": "LoliHouse",
                "resolution": "1080p"
            }"#,
        );

        test_raw_ep_parser_case(
            r#"[LoliHouse] 精灵幻想记 第二季 / Seirei Gensouki S2 [01-12 合集][WebRip 1080p HEVC-10bit AAC][简繁内封字幕][Fin]"#,
            r#"{
                "name_en": "Seirei Gensouki S2",
                "name_en_no_season": "Seirei Gensouki",
                "name_zh": "精灵幻想记 第二季",
                "name_zh_no_season": "精灵幻想记",
                "season": 2,
                "season_raw": "第二季",
                "episode_index": 1,
                "subtitle": "简繁内封字幕",
                "source": "WebRip",
                "fansub": "LoliHouse",
                "resolution": "1080p"
            }"#,
        );

        test_raw_ep_parser_case(
            r#"[喵萌奶茶屋&LoliHouse] 超自然武装当哒当 / 胆大党 / Dandadan [01-12 精校合集][WebRip 1080p HEVC-10bit AAC][简繁日内封字幕][Fin]"#,
            r#" {
                "name_en": "Dandadan",
                "name_en_no_season": "Dandadan",
                "name_zh": "超自然武装当哒当",
                "name_zh_no_season": "超自然武装当哒当",
                "season": 1,
                "episode_index": 1,
                "subtitle": "简繁日内封字幕",
                "source": "WebRip",
                "fansub": "喵萌奶茶屋&LoliHouse",
                "resolution": "1080p"
            }"#,
        );
    }

    // TODO: FIXME
    #[test]
    fn test_bad_cases() {
        test_raw_ep_parser_case(
            r#"[7³ACG x 桜都字幕组] 摇曳露营△ 剧场版/映画 ゆるキャン△/Eiga Yuru Camp△ [简繁字幕] BDrip 1080p x265 FLAC 2.0"#,
            r#"{
                  "name_zh": "摇曳露营△剧场版",
                  "name_zh_no_season": "摇曳露营△剧场版",
                  "season": 1,
                  "season_raw": null,
                  "episode_index": 1,
                  "subtitle": "简繁字幕",
                  "source": "BDrip",
                  "fansub": "7³ACG x 桜都字幕组",
                  "resolution": "1080p"
                }"#,
        );

        test_raw_ep_parser_case(
            r#"【幻樱字幕组】【4月新番】【古见同学有交流障碍症 第二季 Komi-san wa, Komyushou Desu. S02】【22】【GB_MP4】【1920X1080】"#,
            r#"{
                      "name_en": "第二季 Komi-san wa, Komyushou Desu. S02",
                      "name_en_no_season": "Komi-san wa, Komyushou Desu.",
                      "name_zh": "古见同学有交流障碍症",
                      "name_zh_no_season": "古见同学有交流障碍症",
                      "season": 2,
                      "season_raw": "第二季",
                      "episode_index": 22,
                      "subtitle": "GB",
                      "fansub": "幻樱字幕组",
                      "resolution": "1920X1080"
                    }"#,
        );
    }
}

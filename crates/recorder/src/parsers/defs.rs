use std::collections::HashMap;

use fancy_regex::Regex as FancyRegex;
use lazy_static::lazy_static;
use maplit::hashmap;
use regex::Regex;

const LANG_ZH_TW: &str = "zh-tw";
const LANG_ZH: &str = "zh";
const LANG_EN: &str = "en";
const LANG_JP: &str = "jp";

lazy_static! {
    pub static ref SEASON_REGEX: Regex =
        Regex::new(r"(S\|[Ss]eason\s+)(\d+)").expect("Invalid regex");
    pub static ref TORRENT_PRASE_RULE_REGS: Vec<FancyRegex> = vec![
        FancyRegex::new(
            r"(.*) - (\d{1,4}(?!\d|p)|\d{1,4}\.\d{1,2}(?!\d|p))(?:v\d{1,2})?(?: )?(?:END)?(.*)"
        )
        .unwrap(),
        FancyRegex::new(
            r"(.*)[\[\ E](\d{1,4}|\d{1,4}\.\d{1,2})(?:v\d{1,2})?(?: )?(?:END)?[\]\ ](.*)"
        )
        .unwrap(),
        FancyRegex::new(r"(.*)\[(?:第)?(\d*\.*\d*)[话集話](?:END)?\](.*)").unwrap(),
        FancyRegex::new(r"(.*)第?(\d*\.*\d*)[话話集](?:END)?(.*)").unwrap(),
        FancyRegex::new(r"(.*)(?:S\d{2})?EP?(\d+)(.*)").unwrap(),
    ];
    pub static ref SUBTITLE_LANG: Vec<(&'static str, Vec<&'static str>)> = {
        vec![
            (LANG_ZH_TW, vec!["tc", "cht", "繁", "zh-tw"]),
            (LANG_ZH, vec!["sc", "chs", "简", "zh", "zh-cn"]),
            (LANG_EN, vec!["en", "eng", "英"]),
            (LANG_JP, vec!["jp", "jpn", "日"]),
        ]
    };
    pub static ref BRACKETS_REG: Regex = Regex::new(r"[\[\]()【】（）]").unwrap();
    pub static ref DIGIT_1PLUS_REG: Regex = Regex::new(r"\d+").unwrap();
    pub static ref ZH_NUM_MAP: HashMap<&'static str, i32> = {
        hashmap! {
            "〇" => 0,
            "一" => 1,
            "二" => 2,
            "三" => 3,
            "四" => 4,
            "五" => 5,
            "六" => 6,
            "七" => 7,
            "八" => 8,
            "九" => 9,
            "十" => 10,
            "廿" => 20,
            "百" => 100,
            "千" => 1000,
            "零" => 0,
            "壹" => 1,
            "贰" => 2,
            "叁" => 3,
            "肆" => 4,
            "伍" => 5,
            "陆" => 6,
            "柒" => 7,
            "捌" => 8,
            "玖" => 9,
            "拾" => 10,
            "念" => 20,
            "佰" => 100,
            "仟" => 1000,
        }
    };
    pub static ref ZH_NUM_RE: Regex =
        Regex::new(r"[〇一二三四五六七八九十廿百千零壹贰叁肆伍陆柒捌玖拾念佰仟]").unwrap();
}

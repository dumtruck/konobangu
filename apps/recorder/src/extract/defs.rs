use fancy_regex::Regex as FancyRegex;
use lazy_static::lazy_static;
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
}

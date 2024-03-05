use quirks_path::Path;

use super::defs::{
    BRACKETS_REG, DIGIT_1PLUS_REG, SEASON_REGEX, SUBTITLE_LANG, TORRENT_PRASE_RULE_REGS,
};

pub fn get_path_basename(path: &Path) -> &str {
    path.parent().map_or("", |s| s.as_str())
}

pub fn get_fansub(group_and_title: &str) -> (Option<&str>, &str) {
    let n = BRACKETS_REG
        .split(group_and_title)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();

    if n.len() > 1 {
        if DIGIT_1PLUS_REG.is_match(n[1]) {
            (None, group_and_title)
        } else {
            (Some(n[0]), n[1])
        }
    } else {
        (None, n[0])
    }
}

pub fn get_season_and_title(season_and_title: &str) -> (String, i32) {
    let title = SEASON_REGEX.replace(season_and_title, "");
    let title = title.trim().to_string();
    let mut season = 1;
    if let Some(match_result) = SEASON_REGEX.captures(season_and_title) {
        let season_str = match_result
            .get(2)
            .unwrap_or_else(|| unreachable!("season regex should have 2 groups"))
            .as_str();
        season = season_str
            .parse::<i32>()
            .unwrap_or_else(|_| unreachable!("season should be a number"));
    }
    (title, season)
}

pub fn get_subtitle_lang(subtitle_name: &str) -> Option<&'static str> {
    let subtitle_name_lower = subtitle_name.to_lowercase();
    for (lang, matches) in SUBTITLE_LANG.iter() {
        for m in matches {
            if subtitle_name_lower.contains(m) {
                return Some(lang);
            }
        }
    }
    None
}

pub fn parse_torrent(
    torrent_path: &Path,
    torrent_name: Option<&str>,
    season: Option<i32>,
    file_type: Option<&str>,
) {
    let media_name = get_path_basename(torrent_path);
    for rule in TORRENT_PRASE_RULE_REGS.iter() {
        let match_obj = if let Some(torrent_name) = torrent_name {
            rule.captures(torrent_name)
        } else {
            rule.captures(media_name)
        };

        if let Ok(Some(match_obj)) = match_obj {
            let group_and_title = match_obj
                .get(1)
                .unwrap_or_else(|| unreachable!("should have 1 group"))
                .as_str();
            let (group, title) = get_fansub(group_and_title);
            let season_and_title = get_season_and_title(title);
            let season = season.unwrap_or(season_and_title.1);
            let title = season_and_title.0;
            let episode = match_obj
                .get(2)
                .unwrap_or_else(|| unreachable!("should have 2 group"))
                .as_str()
                .parse::<i32>()
                .unwrap_or_else(|_| unreachable!("episode should be a number"));

            let extension = media_name;
            todo!()
        }
    }
}

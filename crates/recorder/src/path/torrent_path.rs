use std::collections::HashSet;

use quirks_path::{Path, PathBuf};

use crate::{
    downloaders::defs::Torrent,
    models::{bangumi, subscribers},
    parsers::defs::SEASON_REGEX,
};

pub fn check_files(info: &Torrent) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut media_list = vec![];
    let mut subtitle_list = vec![];
    for f in info.iter_files() {
        let file_name = PathBuf::from(f.get_name());
        let extension = file_name.extension().unwrap_or_default().to_lowercase();

        match extension.as_str() {
            ".mp4" | ".mkv" => {
                media_list.push(file_name);
            }
            ".ass" | ".srt" => subtitle_list.push(file_name),
            _ => {}
        }
    }

    (media_list, subtitle_list)
}

pub fn path_to_bangumi<'a>(
    save_path: &'a Path,
    downloader_path: &'a Path,
) -> Option<(&'a str, i32)> {
    let downloader_parts = downloader_path
        .components()
        .map(|s| s.as_str())
        .collect::<HashSet<_>>();

    let mut season = None;
    let mut bangumi_name = None;
    for part in save_path.components().map(|s| s.as_str()) {
        if let Some(match_result) = SEASON_REGEX.captures(part) {
            season = Some(
                match_result
                    .get(2)
                    .unwrap_or_else(|| unreachable!("must have a season"))
                    .as_str()
                    .parse::<i32>()
                    .unwrap_or_else(|e| unreachable!("{}", e.to_string())),
            );
        } else if !downloader_parts.contains(part) {
            bangumi_name = Some(part);
        }
    }
    match (season, bangumi_name) {
        (Some(season), Some(bangumi_name)) => Some((bangumi_name, season)),
        _ => None,
    }
}

pub fn file_depth(path: &Path) -> usize {
    path.components().count()
}

pub fn is_ep(path: &Path) -> bool {
    file_depth(path) <= 2
}

pub fn gen_bangumi_sub_path(data: &bangumi::Model) -> PathBuf {
    PathBuf::from(data.official_title.to_string()).join(format!("Season {}", data.season))
}

pub fn rule_name(bgm: &bangumi::Model, conf: &subscribers::SubscriberBangumiConfig) -> String {
    if let (Some(true), Some(group_name)) = (conf.leading_group_tag, &bgm.fansub) {
        format!("[{}] {} S{}", group_name, bgm.official_title, bgm.season)
    } else {
        format!("{} S{}", bgm.official_title, bgm.season)
    }
}

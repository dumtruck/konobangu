use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TorrentEpisodeMediaMeta {}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TorrentEpisodeSubtitleMeta {}

pub fn parse_episode_media_meta_from_torrent(
    torrent_path: &str,
    torrent_name: Option<&str>,
    season: Option<i32>,
) -> eyre::Result<TorrentEpisodeMediaMeta> {
    todo!()
}

pub fn parse_episode_subtitle_meta_from_torrent(
    torrent_path: &str,
    torrent_name: Option<&str>,
    season: Option<i32>,
) -> eyre::Result<TorrentEpisodeMediaMeta> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::{
        parse_episode_media_meta_from_torrent, parse_episode_subtitle_meta_from_torrent,
        TorrentEpisodeMediaMeta, TorrentEpisodeSubtitleMeta,
    };

    pub fn test_torrent_ep_parser(raw_name: &str, expected: &str) {
        let expected: Option<TorrentEpisodeMediaMeta> = serde_json::from_str(expected).unwrap();
        let found = parse_episode_media_meta_from_torrent(raw_name, None, None).ok();

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

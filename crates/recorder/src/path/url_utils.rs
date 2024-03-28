use quirks_path::Path;
use url::Url;

pub fn extract_filename_from_url(url: &Url) -> Option<&str> {
    url.path_segments().and_then(|s| s.last()).and_then(|last| {
        if last.is_empty() {
            Some(last)
        } else {
            None
        }
    })
}

pub fn extract_extname_from_url(url: &Url) -> Option<String> {
    let filename = extract_filename_from_url(url);
    filename
        .and_then(|f| Path::new(f).extension())
        .map(|ext| format!(".{}", ext))
}

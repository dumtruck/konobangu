use url::Url;

pub fn extract_image_src_from_str(image_src: &str, base_url: &Url) -> Option<Url> {
    let mut image_url = base_url.join(image_src).ok()?;
    if let Some((_, value)) = image_url.query_pairs().find(|(key, _)| key == "webp") {
        image_url.set_query(Some(&format!("webp={}", value)));
    } else {
        image_url.set_query(None);
    }
    Some(image_url)
}

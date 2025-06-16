use std::ops::Bound;

pub fn bound_range_to_content_range(
    r: &(Bound<u64>, Bound<u64>),
    l: u64,
) -> Result<String, String> {
    match r {
        (Bound::Included(start), Bound::Included(end)) => Ok(format!("bytes {start}-{end}/{l}")),
        (Bound::Included(start), Bound::Excluded(end)) => {
            Ok(format!("bytes {start}-{}/{l}", end - 1))
        }
        (Bound::Included(start), Bound::Unbounded) => Ok(format!(
            "bytes {start}-{}/{l}",
            if l > 0 { l - 1 } else { 0 }
        )),
        _ => Err(format!("bytes */{l}")),
    }
}

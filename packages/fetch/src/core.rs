use lazy_static::lazy_static;

lazy_static! {
    static ref DEFAULT_HTTP_CLIENT_USER_AGENT: Vec<String> =
        serde_json::from_str::<Vec<String>>(include_str!("./ua.json")).unwrap();
}

pub fn get_random_ua() -> &'static str {
    DEFAULT_HTTP_CLIENT_USER_AGENT[fastrand::usize(0..DEFAULT_HTTP_CLIENT_USER_AGENT.len())]
        .as_str()
}

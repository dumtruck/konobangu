use std::collections::HashMap;

use lazy_static::lazy_static;
use loco_rs::controller::middleware::static_assets::{FolderConfig, StaticAssets};
use regex::Regex;
use serde::Serialize;
use serde_json::Value;

use crate::app::App;

const ALTAIR_GRAPHQL_HTML: &'static str =
    include_str!("../../../node_modules/altair-static/build/dist/index.html");

lazy_static! {
    static ref ALTAIR_GRAPHQL_BASE_REGEX: Regex = Regex::new(r"<base.*>").unwrap();
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AltairGraphQLPlayground<'a> {
    #[serde(rename = "endpointURL")]
    pub endpoint_url: &'a str,
    /**
     * URL to set as the subscription endpoint. This can be relative or
     * absolute.
     */
    pub subscriptions_endpoint: Option<&'a str>,
    pub initial_headers: Option<HashMap<String, String>>,
    pub initial_settings: Option<HashMap<String, Value>>,
    #[serde(flatten)]
    pub other: Option<HashMap<String, Value>>,
}

impl<'a> AltairGraphQLPlayground<'a> {
    /// Create a config for GraphQL playground.
    pub fn new(endpoint_url: &'a str) -> Self {
        Self {
            endpoint_url,
            subscriptions_endpoint: Default::default(),
            initial_headers: Default::default(),
            initial_settings: Default::default(),
            other: Default::default(),
        }
    }

    pub fn render(&self, base_url: &str) -> loco_rs::Result<String> {
        let option = serde_json::to_string(self)?;
        let render_str = ALTAIR_GRAPHQL_BASE_REGEX
            .replace(ALTAIR_GRAPHQL_HTML, format!(r#"<base href="{base_url}">"#))
            .replace(
                "</body>",
                &format!("<script>AltairGraphQL.init({});</script></body>", option),
            );

        Ok(render_str)
    }
}

pub fn altair_graphql_playground_asset_middleware(base_url: &str) -> StaticAssets {
    StaticAssets {
        enable: true,
        must_exist: true,
        folder: FolderConfig {
            uri: String::from(base_url),
            path: App::get_working_root()
                .join("node_modules/altair-static/build/dist")
                .into(),
        },
        fallback: App::get_working_root()
            .join("assets/static/404.html")
            .into(),
        precompressed: false,
    }
}

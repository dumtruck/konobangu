use std::collections::HashMap;

use chrono::{Duration, Utc};
use fetch::{FetchError, HttpClientConfig, IntoUrl, get_random_ua};
use url::Url;

use crate::{
    errors::RecorderResult,
    extract::mikan::{
        MIKAN_ACCOUNT_MANAGE_PAGE_PATH, MIKAN_LOGIN_PAGE_PATH, MikanClient, MikanConfig,
        MikanCredentialForm,
    },
};

const TESTING_MIKAN_USERNAME: &str = "test_username";
const TESTING_MIKAN_PASSWORD: &str = "test_password";
const TESTING_MIKAN_ANTIFORGERY: &str = "test_antiforgery";
const TESTING_MIKAN_IDENTITY: &str = "test_identity";

pub async fn build_testing_mikan_client(
    base_mikan_url: impl IntoUrl,
) -> RecorderResult<MikanClient> {
    let mikan_client = MikanClient::from_config(MikanConfig {
        http_client: HttpClientConfig {
            ..Default::default()
        },
        base_url: base_mikan_url.into_url().map_err(FetchError::from)?,
    })
    .await?;
    Ok(mikan_client)
}

pub fn build_testing_mikan_credential_form() -> MikanCredentialForm {
    MikanCredentialForm {
        username: String::from(TESTING_MIKAN_USERNAME),
        password: String::from(TESTING_MIKAN_PASSWORD),
        user_agent: get_random_ua().to_string(),
    }
}

pub struct MikanMockServerLoginMock {
    pub login_get_mock: mockito::Mock,
    pub login_post_success_mock: mockito::Mock,
    pub login_post_failed_mock: mockito::Mock,
    pub account_get_success_mock: mockito::Mock,
    pub account_get_failed_mock: mockito::Mock,
}

pub struct MikanMockServer {
    pub server: mockito::ServerGuard,
    base_url: Url,
}

impl MikanMockServer {
    pub async fn new() -> RecorderResult<Self> {
        let server = mockito::Server::new_async().await;
        let base_url = Url::parse(&server.url())?;

        Ok(Self { server, base_url })
    }

    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    pub fn get_has_auth_matcher() -> impl Fn(&mockito::Request) -> bool {
        |req: &mockito::Request| -> bool {
            let test_identity_cookie =
                format!(".AspNetCore.Identity.Application={TESTING_MIKAN_IDENTITY}");
            req.header("Cookie").iter().any(|cookie| {
                cookie
                    .to_str()
                    .is_ok_and(|c| c.contains(&test_identity_cookie))
            })
        }
    }

    pub fn mock_get_login_page(&mut self) -> MikanMockServerLoginMock {
        let login_get_mock = self
            .server
            .mock("GET", MIKAN_LOGIN_PAGE_PATH)
            .match_query(mockito::Matcher::Any)
            .with_status(201)
            .with_header("Content-Type", "text/html; charset=utf-8")
            .with_header(
                "Set-Cookie",
                &format!(
                    ".AspNetCore.Antiforgery.test_app_id={TESTING_MIKAN_ANTIFORGERY}; HttpOnly; \
                     SameSite=Strict; Path=/"
                ),
            )
            .create();

        let test_identity_expires = (Utc::now() + Duration::days(30)).to_rfc2822();

        let match_post_login_body = |req: &mockito::Request| {
            req.body()
                .map(|b| url::form_urlencoded::parse(b))
                .is_ok_and(|queires| {
                    let qs = queires.collect::<HashMap<_, _>>();
                    qs.get("UserName")
                        .is_some_and(|s| s == TESTING_MIKAN_USERNAME)
                        && qs
                            .get("Password")
                            .is_some_and(|s| s == TESTING_MIKAN_PASSWORD)
                        && qs
                            .get("__RequestVerificationToken")
                            .is_some_and(|s| s == TESTING_MIKAN_ANTIFORGERY)
                })
        };

        let login_post_success_mock = {
            let mikan_base_url = self.base_url().clone();
            self.server
                .mock("POST", MIKAN_LOGIN_PAGE_PATH)
                .match_query(mockito::Matcher::Any)
                .match_request(match_post_login_body)
                .with_status(302)
                .with_header(
                    "Set-Cookie",
                    &format!(
                        ".AspNetCore.Identity.Application={TESTING_MIKAN_IDENTITY}; HttpOnly; \
                         SameSite=Lax; Path=/; Expires=${test_identity_expires}"
                    ),
                )
                .with_header_from_request("Location", move |req| {
                    let request_url = mikan_base_url.join(req.path_and_query()).ok();
                    request_url
                        .and_then(|u| {
                            u.query_pairs()
                                .find(|(key, _)| key == "ReturnUrl")
                                .map(|(_, value)| value.to_string())
                        })
                        .unwrap_or(String::from("/"))
                })
                .create()
        };

        let login_post_failed_mock = self
            .server
            .mock("POST", MIKAN_LOGIN_PAGE_PATH)
            .match_query(mockito::Matcher::Any)
            .match_request(move |req| !match_post_login_body(req))
            .with_status(200)
            .with_body_from_file("tests/resources/mikan/LoginError.html")
            .create();

        let account_get_success_mock = self
            .server
            .mock("GET", MIKAN_ACCOUNT_MANAGE_PAGE_PATH)
            .match_query(mockito::Matcher::Any)
            .match_request(move |req| Self::get_has_auth_matcher()(req))
            .with_status(200)
            .create();

        let account_get_failed_mock = self
            .server
            .mock("GET", MIKAN_ACCOUNT_MANAGE_PAGE_PATH)
            .match_query(mockito::Matcher::Any)
            .match_request(move |req| !Self::get_has_auth_matcher()(req))
            .with_status(302)
            .with_header("Location", MIKAN_LOGIN_PAGE_PATH)
            .create();

        MikanMockServerLoginMock {
            login_get_mock,
            login_post_success_mock,
            login_post_failed_mock,
            account_get_success_mock,
            account_get_failed_mock,
        }
    }
}

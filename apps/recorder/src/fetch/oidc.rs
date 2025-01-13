use std::{future::Future, pin::Pin};

use axum::http;

use super::{client::HttpClientError, HttpClient};

impl<'c> openidconnect::AsyncHttpClient<'c> for HttpClient {
    type Error = HttpClientError;

    #[cfg(target_arch = "wasm32")]
    type Future = Pin<Box<dyn Future<Output = Result<HttpResponse, Self::Error>> + 'c>>;
    #[cfg(not(target_arch = "wasm32"))]
    type Future =
        Pin<Box<dyn Future<Output = Result<openidconnect::HttpResponse, Self::Error>> + Send + 'c>>;

    fn call(&'c self, request: openidconnect::HttpRequest) -> Self::Future {
        Box::pin(async move {
            let response = self.execute(request.try_into()?).await?;

            let mut builder = http::Response::builder().status(response.status());

            #[cfg(not(target_arch = "wasm32"))]
            {
                builder = builder.version(response.version());
            }

            for (name, value) in response.headers().iter() {
                builder = builder.header(name, value);
            }

            builder
                .body(response.bytes().await?.to_vec())
                .map_err(HttpClientError::HttpError)
        })
    }
}

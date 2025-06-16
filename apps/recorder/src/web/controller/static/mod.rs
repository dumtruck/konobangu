use std::sync::Arc;

use async_stream::try_stream;
use axum::{
    Extension, Router,
    body::Body,
    extract::{Path, State},
    middleware::from_fn_with_state,
    response::Response,
    routing::get,
};
use axum_extra::{TypedHeader, headers::Range};
use bytes::Bytes;
use futures::{Stream, StreamExt};
use http::{HeaderMap, HeaderValue, StatusCode, header};
use itertools::Itertools;
use uuid::Uuid;

use crate::{
    app::AppContextTrait,
    auth::{AuthError, AuthUserInfo, auth_middleware},
    errors::{RecorderError, RecorderResult},
    utils::http::bound_range_to_content_range,
    web::controller::Controller,
};

pub const CONTROLLER_PREFIX: &str = "/api/static";

async fn serve_file_with_cache(
    State(ctx): State<Arc<dyn AppContextTrait>>,
    Path((subscriber_id, path)): Path<(i32, String)>,
    Extension(auth_user_info): Extension<AuthUserInfo>,
    range: Option<TypedHeader<Range>>,
) -> RecorderResult<Response> {
    if subscriber_id != auth_user_info.subscriber_auth.id {
        Err(AuthError::PermissionError)?;
    }

    let storage = ctx.storage();

    let storage_path = storage.build_subscriber_path(subscriber_id, &path);

    let metadata = storage
        .stat(&storage_path)
        .await
        .map_err(|_| RecorderError::from_status(StatusCode::NOT_FOUND))?;

    if !metadata.is_file() {
        return Err(RecorderError::from_status(StatusCode::NOT_FOUND));
    }

    let mime_type = mime_guess::from_path(&path).first_or_octet_stream();

    let response = if let Some(TypedHeader(range)) = range {
        let ranges = range
            .satisfiable_ranges(metadata.content_length())
            .collect_vec();

        if ranges.is_empty() {
            Response::builder()
                .status(StatusCode::PARTIAL_CONTENT)
                .header(header::CONTENT_TYPE, mime_type.as_ref())
                .body(Body::empty())?
        } else if ranges.len() == 1 {
            let r = ranges[0];
            let reader = storage.reader(&storage_path).await?;
            let content_range = bound_range_to_content_range(&r, metadata.content_length())
                .map_err(|s| {
                    RecorderError::from_status_and_headers(
                        StatusCode::RANGE_NOT_SATISFIABLE,
                        HeaderMap::from_iter(
                            [(header::CONTENT_RANGE, HeaderValue::from_str(&s).unwrap())]
                                .into_iter(),
                        ),
                    )
                })?;
            let stream = reader.into_bytes_stream(r).await?;

            Response::builder()
                .status(StatusCode::PARTIAL_CONTENT)
                .header(header::CONTENT_TYPE, mime_type.as_ref())
                .header(header::CONTENT_RANGE, content_range)
                .body(Body::from_stream(stream))?
        } else {
            let boundary = Uuid::new_v4().to_string();
            let reader = storage.reader(&storage_path).await?;
            let stream: impl Stream<Item = Result<Bytes, RecorderError>> = {
                let boundary = boundary.clone();
                try_stream! {
                    for r in ranges {
                        let content_range = bound_range_to_content_range(&r, metadata.content_length())
                        .map_err(|s| {
                            RecorderError::from_status_and_headers(
                                StatusCode::RANGE_NOT_SATISFIABLE,
                                HeaderMap::from_iter([(header::CONTENT_RANGE, HeaderValue::from_str(&s).unwrap())].into_iter()),
                            )
                        })?;
                        let part_header = format!("--{boundary}\r\nContent-Type: {}\r\nContent-Range: {}\r\n\r\n",
                            mime_type.as_ref(),
                            content_range,
                        );
                        yield part_header.into();
                        let mut part_stream = reader.clone().into_bytes_stream(r).await?;
                        while let Some(chunk) = part_stream.next().await {
                            yield chunk?;
                        }
                        yield "\r\n".into();
                    }
                    yield format!("--{boundary}--").into();
                }
            };
            let body = Body::from_stream(stream);

            Response::builder()
                .status(StatusCode::PARTIAL_CONTENT)
                .header(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str(
                        format!("multipart/byteranges; boundary={boundary}").as_str(),
                    )
                    .unwrap(),
                )
                .body(body)?
        }
    } else {
        let reader = storage.reader(&storage_path).await?;
        let stream = reader.into_bytes_stream(..).await?;

        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime_type.as_ref())
            .body(Body::from_stream(stream))?
    };

    Ok(response)
}

pub async fn create(ctx: Arc<dyn AppContextTrait>) -> RecorderResult<Controller> {
    let router = Router::<Arc<dyn AppContextTrait>>::new().route(
        "/subscribers/{subscriber_id}/*path",
        get(serve_file_with_cache).layer(from_fn_with_state(ctx, auth_middleware)),
    );

    Ok(Controller::from_prefix(CONTROLLER_PREFIX, router))
}

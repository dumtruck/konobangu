use std::fmt;

use async_stream::try_stream;
use axum::{body::Body, response::Response};
use axum_extra::{TypedHeader, headers::Range};
use bytes::Bytes;
use futures::{Stream, StreamExt};
use http::{HeaderValue, StatusCode, header};
use opendal::{Buffer, Metadata, Operator, Reader, Writer, layers::LoggingLayer};
use quirks_path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use url::Url;
use uuid::Uuid;

use super::StorageConfig;
use crate::{
    errors::{RecorderError, RecorderResult},
    utils::http::{bound_range_to_content_range, build_no_satisfiable_content_range},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageContentCategory {
    Image,
}

impl AsRef<str> for StorageContentCategory {
    fn as_ref(&self) -> &str {
        match self {
            Self::Image => "image",
        }
    }
}

pub enum StorageStoredUrl {
    RelativePath { path: String },
    Absolute { url: Url },
}

impl AsRef<str> for StorageStoredUrl {
    fn as_ref(&self) -> &str {
        match &self {
            Self::Absolute { url } => url.as_str(),
            Self::RelativePath { path } => path,
        }
    }
}

impl fmt::Display for StorageStoredUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

#[derive(Debug, Clone)]
pub struct StorageService {
    pub data_dir: String,
}

impl StorageService {
    pub async fn from_config(config: StorageConfig) -> RecorderResult<Self> {
        Ok(Self {
            data_dir: config.data_dir.to_string(),
        })
    }

    pub fn get_operator(&self) -> Result<Operator, opendal::Error> {
        let op = if cfg!(test) {
            Operator::new(opendal::services::Memory::default())?
                .layer(LoggingLayer::default())
                .finish()
        } else {
            Operator::new(opendal::services::Fs::default().root(&self.data_dir))?
                .layer(LoggingLayer::default())
                .finish()
        };

        Ok(op)
    }

    pub fn build_subscriber_path(&self, subscriber_id: i32, path: impl AsRef<Path>) -> PathBuf {
        let mut p = PathBuf::from("/subscribers");
        p.push(subscriber_id.to_string());
        p.push(path);
        p
    }

    pub fn build_public_path(&self, path: impl AsRef<Path>) -> PathBuf {
        let mut p = PathBuf::from("/public");
        p.push(path);
        p
    }

    pub fn build_subscriber_object_path(
        &self,
        subscriber_id: i32,
        content_category: StorageContentCategory,
        bucket: &str,
        object_name: &str,
    ) -> PathBuf {
        self.build_subscriber_path(
            subscriber_id,
            [content_category.as_ref(), bucket, object_name]
                .iter()
                .collect::<PathBuf>(),
        )
    }

    pub fn build_public_object_path(
        &self,
        content_category: StorageContentCategory,
        bucket: &str,
        object_name: &str,
    ) -> PathBuf {
        self.build_public_path(
            [content_category.as_ref(), bucket, object_name]
                .iter()
                .collect::<PathBuf>(),
        )
    }

    pub async fn write<P: Into<PathBuf> + Send>(
        &self,
        path: P,
        data: Bytes,
    ) -> Result<StorageStoredUrl, opendal::Error> {
        let operator = self.get_operator()?;

        let path = path.into();

        if let Some(dirname) = path.parent() {
            let dirname = dirname.join("/");
            operator.create_dir(dirname.as_str()).await?;
        }

        operator.write(path.as_str(), data).await?;

        Ok(StorageStoredUrl::RelativePath {
            path: path.to_string(),
        })
    }

    pub async fn exists<P: ToString + Send>(
        &self,
        path: P,
    ) -> Result<Option<StorageStoredUrl>, opendal::Error> {
        let operator = self.get_operator()?;

        let path = path.to_string();

        if operator.exists(&path).await? {
            Ok(Some(StorageStoredUrl::RelativePath { path }))
        } else {
            Ok(None)
        }
    }

    pub async fn read(&self, path: impl AsRef<str>) -> Result<Buffer, opendal::Error> {
        let operator = self.get_operator()?;

        let data = operator.read(path.as_ref()).await?;

        Ok(data)
    }

    pub async fn reader(&self, path: impl AsRef<str>) -> Result<Reader, opendal::Error> {
        let operator = self.get_operator()?;

        let reader = operator.reader(path.as_ref()).await?;

        Ok(reader)
    }

    pub async fn writer(&self, path: impl AsRef<str>) -> Result<Writer, opendal::Error> {
        let operator = self.get_operator()?;

        let writer = operator.writer(path.as_ref()).await?;

        Ok(writer)
    }

    pub async fn stat(&self, path: impl AsRef<str>) -> Result<Metadata, opendal::Error> {
        let operator = self.get_operator()?;

        let metadata = operator.stat(path.as_ref()).await?;

        Ok(metadata)
    }

    #[instrument(skip_all, err, fields(storage_path = %storage_path.as_ref(), range = ?range))]
    pub async fn serve_file(
        &self,
        storage_path: impl AsRef<str>,
        range: Option<TypedHeader<Range>>,
    ) -> RecorderResult<Response> {
        let metadata = self
            .stat(&storage_path)
            .await
            .map_err(|_| RecorderError::from_status(StatusCode::NOT_FOUND))?;

        if !metadata.is_file() {
            return Err(RecorderError::from_status(StatusCode::NOT_FOUND));
        }

        let mime_type = mime_guess::from_path(storage_path.as_ref()).first_or_octet_stream();

        let content_type = HeaderValue::from_str(mime_type.as_ref())?;

        let response = if let Some(TypedHeader(range)) = range {
            let ranges = range
                .satisfiable_ranges(metadata.content_length())
                .map(|r| -> Option<(_, _)> {
                    let a = bound_range_to_content_range(&r, metadata.content_length())?;
                    Some((r, a))
                })
                .collect::<Option<Vec<_>>>();

            if let Some(mut ranges) = ranges {
                if ranges.len() > 1 {
                    let boundary = Uuid::new_v4().to_string();
                    let reader = self.reader(storage_path.as_ref()).await?;
                    let stream: impl Stream<Item = Result<Bytes, RecorderError>> = {
                        let boundary = boundary.clone();
                        try_stream! {
                            for (r, content_range) in ranges {
                                let part_header = format!("--{boundary}\r\nContent-Type: {}\r\nContent-Range: {}\r\n\r\n",
                                    mime_type.as_ref(),
                                    content_range.clone().to_str().unwrap(),
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
                } else if let Some((r, content_range)) = ranges.pop() {
                    let reader = self.reader(storage_path.as_ref()).await?;
                    let stream = reader.into_bytes_stream(r).await?;

                    Response::builder()
                        .status(StatusCode::PARTIAL_CONTENT)
                        .header(header::CONTENT_TYPE, content_type.clone())
                        .header(header::CONTENT_RANGE, content_range)
                        .body(Body::from_stream(stream))?
                } else {
                    unreachable!("ranges length should be greater than 0")
                }
            } else {
                Response::builder()
                    .status(StatusCode::RANGE_NOT_SATISFIABLE)
                    .header(header::CONTENT_TYPE, content_type)
                    .header(
                        header::CONTENT_RANGE,
                        build_no_satisfiable_content_range(metadata.content_length()),
                    )
                    .body(Body::empty())?
            }
        } else {
            let reader = self.reader(storage_path.as_ref()).await?;
            let stream = reader.into_bytes_stream(..).await?;

            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, content_type)
                .body(Body::from_stream(stream))?
        };

        Ok(response)
    }
}

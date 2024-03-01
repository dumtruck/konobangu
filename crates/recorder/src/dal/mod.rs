use bytes::Bytes;
use opendal::{layers::LoggingLayer, services, Operator};
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use crate::{
    config::AppDalConf,
    path::{VFSSubPath, VFSSubPathBuf},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppDalContentCategory {
    Poster,
}

impl AsRef<str> for AppDalContentCategory {
    fn as_ref(&self) -> &str {
        match self {
            Self::Poster => "poster",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppDalContext {
    pub config: AppDalConf,
}

pub enum DalStoredUrl {
    RelativePath { path: String },
    Absolute { url: Url },
}

impl AppDalContext {
    pub fn new(app_dal_conf: AppDalConf) -> Self {
        Self {
            config: app_dal_conf,
        }
    }

    pub async fn store_blob(
        &self,
        content_category: AppDalContentCategory,
        extname: &str,
        data: Bytes,
        subscriber_pid: &str,
    ) -> eyre::Result<DalStoredUrl> {
        let basename = format!("{}{}", Uuid::new_v4(), extname);
        let mut dirname = [subscriber_pid, content_category.as_ref()]
            .into_iter()
            .map(VFSSubPath::new)
            .collect::<VFSSubPathBuf>();

        let mut fs_builder = services::Fs::default();
        fs_builder.root(self.config.fs_root.as_str());

        let fs_op = Operator::new(fs_builder)?
            .layer(LoggingLayer::default())
            .finish();

        fs_op.create_dir(dirname.as_str()).await?;

        let fullname = {
            dirname.push(basename);
            dirname
        };

        fs_op.write_with(fullname.as_str(), data).await?;

        Ok(DalStoredUrl::RelativePath {
            path: fullname.to_string(),
        })
    }
}

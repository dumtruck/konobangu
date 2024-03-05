use std::{borrow::Cow, collections::VecDeque, path::PathBuf};

use lazy_static::lazy_static;
pub use uni_path::{Path as VFSSubPath, PathBuf as VFSSubPathBuf};
use url::Url;

use crate::parsers::errors::ParseError;

pub fn path_str_to_file_url(path: &str) -> eyre::Result<Url> {
    Url::parse(&format!("file:///{path}")).map_err(|e| e.into())
}

pub fn path_str_equals(p1: &str, p2: &str) -> eyre::Result<bool> {
    let p1 = path_str_to_file_url(p1)?;
    let p2 = path_str_to_file_url(p2)?;
    Ok(p1.as_str() == p2.as_str())
}

const VFS_EMPTY_STR: &str = "";

lazy_static! {
    pub static ref VFS_SUB_ROOT_BUF: VFSSubPathBuf = VFSSubPathBuf::from("/");
    pub static ref VFS_SUB_ROOT: &'static VFSSubPath = &VFS_SUB_ROOT_BUF.as_path();
}

pub type VFSComponents<'a> = uni_path::Components<'a>;
pub type VFSComponent<'a> = uni_path::Component<'a>;

#[derive(Debug, Clone)]
pub struct VFSPath<'a> {
    pub root: &'a str,
    pub sub: &'a VFSSubPath,
}

impl<'a> VFSPath<'a> {
    pub fn new(root: &'a str, sub: &'a VFSSubPath) -> VFSPath<'a> {
        Self { root, sub }
    }

    pub fn file_name(&self) -> Option<&str> {
        self.sub.file_name()
    }

    pub fn parent(&self) -> Option<VFSPath> {
        self.sub.parent().map(|p| Self::new(self.root, p))
    }

    pub fn dirname(&'a self) -> VFSPath<'a> {
        self.parent()
            .unwrap_or_else(|| Self::new(self.root, &VFS_SUB_ROOT))
    }

    pub fn basename(&self) -> &str {
        self.file_name().unwrap_or(VFS_EMPTY_STR)
    }

    pub fn components(&self) -> VFSComponents<'a> {
        self.sub.components()
    }

    pub fn join<P: AsRef<VFSSubPath>>(&self, path: P) -> VFSPathBuf {
        VFSPathBuf::new(self.root, self.sub.join(path))
    }

    pub fn extension(&self) -> Option<&str> {
        self.sub.extension()
    }

    pub fn extname(&self) -> &str {
        self.extension().unwrap_or_default()
    }

    pub fn to_std_path_buf(&self) -> PathBuf {
        PathBuf::from(self.root).join(self.sub.as_str())
    }
}

#[derive(Clone, Debug)]
pub struct VFSPathBuf {
    root: String,
    sub: VFSSubPathBuf,
}

impl VFSPathBuf {
    pub fn new<R: Into<String>, S: Into<VFSSubPathBuf>>(root: R, sub: S) -> Self {
        Self {
            root: root.into(),
            sub: sub.into(),
        }
    }

    pub fn from_root(root: &str) -> Result<Self, ParseError> {
        Ok(Self {
            root: root.to_string(),
            sub: VFS_SUB_ROOT_BUF.clone(),
        })
    }

    pub fn as_path(&self) -> VFSPath {
        VFSPath::new(&self.root as &str, self.sub.as_path())
    }

    pub fn push<P: AsRef<VFSSubPath>>(&mut self, path: P) {
        self.sub.push(path);
    }

    pub fn pop(&mut self) -> bool {
        self.sub.pop()
    }

    pub fn set_extension<S: AsRef<str>>(&mut self, ext: S) {
        self.sub.set_extension(ext);
    }

    pub fn set_file_name<S: AsRef<str>>(&mut self, file_name: S) {
        self.sub.set_file_name(file_name);
    }
}

impl Into<PathBuf> for VFSPathBuf {
    fn into(self) -> PathBuf {
        let root = self.root;
        PathBuf::from(root).join(self.sub.as_str())
    }
}

#![feature(extend_one)]

mod url;
pub mod windows;

use std::{
    borrow::{Borrow, Cow},
    cmp,
    collections::TryReserveError,
    error::Error,
    fmt,
    fmt::Formatter,
    hash::{Hash, Hasher},
    io,
    iter::FusedIterator,
    ops::{Deref, DerefMut},
    rc::Rc,
    str::FromStr,
    sync::Arc,
};

use ::url::Url;
pub use url::{path_equals_as_file_url, PathToUrlError};
use windows::is_windows_sep;

use crate::{
    url::path_to_file_url_segments,
    windows::{is_windows_verbatim_sep, parse_windows_path_prefix},
};

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum Prefix<'a> {
    Verbatim { prefix: &'a str },
    VerbatimUNC { server: &'a str, share: &'a str },
    VerbatimDisk { drive: char },
    DeviceNS { device: &'a str },
    UNC { server: &'a str, share: &'a str },
    Disk { drive: char },
}

impl Prefix<'_> {
    #[inline]
    pub fn is_verbatim(&self) -> bool {
        use Prefix::*;

        matches!(
            *self,
            Verbatim { .. } | VerbatimDisk { .. } | VerbatimUNC { .. }
        )
    }

    pub fn len(&self) -> usize {
        use Prefix::*;

        match *self {
            Verbatim { prefix } => 4 + prefix.len(),
            VerbatimUNC { server, share } => {
                8 + server.len()
                    + if !share.is_empty() {
                        1 + share.len()
                    } else {
                        0
                    }
            }
            VerbatimDisk { .. } => 6,
            UNC { server, share } => {
                2 + server.len()
                    + if !share.is_empty() {
                        1 + share.len()
                    } else {
                        0
                    }
            }
            DeviceNS { device } => 4 + device.len(),
            Disk { .. } => 2,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn is_drive(&self) -> bool {
        use Prefix::*;

        matches!(*self, Disk { .. } | VerbatimDisk { .. })
    }

    #[inline]
    pub fn has_implicit_root(&self) -> bool {
        !self.is_drive()
    }
}

fn iter_after<'a, 'b, I, J>(mut iter: I, mut prefix: J) -> Option<I>
where
    I: Iterator<Item = Component<'a>> + Clone,
    J: Iterator<Item = Component<'b>>,
{
    loop {
        let mut iter_next = iter.clone();
        match (iter_next.next(), prefix.next()) {
            (Some(ref x), Some(ref y)) if x == y => (),
            (Some(_), Some(_)) => return None,
            (Some(_), None) => return Some(iter),
            (None, None) => return Some(iter),
            (None, Some(_)) => return None,
        }
        iter = iter_next;
    }
}

pub fn is_separator(c: char) -> bool {
    is_windows_sep(c)
}

pub fn is_sep_byte(c: char) -> bool {
    is_windows_sep(c)
}

pub fn is_verbatim_sep(c: char) -> bool {
    is_windows_verbatim_sep(c)
}

fn has_physical_root<'a>(s: &'a str, prefix: Option<Prefix<'_>>) -> Option<&'a str> {
    let (len, path) = if let Some(p) = prefix {
        (p.len(), &s[p.len()..])
    } else {
        (0, s)
    };
    let path_bytes = path.as_bytes();
    if !path.is_empty() && is_separator(path_bytes[0] as char) {
        Some(&s[len..len + 1])
    } else {
        None
    }
}

fn rsplit_file_at_dot(file: &str) -> (Option<&str>, Option<&str>) {
    if file == ".." {
        return (Some(file), None);
    }

    let mut iter = file.rsplitn(2, '.');
    let after = iter.next();
    let before = iter.next();
    if before == Some("") {
        (Some(file), None)
    } else {
        (before, after)
    }
}

fn split_file_at_dot(file: &str) -> (&str, Option<&str>) {
    if file == ".." {
        return (file, None);
    }

    let i = match file[1..].bytes().position(|b| b == b'.') {
        Some(i) => i + 1,
        None => return (file, None),
    };
    let before = &file[..i];
    let after = &file[i + 1..];
    (before, Some(after))
}

#[derive(Debug, PartialEq, Clone, PartialOrd, Copy)]
enum State {
    Prefix = 0,
    StartDir = 1,
    Body = 2,
    Done = 3,
}

#[derive(Debug, Eq, Clone, Copy)]
pub struct PrefixComponent<'a> {
    raw: &'a str,
    parsed: Prefix<'a>,
}

impl<'a> PrefixComponent<'a> {
    #[inline]
    pub fn kind(&self) -> Prefix<'a> {
        self.parsed
    }

    #[inline]
    pub fn as_str(&self) -> &'a str {
        self.raw
    }
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for PrefixComponent<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.parsed.partial_cmp(&other.parsed)
    }
}

impl PartialEq for PrefixComponent<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.parsed == other.parsed
    }
}

impl Ord for PrefixComponent<'_> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.parsed.cmp(&other.parsed)
    }
}

impl Hash for PrefixComponent<'_> {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.parsed.hash(h)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Component<'a> {
    Prefix(PrefixComponent<'a>),
    RootDir(Option<&'a str>),
    CurDir,
    ParentDir,
    Normal(&'a str),
}

impl<'a> Component<'a> {
    pub fn as_str(self) -> &'a str {
        match self {
            Component::Prefix(p) => p.as_str(),
            Component::RootDir(root) => root.unwrap_or("/"),
            Component::CurDir => ".",
            Component::ParentDir => "..",
            Component::Normal(path) => path,
        }
    }
}

impl AsRef<str> for Component<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<Path> for Component<'_> {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.as_str().as_ref()
    }
}

#[derive(Clone)]
pub struct Components<'a> {
    path: &'a str,
    prefix: Option<Prefix<'a>>,
    has_physical_root: Option<&'a str>,
    front: State,
    back: State,
}

#[derive(Clone)]
pub struct Iter<'a> {
    inner: Components<'a>,
}

impl fmt::Debug for Components<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct DebugHelper<'a>(&'a Path);

        impl fmt::Debug for DebugHelper<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list().entries(self.0.components()).finish()
            }
        }

        f.debug_tuple("Components")
            .field(&DebugHelper(self.as_path()))
            .finish()
    }
}

impl<'a> Components<'a> {
    #[inline]
    fn prefix_len(&self) -> usize {
        self.prefix.as_ref().map(Prefix::len).unwrap_or(0)
    }

    #[inline]
    fn prefix_verbatim(&self) -> bool {
        self.prefix
            .as_ref()
            .map(Prefix::is_verbatim)
            .unwrap_or(false)
    }

    #[inline]
    fn prefix_remaining(&self) -> usize {
        if self.front == State::Prefix {
            self.prefix_len()
        } else {
            0
        }
    }

    // Given the iteration so far, how much of the pre-State::Body path is left?
    #[inline]
    fn len_before_body(&self) -> usize {
        let root = if self.front <= State::StartDir && matches!(self.has_physical_root, Some(..)) {
            1
        } else {
            0
        };
        let cur_dir = if self.front <= State::StartDir && self.include_cur_dir() {
            1
        } else {
            0
        };
        self.prefix_remaining() + root + cur_dir
    }

    #[inline]
    fn finished(&self) -> bool {
        self.front == State::Done || self.back == State::Done || self.front > self.back
    }

    #[inline]
    fn is_sep_byte(&self, b: u8) -> bool {
        if self.prefix_verbatim() {
            is_verbatim_sep(b as char)
        } else {
            is_sep_byte(b as char)
        }
    }

    pub fn as_path(&self) -> &'a Path {
        let mut comps = self.clone();
        if comps.front == State::Body {
            comps.trim_left();
        }
        if comps.back == State::Body {
            comps.trim_right();
        }
        Path::new(comps.path)
    }

    fn has_root(&self) -> bool {
        if matches!(self.has_physical_root, Some(..)) {
            return true;
        }
        if let Some(p) = self.prefix {
            if p.has_implicit_root() {
                return true;
            }
        }
        false
    }

    fn include_cur_dir(&self) -> bool {
        if self.has_root() {
            return false;
        }
        let mut iter = self.path[self.prefix_remaining()..].as_bytes().iter();
        match (iter.next(), iter.next()) {
            (Some(&b'.'), None) => true,
            (Some(&b'.'), Some(&b)) => self.is_sep_byte(b),
            _ => false,
        }
    }

    fn parse_single_component<'b>(&self, comp: &'b str) -> Option<Component<'b>> {
        match comp {
            "." if self.prefix_verbatim() => Some(Component::CurDir),
            "." => None,
            ".." => Some(Component::ParentDir),
            "" => None,
            _ => Some(Component::Normal(comp)),
        }
    }

    fn parse_next_component(&self) -> (usize, Option<Component<'a>>) {
        debug_assert!(self.front == State::Body);
        let (extra, comp) = match self
            .path
            .as_bytes()
            .iter()
            .position(|b| self.is_sep_byte(*b))
        {
            None => (0, self.path),
            Some(i) => (1, &self.path[..i]),
        };
        (comp.len() + extra, self.parse_single_component(comp))
    }

    fn parse_next_component_back(&self) -> (usize, Option<Component<'a>>) {
        debug_assert!(self.back == State::Body);
        let start = self.len_before_body();
        let (extra, comp) = match self.path[start..]
            .as_bytes()
            .iter()
            .rposition(|b| self.is_sep_byte(*b))
        {
            None => (0, &self.path[start..]),
            Some(i) => (1, &self.path[start + i + 1..]),
        };
        (comp.len() + extra, self.parse_single_component(comp))
    }

    fn trim_left(&mut self) {
        while !self.path.is_empty() {
            let (size, comp) = self.parse_next_component();
            if comp.is_some() {
                return;
            } else {
                self.path = &self.path[size..];
            }
        }
    }

    fn trim_right(&mut self) {
        while self.path.len() > self.len_before_body() {
            let (size, comp) = self.parse_next_component_back();
            if comp.is_some() {
                return;
            } else {
                self.path = &self.path[..self.path.len() - size];
            }
        }
    }
}

impl AsRef<Path> for Components<'_> {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl AsRef<str> for Components<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_path().as_str()
    }
}

impl fmt::Debug for Iter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct DebugHelper<'a>(&'a Path);

        impl fmt::Debug for DebugHelper<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list().entries(self.0.iter()).finish()
            }
        }

        f.debug_tuple("Iter")
            .field(&DebugHelper(self.as_path()))
            .finish()
    }
}

impl<'a> Iter<'a> {
    #[inline]
    pub fn as_path(&self) -> &'a Path {
        self.inner.as_path()
    }
}

impl AsRef<Path> for Iter<'_> {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl AsRef<str> for Iter<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_path().as_str()
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<&'a str> {
        self.inner.next().map(Component::as_str)
    }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a str> {
        self.inner.next_back().map(Component::as_str)
    }
}

impl FusedIterator for Iter<'_> {}

impl<'a> Iterator for Components<'a> {
    type Item = Component<'a>;

    fn next(&mut self) -> Option<Component<'a>> {
        while !self.finished() {
            match self.front {
                State::Prefix if self.prefix_len() > 0 => {
                    self.front = State::StartDir;
                    debug_assert!(self.prefix_len() <= self.path.len());
                    let raw = &self.path[..self.prefix_len()];
                    self.path = &self.path[self.prefix_len()..];
                    return Some(Component::Prefix(PrefixComponent {
                        raw,
                        parsed: self.prefix.unwrap(),
                    }));
                }
                State::Prefix => {
                    self.front = State::StartDir;
                }
                State::StartDir => {
                    self.front = State::Body;
                    if let Some(root) = self.has_physical_root {
                        debug_assert!(!self.path.is_empty());
                        self.path = &self.path[1..];
                        return Some(Component::RootDir(Some(root)));
                    } else if let Some(p) = self.prefix {
                        if p.has_implicit_root() && !p.is_verbatim() {
                            return Some(Component::RootDir(None));
                        }
                    } else if self.include_cur_dir() {
                        debug_assert!(!self.path.is_empty());
                        self.path = &self.path[1..];
                        return Some(Component::CurDir);
                    }
                }
                State::Body if !self.path.is_empty() => {
                    let (size, comp) = self.parse_next_component();
                    self.path = &self.path[size..];
                    if comp.is_some() {
                        return comp;
                    }
                }
                State::Body => {
                    self.front = State::Done;
                }
                State::Done => unreachable!(),
            }
        }
        None
    }
}

impl<'a> DoubleEndedIterator for Components<'a> {
    fn next_back(&mut self) -> Option<Component<'a>> {
        while !self.finished() {
            match self.back {
                State::Body if self.path.len() > self.len_before_body() => {
                    let (size, comp) = self.parse_next_component_back();
                    self.path = &self.path[..self.path.len() - size];
                    if comp.is_some() {
                        return comp;
                    }
                }
                State::Body => {
                    self.back = State::StartDir;
                }
                State::StartDir => {
                    self.back = State::Prefix;
                    if let Some(root) = self.has_physical_root {
                        self.path = &self.path[..self.path.len() - 1];
                        return Some(Component::RootDir(Some(root)));
                    } else if let Some(p) = self.prefix {
                        if p.has_implicit_root() && !p.is_verbatim() {
                            return Some(Component::RootDir(None));
                        }
                    } else if self.include_cur_dir() {
                        self.path = &self.path[..self.path.len() - 1];
                        return Some(Component::CurDir);
                    }
                }
                State::Prefix if self.prefix_len() > 0 => {
                    self.back = State::Done;
                    return Some(Component::Prefix(PrefixComponent {
                        raw: self.path,
                        parsed: self.prefix.unwrap(),
                    }));
                }
                State::Prefix => {
                    self.back = State::Done;
                    return None;
                }
                State::Done => unreachable!(),
            }
        }
        None
    }
}

impl FusedIterator for Components<'_> {}

impl<'a> PartialEq for Components<'a> {
    #[inline]
    fn eq(&self, other: &Components<'a>) -> bool {
        let Components {
            path: _,
            front: _,
            back: _,
            has_physical_root: _,
            prefix: _,
        } = self;

        if self.path.len() == other.path.len()
            && self.front == other.front
            && self.back == State::Body
            && other.back == State::Body
            && self.prefix_verbatim() == other.prefix_verbatim()
            && self.path == other.path
        {
            return true;
        }

        Iterator::eq(self.clone().rev(), other.clone().rev())
    }
}

impl Eq for Components<'_> {}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl<'a> PartialOrd for Components<'a> {
    #[inline]
    fn partial_cmp(&self, other: &Components<'a>) -> Option<cmp::Ordering> {
        Some(compare_components(self.clone(), other.clone()))
    }
}

impl Ord for Components<'_> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        compare_components(self.clone(), other.clone())
    }
}

fn compare_components(mut left: Components<'_>, mut right: Components<'_>) -> cmp::Ordering {
    if left.prefix.is_none() && right.prefix.is_none() && left.front == right.front {
        let first_difference = match left
            .path
            .as_bytes()
            .iter()
            .zip(right.path.as_bytes())
            .position(|(&a, &b)| a != b)
        {
            None if left.path.len() == right.path.len() => return cmp::Ordering::Equal,
            None => left.path.len().min(right.path.len()),
            Some(diff) => diff,
        };

        if let Some(previous_sep) = left.path[..first_difference]
            .as_bytes()
            .iter()
            .rposition(|&b| left.is_sep_byte(b))
        {
            let mismatched_component_start = previous_sep + 1;
            left.path = &left.path[mismatched_component_start..];
            left.front = State::Body;
            right.path = &right.path[mismatched_component_start..];
            right.front = State::Body;
        }
    }

    Iterator::cmp(left, right)
}

#[derive(Copy, Clone, Debug)]
pub struct Ancestors<'a> {
    next: Option<&'a Path>,
}

impl<'a> Iterator for Ancestors<'a> {
    type Item = &'a Path;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next;
        self.next = next.and_then(Path::parent);
        next
    }
}

impl FusedIterator for Ancestors<'_> {}

pub struct PathBuf {
    inner: String,
}

impl PathBuf {
    #[inline]
    fn as_mut_vec(&mut self) -> &mut Vec<u8> {
        unsafe { self.inner.as_mut_vec() }
    }

    #[inline]
    pub fn new() -> PathBuf {
        PathBuf {
            inner: String::new(),
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> PathBuf {
        PathBuf {
            inner: String::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn as_path(&self) -> &Path {
        self
    }

    pub fn push<P: AsRef<Path>>(&mut self, path: P) {
        self._push(path.as_ref())
    }

    fn _push(&mut self, path: &Path) {
        let main_sep_str = self.get_main_sep();

        let mut need_sep = self
            .as_mut_vec()
            .last()
            .is_some_and(|c| !is_separator(*c as char));

        let comps = self.components();

        if comps.prefix_len() > 0
            && comps.prefix_len() == comps.path.len()
            && comps.prefix.unwrap().is_drive()
        {
            need_sep = true;
        }

        if path.is_absolute() || path.prefix().is_some() {
            self.as_mut_vec().truncate(0);
        } else if comps.prefix_verbatim() && !path.inner.is_empty() {
            let mut buf: Vec<_> = comps.collect();
            for c in path.components() {
                match c {
                    Component::RootDir { .. } => {
                        buf.truncate(1);
                        buf.push(c);
                    }
                    Component::CurDir => (),
                    Component::ParentDir => {
                        if let Some(Component::Normal(_)) = buf.last() {
                            buf.pop();
                        }
                    }
                    _ => buf.push(c),
                }
            }
            let mut res = String::new();
            let mut need_sep = false;
            for c in buf {
                if need_sep && !matches!(c, Component::RootDir { .. }) {
                    res.push(main_sep_str);
                }
                res.push_str(c.as_str());

                need_sep = match c {
                    Component::RootDir { .. } => false,
                    Component::Prefix(prefix) => {
                        !prefix.parsed.is_drive() && !prefix.parsed.is_empty()
                    }
                    _ => true,
                };
            }
            self.inner = res;
            return;
        } else if path.has_root() {
            let prefix_len = self.components().prefix_remaining();
            self.as_mut_vec().truncate(prefix_len);
        } else if need_sep {
            self.inner.push(main_sep_str)
        }

        self.inner.push_str(path.as_str())
    }

    pub fn pop(&mut self) -> bool {
        match self.parent().map(|p| p.inner.len()) {
            Some(len) => {
                self.as_mut_vec().truncate(len);
                true
            }
            None => false,
        }
    }

    pub fn set_file_name<S: AsRef<str>>(&mut self, file_name: S) {
        self._set_file_name(file_name.as_ref())
    }

    fn _set_file_name(&mut self, file_name: &str) {
        if self.file_name().is_some() {
            let popped = self.pop();
            debug_assert!(popped);
        }
        self.push(file_name);
    }

    pub fn set_extension<S: AsRef<str>>(&mut self, extension: S) -> bool {
        self._set_extension(extension.as_ref())
    }

    fn _set_extension(&mut self, extension: &str) -> bool {
        let file_stem = match self.file_stem() {
            None => return false,
            Some(f) => f.as_bytes(),
        };

        // truncate until right after the file stem
        let end_file_stem = file_stem[file_stem.len()..].as_ptr().addr();
        let start = self.inner.as_bytes().as_ptr().addr();
        let v = self.as_mut_vec();
        v.truncate(end_file_stem.wrapping_sub(start));

        // add the new extension, if any
        let new = extension.as_bytes();
        if !new.is_empty() {
            v.reserve_exact(new.len() + 1);
            v.push(b'.');
            v.extend_from_slice(new);
        }

        true
    }

    #[inline]
    pub fn as_mut_string(&mut self) -> &mut String {
        &mut self.inner
    }

    #[inline]
    pub fn into_string(self) -> String {
        self.inner
    }

    #[inline]
    pub fn into_boxed_path(self) -> Box<Path> {
        let rw = Box::into_raw(self.inner.into_boxed_str()) as *mut Path;
        unsafe { Box::from_raw(rw) }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear()
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional)
    }

    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.inner.try_reserve(additional)
    }

    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.inner.reserve_exact(additional)
    }

    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.inner.try_reserve_exact(additional)
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit()
    }

    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.inner.shrink_to(min_capacity)
    }
}

impl Clone for PathBuf {
    #[inline]
    fn clone(&self) -> Self {
        PathBuf {
            inner: self.inner.clone(),
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.inner.clone_from(&source.inner)
    }
}

impl From<&Path> for Box<Path> {
    /// Creates a boxed [`Path`] from a reference.
    ///
    /// This will allocate and clone `path` to it.
    fn from(path: &Path) -> Box<Path> {
        let boxed: Box<str> = path.inner.into();
        let rw = Box::into_raw(boxed) as *mut Path;
        unsafe { Box::from_raw(rw) }
    }
}

impl From<Cow<'_, Path>> for Box<Path> {
    #[inline]
    fn from(cow: Cow<'_, Path>) -> Box<Path> {
        match cow {
            Cow::Borrowed(path) => Box::from(path),
            Cow::Owned(path) => Box::from(path),
        }
    }
}

impl From<Box<Path>> for PathBuf {
    #[inline]
    fn from(boxed: Box<Path>) -> PathBuf {
        boxed.into_path_buf()
    }
}

impl From<PathBuf> for Box<Path> {
    #[inline]
    fn from(p: PathBuf) -> Box<Path> {
        p.into_boxed_path()
    }
}

impl Clone for Box<Path> {
    #[inline]
    fn clone(&self) -> Self {
        self.to_path_buf().into_boxed_path()
    }
}

impl<T: ?Sized + AsRef<str>> From<&T> for PathBuf {
    #[inline]
    fn from(s: &T) -> PathBuf {
        PathBuf::from(s.as_ref().to_string())
    }
}

impl From<String> for PathBuf {
    #[inline]
    fn from(s: String) -> PathBuf {
        PathBuf { inner: s }
    }
}

impl From<PathBuf> for String {
    #[inline]
    fn from(path_buf: PathBuf) -> String {
        path_buf.inner
    }
}

impl FromStr for PathBuf {
    type Err = core::convert::Infallible;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(PathBuf::from(s))
    }
}

impl<P: AsRef<Path>> FromIterator<P> for PathBuf {
    fn from_iter<I: IntoIterator<Item = P>>(iter: I) -> PathBuf {
        let mut buf = PathBuf::new();
        buf.extend(iter);
        buf
    }
}

impl<P: AsRef<Path>> Extend<P> for PathBuf {
    fn extend<I: IntoIterator<Item = P>>(&mut self, iter: I) {
        iter.into_iter().for_each(move |p| self.push(p.as_ref()));
    }

    #[inline]
    fn extend_one(&mut self, p: P) {
        self.push(p.as_ref());
    }
}

impl fmt::Debug for PathBuf {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, formatter)
    }
}

impl fmt::Display for PathBuf {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, formatter)
    }
}

impl Deref for PathBuf {
    type Target = Path;
    #[inline]
    fn deref(&self) -> &Path {
        Path::new(&self.inner)
    }
}

impl DerefMut for PathBuf {
    #[inline]
    fn deref_mut(&mut self) -> &mut Path {
        Path::from_inner_mut(&mut self.inner)
    }
}

impl Borrow<Path> for PathBuf {
    #[inline]
    fn borrow(&self) -> &Path {
        self.deref()
    }
}

impl Default for PathBuf {
    #[inline]
    fn default() -> Self {
        PathBuf::new()
    }
}

impl<'a> From<&'a Path> for Cow<'a, Path> {
    #[inline]
    fn from(s: &'a Path) -> Cow<'a, Path> {
        Cow::Borrowed(s)
    }
}

impl<'a> From<PathBuf> for Cow<'a, Path> {
    #[inline]
    fn from(s: PathBuf) -> Cow<'a, Path> {
        Cow::Owned(s)
    }
}

impl<'a> From<&'a PathBuf> for Cow<'a, Path> {
    #[inline]
    fn from(p: &'a PathBuf) -> Cow<'a, Path> {
        Cow::Borrowed(p.as_path())
    }
}

impl<'a> From<Cow<'a, Path>> for PathBuf {
    #[inline]
    fn from(p: Cow<'a, Path>) -> Self {
        p.into_owned()
    }
}

impl From<PathBuf> for Arc<Path> {
    #[inline]
    fn from(s: PathBuf) -> Arc<Path> {
        let arc: Arc<str> = Arc::from(s.into_string());
        unsafe { Arc::from_raw(Arc::into_raw(arc) as *const Path) }
    }
}

impl From<&Path> for Arc<Path> {
    /// Converts a [`Path`] into an [`Arc`] by copying the [`Path`] data into a
    /// new [`Arc`] buffer.
    #[inline]
    fn from(s: &Path) -> Arc<Path> {
        let arc: Arc<str> = Arc::from(s.as_str());
        unsafe { Arc::from_raw(Arc::into_raw(arc) as *const Path) }
    }
}

impl From<PathBuf> for Rc<Path> {
    /// Converts a [`PathBuf`] into an <code>[Rc]<[Path]></code> by moving the
    /// [`PathBuf`] data into a new [`Rc`] buffer.
    #[inline]
    fn from(s: PathBuf) -> Rc<Path> {
        let rc: Rc<str> = Rc::from(s.into_string());
        unsafe { Rc::from_raw(Rc::into_raw(rc) as *const Path) }
    }
}

impl From<&Path> for Rc<Path> {
    #[inline]
    fn from(s: &Path) -> Rc<Path> {
        let rc: Rc<str> = Rc::from(s.as_str());
        unsafe { Rc::from_raw(Rc::into_raw(rc) as *const Path) }
    }
}

impl ToOwned for Path {
    type Owned = PathBuf;
    #[inline]
    fn to_owned(&self) -> PathBuf {
        self.to_path_buf()
    }
    #[inline]
    fn clone_into(&self, target: &mut PathBuf) {
        self.inner.clone_into(&mut target.inner);
    }
}

impl PartialEq for PathBuf {
    #[inline]
    fn eq(&self, other: &PathBuf) -> bool {
        self.components() == other.components()
    }
}

impl Hash for PathBuf {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.as_path().hash(h)
    }
}

impl Eq for PathBuf {}

impl PartialOrd for PathBuf {
    #[inline]
    fn partial_cmp(&self, other: &PathBuf) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathBuf {
    #[inline]
    fn cmp(&self, other: &PathBuf) -> cmp::Ordering {
        compare_components(self.components(), other.components())
    }
}

impl AsRef<str> for PathBuf {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.inner[..]
    }
}

pub struct Path {
    inner: str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StripPrefixError(());

impl Path {
    pub fn new<S: AsRef<str> + ?Sized>(s: &S) -> &Path {
        unsafe { &*(s.as_ref() as *const str as *const Path) }
    }

    pub fn from_inner_mut(s: &mut str) -> &mut Path {
        unsafe { &mut *(s as *mut str as *mut Path) }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    #[inline]
    pub fn as_mut_str(&mut self) -> &mut str {
        &mut self.inner
    }

    pub fn to_path_buf(&self) -> PathBuf {
        PathBuf {
            inner: self.inner.to_owned(),
        }
    }

    pub fn is_absolute(&self) -> bool {
        self.has_root()
    }

    pub fn is_relative(&self) -> bool {
        !self.is_absolute()
    }

    fn prefix(&self) -> Option<Prefix<'_>> {
        self.components().prefix
    }

    pub fn has_root(&self) -> bool {
        self.components().has_root()
    }

    pub fn parent(&self) -> Option<&Path> {
        let mut comps = self.components();
        let comp = comps.next_back();
        comp.and_then(|p| match p {
            Component::Normal(_) | Component::CurDir | Component::ParentDir => {
                Some(comps.as_path())
            }
            _ => None,
        })
    }

    pub fn ancestors(&self) -> Ancestors<'_> {
        Ancestors { next: Some(self) }
    }

    pub fn file_name(&self) -> Option<&str> {
        self.components().next_back().and_then(|p| match p {
            Component::Normal(p) => Some(p),
            _ => None,
        })
    }

    pub fn strip_prefix<P>(&self, base: P) -> Result<&Path, StripPrefixError>
    where
        P: AsRef<Path>,
    {
        self._strip_prefix(base.as_ref())
    }

    fn _strip_prefix(&self, base: &Path) -> Result<&Path, StripPrefixError> {
        iter_after(self.components(), base.components())
            .map(|c| c.as_path())
            .ok_or(StripPrefixError(()))
    }

    pub fn starts_with<P: AsRef<Path>>(&self, base: P) -> bool {
        self._starts_with(base.as_ref())
    }

    fn _starts_with(&self, base: &Path) -> bool {
        iter_after(self.components(), base.components()).is_some()
    }

    pub fn ends_with<P: AsRef<Path>>(&self, child: P) -> bool {
        self._ends_with(child.as_ref())
    }

    fn _ends_with(&self, child: &Path) -> bool {
        iter_after(self.components().rev(), child.components().rev()).is_some()
    }

    pub fn file_stem(&self) -> Option<&str> {
        self.file_name()
            .map(rsplit_file_at_dot)
            .and_then(|(before, after)| before.or(after))
    }

    pub fn file_prefix(&self) -> Option<&str> {
        self.file_name()
            .map(split_file_at_dot)
            .map(|(before, _after)| before)
    }

    pub fn extension(&self) -> Option<&str> {
        self.file_name()
            .map(rsplit_file_at_dot)
            .and_then(|(before, after)| before.and(after))
    }

    pub fn join<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self._join(path.as_ref())
    }

    fn _join(&self, path: &Path) -> PathBuf {
        let mut buf = self.to_path_buf();
        buf.push(path);
        buf
    }

    pub fn with_file_name<S: AsRef<str>>(&self, file_name: S) -> PathBuf {
        self._with_file_name(file_name.as_ref())
    }

    fn _with_file_name(&self, file_name: &str) -> PathBuf {
        let mut buf = self.to_path_buf();
        buf.set_file_name(file_name);
        buf
    }

    pub fn with_extension<S: AsRef<str>>(&self, extension: S) -> PathBuf {
        self._with_extension(extension.as_ref())
    }

    fn _with_extension(&self, extension: &str) -> PathBuf {
        let self_len = self.as_str().len();
        let self_bytes = self.as_str().as_bytes();

        let (new_capacity, slice_to_copy) = match self.extension() {
            None => {
                // Enough capacity for the extension and the dot
                let capacity = self_len + extension.len() + 1;
                let whole_path = self_bytes.iter();
                (capacity, whole_path)
            }
            Some(previous_extension) => {
                let capacity = self_len + extension.len() - previous_extension.len();
                let path_till_dot = self_bytes[..self_len - previous_extension.len()].iter();
                (capacity, path_till_dot)
            }
        };

        let mut new_path = PathBuf::with_capacity(new_capacity);
        new_path.as_mut_vec().extend(slice_to_copy);
        new_path.set_extension(extension);
        new_path
    }

    pub fn components(&self) -> Components<'_> {
        let prefix = parse_windows_path_prefix(self.as_str());
        let prefix = prefix.ok().map(|(_, prefix)| prefix);
        Components {
            path: self.as_str(),
            has_physical_root: has_physical_root(self.as_str(), prefix),
            prefix,
            front: State::Prefix,
            back: State::Body,
        }
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            inner: self.components(),
        }
    }

    pub fn into_path_buf(self: Box<Path>) -> PathBuf {
        let rw = Box::into_raw(self) as *mut str;
        let inner = unsafe { Box::from_raw(rw) };
        PathBuf {
            inner: String::from(inner),
        }
    }

    pub fn get_main_sep(&self) -> char {
        if let Ok((_, prefix)) = parse_windows_path_prefix(self.as_str()) {
            if prefix.is_verbatim() {
                return '\\';
            }
        }
        '/'
    }

    pub fn get_main_sep_str(&self) -> &'static str {
        if let Ok((_, prefix)) = parse_windows_path_prefix(self.as_str()) {
            if prefix.is_verbatim() {
                return r"\\";
            }
        }
        "/"
    }

    pub fn to_file_url(&self) -> Result<Url, PathToUrlError> {
        let mut serialization = "file://".to_owned();
        let _host_start = serialization.len() as u32;
        let (_host_end, _host) = path_to_file_url_segments(self, &mut serialization)?;
        let u = Url::parse(&serialization)?;
        Ok(u)
    }
}

impl AsRef<str> for Path {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl fmt::Debug for Path {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, formatter)
    }
}

impl fmt::Display for Path {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, formatter)
    }
}

impl PartialEq for Path {
    #[inline]
    fn eq(&self, other: &Path) -> bool {
        self.components() == other.components()
    }
}

impl Hash for Path {
    fn hash<H: Hasher>(&self, h: &mut H) {
        let path_str = self.as_str();
        let path_bytes = path_str.as_bytes();
        let (prefix_len, verbatim) = match parse_windows_path_prefix(path_str) {
            Ok((_, prefix)) => {
                prefix.hash(h);
                (prefix.len(), prefix.is_verbatim())
            }
            _ => (0, false),
        };
        let bytes = &path_bytes[prefix_len..];

        let mut component_start = 0;
        let mut bytes_hashed = 0;

        for i in 0..bytes.len() {
            let is_sep = if verbatim {
                is_verbatim_sep(bytes[i] as char)
            } else {
                is_sep_byte(bytes[i] as char)
            };
            if is_sep {
                if i > component_start {
                    let to_hash = &bytes[component_start..i];
                    h.write(to_hash);
                    bytes_hashed += to_hash.len();
                }

                // skip over separator and optionally a following CurDir item
                // since components() would normalize these away.
                component_start = i + 1;

                let tail = &bytes[component_start..];

                if !verbatim {
                    component_start += match tail {
                        [b'.'] => 1,
                        [b'.', sep, ..] if is_sep_byte(*sep as char) => 1,
                        _ => 0,
                    };
                }
            }
        }

        if component_start < bytes.len() {
            let to_hash = &bytes[component_start..];
            h.write(to_hash);
            bytes_hashed += to_hash.len();
        }

        h.write_usize(bytes_hashed);
    }
}

impl Eq for Path {}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for Path {
    #[inline]
    fn partial_cmp(&self, other: &Path) -> Option<cmp::Ordering> {
        Some(compare_components(self.components(), other.components()))
    }
}

impl Ord for Path {
    #[inline]
    fn cmp(&self, other: &Path) -> cmp::Ordering {
        compare_components(self.components(), other.components())
    }
}

impl AsRef<Path> for Path {
    #[inline]
    fn as_ref(&self) -> &Path {
        self
    }
}

impl AsRef<Path> for str {
    #[inline]
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl AsRef<Path> for Cow<'_, str> {
    #[inline]
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl AsRef<Path> for String {
    #[inline]
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl AsRef<Path> for PathBuf {
    #[inline]
    fn as_ref(&self) -> &Path {
        self
    }
}

impl<'a> IntoIterator for &'a PathBuf {
    type Item = &'a str;
    type IntoIter = Iter<'a>;
    #[inline]
    fn into_iter(self) -> Iter<'a> {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a Path {
    type Item = &'a str;
    type IntoIter = Iter<'a>;
    #[inline]
    fn into_iter(self) -> Iter<'a> {
        self.iter()
    }
}

macro_rules! impl_cmp {
    (<$($life:lifetime),*> $lhs:ty, $rhs: ty) => {
        impl<$($life),*> PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                <Path as PartialEq>::eq(self, other)
            }
        }

        impl<$($life),*> PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool {
                <Path as PartialEq>::eq(self, other)
            }
        }

        impl<$($life),*> PartialOrd<$rhs> for $lhs {
            #[inline]
            fn partial_cmp(&self, other: &$rhs) -> Option<cmp::Ordering> {
                <Path as PartialOrd>::partial_cmp(self, other)
            }
        }

        impl<$($life),*> PartialOrd<$lhs> for $rhs {
            #[inline]
            fn partial_cmp(&self, other: &$lhs) -> Option<cmp::Ordering> {
                <Path as PartialOrd>::partial_cmp(self, other)
            }
        }
    };
}

impl_cmp!(<> PathBuf, Path);
impl_cmp!(<'a> PathBuf, &'a Path);
impl_cmp!(<'a> Cow<'a, Path>, Path);
impl_cmp!(<'a, 'b> Cow<'a, Path>, &'b Path);
impl_cmp!(<'a> Cow<'a, Path>, PathBuf);

macro_rules! impl_cmp_str {
    (<$($life:lifetime),*> $lhs:ty, $rhs: ty) => {
        impl<$($life),*> PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                <Path as PartialEq>::eq(self, other.as_ref())
            }
        }

        impl<$($life),*> PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool {
                <Path as PartialEq>::eq(self.as_ref(), other)
            }
        }

        impl<$($life),*> PartialOrd<$rhs> for $lhs {
            #[inline]
            fn partial_cmp(&self, other: &$rhs) -> Option<cmp::Ordering> {
                <Path as PartialOrd>::partial_cmp(self, other.as_ref())
            }
        }

        impl<$($life),*> PartialOrd<$lhs> for $rhs {
            #[inline]
            fn partial_cmp(&self, other: &$lhs) -> Option<cmp::Ordering> {
                <Path as PartialOrd>::partial_cmp(self.as_ref(), other)
            }
        }
    };
}

impl_cmp_str!(<> PathBuf, str);
impl_cmp_str!(<'a> PathBuf, &'a str);
impl_cmp_str!(<'a> PathBuf, Cow<'a, str>);
impl_cmp_str!(<> PathBuf, String);
impl_cmp_str!(<> Path, str);
impl_cmp_str!(<'a> Path, &'a str);
impl_cmp_str!(<'a> Path, Cow<'a, str>);
impl_cmp_str!(<> Path, String);
impl_cmp_str!(<'a> &'a Path, str);
impl_cmp_str!(<'a, 'b> &'a Path, Cow<'b, str>);
impl_cmp_str!(<'a> &'a Path, String);

impl fmt::Display for StripPrefixError {
    #[allow(deprecated, deprecated_in_future)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.description().fmt(f)
    }
}

impl Error for StripPrefixError {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        "prefix not found"
    }
}

pub fn absolute<P: AsRef<Path>>(path: P) -> io::Result<PathBuf> {
    let path = path.as_ref();
    let path_str = path.as_str();
    if path_str.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "cannot make an empty path absolute",
        ));
    }
    let prefix = parse_windows_path_prefix(path_str);

    if prefix.map(|(_, p)| p.is_verbatim()).unwrap_or(false) {
        if path_str.contains('\0') {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "strings passed to WinAPI cannot contain NULs",
            ));
        }
        return Ok(PathBuf::from(path_str.to_owned()));
    }

    let mut components = path.strip_prefix(".").unwrap_or(path).components();

    let path_str = path.as_str();
    let mut normalized = if path.is_absolute() {
        if path_str.starts_with("//") && !path_str.starts_with("///") {
            components.next();
            PathBuf::from("//")
        } else {
            PathBuf::new()
        }
    } else {
        PathBuf::new()
    };
    normalized.extend(components);

    if path_str.ends_with('/') || path_str.ends_with('\\') {
        normalized.push("");
    }

    Ok(normalized)
}

#[cfg(test)]
mod tests {

    use url::Url;

    use crate::{url::PathToUrlError, Path, PathBuf};

    #[test]
    fn test_black_slash_and_slash_mix_path() {
        let mut path = PathBuf::from(r"\\?\C:\Users\test\Desktop/1.txt");
        path.push("abc");
        assert_eq!(path.as_str(), r"\\?\C:\Users\test\Desktop/1.txt\abc");
    }

    #[test]
    fn test_to_file_url() {
        let test_fn = |source: &str| -> Result<String, PathToUrlError> {
            let p = Path::new(source);
            let u: Url = p.to_file_url()?;
            Ok(u.to_string())
        };
        assert_eq!(
            test_fn(r"\\?\UNC\server\share\path").expect("parse success"),
            "file://server/share/path"
        );
        assert_eq!(
            test_fn(r"\\?\C:\path").expect("parse success"),
            "file:///C:/path"
        );
        assert_eq!(
            test_fn(r"\\server\share\path").expect("parse success"),
            "file://server/share/path"
        );
        assert_eq!(
            test_fn(r"C:\path").expect("parse success"),
            "file:///C:/path"
        );
        assert!(matches!(
            test_fn(r"\\?\abc\path"),
            Err(PathToUrlError::UrlNotSupportedPrefix { .. })
        ));
        assert!(matches!(
            test_fn(r"\\.\device\path"),
            Err(PathToUrlError::UrlNotSupportedPrefix { .. })
        ));
        assert!(matches!(
            test_fn(r"~/a"),
            Err(PathToUrlError::PathNotAbsoluteError { .. })
        ));
    }
}

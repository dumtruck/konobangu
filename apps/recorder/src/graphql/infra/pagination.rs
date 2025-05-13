use async_graphql::{InputObject, SimpleObject};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, InputObject)]
pub struct CursorInput {
    pub cursor: Option<String>,
    pub limit: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, InputObject)]
pub struct PageInput {
    pub page: u64,
    pub limit: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, InputObject)]
pub struct OffsetInput {
    pub offset: u64,
    pub limit: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, InputObject)]
pub struct PaginationInput {
    pub cursor: Option<CursorInput>,
    pub page: Option<PageInput>,
    pub offset: Option<OffsetInput>,
}

pub type PageInfo = async_graphql::connection::PageInfo;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, SimpleObject)]
pub struct PaginationInfo {
    pub pages: u64,
    pub current: u64,
    pub offset: u64,
    pub total: u64,
}

use sea_orm::sea_query;

#[derive(sea_query::Iden)]
pub enum ApalisJob {
    Table,
    Id,
}

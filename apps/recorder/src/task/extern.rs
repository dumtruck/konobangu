use sea_orm::sea_query;

#[derive(sea_query::Iden)]

pub enum ApalisSchema {
    #[iden = "apalis"]
    Schema,
}

#[derive(sea_query::Iden)]

pub enum ApalisJobs {
    #[iden = "jobs"]
    Table,
    Id,
}

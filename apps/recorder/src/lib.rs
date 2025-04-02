#![feature(
    duration_constructors,
    assert_matches,
    unboxed_closures,
    impl_trait_in_bindings,
    iterator_try_collect,
    async_fn_traits,
    let_chains,
    error_generic_member_access
)]
#![feature(associated_type_defaults)]

pub mod app;
pub mod auth;
pub mod cache;
pub mod database;
pub mod downloader;
pub mod errors;
pub mod extract;
pub mod fetch;
pub mod graphql;
pub mod logger;
pub mod migrations;
pub mod models;
pub mod storage;
pub mod tasks;
#[cfg(test)]
pub mod test_utils;
pub mod utils;
pub mod web;

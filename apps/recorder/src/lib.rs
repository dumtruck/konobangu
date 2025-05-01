#![feature(
    duration_constructors,
    assert_matches,
    unboxed_closures,
    impl_trait_in_bindings,
    iterator_try_collect,
    async_fn_traits,
    error_generic_member_access,
    associated_type_defaults,
    let_chains
)]
pub use downloader;

pub mod app;
pub mod auth;
pub mod cache;
pub mod crypto;
pub mod database;
pub mod errors;
pub mod extract;
pub mod graphql;
pub mod logger;
pub mod migrations;
pub mod models;
pub mod storage;
pub mod tasks;
#[cfg(test)]
pub mod test_utils;
pub mod web;

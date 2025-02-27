#![feature(
    duration_constructors,
    assert_matches,
    unboxed_closures,
    impl_trait_in_bindings,
    iterator_try_collect,
    async_fn_traits,
    let_chains
)]

pub mod app;
pub mod auth;
pub mod cache;
pub mod controllers;
pub mod errors;
pub mod extract;
pub mod fetch;
pub mod graphql;
pub mod migrations;
pub mod models;
pub mod storage;
pub mod sync;
pub mod tasks;
#[cfg(test)]
pub mod test_utils;
pub mod views;

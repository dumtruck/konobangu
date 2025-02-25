#![feature(
    duration_constructors,
    assert_matches,
    unboxed_closures,
    impl_trait_in_bindings,
    iterator_try_collect
)]

pub mod app;
pub mod auth;
pub mod config;
pub mod controllers;
pub mod dal;
pub mod errors;
pub mod extract;
pub mod fetch;
pub mod graphql;
pub mod migrations;
pub mod models;
pub mod sync;
pub mod tasks;
#[cfg(test)]
pub mod test_utils;
pub mod views;
pub mod workers;

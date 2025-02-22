#![feature(duration_constructors, assert_matches, unboxed_closures)]

pub mod app;
pub mod auth;
pub mod config;
pub mod controllers;
pub mod dal;
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

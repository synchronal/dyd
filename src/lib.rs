#![cfg_attr(feature = "strict", deny(warnings))]

//! DYD is a CLI for diffing multiple git repositories.
//!
//! It relies on the shell for authentication to remote origins, and uses
//! the configured GUI git difftool for opening diffs.
pub mod app;
pub mod cli;
pub mod config;
pub mod difftool;
pub mod git;
pub mod manifest;
pub mod terminal;
pub mod time;
pub mod ui;
pub mod widget;

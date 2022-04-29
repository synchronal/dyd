//! DYD is a CLI for diffing multiple git repositories.
//!
//! It relies on the shell for authentication to remote origins, and uses
//! the configured GUI git difftool for opening diffs.
pub mod app;
pub mod cli;
pub mod event;
pub mod git;
pub mod handler;
pub mod manifest;
pub mod repo;
pub mod time;
pub mod tui;
pub mod ui;

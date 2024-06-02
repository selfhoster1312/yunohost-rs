//! # yunohost
//!
//! [Yunohost](https://yunohost.org/) is a self-hosting distribution based on Debian to easily manage a server for your friends, association, or entreprise.
//!
//! This crate contains a Rust reimplementation of the core Yunohost concepts/structures to write components that integrate with Yunohost in Rust. If you're looking for the
//! scope and reasons for this project, it's all explained in the [README.md](https://github.com/selfhoster1312/yunohost-rs) file.
//!
//! **NOTE:** This is not an official Yunohost project and is not supported by Yunohost. This is a very young project and may break things. Please do not use this on a server you care about, and do not address the Yunohost project any support request if something breaks.

#[macro_use]
extern crate log;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate snafu;

/// The Yunohost CLI subcommands
pub mod cmd;
/// The top-level error cases
pub mod error;
/// High-level helpers to faciliate your life
pub mod helpers;
// /// [moulinette](https://github.com/YunoHost/moulinette/) reimplementation for i18n
// pub mod moulinette;

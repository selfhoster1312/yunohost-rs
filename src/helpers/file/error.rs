use camino::Utf8PathBuf;
use snafu::prelude::*;

use crate::helpers::file::StrPath;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum FileError {
    // StrPath::from_str
    #[snafu(display(
        "Failed to parse path because it's not valid UTF-8. It's approximately: {path}"
    ))]
    PathUnicode { path: String },

    // StrPath::owner_get, StrPath::group_get
    #[snafu(display("Failed to read ownership/group metadata for path: {path}"))]
    PathOwnershipMetadata {
        path: StrPath,
        source: file_owner::FileOwnerError,
    },

    // StrPath::owner_get
    #[snafu(display("Failed to lookup user information for owner of path: {path}"))]
    PathOwnershipUser {
        path: StrPath,
        source: file_owner::FileOwnerError,
    },

    // StrPath::owner_get
    #[snafu(display("Failed to find username for owner of path: {path}"))]
    PathOwnershipUserNotFound { path: StrPath },

    // StrPath::owner_set
    #[snafu(display("Failed to set owner {owner} for path {path}"))]
    PathOwnershipSetUser {
        owner: String,
        path: StrPath,
        source: file_owner::FileOwnerError,
    },

    // StrPath::group_get
    #[snafu(display("Failed to lookup group information for group of path: {path}"))]
    PathOwnershipGroup {
        path: StrPath,
        source: file_owner::FileOwnerError,
    },

    // StrPath::group_get
    #[snafu(display("Failed to find group name for group of path: {path}"))]
    PathOwnershipGroupNotFound { path: StrPath },

    // StrPath::group_set
    #[snafu(display("Failed to set group {group} for path {path}"))]
    PathOwnershipSetGroup {
        group: String,
        path: StrPath,
        source: file_owner::FileOwnerError,
    },

    // StrPath::mode_get
    #[snafu(display("Failed to read permissions (mode) for path: {path}"))]
    PathMode {
        path: StrPath,
        source: std::io::Error,
    },

    // StrPath::mode_set
    #[snafu(display("Failed to set permissions (mode) to {mode:o} for path: {path}"))]
    PathModeSet {
        mode: u32,
        path: StrPath,
        source: std::io::Error,
    },

    // StrPath::chown
    #[snafu(display("Failed to chown path {path} to {owner}:{group}"))]
    PathChown {
        path: StrPath,
        owner: String,
        group: String,
        #[snafu(source(from(FileError, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    // StrPath::chown
    #[snafu(display("Failed to chown {owner}:{group} for path: {path}"))]
    PathChownSet {
        owner: String,
        group: String,
        path: StrPath,
        source: file_owner::FileOwnerError,
    },

    // StrPath::mkdir_p
    #[snafu(display("Failed to create recursive directory (mkdir -p) until path: {path}"))]
    PathMkdirP {
        path: StrPath,
        source: std::io::Error,
    },

    // StrPath::copy_to
    #[snafu(display("Cannot copy {path} to {dest} because it is not a directory!"))]
    PathCopyToNonDir { path: StrPath, dest: StrPath },

    // StrPath::copy_to
    #[snafu(display("Failed to copy {path} to {dest}."))]
    PathCopyFail {
        path: StrPath,
        dest: StrPath,
        source: std::io::Error,
    },

    // StrPath::read
    #[snafu(display("read failed to read {path}"))]
    PathRead {
        path: StrPath,
        source: std::io::Error,
    },

    // StrPath::symlink_to_target
    #[snafu(display("symlink failed to create a symlink to {target} due to failing to remove existing file: {link}"))]
    PathSymlinkRemove {
        target: StrPath,
        link: StrPath,
        #[snafu(source(from(FileError, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    // StrPath::symlink_to_target
    #[snafu(display("symlink to create a symlink to {target} at: {link}"))]
    PathSymlinkCreate {
        target: StrPath,
        link: StrPath,
        source: std::io::Error,
    },

    // StrPath::file_remove
    #[snafu(display("file_remove failed to remove file {path}"))]
    PathFileRemove {
        path: StrPath,
        source: std::io::Error,
    },

    // StrPath::canonicalize
    #[snafu(display("canonicalize failed to resolve links of {path}"))]
    PathCanonicalize {
        path: StrPath,
        source: std::io::Error,
    },

    // StrPath::canonicalize
    #[snafu(display("Failed to read link from {link} because the target is not valid UTF-8"))]
    PathCanonicalizeParse {
        link: StrPath,
        #[snafu(source(from(FileError, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    // StrPath::read_link
    #[snafu(display("Failed to read_link on path because it's not a symlink: {path}"))]
    PathReadLinkNotSymlink { path: StrPath },

    // StrPath::read_link
    #[snafu(display("Failed to read_link on path: {path}"))]
    PathReadLink {
        path: StrPath,
        source: std::io::Error,
    },

    // StrPath::read_link
    #[snafu(display("Failed to read_link on path: {link}"))]
    PathReadLinkParse {
        link: StrPath,
        #[snafu(source(from(FileError, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[snafu(display("Failed to parse TOML from string"))]
    Toml { source: toml::de::Error },

    // StrPath::read_toml
    PathTomlRead {
        path: StrPath,
        #[snafu(source(from(FileError, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[snafu(display("Failed to parse YAML from string"))]
    Yaml { source: serde_yaml_ng::Error },

    // StrPath::read_yaml
    PathYamlRead {
        path: StrPath,
        #[snafu(source(from(FileError, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[snafu(display("Failed to parse JSON from string"))]
    Json { source: serde_json::Error },

    // StrPath::read_json
    PathJsonRead {
        path: StrPath,
        #[snafu(source(from(FileError, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    // ensure_file_remove
    #[snafu(display("ensure_file_remove failed to remove file {path}"))]
    EnsureFileRemove {
        path: StrPath,
        #[snafu(source(from(FileError, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[snafu(display("read_dir failed to read {}", path))]
    ReadDir {
        path: Utf8PathBuf,
        source: std::io::Error,
    },

    //     fn glob
    #[snafu(display("glob invalid pattern: {}", pattern))]
    GlobPattern {
        pattern: String,
        source: glob::PatternError,
    },

    // fn glob
    #[snafu(display("glob invalid read: {}", source.path().display()))]
    Glob { source: glob::GlobError },
}

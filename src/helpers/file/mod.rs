//! File manipulation helpers.
//!
//! In this module, [`path`] will help the Python afficionados create path objects,
//! and the [`StrPath`] class contains helper methods to interact with those path objects.

use camino::{Utf8Path, Utf8PathBuf};
use derive_deref::Deref;
use file_owner::PathExt;
use serde::Deserialize;
use snafu::prelude::*;

use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;

pub mod error;
use error::*;

/// A high-level wrapper for paths.
///
/// This type enables higher-level convenience methods for operations like read_dir (TODO), as well
/// as integration with Yunohost [`FileError`]. Inside is actually a [`Utf8PathBuf`],
/// and so it can also be used both as a standard library [`&str`] and [`Path`].
///
/// `StrPath` can be build from any stringy value, like so:
///
/// ```rust
/// let path = StrPath::from("/etc/yunohost");
/// ```
// ASYNC NOTE: If we ever go async, this entire struct needs to be rewritten with async primitives...
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Deref)]
pub struct StrPath(Utf8PathBuf);

impl StrPath {
    /// Build a new stringy path from a string slice. For a more lax method, use [`StrPath::from`].
    pub fn new(path: &str) -> StrPath {
        StrPath(Utf8PathBuf::from(path))
    }

    /// Tries to parse a real Path into a StrPath. Fails if the path is not valid unicode.
    pub fn from_path(path: &Path) -> Result<Self, FileError> {
        if let Some(path) = path.to_str() {
            Ok(Self::new(path))
        } else {
            Err(FileError::PathUnicode {
                path: path.to_string_lossy().to_string(),
            })
        }
    }

    /// Return a reference to the underlying UTF8 path.
    pub fn as_path(&self) -> &Utf8Path {
        self.0.as_path()
    }

    /// Lookup ownership information for this path.
    pub fn owner_get(&self) -> Result<String, FileError> {
        let owner = self
            .as_path()
            .owner()
            .context(PathOwnershipMetadataSnafu { path: self.clone() })?
            .name()
            .context(PathOwnershipUserSnafu { path: self.clone() })?
            .context(PathOwnershipUserNotFoundSnafu { path: self.clone() })?;

        Ok(owner)
    }

    /// Sets ownership information for this path. For both owner/group setting, use [`chown`](Self::chown).
    pub fn owner_set(&self, owner: &str) -> Result<(), FileError> {
        self.set_owner(owner).context(PathOwnershipSetUserSnafu {
            owner: owner.to_string(),
            path: self.clone(),
        })
    }

    /// Lookup group information for this path.
    pub fn group_get(&self) -> Result<String, FileError> {
        let group = self
            .as_path()
            .group()
            .context(PathOwnershipMetadataSnafu { path: self.clone() })?
            .name()
            .context(PathOwnershipGroupSnafu { path: self.clone() })?
            .context(PathOwnershipGroupNotFoundSnafu { path: self.clone() })?;

        Ok(group)
    }

    /// Sets group information for this path. For both owner/group setting, use [`chown`](Self::chown).
    pub fn group_set(&self, group: &str) -> Result<(), FileError> {
        self.set_group(group).context(PathOwnershipSetGroupSnafu {
            group: group.to_string(),
            path: self.clone(),
        })
    }

    /// Lookup mode information for this path.
    pub fn mode_get(&self) -> Result<u32, FileError> {
        let meta = fs::metadata(&self).context(PathModeSnafu { path: self.clone() })?;
        Ok(meta.mode())
    }

    /// Sets mode information for this path.
    pub fn mode_set(&self, mode: u32) -> Result<(), FileError> {
        fs::set_permissions(&self, fs::Permissions::from_mode(mode)).context(PathModeSetSnafu {
            path: self.clone(),
            mode,
        })
    }

    /// Sets both owner and group information for this path. Use [`owner_set`](Self::owner_set) or [`group_set`](Self::group_set)
    /// if that's not what you want.
    pub fn chown(&self, req_owner: &str, req_group: &str) -> Result<(), FileError> {
        let (owner, group) = self
            .owner_group()
            .context(PathOwnershipMetadataSnafu { path: self.clone() })
            .context(PathChownSnafu {
                owner: req_owner.to_string(),
                group: req_group.to_string(),
                path: self.clone(),
            })?;

        // TODO: this is a bit long, replacing file_owner crate would make it easier to articulate
        let found_owner = owner
            .name()
            .context(PathOwnershipUserSnafu { path: self.clone() })
            // Wrap in outer context so we don't need to duplicate error types
            .context(PathChownSnafu {
                owner: req_owner.to_string(),
                group: req_group.to_string(),
                path: self.clone(),
            })?
            // Now we have to unwrap the inner option
            .context(PathOwnershipUserNotFoundSnafu { path: self.clone() })
            // Wrap in outer context so we don't need to duplicate error types
            .context(PathChownSnafu {
                owner: req_owner.to_string(),
                group: req_group.to_string(),
                path: self.clone(),
            })?;

        // TODO: this is a bit long, replacing file_owner crate would make it easier to articulate
        let found_group = group
            .name()
            .context(PathOwnershipGroupSnafu { path: self.clone() })
            // Wrap in outer context so we don't need to duplicate error types
            .context(PathChownSnafu {
                owner: req_owner.to_string(),
                group: req_group.to_string(),
                path: self.clone(),
            })?
            // Now we have to unwrap the inner option
            .context(PathOwnershipGroupNotFoundSnafu { path: self.clone() })
            // Wrap in outer context so we don't need to duplicate error types
            .context(PathChownSnafu {
                owner: req_owner.to_string(),
                group: req_group.to_string(),
                path: self.clone(),
            })?;

        if found_owner != req_owner || found_group != req_group {
            self.set_owner_group(req_owner, req_group)
                .context(PathChownSetSnafu {
                    owner: req_owner.to_string(),
                    group: req_group.to_string(),
                    path: self.clone(),
                })?;
        }

        Ok(())
    }

    /// Set mode and ownership information at the same time.
    pub fn chown_and_mode(
        &self,
        mode: u32,
        user: &str,
        group: Option<&str>,
    ) -> Result<(), FileError> {
        self.mode_set(mode)?;
        if let Some(group) = group {
            self.chown(user, group)?;
        } else {
            self.owner_set(user)?;
        }

        Ok(())
    }

    /// Recursive chown operation
    pub fn chown_recurse(&self, owner: &str, group: &str) -> Result<(), FileError> {
        if self.is_dir() {
            // TODO: unwrap needs readdir...
            for entry in fs::read_dir(self).unwrap() {
                // TODO: change this re-parsing here...
                let entry = StrPath::from(entry.unwrap().path().to_str().unwrap());
                entry.chown_recurse(owner, group)?;
            }
        } else {
            self.chown(owner, group)?;
        }

        Ok(())
    }

    /// Wrapper for [`Self::mode_set`]
    pub fn chmod(&self, mode: u32) -> Result<(), FileError> {
        self.mode_set(mode)
    }

    /// Recursive chmod operation.
    pub fn chmod_recurse(&self, mode: u32) -> Result<(), FileError> {
        if self.is_dir() {
            // TODO: unwrap here needs readdir integration
            for entry in fs::read_dir(self).unwrap() {
                // TODO
                let entry = StrPath::from(entry.unwrap().path().to_str().unwrap());
                entry.chmod_recurse(mode)?;
            }
        } else {
            self.mode_set(mode)?;
        }

        Ok(())
    }

    /// Ensures the path exists, creating parent directories if necessary.
    ///
    /// Does not error when the folder already exists.
    ///
    /// Errors when:
    /// - the path did not previously exist and failed to be created
    /// - the path previously existed, but was not a directory? (TODO: is this true?)
    pub fn mkdir_p(&self) -> Result<(), FileError> {
        if !self.is_dir() {
            fs::create_dir_all(self).context(PathMkdirPSnafu { path: self.clone() })?;
        }

        Ok(())
    }

    /// Copy to the `dest` folder, keeping the same file/dir name.
    ///
    /// Errors when:
    /// - `dest` is not a directory
    /// - copying to `dest` failed
    pub fn copy_to(&self, dest: &StrPath) -> Result<(), FileError> {
        if !dest.is_dir() {
            return Err(FileError::PathCopyToNonDir {
                path: self.clone(),
                dest: dest.clone(),
            });
        }

        // UNWRAP NOTE: This should be safe, unless the filename is '..'. TODO: Should this be an error case?
        // Or do we prevent it when creating StrPath?
        // See: https://doc.rust-lang.org/stable/std/path/struct.Path.html#method.file_name
        let file_name = self.file_name().unwrap();
        fs::copy(self, dest.join(file_name)).context(PathCopyFailSnafu {
            path: self.clone(),
            dest: dest.clone(),
        })?;

        Ok(())
    }

    /// Reads UTF-8 content to an owned string.
    ///
    /// Errors when:
    /// - the path does not exist or is not a file
    /// - the path is not readable
    /// - file content is not UTF-8
    pub fn read(&self) -> Result<String, FileError> {
        fs::read_to_string(self).context(PathReadSnafu { path: self.clone() })
    }

    /// Reads UTF-8 lines to an owned list of strings.
    ///
    /// Example:
    ///
    /// ```rust
    /// for line in path("/tmp/res.output").read_lines()? {
    ///     info!("{line}");
    /// }
    /// ```
    ///
    /// Errors when:
    /// - the path does not exist or is not a file
    /// - the path is not readable
    /// - file content is not UTF-8
    pub fn read_lines(&self) -> Result<Vec<String>, FileError> {
        Ok(self.read()?.lines().map(String::from).collect())
    }

    /// Creates a symlink on `self` pointing to `target`.
    ///
    /// If `force` is true, an existing file `self` will be deleted first.
    ///
    /// Errors when:
    ///   - `self` exists and `force` is not true
    ///   - creating the symlink failed
    pub fn symlink_to_target(&self, target: &Utf8Path, force: bool) -> Result<(), FileError> {
        if force && self.exists() {
            self.file_remove().context(PathSymlinkRemoveSnafu {
                target: StrPath::from(target),
                link: self.clone(),
            })?;
        }

        std::os::unix::fs::symlink(target, &self).context(PathSymlinkCreateSnafu {
            target: StrPath::from(target),
            link: self.clone(),
        })?;

        Ok(())
    }

    /// Removes an existing file.
    ///
    /// Errors when:
    ///   - the file does not exist
    ///   - the file is not a file, but a directory
    ///   - removing the file failed
    pub fn file_remove(&self) -> Result<(), FileError> {
        fs::remove_file(self).context(PathFileRemoveSnafu { path: self.clone() })?;

        Ok(())
    }

    /// Resolves symlinks until they've reached a stable place on the filesystem.
    ///
    /// Errors when:
    ///   - `path` does not exist
    ///   - an intermediate path is not a directory or does not exist
    ///   - the final path does not exist
    pub fn canonicalize(&self) -> Result<Self, FileError> {
        let target = self
            .0
            .canonicalize()
            .context(PathCanonicalizeSnafu { path: self.clone() })?;

        Self::from_path(&target).context(PathCanonicalizeParseSnafu { link: self.clone() })
    }

    /// Follows a symlink, a single time.
    ///
    /// Does not follow symlinks recursively. Use [`StrPath::canonicalize`] for that.
    ///
    /// Errors when:
    /// - `path` does not exist, or is not a symlink
    pub fn read_link(&self) -> Result<Self, FileError> {
        if !self.is_symlink() {
            return Err(FileError::PathReadLinkNotSymlink { path: self.clone() });
        }

        let target = std::fs::read_link(&self).context(PathReadLinkSnafu { path: self.clone() })?;

        Self::from_path(&target).context(PathReadLinkParseSnafu { link: self.clone() })
    }

    /// Deserialize a TOML file directly to a struct.
    ///
    /// Errors when:
    /// - [`StrPath::read`] fails
    /// - `toml::from_str` fails
    pub fn read_toml<T: for<'a> Deserialize<'a>>(&self) -> Result<T, FileError> {
        let content = self
            .read()
            .context(PathTomlReadSnafu { path: self.clone() })?;
        let value: T = toml::from_str(&content)
            .context(TomlSnafu)
            .context(PathTomlReadSnafu { path: self.clone() })?;
        Ok(value)
    }

    /// Deserialize a YAML file directly to a struct.
    ///
    /// Errors when:
    /// - [`StrPath::read`] fails
    /// - `toml::from_str` fails
    pub fn read_yaml<T: for<'a> Deserialize<'a>>(&self) -> Result<T, FileError> {
        let content = self
            .read()
            .context(PathYamlReadSnafu { path: self.clone() })?;
        let value: T = serde_yaml_ng::from_str(&content)
            .context(YamlSnafu)
            .context(PathYamlReadSnafu { path: self.clone() })?;
        Ok(value)
    }

    /// Deserialize a JSON file directly to a struct.
    ///
    /// Errors when:
    /// - [`StrPath::read`] fails
    /// - `serde_json::from_str` fails
    pub fn read_json<T: for<'a> Deserialize<'a>>(&self) -> Result<T, FileError> {
        let content = self
            .read()
            .context(PathJsonReadSnafu { path: self.clone() })?;
        let value: T = serde_json::from_str(&content)
            .context(JsonSnafu)
            .context(PathJsonReadSnafu { path: self.clone() })?;
        Ok(value)
    }
}

impl<T: AsRef<str>> From<T> for StrPath {
    fn from(path: T) -> StrPath {
        StrPath::new(path.as_ref())
    }
}

impl std::fmt::Display for StrPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl AsRef<Path> for StrPath {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

/// Generate a new path from any valid UTF-8 string.
///
/// [`StrPath`] contains high-level helpers that integrate well
/// with other Yunohost helpers and errors, ignoring weird non-UTF8 files.
///
/// Example:
/// ```rust
/// let p = path("/etc/yunohost");
/// ```
pub fn path<T: AsRef<str>>(path: T) -> StrPath {
    StrPath::from(path.as_ref())
}

/// Make sure a file does not exist.
///
/// Errors when:
///   - removing the file failed
///
/// Does not error when the file does not exist.
pub fn ensure_file_remove<T: AsRef<Path>>(path: T) -> Result<(), FileError> {
    let path = StrPath::from_path(path.as_ref())?;
    if path.is_file() {
        path.file_remove()
            .context(EnsureFileRemoveSnafu { path: path.clone() })?;
    }

    Ok(())
}

/// Lists the paths in a directory. See [`ReadDir::new`] to instantiate it.
pub struct ReadDir {
    paths: Vec<Utf8PathBuf>,
}

impl ReadDir {
    /// Lists the paths in a directory.
    ///
    /// Errors when:
    /// - `path` does not exist or is not a directory
    /// - we don't have sufficient permissions
    /// - intermittent IO errors
    pub fn new<T: AsRef<Utf8Path>>(path: T) -> Result<ReadDir, FileError> {
        let path = path.as_ref().to_path_buf();

        let mut files: Vec<Utf8PathBuf> = vec![];
        for dir_entry in path
            .read_dir_utf8()
            .context(ReadDirSnafu { path: path.clone() })?
        {
            files.push(
                dir_entry
                    .context(ReadDirSnafu { path: path.clone() })?
                    .into_path(),
            );
        }
        Ok(Self { paths: files })
    }

    pub fn paths(self) -> Vec<Utf8PathBuf> {
        self.paths
    }

    /// Returns the names of entries in the read directory.
    pub fn filenames(&self) -> Vec<String> {
        self.paths
            .iter()
            .map(|path| {
                // UNWRAP NOTE: This is safe unwrap because file_name() only fails when the path ends with '/..' which
                // cannot happen in a read_dir which filters those special directories.
                path.file_name().unwrap().to_string()
            })
            .collect()
    }
}

pub fn glob(pattern: &str) -> Result<Vec<Utf8PathBuf>, FileError> {
    let mut files: Vec<Utf8PathBuf> = vec![];

    for entry in glob::glob(pattern).context(GlobPatternSnafu {
        pattern: pattern.to_string(),
    })? {
        let entry = entry.context(GlobSnafu)?;
        let utf8_entry = match Utf8PathBuf::from_path_buf(entry) {
            Ok(p) => p,
            Err(entry_path) => {
                return Err(FileError::PathUnicode {
                    path: entry_path.to_string_lossy().to_string(),
                });
            }
        };
        files.push(utf8_entry);
    }

    Ok(files)
}

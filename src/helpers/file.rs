//! File manipulation helpers.
//!
//! In this module, [`path`] will help the Python afficionados create path objects,
//! and the [`StrPath`] class contains helper methods to interact with those path objects.

use camino::{Utf8Path, Utf8PathBuf};
use derive_deref::Deref;
use file_owner::PathExt;
use snafu::prelude::*;

use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

use crate::error::*;

/// A high-level wrapper for paths.
///
/// This type enables higher-level convenience methods for operations like read_dir (TODO), as well
/// as integration with Yunohost [`Error`]. Inside is actually a [`Utf8PathBuf`],
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

    /// Return a reference to the underlying UTF8 path.
    pub fn as_path(&self) -> &Utf8Path {
        self.0.as_path()
    }

    /// Lookup ownership information for this path.
    pub fn owner_get(&self) -> Result<String, Error> {
        let owner = self
            .as_path()
            .owner()
            .context(PathOwnerMetadataSnafu { path: self.clone() })?
            .name()
            .context(PathOwnerNameSnafu { path: self.clone() })?
            .context(PathOwnerNameNotFoundSnafu { path: self.clone() })?;

        Ok(owner)
    }

    /// Sets ownership information for this path. For both owner/group setting, use [`chown`](Self::chown).
    pub fn owner_set(&self, owner: &str) -> Result<(), Error> {
        self.set_owner(owner).context(PathOwnerSetSnafu {
            owner: owner.to_string(),
            path: self.clone(),
        })
    }

    /// Lookup group information for this path.
    pub fn group_get(&self) -> Result<String, Error> {
        let group = self
            .as_path()
            .group()
            .context(PathGroupMetadataSnafu { path: self.clone() })?
            .name()
            .context(PathGroupNameSnafu { path: self.clone() })?
            .context(PathGroupNameNotFoundSnafu { path: self.clone() })?;

        Ok(group)
    }

    /// Sets group information for this path. For both owner/group setting, use [`chown`](Self::chown).
    pub fn group_set(&self, group: &str) -> Result<(), Error> {
        self.set_group(group).context(PathGroupSetSnafu {
            group: group.to_string(),
            path: self.clone(),
        })
    }

    /// Lookup mode information for this path.
    pub fn mode_get(&self) -> Result<u32, Error> {
        let meta = fs::metadata(&self).context(PathModeSnafu { path: self.clone() })?;
        Ok(meta.mode())
    }

    /// Sets mode information for this path.
    pub fn mode_set(&self, mode: u32) -> Result<(), Error> {
        fs::set_permissions(&self, fs::Permissions::from_mode(mode)).context(PathModeSetSnafu {
            path: self.clone(),
            mode,
        })
    }

    /// Sets both owner and group information for this path. Use [`owner_set`](Self::owner_set) or [`group_set`](Self::group_set)
    /// if that's not what you want.
    pub fn chown(&self, req_owner: &str, req_group: &str) -> Result<(), Error> {
        let (owner, group) = self
            .owner_group()
            .context(PathChownMetadataSnafu { path: self.clone() })?;

        let found_owner = owner
            .name()
            .context(PathOwnerNameSnafu { path: self.clone() })
            // Wrap in outer context so we don't need to duplicate error types
            .context(PathChownOwnerSnafu)?
            // Now we have to unwrap the inner option
            .context(PathOwnerNameNotFoundSnafu { path: self.clone() })
            // Wrap in outer context so we don't need to duplicate error types
            .context(PathChownOwnerSnafu)?;

        let found_group = group
            .name()
            .context(PathGroupNameSnafu { path: self.clone() })
            // Wrap in outer context so we don't need to duplicate error types
            .context(PathChownGroupSnafu)?
            // Now we have to unwrap the inner option
            .context(PathGroupNameNotFoundSnafu { path: self.clone() })
            // Wrap in outer context so we don't need to duplicate error types
            .context(PathChownGroupSnafu)?;

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

    /// Two operations at the same time...
    pub fn chown_and_mode(&self, mode: u32, user: &str, group: Option<&str>) -> Result<(), Error> {
        self.mode_set(mode)?;
        if let Some(group) = group {
            self.chown(user, group)?;
        } else {
            self.owner_set(user)?;
        }

        Ok(())
    }

    /// Recursive chown operation
    pub fn chown_recurse(&self, owner: &str, group: &str) -> Result<(), Error> {
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
    pub fn chmod(&self, mode: u32) -> Result<(), Error> {
        self.mode_set(mode)
    }

    pub fn chmod_recurse(&self, mode: u32) -> Result<(), Error> {
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

    pub fn mkdir_p(&self) -> Result<(), Error> {
        if !self.is_dir() {
            fs::create_dir_all(self).context(PathMkdirPSnafu { path: self.clone() })?;
        }

        Ok(())
    }

    pub fn copy_to(&self, dest: &StrPath) -> Result<(), Error> {
        if !dest.is_dir() {
            return Err(Error::PathCopyToNonDir { path: self.clone(), dest: dest.clone() });
        }

        // TODO
        let file_name = self.file_name().unwrap();
        fs::copy(self, dest.join(file_name)).context(PathCopyFailSnafu { path: self.clone(), dest: dest.clone() })?;

        Ok(())
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
/// This is a special class that contains high-level helpers that integrate well
/// with other Yunohost helpers and errors. See [`StrPath`] for more.
pub fn path<T: AsRef<str>>(path: T) -> StrPath {
    StrPath::from(path.as_ref())
}



/// Resolves symlinks until they've reached a stable place on the filesystem.
///
/// Errors when:
///   - `path` does not exist
///   - an intermediate path is not a directory or does not exist
///   - the final path does not exist, maybe????
pub fn readlink_canonicalize<T: AsRef<Path>>(path: T) -> Result<PathBuf, Error> {
    let path = path.as_ref();
    path.canonicalize().context(ReadLinkCanonicalizeSnafu {
        path: path.to_path_buf(),
    })
}

/// Make sure a file does not exist.
///
/// Errors when:
///   - removing the file failed
///
/// Does not error when the file does not exist.
pub fn ensure_file_remove<T: AsRef<Path>>(path: T) -> Result<(), Error> {
    let path = path.as_ref();
    if path.is_file() {
        file_remove(path).context(EnsureFileRemoveSnafu {
            path: path.to_path_buf(),
        })?;
    }

    Ok(())
}

/// Removes an existing file.
///
/// Errors when:
///   - the file does not exist
///   - removing the file failed
pub fn file_remove<T: AsRef<Path>>(path: T) -> Result<(), Error> {
    let path = path.as_ref();
    fs::remove_file(path).context(FileRemoveSnafu {
        path: path.to_path_buf(),
    })?;

    Ok(())
}

/// Creates a symlink `link` pointing to `source`.
///
/// If `force` is true, an existing file `source` will be deleted first.
///
/// Errors when:
///   - `source` exists and `force` is not true
pub fn symlink<T: AsRef<Path>, U: AsRef<Path>>(
    source: T,
    link: U,
    force: bool,
) -> Result<(), Error> {
    let source = source.as_ref();
    let link = link.as_ref();

    if force && link.exists() {
        ensure_file_remove(link).context(SymlinkCreateRemoveSnafu {
            symlink_source: source.to_path_buf(),
            symlink_link: link.to_path_buf(),
        })?;
    }
    std::os::unix::fs::symlink(source, link).context(SymlinkCreateSymlinkSnafu {
        symlink_source: source.to_path_buf(),
        symlink_link: link.to_path_buf(),
    })?;

    Ok(())
}

pub fn read<T: AsRef<str>>(path: T) -> Result<String, Error> {
    let path = Utf8PathBuf::from(path.as_ref());
    fs::read_to_string(&path).context(ReadSnafu { path: path })
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
    pub fn new<T: AsRef<Utf8Path>>(path: T) -> Result<ReadDir, Error> {
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

pub fn glob(pattern: &str) -> Result<Vec<Utf8PathBuf>, Error> {
    let mut files: Vec<Utf8PathBuf> = vec![];

    for entry in glob::glob(pattern).context(GlobPatternSnafu {
        pattern: pattern.to_string(),
    })? {
        let entry = entry.context(GlobSnafu)?;
        let utf8_entry = match Utf8PathBuf::from_path_buf(entry) {
            Ok(p) => p,
            Err(entry_path) => {
                return Err(Error::InvalidUnicodePath { path: entry_path });
            }
        };
        files.push(utf8_entry);
    }

    Ok(files)
}

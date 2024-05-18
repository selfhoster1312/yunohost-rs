use camino::{Utf8Path, Utf8PathBuf};
use file_owner::PathExt;
use snafu::prelude::*;

use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

use crate::error::*;

pub fn path_exists<T: AsRef<Path>>(path: T) -> bool {
    path.as_ref().exists()
}

pub fn is_dir<T: AsRef<Path>>(path: T) -> bool {
    path.as_ref().is_dir()
}

pub fn is_file<T: AsRef<Path>>(path: T) -> bool {
    path.as_ref().is_file()
}

pub fn file_owner<T: AsRef<Path>>(path: T) -> String {
    path.as_ref().owner().unwrap().name().unwrap().unwrap()
}

pub fn set_file_owner<T: AsRef<Path>>(path: T, owner: &str) {
    path.as_ref().set_owner(owner).unwrap();
}

pub fn file_group<T: AsRef<Path>>(path: T) -> String {
    path.as_ref().group().unwrap().name().unwrap().unwrap()
}
/// - intermittent IO errors

pub fn set_file_group<T: AsRef<Path>>(path: T, group: &str) {
    path.as_ref().set_group(group).unwrap();
}

pub fn file_mode<T: AsRef<Path>>(path: T) -> u32 {
    let meta = fs::metadata(path).unwrap();
    meta.mode()
}

pub fn set_file_mode<T: AsRef<Path>>(path: T, mode: u32) {
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
}

pub fn chown<T: AsRef<Path>>(path: T, req_owner: &str, req_group: Option<&str>) {
    let path = path.as_ref();

    let (owner, group) = path.owner_group().unwrap();

    if owner.name().unwrap().unwrap() != req_owner {
        set_file_owner(path, req_owner);
    }

    if let Some(req_group) = req_group {
        if group.name().unwrap().unwrap() != req_group {
            set_file_group(path, req_group);
        }
    }
}

pub fn chown_recurse<T: AsRef<Path>>(path: T, owner: &str, group: &str) {
    let path = path.as_ref();
    if path.is_dir() {
        for entry in fs::read_dir(path).unwrap() {
            chown_recurse(entry.unwrap().path(), owner, group);
        }
    } else {
        chown(path, owner, Some(group));
    }
}

pub fn chmod<T: AsRef<Path>>(path: T, mode: u32) {
    let path = path.as_ref();

    let permissions = fs::metadata(path).unwrap().permissions();
    if permissions.mode() != mode {
        fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
    }
}

pub fn chmod_recurse<T: AsRef<Path>>(path: T, mode: u32) {
    let path = path.as_ref();
    if path.is_dir() {
        for entry in fs::read_dir(path).unwrap() {
            chmod_recurse(entry.unwrap().path(), mode);
        }
    } else {
        chmod(path, mode);
    }
}

pub fn mkdir_p<T: AsRef<Path>>(path: T) {
    let path = path.as_ref();
    if !path.is_dir() {
        fs::create_dir_all(path).unwrap();
    }
}

pub fn copy_to<T: AsRef<Path>, U: AsRef<Path>>(source: T, dest: U) {
    let source = source.as_ref();
    let dest = dest.as_ref();
    let file_name = source.file_name().unwrap();

    fs::copy(source, dest.join(file_name)).unwrap();
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

pub fn read<T: AsRef<Path>>(source: T) -> Result<String, std::io::Error> {
    fs::read_to_string(source)
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

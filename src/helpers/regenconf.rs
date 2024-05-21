use camino::{Utf8Path, Utf8PathBuf};
use difflib::differ::Differ;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use snafu::prelude::*;

use std::collections::BTreeMap;
use std::fs::remove_dir_all;

use crate::{error::*, helpers::file::*};

pub const BASE_CONF_DIR: &'static str = "/var/cache/yunohost/regenconf";
pub const BACKUP_CONF_DIR: &'static str = "/var/cache/yunohost/backup";
pub const PENDING_CONF_DIR: &'static str = "/var/cache/yunohost/pending";
pub const REGEN_CONF_FILE: &'static str = "/etc/yunohost/regenconf.yml";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegenConfFile {
    #[serde(flatten)]
    pub categories: BTreeMap<String, RegenCategoryConfFiles>,
}

impl RegenConfFile {
    pub fn load() -> Result<Self, serde_yaml_ng::Error> {
        serde_yaml_ng::from_str(&read(REGEN_CONF_FILE).unwrap())
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
/// We want to store as a relative path to avoid logic errors, but serialize/deserialize as an absolute path
/// for yunohost compat
/// TODO: maybe in future we want to make serialization/deserialization less work by storing the absolute Path
/// and returning the relative path in some situations...
pub struct RelativeConfFile {
    pub path: Utf8PathBuf,
}

impl RelativeConfFile {
    pub fn from_relative(path: Utf8PathBuf) -> Self {
        Self { path }
    }

    pub fn from_absolute(path: Utf8PathBuf) -> Self {
        Self {
            path: path.strip_prefix("/").unwrap().to_path_buf(),
        }
    }

    pub fn from_basedir_prefix(path: &Utf8Path, prefix: &Utf8Path) -> Self {
        Self::from_relative(path.strip_prefix(&prefix).unwrap().to_path_buf())
    }

    pub fn to_relative(&self) -> Utf8PathBuf {
        self.path.clone()
    }

    pub fn to_absolute(&self) -> Utf8PathBuf {
        Utf8PathBuf::from("/").join(&self.path)
    }
}

impl Serialize for RelativeConfFile {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_absolute().as_str())
    }
}

impl<'de> Deserialize<'de> for RelativeConfFile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let path = Utf8PathBuf::from(&String::deserialize(deserializer)?);
        Ok(Self::from_absolute(path))
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RegenCategoryConfFiles {
    pub conffiles: BTreeMap<RelativeConfFile, String>,
}

pub fn _get_pending_conf(
    categories: &[String],
) -> Result<BTreeMap<String, BTreeMap<RelativeConfFile, Utf8PathBuf>>, Error> {
    let pending_dir = Utf8PathBuf::from(PENDING_CONF_DIR);
    let mut res: BTreeMap<String, BTreeMap<RelativeConfFile, Utf8PathBuf>> = BTreeMap::new();

    // No pending directory, nothing to see here.
    if !is_dir(&pending_dir) {
        return Ok(res);
    }

    // If no categories specified, populate
    let categories = if categories.is_empty() {
        // Only take file names
        ReadDir::new(&pending_dir)?.filenames()
    } else {
        categories.to_vec()
    };

    for name in categories {
        let category_pending_path = pending_dir.join(&name);
        if !is_dir(&category_pending_path) {
            continue;
        }

        let mut category_conf: BTreeMap<RelativeConfFile, Utf8PathBuf> = BTreeMap::new();

        // Only take files not folders
        for path in
            glob(&format!("{}/**/*", category_pending_path)).context(GetPendingConfGlobSnafu {
                category: name.to_string(),
                path: category_pending_path.clone(),
            })?
        {
            // Ignore folders, we only want files
            // TODO: maybe we want to keep symlinks here??
            if !path.is_file() {
                continue;
            }

            // Remove the category_pending_path prefix from the entry
            // let index = path.strip_prefix(&category_pending_path).unwrap();
            let index = RelativeConfFile::from_basedir_prefix(&path, &category_pending_path);
            category_conf.insert(index, path.to_path_buf());
        }

        if category_conf.is_empty() {
            remove_dir_all(category_pending_path).unwrap();
        } else {
            res.insert(name, category_conf);
        }
    }

    Ok(res)
}

fn _get_regenconf_infos() -> RegenConfFile {
    RegenConfFile::load().unwrap()
}

fn _get_regenconf_hashes(
    regen_conf_file: &RegenConfFile,
    category: &str,
) -> RegenCategoryConfFiles {
    if let Some(category_conffiles) = regen_conf_file.categories.get(category) {
        if category_conffiles.conffiles.is_empty() {
            debug!("No configuration files for category {category}.");
            return RegenCategoryConfFiles::default();
        }

        return category_conffiles.to_owned();
    } else {
        debug!("category {category} not in categories.yml yet");
        return RegenCategoryConfFiles::default();
    }
}

/// Empty files are emptied out later... for now they have a hash
fn _calculate_hash(path: &Utf8Path) -> Option<String> {
    if !path.is_file() {
        return None;
    }

    let digest = md5::compute(&read(path).unwrap());
    Some(format!("{:x}", digest))
}

/// Updates existing [`RegenConfFile`] with new `hashes` in `category`.
/// How could hash be None if the file exists?
fn _update_conf_hashes(
    regen_conf_file: &mut RegenConfFile,
    category: &str,
    hashes: RegenCategoryConfFiles,
) {
    debug!("Updating conf hashes for '{category}' with: {hashes:?}");

    if let Some(category_conf) = regen_conf_file.categories.get_mut(category) {
        category_conf.conffiles = hashes.conffiles;
    }
}

fn _get_files_diff(orig_file: &Utf8Path, new_file: &Utf8Path) -> String {
    let orig_file = read(orig_file).unwrap_or(String::new());
    let orig_lines = orig_file.lines().collect::<Vec<&str>>();
    let new_file = read(new_file).unwrap_or(String::new());
    let new_lines = new_file.lines().collect::<Vec<&str>>();
    let differ = Differ::new();
    differ.compare(&orig_lines, &new_lines).join("")
}

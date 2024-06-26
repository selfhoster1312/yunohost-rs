use camino::{Utf8Path, Utf8PathBuf};
use difflib::differ::Differ;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use snafu::prelude::*;

use std::collections::BTreeMap;
use std::fs::remove_dir_all;

use crate::{error::*, helpers::file::*};

pub const BASE_CONF_DIR: &'static str = "/var/cache/yunohost/regenconf";
pub const BACKUP_CONF_DIR: &'static str = "/var/cache/yunohost/regenconf/backup";
pub const PENDING_CONF_DIR: &'static str = "/var/cache/yunohost/regenconf/pending";
pub const REGEN_CONF_FILE: &'static str = "/etc/yunohost/regenconf.yml";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegenConfFile {
    #[serde(flatten)]
    pub categories: BTreeMap<String, RegenCategoryConfFiles>,
}

impl RegenConfFile {
    pub fn load() -> Result<Self, serde_yaml_ng::Error> {
        serde_yaml_ng::from_str(&path(REGEN_CONF_FILE).read().unwrap())
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

/// An absolute path to the pending conf in PENDING_CONF_DIR.
///
/// This is the default output of `yunohost tools regen-conf --list-pending`,
/// use the [`PendingConfFile::into_pending_diff`] to generate the `--with-diff` output.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PendingConfFile(Utf8PathBuf);

impl PendingConfFile {
    pub fn new(path: Utf8PathBuf) -> PendingConfFile {
        PendingConfFile(path)
    }

    // We need to remove PENDING_CONF_DIR and the category
    // /var/cache/yunohost/regenconf/pending/nginx/FOO
    // => FOO
    pub fn to_system_path(&self) -> RelativeConfFile {
        let category_conffile = self.0.strip_prefix(PENDING_CONF_DIR).unwrap();
        let conffile: Utf8PathBuf = category_conffile.components().skip(1).collect();
        RelativeConfFile::from_relative(conffile)
    }

    pub fn into_pending_diff(self) -> Result<PendingConfDiff, Error> {
        Ok(PendingConfDiff {
            diff: _get_files_diff(&self.to_system_path().path, &self.0),
            pending_conf: self,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PendingConfDiff {
    diff: String,
    pending_conf: PendingConfFile,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RegenCategoryConfFiles {
    pub conffiles: BTreeMap<RelativeConfFile, String>,
}

pub fn _get_pending_conf(
    categories: &[String],
) -> Result<BTreeMap<String, BTreeMap<RelativeConfFile, PendingConfFile>>, Error> {
    let pending_dir = Utf8PathBuf::from(PENDING_CONF_DIR);
    let mut res: BTreeMap<String, BTreeMap<RelativeConfFile, PendingConfFile>> = BTreeMap::new();

    // No pending directory, nothing to see here.
    if !path(&pending_dir).is_dir() {
        debug!("No such regen-conf pending directory: {pending_dir}");
        return Ok(res);
    }

    // If no categories specified, populate
    let categories = if categories.is_empty() {
        // Only take file names
        ReadDir::new(&pending_dir).context(FileSnafu)?.filenames()
    } else {
        categories.to_vec()
    };

    for name in categories {
        let category_pending_path = pending_dir.join(&name);
        if !path(&category_pending_path).is_dir() {
            debug!("regen-conf: Skip non-dir {category_pending_path}");
            continue;
        }

        let mut category_conf: BTreeMap<RelativeConfFile, PendingConfFile> = BTreeMap::new();

        // Only take files not folders
        for path in glob(&format!("{}/**/*", category_pending_path))
            .context(FileSnafu)
            .context(GetPendingConfGlobSnafu {
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
            category_conf.insert(index, PendingConfFile::new(path.to_path_buf()));
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
// TODO: File helpers and error cases
fn _calculate_hash(path: &Utf8Path) -> Option<String> {
    let path = StrPath::from(path);

    if !path.is_file() {
        return None;
    }

    let digest = md5::compute(path.read().unwrap());
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

// TODO: file helper and error cases
fn _get_files_diff(orig_file: &Utf8Path, new_file: &Utf8Path) -> String {
    let orig_file = StrPath::from(orig_file);
    let new_file = StrPath::from(new_file);
    let orig_lines = orig_file.read_lines().unwrap_or(vec![]);
    let new_lines = new_file.read_lines().unwrap_or(vec![]);
    let differ = Differ::new();
    differ
        .compare(
            &orig_lines.iter().map(|x| x.as_str()).collect::<Vec<&str>>(),
            &new_lines.iter().map(|x| x.as_str()).collect::<Vec<&str>>(),
        )
        .join("")
}

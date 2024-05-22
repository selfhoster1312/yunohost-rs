use std::fs::read_dir;
use std::path::{Path, PathBuf};

pub const HOOK_FOLDER: &'static str = "/usr/share/yunohost/hooks";
pub const CUSTOM_HOOK_FOLDER: &'static str = "/etc/yunohost/hooks.d";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Hook {
    pub priority: String,
    pub name: String,
    pub path: PathBuf,
}

impl Hook {
    pub(crate) fn new(priority: String, name: String, path: PathBuf) -> Self {
        Self {
            priority,
            name,
            path,
        }
    }

    pub fn from_path<T: AsRef<Path>>(path: T) -> Option<Self> {
        let path = path.as_ref();
        let name = path.file_name().unwrap().to_str().unwrap();
        if name.starts_with(".")
            || name.ends_with("~")
            || name.ends_with(".pyc")
            || (name.starts_with("__") && name.ends_with("__"))
        {
            return None;
        }

        let (hook_priority, hook_name) = _extract_filename_parts(&name);
        Some(Hook::new(hook_priority, hook_name, path.to_path_buf()))
    }

    pub fn folder_hooks<T: AsRef<Path>>(path: T) -> Vec<Self> {
        let path = path.as_ref();
        if !path.is_dir() {
            return vec![];
        }

        let mut result: Vec<Hook> = vec![];
        for entry in read_dir(path).unwrap() {
            let entry = entry.unwrap().path();
            if let Some(hook) = Hook::from_path(&entry) {
                result.push(hook);
            }
        }
        result
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct HookListNames {
    pub hooks: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct HookList {
    pub hooks: Vec<Hook>,
}

impl HookList {
    pub fn for_action(action: &str) -> HookList {
        let mut hooks = Hook::folder_hooks(&PathBuf::from(HOOK_FOLDER).join(action));
        hooks.extend(Hook::folder_hooks(
            &PathBuf::from(CUSTOM_HOOK_FOLDER).join(action),
        ));
        HookList { hooks }
    }

    pub fn names(&self) -> HookListNames {
        HookListNames {
            hooks: self.hooks.iter().map(|x| x.name.to_string()).collect(),
        }
    }
}

// We use stringy priority because that's what Python did
// but we could actually parse actual numbers
// This function returns the first and last part without extension
pub fn _extract_filename_parts(name: &str) -> (String, String) {
    assert!(!name.contains("/"));

    let (priority, name) = if name.contains("-") {
        let (priority, name) = name.split_once("-").unwrap();
        (priority.to_string(), name.to_string())
    } else {
        (String::from("50"), name.to_string())
    };

    if let Some((name, _ext)) = name.rsplit_once(".") {
        (priority, name.to_string())
    } else {
        (priority, name)
    }
}

// Not reimplemented:
// - list_by
// - show_info
pub fn hook_list(action: &str) -> Vec<String> {
    HookList::for_action(action)
        .hooks
        .into_iter()
        .map(|x| x.name)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alphanumeric_parts() {
        assert_eq!(
            (String::from("01"), String::from("yunohost")),
            _extract_file_parts("01-yunohost")
        );
        assert_eq!(
            (String::from("abc"), String::from("yunohost")),
            _extract_file_parts("abc-yunohost")
        );
        assert_eq!(
            (String::from("01"), String::from("yunohost-01")),
            _extract_file_parts("01-yunohost-01")
        );
        assert_eq!(
            (String::from("01"), String::from("yunohost")),
            _extract_file_parts("01-yunohost.sh")
        );
        assert_eq!(
            (String::from("50"), String::from("yunohost")),
            _extract_file_parts("yunohost")
        );
    }
}

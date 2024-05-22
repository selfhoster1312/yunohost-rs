use camino::{Utf8Path, Utf8PathBuf};
use snafu::prelude::*;
use strum::Display;
use toml::{Table, Value};

use std::str::FromStr;

use crate::error::*;
use crate::helpers::{file::*, form::*, i18n::*, string::*};
use crate::moulinette::*;

// [x] _get_config_panel
// [x] _build_internal_config_panel
// [x] _get_raw_settings
// [x] _hydrate
// [x] _get_raw_config
// [x] _get_default_values

pub const CONFIG_PATH_TEMPLATE: &'static str = "/usr/share/yunohost/config_{entity}.toml";
pub const CONFIG_PANEL_VERSION_SUPPORTED: &'static f64 = &1.0;
pub const FORBIDDEN_KEYWORDS: &'static [&'static str] = &[
    "old",
    "app",
    "changed",
    "file_hash",
    "binds",
    "types",
    "formats",
    "getter",
    "setter",
    "short_setting",
    "type",
    "bind",
    "nothing_changed",
    "changes_validated",
    "result",
    "max_progression",
    // Reserved keys from FormatDescriptionLevel
    "properties",
    "defaults",
];

const ALLOWED_EMPTY_TYPES: &'static [OptionType] = &[
    OptionType::Alert,
    OptionType::DisplayText,
    OptionType::Markdown,
    OptionType::File,
    OptionType::Button,
];

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Display, PartialEq, Eq)]
pub enum ConfigPanelVersion {
    #[strum(serialize = "1.0")]
    V1_0,
}

impl FromStr for ConfigPanelVersion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1.0" => Ok(Self::V1_0),
            _ => Err(Error::ConfigPanelConfigVersionWrongStr {
                version: s.to_string(),
            }),
        }
    }
}

impl ConfigPanelVersion {
    pub fn to_f64(&self) -> f64 {
        match self {
            Self::V1_0 => 1.0,
        }
    }

    pub fn from_f64(version: f64) -> Result<Self, Error> {
        match version {
            1.0 => Ok(Self::V1_0),
            _ => Err(Error::ConfigPanelConfigVersionWrongFloat { version: version }),
        }
    }

    /// Ensures that a TOML table from a config panel
    /// has a "version" key which we can work with.
    pub fn from_config_panel_table(table: &Table) -> Result<Self, Error> {
        let version = table
            .get("version")
            .context(ConfigPanelConfigVersionMissingSnafu)?;

        let version = if let Some(version_f64) = version.as_float() {
            Self::from_f64(version_f64)?
        } else if let Some(version_str) = version.as_str() {
            Self::from_str(version_str)?
        } else {
            return Err(Error::ConfigPanelConfigVersionWrongType {
                value: version.clone(),
            });
        };

        Ok(version)

        // // Sanity check that config panel version >= CONFIG_PANEL_VERSION_SUPPORTED
        // if let Some(version) = toml_config_panel.get("version") {
        //     if let Some(float_version) = version.as_float() {
        //         if float_version < *CONFIG_PANEL_VERSION_SUPPORTED {
        //             error!(
        //                 "Config panel version {} in {} are not supported",
        //                 float_version, self.config_path
        //             );
        //             return Err(Error::TODO);
        //         }
        //     } else {
        //         error!(
        //             "Config panel version {:?} in {} is not a floating point number",
        //             version, self.config_path
        //         );
        //         return Err(Error::TODO);
        //     }
        // } else {
        //     error!("Config panel has no version in {}", self.config_path);
        //     return Err(Error::TODO);
        // }
    }

    /// Checks that the version declared in table matches expectations, otherwise error.
    pub fn expect_version_in(
        version: Self,
        table: &Table,
        entity: &str,
        path: &Utf8Path,
    ) -> Result<(), Error> {
        let found_version = ConfigPanelVersion::from_config_panel_table(table).context(
            ConfigPanelVersionSnafu {
                entity: entity.to_string(),
                path: path.to_path_buf(),
            },
        )?;

        if version != found_version {
            return Err(Error::ConfigPanelVersionUnsupported {
                entity: entity.to_string(),
                path: path.to_path_buf(),
                version: found_version.clone(),
            });
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GetMode {
    Classic,
    Export,
    Full,
}

pub enum SaveMode {
    Diff,
    Full,
}

/// Checks that there is recursively entries in the table named after entries in the list, that are arrays containing each other.
///
/// Pseudo-code example:
///
/// ```
/// table = { a: [ { b: [ FOO ] } ] }
/// list = [ "a", "b" ];
/// has_first_entry_in_toml_table_sub_tables(table, list)?;
/// ```
fn has_first_entry_in_toml_table_sub_tables(
    table: &Table,
    list: &'static [&'static str],
) -> Result<(), Error> {
    if list.len() == 0 {
        // Recursion finished successfully
        return Ok(());
    }

    // UNWRAP NOTE: Safe unwrap because of checked condition
    let subentry_name = list.first().unwrap().to_string();

    // Get the key named `entry_name` in the table, if it exists and is an array that's not empty
    if let Some(subentry_list) = table.get(&subentry_name).map(|x| x.as_array()).flatten() {
        if subentry_list.len() > 0 {
            // The array is not empty... if this is the last depth level we have to check (list.len() == 1), stop the recursion
            // and everyone is happy. Otherwise, recurse with the contained table.
            if let Some(subentry_list_first) = subentry_list.iter().nth(0) {
                if list.len() == 1 {
                    return Ok(());
                }

                if let Some(subentry_list_first_table) = subentry_list_first.as_table() {
                    return Ok(has_first_entry_in_toml_table_sub_tables(
                        &subentry_list_first_table,
                        &list[1..],
                    )?);
                }
            }
        }
    }

    // The good condition was not matched
    // Err(Error::TODO)
    Err(Error::ConfigPanelMalformed {
        table: table.clone(),
    })
}

pub struct ConfigPanel {
    entity: String,
    config_path: Utf8PathBuf,
    save_path: Utf8PathBuf,
    _save_mode: SaveMode,
    // The actual "parsed" configuration

    // TODO: why is this initialized later???? It makes no fuckign sense
    config: Table,
    values: Table,
}

impl ConfigPanel {
    pub fn new(
        entity: &str,
        config_path: &Utf8Path,
        save_path: &Utf8Path,
        save_mode: SaveMode,
    ) -> ConfigPanel {
        ConfigPanel {
            // TODO: we initialize empty config/values here but what about loading the config directly when Config::new()???
            config: Table::default(),
            values: Table::default(),
            entity: entity.to_string(),
            config_path: config_path.to_path_buf(),
            save_path: save_path.to_path_buf(),
            _save_mode: save_mode,
        }
    }

    pub fn _get_default_values(&self) -> Table {
        let mut res = Table::new();

        for (_panel, _section, option) in self._iterate(Some(&[FormatDescriptionLevel::Options])) {
            if let Some(default) = option.get("default") {
                res.insert(
                    option.get("id").unwrap().as_str().unwrap().to_string(),
                    default.clone(),
                );
            }
        }

        res
    }

    pub fn _get_raw_settings(&mut self) -> Result<(), Error> {
        let mut defaults = self._get_default_values();

        if self.save_path.is_file() {
            let saved_values: serde_yaml_ng::Mapping = serde_yaml_ng::from_str(
                &read(&self.save_path).context(ConfigPanelReadSavePathSnafu {
                    entity: self.entity.to_string(),
                })?,
            )
            .context(ConfigPanelReadSavePathYamlSnafu {
                entity: self.entity.to_string(),
            })?;

            // serde_yaml_ng::Mapping<Value, Value> needs to have Mapping<String, Value> for extending to toml::Table
            // let saved_values: HashMap<String, serde_yaml_ng::Value> = saved_values.into_iter().map(|(k, v)| (k.as_str().unwrap().to_string(), v)).collect();
            let saved_values = Table::try_from(saved_values).unwrap();

            defaults.extend(saved_values);
        }

        self.values = defaults;
        // println!("DEFAULT VALUES\n{:#?}", &self.values);

        Ok(())
    }

    // TODO: here in python code default key is "" but that makes no sense? and doesn't work from CLI?
    // Actually key="" is how yunohst python implements settings_list........ u_u FACEPALM
    pub fn get(&mut self, key: &str, mode: GetMode) -> Result<Value, Error> {
        self._get_config_panel(key).context(ConfigNoPanelSnafu)?;
        self._get_raw_settings()?;
        self._hydrate()?;

        // info!("CONFIG: {}", &self.config);
        // info!("-------------------------------------");
        // info!("-------------------------------------");
        // info!("-------------------------------------");
        // info!("-------------------------------------");
        // info!("-------------------------------------");
        // info!("VALUES: {}", &self.values);
        // info!("-------------------------------------");
        // info!("-------------------------------------");
        // info!("-------------------------------------");

        if key.chars().filter(|c| *c == '.').count() == 2 && mode == GetMode::Classic {
            // UNWRAP NOTE: Safe unwrap because there's at least 1 '.' character
            let option = key.split('.').rev().nth(0).unwrap();

            // NOPE NOPE NOPE
            let value = self.values.get(option).unwrap();

            for (_panels, _sections, option_iter) in self._iterate(None) {
                if option_iter.get("id").unwrap().as_str().unwrap() == option {
                    let option_type = OptionType::from_str(
                        option_iter
                            .get("type")
                            .map(|x| x.as_str())
                            .flatten()
                            .unwrap_or("string"),
                    )
                    .unwrap()
                    .to_option_type();

                    let normalized = option_type.normalize(value, &option_iter);
                    let val: Result<toml::Value, serde_json::Error> =
                        serde_json::from_str(&normalized);
                    // match serde_json::Value::from_str(&normalized) {
                    match val {
                        Ok(normalized_value) => return Ok(normalized_value),
                        Err(e) => {
                            eprintln!("Failed to value normalized str: {:?}", normalized);
                            eprintln!("{e}");
                            panic!();
                        }
                    }
                    // let normalized_value = Value::from_str(&normalized).unwrap();
                    // return Ok(normalized_value);
                }
            }

            return Ok(value.clone());
        }

        debug!("Formatting result in '{mode:?}' mode");
        let mut result: Table = Table::new();

        for (panel, section, option) in self._iterate(None) {
            // UNWRAP NOTE: Safe unwrap as long as Self::_iterate works as expected
            let section = section.unwrap();

            if section.get("is_action_section").unwrap().as_bool().unwrap() && mode != GetMode::Full
            {
                continue;
            }

            let panel_id = panel.get("id").unwrap().as_str().unwrap().to_string();
            let section_id = section.get("id").unwrap().as_str().unwrap().to_string();
            let option_id = option.get("id").unwrap().as_str().unwrap().to_string();
            let key = format!("{panel_id}.{section_id}.{option_id}");

            if mode == GetMode::Export {
                result.insert(
                    option_id.clone(),
                    option.get("current_value").unwrap().clone(),
                );
                continue;
            }

            let mut ask = None;
            if let Some(option_ask_table) = option.get("ask").map(|x| x.as_table()).flatten() {
                // NOTE: Wtf is this?
                ask = Some(_value_for_locale(option_ask_table));
            } else if let Some(config_i18n_key) =
                self.config.get("i18n").map(|x| x.as_str()).flatten()
            {
                let option_i18n_key = format!("{}_{}", config_i18n_key, option_id);
                ask = Some(i18n::n(&option_i18n_key, None));
            }

            // We start to modify option
            let option = self._get_config_option_mut(&panel, &section, &option);
            let question_class = OptionType::from_str(
                option
                    .get("type")
                    .map(|x| x.as_str())
                    .flatten()
                    .unwrap_or("string"),
            )
            .unwrap()
            .to_option_type();

            if mode == GetMode::Full {
                if let Some(ask) = ask {
                    option.insert("ask".to_string(), ask.into());
                }

                // TODO: wtf choices/default/pattern??? dynamic types and all
                unimplemented!();
            } else {
                let mut result_key = Table::new();
                if let Some(ask) = ask {
                    // TODO: ask stuff
                    result_key.insert("ask".to_string(), ask.into());
                }

                result.insert(key.to_string(), result_key.into());

                // Now borrow the result key again for further stuff
                // UNWRAP NOTE: Safe unwrap because it was inserted just before
                let result_key = result.get_mut(&key).unwrap().as_table_mut().unwrap();

                if let Some(current_value) = option.get("current_value") {
                    if let Some(humanized) = question_class.humanize(
                        // TODO: is current_value always a string????
                        // Maybe it's just a toml::Value and we need to handle all cases?
                        option.get("current_value").unwrap(),
                        &option,
                    ) {
                        let humanized_value = Value::from_str(&humanized).unwrap();
                        result_key.insert("value".to_string(), humanized_value);
                    } else {
                        // OptionType has no specific opinion about humanization
                        result_key.insert("value".to_string(), current_value.clone());
                    }

                    // FIXME semantics upstream
                    if question_class.hide_user_input_in_prompt() {
                        result_key.insert(
                            "value".to_string(),
                            Value::String("*************".to_string()),
                        );
                    }
                }
            }
        }

        if mode == GetMode::Full {
            Ok(self.config.clone().into())
        } else {
            Ok(result.into())
        }
    }

    // WTF is this???
    pub fn _hydrate(&mut self) -> Result<(), Error> {
        // HUGE HACK: We need the panel, to reconstruct the path to an option to rewrite it
        // when option.extend(value);
        for (panel, section, option) in self._iterate(Some(&[FormatDescriptionLevel::Options])) {
            // UNWRAP NOTE: This should be type-level guaranteed but for now this unwrap is safe as long as Self::_iterate
            // guarantees that section is not None when not requesting Panels
            let section = section.unwrap();

            let option_id = option.get("id").unwrap().as_str().unwrap().to_string();
            if !self.values.contains_key(&option_id) {
                if section.get("is_action_section").unwrap().as_bool().unwrap()
                    && option.get("default").is_some()
                {
                    let id = option.get("id").map(|x| x.as_str()).flatten().unwrap();
                    // UNWRAP NOTE: Safe unwrap because of checked condition
                    let default = option.get("default").unwrap();
                    self.values.insert(id.to_string(), default.clone());
                } else {
                    // THERE IS A PANIC BELOW, DEBUG MSG HERE
                    // That was because we requested FormatDescriptionLevel::Sections so sometimes section == option... i think it's fixed?
                    // info!("{:#?}\n  -> {:#?}\n      -> {:#?}", &panel, &section, &option);
                    let option_type: OptionType = OptionType::from_str(
                        // PANIC PANIC
                        &option.get("type").map(|x| x.as_str()).flatten().unwrap(),
                    )
                    .unwrap();
                    // WTF IS THIS STRINGY TYPE NULL?????
                    if ALLOWED_EMPTY_TYPES.contains(&option_type)
                        || option.get("bind").unwrap().as_str().unwrap() == "null"
                    {
                        continue;
                    } else {
                        let id = option.get("id").unwrap().as_str().unwrap();
                        // error!("Question {} should be initialized with a value during install or upgrade", option.get("id").unwrap().as_str().unwrap());
                        // return Err(Error::TODO);
                        return Err(Error::ConfigPanelHydrateValueNotSet { id: id.to_string() });
                    }
                }
            }

            // UNWRAP NOTE: Safe unwrap because of checked condition... values[option_id] is either filled, or continue, or return Err
            let value = self.values.get_mut(&option_id).unwrap();
            // PANIC PANIC BECAUSE NO VALUE?!
            // println!("{option_id} VALUE: {:#?}", value);

            let value = if let Some(value) = value.as_table_mut() {
                if value.contains_key("value") && !value.contains_key("current_value") {
                    value.insert(
                        "current_value".to_string(),
                        value.get("current").unwrap().clone(),
                    );
                }
                value.clone()
            } else {
                // Treat value as table of current_value => value
                let mut new_value = Table::new();
                new_value.insert("current_value".to_string(), value.clone());
                new_value
            };

            // NOOOOOOO option is readonly because it's generated from _iterate
            // Now we need to hack around to get a mutable ref to self.config -> panel -> section -> option
            let option = self._get_config_option_mut(&panel, &section, &option);
            option.extend(value.clone());

            // println!("VALUE: {:#?}", value);
            self.values
                .insert(option_id, value.get("current_value").unwrap().clone());
        }

        Ok(())
    }

    fn _get_config_option_mut(
        &mut self,
        req_panel: &Table,
        req_section: &Table,
        req_option: &Table,
    ) -> &mut Table {
        let panels = self
            .config
            .get_mut("panels")
            .unwrap()
            .as_array_mut()
            .unwrap();
        for panel in panels {
            let panel = panel.as_table_mut().unwrap();
            if Self::table_id_equals(&panel, &req_panel) {
                // if panel == req_panel {
                let sections = panel.get_mut("sections").unwrap().as_array_mut().unwrap();
                for section in sections {
                    let section = section.as_table_mut().unwrap();
                    if Self::table_id_equals(&section, &req_section) {
                        // if section == req_section {
                        let options = section.get_mut("options").unwrap().as_array_mut().unwrap();
                        for option in options {
                            let option = option.as_table_mut().unwrap();
                            // if option == req_option {
                            if Self::table_id_equals(&option, &req_option) {
                                return option;
                            }
                        }
                    }
                }
            }
        }

        // println!("Option not found: {}.{}.{}",
        //     req_panel.get("id").unwrap().as_str().unwrap(),
        //     req_section.get("id").unwrap().as_str().unwrap(),
        //     req_option.get("id").unwrap().as_str().unwrap(),
        // );

        panic!();
    }

    fn table_id_equals(a: &Table, b: &Table) -> bool {
        if a.get("id").unwrap() == b.get("id").unwrap() {
            return true;
        }

        false
    }

    /// What does this do? Why does it have a filter key? Why 3 levels maximum?
    /// Why is the filter_key not mandatory in python?
    fn _get_config_panel(&mut self, filter_key: &str) -> Result<(), Error> {
        let filter_list: Vec<&str> = filter_key.split('.').collect();
        if filter_list.len() > 3 {
            return Err(Error::ConfigPanelTooManySublevels {
                filter_key: filter_key.to_string(),
            });
        }

        if !self.config_path.exists() {
            // debug!("Config panel {} doesn't exist", self.config_path);
            return Err(Error::ConfigPanelReadConfigNotPath {
                path: self.config_path.clone(),
            });
        }

        // Sanity check that config panel version >= CONFIG_PANEL_VERSION_SUPPORTED
        let toml_config_panel = self._get_raw_config()?;
        ConfigPanelVersion::expect_version_in(
            ConfigPanelVersion::V1_0,
            &toml_config_panel,
            &self.entity,
            &self.config_path,
        )?;

        let config = Self::_build_internal_config_panel(
            toml_config_panel,
            FormatDescriptionLevel::Root,
            filter_list,
        );
        self.config = config.clone();
        // println!("000000000000");
        // println!("{}", &config);
        has_first_entry_in_toml_table_sub_tables(&config, &["panels", "sections", "options"])?;
        // println!("000000000000");

        // for _, _, option in self._iterate():
        //     if option["id"] in forbidden_keywords: error
        for (_panel, _section, option) in self._iterate(Some(&[FormatDescriptionLevel::Options])) {
            if let Some(option_id) = option.get("id").map(|x| x.as_str()).flatten() {
                if FORBIDDEN_KEYWORDS.contains(&option_id) {
                    return Err(Error::ConfigPanelReadConfigForbiddenKeyword {
                        id: option_id.to_string(),
                    });
                }
            } else {
                return Err(Error::ConfigPanelReadConfigOptionNoId {
                    option: option.clone().into(),
                });
                // return Err(Error::TODO);
            }
        }

        return Ok(());
    }

    /// Return a list of config panels, and/or sections, and/or options.
    ///
    /// The list entries in the return tuple are:
    /// - the current panel, in all cases
    /// - either:
    ///   - the current section, if "sections" or "options" trigger is requested, and the current entry is not a top-level panel
    ///   - None if the current entry is a top-level panel
    /// - either:
    ///   - the current panel, if "panel" trigger is requested, and the current entry is a panel
    ///   - the current section, if "sections" trigger is requested, and the current entry is a section
    ///   - the current option, if "options" trigger is requested, and the current entry is an option
    /// TODO: Why does this function do so many things? Is it that common to want to iterate over everything? Apparently this has to do with
    /// javascript and visibility stuff...
    fn _iterate(
        &self,
        triggers: Option<&'static [FormatDescriptionLevel]>,
    ) -> Vec<(Table, Option<Table>, Table)> {
        // If no triggers were requested, only request options
        let triggers = triggers.unwrap_or(&[FormatDescriptionLevel::Options]);

        let mut res: Vec<(Table, Option<Table>, Table)> = Vec::new();

        if let Some(panels_list) = self
            .config
            .get(FormatDescriptionLevel::Panels.as_str())
            .map(|x| x.as_array())
            .flatten()
        {
            for panel in panels_list {
                let panel: Table = panel.as_table().unwrap().clone();
                if triggers.contains(&FormatDescriptionLevel::Panels) {
                    res.push((panel.clone(), None, panel.clone()));
                }

                // Don't go deeper if sections/options was not requested
                if !triggers.contains(&FormatDescriptionLevel::Sections)
                    && !triggers.contains(&FormatDescriptionLevel::Options)
                {
                    continue;
                }

                if let Some(sections_list) = panel
                    .get(&FormatDescriptionLevel::Sections.to_string())
                    .map(|x| x.as_array())
                    .flatten()
                {
                    for section in sections_list {
                        let section: Table = section.as_table().unwrap().clone();
                        if triggers.contains(&FormatDescriptionLevel::Sections) {
                            res.push((panel.clone(), Some(section.clone()), section.clone()));
                        }

                        // Don't go deeper if options was not requested
                        if !triggers.contains(&FormatDescriptionLevel::Options) {
                            continue;
                        }

                        if let Some(options_list) = section
                            .get(FormatDescriptionLevel::Options.as_str())
                            .map(|x| x.as_array())
                            .flatten()
                        {
                            for option in options_list {
                                let option: Table = option.as_table().unwrap().clone();
                                // Trigger options was requested if we reached here
                                res.push((panel.clone(), Some(section.clone()), option.clone()));
                            }
                        }
                    }
                }
            }
        }

        return res;
    }

    fn _get_raw_config(&self) -> Result<Table, Error> {
        let content = read(&self.config_path).context(ConfigPanelReadConfigPathSnafu {
            entity: self.entity.to_string(),
        })?;
        toml::from_str(&content).context(ConfigPanelReadConfigPathTomlSnafu {
            entity: self.entity.to_string(),
        })
    }

    fn _build_internal_config_panel(
        raw_infos: Table,
        level: FormatDescriptionLevel,
        filter_list: Vec<&str>,
    ) -> Table {
        let defaults = level.defaults();
        let properties = level.properties();

        let mut out = Table::new();

        // First, to populate out, copy default values from level defaults, overwritten by
        // raw_infos entries. So keys which don't have a default in raw_infos don't get copied over to out yet.
        for (key, val) in defaults {
            out.insert(key.clone(), raw_infos.get(&key).unwrap_or(&val).clone());
        }

        // Let's figure out some stuff about the next level
        let sublevel = level.next_level();
        // And about what we're looking for at this level)
        let search_key = filter_list.iter().nth(level.as_usize());

        // This is magic shit
        for (key, value) in raw_infos {
            // If the entry below is a table, and we have yet another FormatDescriptionLevel below,
            // we may want to recurse...
            if value.is_table() && !properties.contains(&key.as_str()) && sublevel.is_some() {
                // UNWRAP NOTE: Safe unwrap because we checked that sublevel.is_some() and value.is_table()
                let value = value.as_table().unwrap();
                let sublevel = sublevel.clone().unwrap();

                // We have explicitly requested a search_key, such as the user performing
                // `yunohost settings get security.webadmin`. In this example, on the root level,
                // search_key will be Some("security"), while on the panel level it will be Some("webadmin").
                if let Some(search_key) = search_key {
                    // The search key does not match the key below, so let's ignore it.
                    if *search_key != key {
                        continue;
                    }
                }

                // There is a sub-entry which either matches the filter_list we're looking for, or we're
                // don't have a filter_list at all for this level. In both cases, we'd like to recurse.
                let mut subnode = Self::_build_internal_config_panel(
                    value.clone(),
                    sublevel.clone(),
                    filter_list.clone(),
                );
                subnode.insert("id".to_string(), key.clone().into());

                match level {
                    FormatDescriptionLevel::Root => {
                        // If the sub-entry (panel) doesn't have a proper name,
                        // use the capitalized key name as name.
                        // For example, "Security", "Email", or "Misc" in the Yunohost settings.
                        if subnode.get("name").is_none() {
                            let mut name_table = Table::new();
                            name_table.insert("en".to_string(), capitalize(&key).into());
                            subnode.insert("name".to_string(), name_table.into());
                        }
                    }
                    FormatDescriptionLevel::Sections => {
                        // If this section contains at least one button, it becomes an "action" section.
                        if let Some(subnode_type) = subnode.get("type").map(|x| x.as_str()) {
                            if subnode_type == Some("button") {
                                out.insert("is_action_section".to_string(), true.into());
                            }
                        }
                    }
                    _ => {
                        // Nothing special to do in these cases
                    }
                }

                // Check the table entry for the (next) sublevel (eg. panels if current level is root)
                // and make it an empty array if it does not exist yet.
                let sublevel_entry = out
                    .entry(sublevel.to_string())
                    .or_insert(toml::value::Array::new().into());
                // Borrow the array as mutable to push the new subnode.
                // TODO: Is this a safe unwrap?
                let sublevel_entry = sublevel_entry.as_array_mut().unwrap();
                sublevel_entry.push(Value::from(subnode));
            }
            // Otherwise key/value are a property... which can be a dict!
            else {
                if !properties.contains(&key.as_str()) {
                    warn!("Unknown key '{key}' fuond in config panel");
                }

                // Why these?
                let special_keys: &'static [&'static str] = &["ask", "help", "name"];

                let key_value = if !special_keys.contains(&key.as_str()) || value.is_table() {
                    value
                } else {
                    let mut table = Table::new();
                    table.insert("en".to_string(), value);
                    table.into()
                };
                out.insert(key, key_value);
            }
        }

        return out;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormatDescriptionLevel {
    Root,
    Panels,
    Sections,
    Options,
}

impl std::fmt::Display for FormatDescriptionLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FormatDescriptionLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Panels => "panels",
            Self::Sections => "sections",
            Self::Options => "options",
        }
    }

    pub fn as_usize(&self) -> usize {
        match self {
            Self::Root => 0,
            Self::Panels => 1,
            Self::Sections => 2,
            Self::Options => 3,
        }
    }

    pub fn properties(&self) -> &'static [&'static str] {
        match self {
            Self::Root => &["version", "i18n"],
            Self::Panels => &["name", "services", "actions", "help", "bind"],
            Self::Sections => &["name", "services", "optional", "help", "visible", "bind"],
            Self::Options => &[
                "ask",
                "type",
                "bind",
                "help",
                "example",
                "default",
                "style",
                "icon",
                "placeholder",
                "visible",
                "optional",
                "choices",
                "yes",
                "no",
                "pattern",
                "limit",
                "min",
                "max",
                "step",
                "accept",
                "redact",
                "filter",
                "readonly",
                "enabled",
            ],
        }
    }

    pub fn defaults(&self) -> Table {
        let mut table = Table::new();

        match self {
            Self::Root => {
                table.insert("version".to_string(), Value::Float(1.0));
            }
            Self::Panels => {
                let mut actions = Table::new();
                let mut apply = Table::new();
                apply.insert("en".to_string(), Value::String("Apply".to_string()));
                actions.insert("apply".to_string(), apply.into());

                table.insert(
                    "services".to_string(),
                    Value::Array(toml::value::Array::new()),
                );
                table.insert("actions".to_string(), actions.into());
            }
            Self::Sections => {
                table.insert("name".to_string(), Value::String(String::new()));
                table.insert(
                    "services".to_string(),
                    Value::Array(toml::value::Array::new()),
                );
                table.insert("optional".to_string(), Value::Boolean(true));
                table.insert("is_action_section".to_string(), Value::Boolean(false));
            }
            Self::Options => {}
        }

        table
    }

    pub fn next_level(&self) -> Option<FormatDescriptionLevel> {
        match self {
            Self::Root => Some(Self::Panels),
            Self::Panels => Some(Self::Sections),
            Self::Sections => Some(Self::Options),
            Self::Options => None,
        }
    }
}

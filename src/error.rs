use camino::Utf8PathBuf;

use std::collections::HashMap;
use std::path::PathBuf;

use crate::helpers;
use crate::helpers::file::StrPath;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    // ===================
    // src/helpers/apt.rs
    // ===================

    //    fn change_dpkg_vendor
    #[snafu(display("Failed to read existing DPKG vendor"))]
    ChangeDPKGVendorRead {
        #[snafu(source(from(Error, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[snafu(display("Failed to change DPKG vendor"))]
    ChangeDPKGVendorWrite {
        #[snafu(source(from(Error, Box::from)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    // ===================
    // src/helpers/configpanel.rs
    // ===================
    //     fn get (ConfigPanel::get)
    #[snafu(display("No config panel could be loaded"))]
    ConfigNoPanel {
        #[snafu(source(from(Error, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    //     fn _get_config_panel (ConfigPanel::_get_config_panel)
    #[snafu(display(
        "ConfigPanel::_get_config_panel cannot have so many levels in filter_key: {}",
        filter_key
    ))]
    ConfigPanelTooManySublevels { filter_key: String },

    #[snafu(display("ConfigPanel::_get_config_panel cannot find config file: {path}"))]
    ConfigPanelReadConfigNotPath { path: StrPath },

    #[snafu(display("ConfigPanel::_get_config_panel: Option id {id} is a forbidden keyword."))]
    ConfigPanelReadConfigForbiddenKeyword { id: String },

    #[snafu(display("ConfigPanel::_get_config_panel: Option has no id?! See:\n{}", option))]
    ConfigPanelReadConfigOptionNoId { option: toml::Value },

    //     fn _get_raw_config (ConfigPanel::_get_raw_config)
    #[snafu(display("ConfigPanel::_get_raw_config failed to read config for {}", entity))]
    ConfigPanelReadConfigPath {
        entity: String,
        #[snafu(source(from(Error, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[snafu(display(
        "ConfigPanel::_get_raw_config failed to parse invalid TOML for {}",
        entity
    ))]
    ConfigPanelReadConfigPathToml {
        entity: String,
        source: toml::de::Error,
    },

    //     fn _hydrate (ConfigPanel::_hydrate)
    #[snafu(display(
        "ConfigPanel::_hydrate: Question {id} should be initialized during install or upgrade"
    ))]
    ConfigPanelHydrateValueNotSet { id: String },

    //     fn has_first_entry_in_toml_table_sub_tables
    #[snafu(display(
        "ConfigPanel::has_first_entry_in_toml_table_sub_tables: MALFORMED:\n{:#?}",
        table
    ))]
    ConfigPanelMalformed { table: toml::Table },

    //     fn from_config_panel_table (ConfigPanelVersion::from_config_panel_table)
    #[snafu(display("No version field in config panel"))]
    ConfigPanelConfigVersionMissing,

    #[snafu(display("Unknown (float) version field in config panel: {version}"))]
    ConfigPanelConfigVersionWrongFloat { version: f64 },

    #[snafu(display("Unknown (str) version field in config panel: {version}"))]
    ConfigPanelConfigVersionWrongStr { version: String },

    #[snafu(display("Unknown type of version field in config panel: {value:?}"))]
    ConfigPanelConfigVersionWrongType { value: toml::Value },

    #[snafu(display(
        "Failed to find valid config panel version for entity {entity} at path {path}"
    ))]
    ConfigPanelVersion {
        entity: String,
        path: Utf8PathBuf,

        #[snafu(source(from(Error, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[snafu(display("Unsupported version {} in config panel for entity {entity} at path {path}"))]
    ConfigPanelVersionUnsupported {
        entity: String,
        version: helpers::configpanel::ConfigPanelVersion,
        path: Utf8PathBuf,
    },

    //     _get_raw_settings (ConfigPanel::_get_raw_settings)
    #[snafu(display("Config::_get_raw_settings failed to read config for {}", entity))]
    ConfigPanelReadSavePath {
        entity: String,
        #[snafu(source(from(Error, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[snafu(display("Config::_get_raw_settings failed to read config for {}", entity))]
    ConfigPanelReadSavePathYaml {
        entity: String,
        source: serde_yaml_ng::Error,
    },

    // ===================
    // src/helpers/file.rs
    // ===================
    #[snafu(display("Failed to read ownership metadata for path: {path}"))]
    PathOwnerMetadata {
        path: StrPath,
        source: file_owner::FileOwnerError,
    },
    #[snafu(display("Failed to lookup user information for owner of path: {path}"))]
    PathOwnerName {
        path: StrPath,
        source: file_owner::FileOwnerError,
    },
    #[snafu(display("Failed to find username for owner of path: {path}"))]
    PathOwnerNameNotFound { path: StrPath },

    #[snafu(display("Failed to set owner {owner} for path {path}"))]
    PathOwnerSet {
        owner: String,
        path: StrPath,
        source: file_owner::FileOwnerError,
    },

    #[snafu(display("Failed to read group metadata for path: {path}"))]
    PathGroupMetadata {
        path: StrPath,
        source: file_owner::FileOwnerError,
    },
    #[snafu(display("Failed to lookup group information for group of path: {path}"))]
    PathGroupName {
        path: StrPath,
        source: file_owner::FileOwnerError,
    },
    #[snafu(display("Failed to find groupname for group of path: {path}"))]
    PathGroupNameNotFound { path: StrPath },

    #[snafu(display("Failed to set group {group} for path: {path}"))]
    PathGroupSet {
        group: String,
        path: StrPath,
        source: file_owner::FileOwnerError,
    },

    #[snafu(display("Failed to read permissions (mode) for path: {path}"))]
    PathMode {
        path: StrPath,
        source: std::io::Error,
    },

    #[snafu(display("Failed to set permissions (mode) to {mode:o} for path: {path}"))]
    PathModeSet {
        mode: u32,
        path: StrPath,
        source: std::io::Error,
    },

    #[snafu(display("Failed to read metadata for path: {path}"))]
    PathChownMetadata {
        path: StrPath,
        source: file_owner::FileOwnerError,
    },

    #[snafu(display("Failed to read owner information... see error above"))]
    PathChownOwner {
        #[snafu(source(from(Error, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[snafu(display("Failed to read group information... see error above"))]
    PathChownGroup {
        #[snafu(source(from(Error, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[snafu(display("Failed to chown {owner}:{group} for path: {path}"))]
    PathChownSet {
        owner: String,
        group: String,
        path: StrPath,
        source: file_owner::FileOwnerError,
    },

    #[snafu(display("Failed to create recursive directory (mkdir -p) until path: {path}"))]
    PathMkdirP {
        path: StrPath,
        source: std::io::Error,
    },

    #[snafu(display("Cannot copy {path} to {dest} because it is not a directory!"))]
    PathCopyToNonDir { path: StrPath, dest: StrPath },

    #[snafu(display("Failed to copy {path} to {dest}."))]
    PathCopyFail {
        path: StrPath,
        dest: StrPath,
        source: std::io::Error,
    },

    #[snafu(display("read failed to read {path}"))]
    PathRead {
        path: StrPath,
        source: std::io::Error,
    },

    // -------

    //    fn ensure_remove_file
    #[snafu(display("ensure_file_remove failed to remove file {}", path.display()))]
    EnsureFileRemove {
        path: PathBuf,
        #[snafu(source(from(Error, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    //    fn readlink_canonicalize
    #[snafu(display("readlink_canonicalize failed to resolve links of {}", path.display()))]
    ReadLinkCanonicalize {
        path: PathBuf,
        source: std::io::Error,
    },

    //    fn remove_file
    #[snafu(display("file_remove failed to remove file {}", path.display()))]
    FileRemove {
        path: PathBuf,
        source: std::io::Error,
    },

    //    fn symlink_create
    #[snafu(display("symlink_create failed to create a symlink to {} due to failing to remove existing file: {}", symlink_source.display(), symlink_link.display()))]
    SymlinkCreateRemove {
        symlink_source: PathBuf,
        symlink_link: PathBuf,
        #[snafu(source(from(Error, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[snafu(display("symlink_create to create a symlink to {} at: {}", symlink_source.display(), symlink_link.display()))]
    SymlinkCreateSymlink {
        symlink_source: PathBuf,
        symlink_link: PathBuf,
        source: std::io::Error,
    },

    //     fn read_dir_str / fn read_dir_filenames
    #[snafu(display("read_dir failed to read {}", path))]
    ReadDir {
        path: Utf8PathBuf,
        source: std::io::Error,
    },

    //     fn read
    #[snafu(display("read failed to read {}", path))]
    Read {
        path: Utf8PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("Utf8PathBuf::from_path_buf failed because path is not valid UTF8: {}", path.display()))]
    InvalidUnicodePath { path: PathBuf },

    //     fn glob
    #[snafu(display("glob invalid pattern: {}", pattern))]
    GlobPattern {
        pattern: String,
        source: glob::PatternError,
    },

    #[snafu(display("glob invalid read: {}", source.path().display()))]
    Glob { source: glob::GlobError },

    // ===================
    // src/helpers/ldap.rs
    // ===================

    //     fn exists (YunohostGroup::exists)
    #[snafu(display(
        "YunohostGroup::exists failed to read /etc/group to check whether group {name} exists."
    ))]
    YunohostGroupExistsRead {
        name: String,
        #[snafu(source(from(Error, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    //     fn add (YunohostGroup::add)
    #[snafu(display("YunohostGroup::add failed because group {name} already exists."))]
    YunohostGroupExists { name: String },

    #[snafu(display("YunohostGroup::add failed because groupadd {name} failed."))]
    YunohostGroupCreate { name: String },

    // ===================
    // src/helpers/ldap.rs
    // ===================

    //    fn new_ldap
    #[snafu(display("new_ldap failed to init connection to LDAP database {}", uri))]
    LdapInit {
        uri: String,
        source: ldap3::result::LdapError,
    },

    // TODO
    #[snafu(display("Failed to bind on the LDAP database"))]
    LdapBind { source: ldap3::result::LdapError },

    // TODO
    #[snafu(display("Failed to search the LDAP database"))]
    LdapSearch { source: ldap3::result::LdapError },

    // TODO
    // #[snafu(display("No such user: {}", username.as_str()))]
    #[snafu(display("No such user matching query: {:?}", query))]
    LdapNoSuchUser {
        // username: crate::helpers::credentials::Username,
        query: crate::helpers::user::UserQuery,
    },

    #[snafu(display("Failed to lookup permission {name}"))]
    LdapPermissionNotFound { name: String },

    // TODO
    #[snafu(display("Empty username provided for login"))]
    LdapEmptyUsername,

    // TODO
    #[snafu(display("Empty password provided for login"))]
    LdapEmptyPassword,

    // ===================
    // src/helpers/process.rs
    // ===================

    //     fn json_or_yaml_output
    #[snafu(display("json_or_yaml_output failed to generate a JSON output for the following (lossy) document:\n{:#?}", content))]
    OutputJson {
        content: String,
        source: serde_json::Error,
    },

    #[snafu(display("json_or_yaml_output failed to generate a YAML output for the following (lossy) document:\n{:#?}", content))]
    OutputYaml {
        content: String,
        source: serde_yaml_ng::Error,
    },

    // ===================
    // src/helpers/process.rs
    // ===================

    //     fn cmd
    #[snafu(display("Failed to run the program {} with args: {:?}", cmd, args))]
    Cmd {
        cmd: String,
        args: Vec<String>,
        source: std::io::Error,
    },

    // ===================
    // src/helpers/regenconf.rs
    // ===================

    //     fn get_pending_conf
    #[snafu(display(
        "_get_pending_conf failed to read pending changes for category {} from directory {}",
        category,
        path
    ))]
    GetPendingConfGlob {
        category: String,
        path: Utf8PathBuf,
        #[snafu(source(from(Error, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    // ===================
    // src/helpers/settings.rs
    // ===================
    #[snafu(display("You can't use --full and --export together."))]
    SettingsNoExportAndFull,

    // ===================
    // src/helpers/users.rs
    // ===================

    //     fn load (YunohostUsers::load)
    #[snafu(display("Failed to run `yunohost user list` to load the users list"))]
    YunohostUsersLoadCmd {
        #[snafu(source(from(Error, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[snafu(display(
        "Failed to load `yunohost user list` due to invalid JSON:\n{}",
        content
    ))]
    YunohostUsersLoadJson {
        content: String,
        source: serde_json::Error,
    },

    #[snafu(display("Failed to lookup the mail storage used by user {user}"))]
    MailStorageLookup {
        user: String,
        #[snafu(source(from(Error, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    // ===================
    // src/helpers/ldap.rs
    // ===================

    //     fn from_str (UserAttr::from_str)
    #[snafu(display("UserAttr: unknown user field: {}", field))]
    LdapUserAttrUnknown { field: String },

    #[snafu(display("UserAttr: cannot request user password from LDAP"))]
    LdapUserAttrNotPassword,

    // ===================
    // src/moulinette/i18n.rs
    // ===================

    //     fn new (Translator::new)
    #[snafu(display("Failed to read the locales from {}", path))]
    LocalesReadFailed {
        path: Utf8PathBuf,
        #[snafu(source(from(Error, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[snafu(display("Failed to load JSON locale {}", path))]
    LocalesLoadFailed {
        path: Utf8PathBuf,
        source: serde_json::Error,
    },

    //     fn translate (Translator::translate)
    #[snafu(display("Missing translation key: {key}"))]
    LocalesMissingKey { key: String },

    #[snafu(display("Failed to format translation key {key} with the args:\n{:?}", args))]
    LocalesFormatting {
        key: String,
        args: Option<HashMap<String, String>>,
        source: strfmt::FmtError,
    },

    #[snafu(display("Failed to load Yunohost locales"))]
    Moulinette18nYunohost {
        #[snafu(source(from(Error, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[snafu(display("Failed to load Moulinette locales"))]
    Moulinette18nMoulinette {
        #[snafu(source(from(Error, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    // ===================
    // hooks/conf_regen/01-yunohost.rs
    // ===================

    //     fn init
    #[snafu(display("conf_regen/01-yunohost::init failed to update DPKG vendor"))]
    ConfRegenYunohostInitDPKGVendor {
        #[snafu(source(from(Error, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    //     fn pre
    #[snafu(display("conf_regen/01-yunohost::pre failed to update DPKG vendor"))]
    ConfRegenYunohostPreDPKGVendor {
        #[snafu(source(from(Error, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[snafu(display(
        "conf_regen/01-yunohost::pre failed to remove the check_yunohost_is_installed.sh script"
    ))]
    ConfRegenYunohostPreIsInstalledCheck {
        #[snafu(source(from(Error, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    //    fn post
    #[snafu(display("conf_regen/01-yunohost::post failed to list Yunohost users"))]
    ConfRegenYunohostPostUsers {
        #[snafu(source(from(Error, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[snafu(display("conf_regen/01-yunohost::post failed to set ACL on system directories"))]
    ConfRegenYunohostPostSetfacl {
        #[snafu(source(from(Error, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[snafu(display("conf_regen/01-yunohost::post failed to disable now some systemd services"))]
    ConfRegenYunohostPostSystemCtlDisable {
        #[snafu(source(from(Error, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[snafu(display("conf_regen/01-yunohost::post failed to enable now some systemd services"))]
    ConfRegenYunohostPostSystemCtlEnable {
        #[snafu(source(from(Error, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[snafu(display("conf_regen/01-yunohost::post failed to restart some systemd services"))]
    ConfRegenYunohostPostSystemCtlRestart {
        #[snafu(source(from(Error, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[snafu(display("TODO"))]
    TODO,
}

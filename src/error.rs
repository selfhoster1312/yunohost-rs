use camino::Utf8PathBuf;

use std::path::PathBuf;

use crate::helpers;
use crate::helpers::file::StrPath;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    // TODO: relocate
    #[snafu(display("Failed to parse TOML from string"))]
    Toml { source: toml::de::Error },

    #[snafu(display("Failed to parse YAML from string"))]
    Yaml { source: serde_yaml_ng::Error },

    #[snafu(display("Failed to parse JSON from string"))]
    Json { source: serde_json::Error },

    #[snafu(display("An error happened inside the config panel"))]
    ConfigPanel {
        source: helpers::configpanel::error::ConfigPanelError,
    },

    #[snafu(display("An error happened when translating"))]
    I18N {
        source: helpers::i18n::error::I18NError,
    },

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

    #[snafu(display("symlink failed to create a symlink to {target} due to failing to remove existing file: {link}"))]
    PathSymlinkRemove {
        target: StrPath,
        link: StrPath,
        #[snafu(source(from(Error, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[snafu(display("symlink to create a symlink to {target} at: {link}"))]
    PathSymlinkCreate {
        target: StrPath,
        link: StrPath,
        source: std::io::Error,
    },

    #[snafu(display("file_remove failed to remove file {path}"))]
    PathFileRemove {
        path: StrPath,
        source: std::io::Error,
    },

    #[snafu(display("canonicalize failed to resolve links of {path}"))]
    PathCanonicalize {
        path: StrPath,
        source: std::io::Error,
    },

    #[snafu(display("Failed to read link from {link} because the target is not valid UTF-8"))]
    PathCanonicalizeParse {
        link: StrPath,
        #[snafu(source(from(Error, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[snafu(display(
        "Failed to parse path because it's not valid UTF-8. It's approximately: {path}"
    ))]
    PathUnicode { path: String },

    #[snafu(display("Failed to read_link on path because it's not a symlink: {path}"))]
    PathReadLinkNotSymlink { path: StrPath },

    #[snafu(display("Failed to read_link on path: {path}"))]
    PathReadLink {
        path: StrPath,
        source: std::io::Error,
    },

    #[snafu(display("Failed to read link from {link} because the target is not valid UTF-8"))]
    PathReadLinkParse {
        link: StrPath,
        #[snafu(source(from(Error, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    PathTomlRead {
        path: StrPath,
        #[snafu(source(from(Error, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    PathYamlRead {
        path: StrPath,
        #[snafu(source(from(Error, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    PathJsonRead {
        path: StrPath,
        #[snafu(source(from(Error, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    // -------

    //    fn ensure_remove_file
    #[snafu(display("ensure_file_remove failed to remove file {path}"))]
    EnsureFileRemove {
        path: StrPath,
        #[snafu(source(from(Error, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    //    fn remove_file

    //    fn symlink_create

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
    // src/helpers/distro.rs
    // ===================
    #[snafu(display(
        "Your system version {version} is unsupported by Yunohost (output of lsb-release -rs)"
    ))]
    UnsupportedDebianRelease { version: String },

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

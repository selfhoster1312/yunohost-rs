use camino::Utf8PathBuf;

use std::path::PathBuf;

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
    // src/helpers/file.rs
    // ===================

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
        source: std::io::Error,
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
        query: crate::helpers::users::UserQuery,
    },

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

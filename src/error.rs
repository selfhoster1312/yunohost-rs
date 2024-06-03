use camino::Utf8PathBuf;

use crate::helpers;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("An error happened inside the config panel"))]
    ConfigPanel {
        source: helpers::configpanel::error::ConfigPanelError,
    },

    #[snafu(display("An error happened when translating"))]
    I18N {
        source: helpers::i18n::error::I18NError,
    },

    #[snafu(display("A file error occurred"))]
    File {
        source: helpers::file::error::FileError,
    },

    // ===================
    // src/helpers/apt.rs
    // ===================

    //    fn change_dpkg_vendor
    #[snafu(display("Failed to read existing DPKG vendor"))]
    ChangeDPKGVendorRead {
        #[snafu(source(from(helpers::file::error::FileError, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[snafu(display("Failed to change DPKG vendor"))]
    ChangeDPKGVendorWrite {
        #[snafu(source(from(helpers::file::error::FileError, Box::from)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    // ===================
    // src/helpers/distro.rs
    // ===================
    #[snafu(display("Failed to read Debian version from disk"))]
    Distro {
        #[snafu(source(from(helpers::file::error::FileError, Box::from)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

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
        #[snafu(source(from(helpers::file::error::FileError, Box::new)))]
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

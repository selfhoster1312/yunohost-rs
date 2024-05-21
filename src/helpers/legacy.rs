/// Migrates key names from legacy (when?) settings to ConfigPanel settings.
///
/// If a key is not found in this mapping, it's still considered valid and returned as is.
pub fn translate_legacy_settings_to_configpanel_settings(key: &str) -> &str {
    match key {
        "security.password.admin.strength" => "security.password.admin_strength",
        "security.password.user.strength" => "security.password.user_strength",
        "security.ssh.compatibility" => "security.ssh.ssh_compatibility",
        "security.ssh.port" => "security.ssh.ssh_port",
        "security.ssh.password_authentication" => "security.ssh.ssh_password_authentication",
        "security.nginx.redirect_to_https" => "security.nginx.nginx_redirect_to_https",
        "security.nginx.compatibility" => "security.nginx.nginx_compatibility",
        "security.postfix.compatibility" => "security.postfix.postfix_compatibility",
        "pop3.enabled" => "email.pop3.pop3_enabled",
        "smtp.allow_ipv6" => "email.smtp.smtp_allow_ipv6",
        "smtp.relay.host" => "email.smtp.smtp_relay_host",
        "smtp.relay.port" => "email.smtp.smtp_relay_port",
        "smtp.relay.user" => "email.smtp.smtp_relay_user",
        "smtp.relay.password" => "email.smtp.smtp_relay_password",
        "backup.compress_tar_archives" => "misc.backup.backup_compress_tar_archives",
        "ssowat.panel_overlay.enabled" => "misc.portal.ssowat_panel_overlay_enabled",
        "security.webadmin.allowlist.enabled" => "security.webadmin.webadmin_allowlist_enabled",
        "security.webadmin.allowlist" => "security.webadmin.webadmin_allowlist",
        "security.experimental.enabled" => "security.experimental.security_experimental_enabled",
        _ => key,
    }
}

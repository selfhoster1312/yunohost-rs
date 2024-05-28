use std::str::FromStr;

use super::error::ConfigPanelError;

/// Exclude a panel, section or option in a [`ConfigPanel`](super::ConfigPanel).
///
/// This is usually built from a stringy value to exclude a specific value from
/// a given config panel. For example:
///
/// - `security.webadmin` designates the `webadmin` section in the `security` panel
/// - `security.webadmin.allowlist_enabled` is the `allowlsit_enabled` option in that section
/// - `security` excludes the entire panel from the output
///
/// Any ExcludeKey is relative to a given control panel.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExcludeKey {
    Nothing,
    Panel(String),
    Section(String, String),
    Option(String, String, String),
}

impl ExcludeKey {
    pub fn excludes_panel(&self, panel_id: &str) -> bool {
        match self {
            Self::Nothing => false,
            Self::Panel(panel) => panel_id == panel,
            Self::Section(_panel, _section) => false,
            Self::Option(_panel, _section, _option) => false,
        }
    }

    // TODO: this could be more efficient by storing a single string for the filterkey, with indices...?
    pub fn excludes_section(&self, panel_id: &str, section_id: &str) -> bool {
        match self {
            Self::Nothing => false,
            Self::Panel(panel) => panel == panel_id,
            Self::Section(panel, section) => panel_id == panel && section_id == section,
            Self::Option(_panel, _section, _option) => false,
        }
    }

    // TODO: this could be more efficient by storing a single string for the filterkey, with indices...?
    pub fn excludes_option(&self, panel_id: &str, section_id: &str, option_id: &str) -> bool {
        match self {
            Self::Nothing => false,
            Self::Panel(panel) => panel == panel_id,
            Self::Section(panel, section) => panel == panel_id && section == section_id,
            Self::Option(panel, section, option) => {
                panel_id == panel && section_id == section && option_id == option
            }
        }
    }
}

impl std::fmt::Display for ExcludeKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nothing => write!(f, "NOTHING"),
            Self::Panel(p) => write!(f, "{}", p),
            Self::Section(p, s) => write!(f, "{}.{}", p, s),
            Self::Option(p, s, o) => write!(f, "{}.{}.{}", p, s, o),
        }
    }
}

impl FromStr for ExcludeKey {
    type Err = ConfigPanelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "" {
            // TODO: specific error?
            return Err(ConfigPanelError::FilterKeyNone);
        }

        let mut parts = s.split('.');

        // UNWRAP NOTE: There is always one part, even when there is no '.'
        let panel = parts.next().unwrap().to_string();

        match parts.next() {
            None => Ok(Self::Panel(panel)),
            Some(section) => {
                let section = section.to_string();
                match parts.next() {
                    None => Ok(Self::Section(panel, section)),
                    Some(option) => {
                        if parts.next().is_some() {
                            // TODO: specific error
                            return Err(ConfigPanelError::FilterKeyTooDeep {
                                filter_key: s.to_string(),
                            });
                        }

                        let option = option.to_string();
                        Ok(Self::Option(panel, section, option))
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_panel() {
        assert_eq!(
            ExcludeKey::Panel("panel".to_string()),
            ExcludeKey::from_str("panel").unwrap(),
        );
    }

    #[test]
    fn valid_section() {
        assert_eq!(
            ExcludeKey::Section("panel".to_string(), "section".to_string()),
            ExcludeKey::from_str("panel.section").unwrap(),
        );
    }

    #[test]
    fn valid_option() {
        assert_eq!(
            ExcludeKey::Option(
                "panel".to_string(),
                "section".to_string(),
                "option".to_string()
            ),
            ExcludeKey::from_str("panel.section.option").unwrap(),
        );
    }

    #[test]
    fn invalid_empty_filter_key() {
        assert_eq!(
            Err(ConfigPanelError::FilterKeyNone),
            ExcludeKey::from_str(""),
        );
    }

    #[test]
    fn invalid_too_deep_filter_key() {
        assert_eq!(
            Err(ConfigPanelError::FilterKeyTooDeep {
                filter_key: "a.b.c.d.e".to_string()
            }),
            ExcludeKey::from_str("a.b.c.d.e"),
        );
    }

    #[test]
    fn panel_matches_panel() {
        let filter = ExcludeKey::Panel("foo".to_string());
        assert_eq!(filter.excludes_panel("foo"), true,);
        assert_eq!(filter.excludes_panel("Foo"), false,);
        assert_eq!(filter.excludes_panel("bar"), false,);
    }

    #[test]
    fn panel_matches_section() {
        let filter = ExcludeKey::Panel("foo".to_string());
        assert_eq!(filter.excludes_section("foo", "bar"), true,);
        assert_eq!(filter.excludes_section("Foo", "bar"), false,);
        assert_eq!(filter.excludes_section("bar", "foo"), false,);
    }

    #[test]
    fn panel_matches_option() {
        let filter = ExcludeKey::Panel("foo".to_string());
        assert_eq!(filter.excludes_option("foo", "bar", "baz"), true,);
        assert_eq!(filter.excludes_option("Foo", "bar", "baz"), false,);
        assert_eq!(filter.excludes_option("bar", "bar", "baz"), false,);
    }

    #[test]
    fn section_matches_panel() {
        let filter = ExcludeKey::Section("foo".to_string(), "bar".to_string());
        assert_eq!(filter.excludes_panel("foo"), false,);
        assert_eq!(filter.excludes_panel("Foo"), false,);
        assert_eq!(filter.excludes_panel("bar"), false,);
    }

    #[test]
    fn section_matches_section() {
        let filter = ExcludeKey::Section("foo".to_string(), "bar".to_string());
        assert_eq!(filter.excludes_section("foo", "bar"), true,);
        assert_eq!(filter.excludes_section("Foo", "bar"), false,);
        assert_eq!(filter.excludes_section("bar", "foo"), false,);
    }

    #[test]
    fn section_matches_option() {
        let filter = ExcludeKey::Section("foo".to_string(), "bar".to_string());
        assert_eq!(filter.excludes_option("foo", "bar", "baz"), true,);
        assert_eq!(filter.excludes_option("Foo", "bar", "baz"), false,);
        assert_eq!(filter.excludes_option("bar", "foo", "baz"), false,);
    }

    #[test]
    fn option_matches_panel() {
        let filter = ExcludeKey::Option("foo".to_string(), "bar".to_string(), "baz".to_string());
        assert_eq!(filter.excludes_panel("foo"), false,);
        assert_eq!(filter.excludes_panel("Foo"), false,);
        assert_eq!(filter.excludes_panel("bar"), false,);
    }

    #[test]
    fn option_matches_section() {
        let filter = ExcludeKey::Option("foo".to_string(), "bar".to_string(), "baz".to_string());
        assert_eq!(filter.excludes_section("foo", "bar"), false,);
        assert_eq!(filter.excludes_section("Foo", "bar"), false,);
        assert_eq!(filter.excludes_section("bar", "foo"), false,);
    }

    #[test]
    fn option_matches_option() {
        let filter = ExcludeKey::Option("foo".to_string(), "bar".to_string(), "baz".to_string());
        assert_eq!(filter.excludes_option("foo", "bar", "baz"), true,);
        assert_eq!(filter.excludes_option("Foo", "bar", "baz"), false,);
        assert_eq!(filter.excludes_option("bar", "foo", "baz"), false,);
    }
}

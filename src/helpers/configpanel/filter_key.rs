use std::str::FromStr;

use super::error::ConfigPanelError;

/// Extract a panel, section or option in a [`ConfigPanel`](super::ConfigPanel).
///
/// This is usually built from a stringy value to extract a specific value from
/// a given config panel. For example:
///
/// - `security.webadmin` designates the `webadmin` section in the `security` panel
/// - `security.webadmin.allowlist_enabled` is the `allowlsit_enabled` option in that section
///
/// Any FilterKey is relative to a given control panel.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterKey {
    Everything,
    Panel(String),
    Section(String, String),
    Option(String, String, String),
}

impl FilterKey {
    pub fn matches_panel(&self, panel_id: &str) -> bool {
        match self {
            Self::Panel(panel) => panel_id == panel,
            Self::Section(panel, _section) => panel_id == panel,
            Self::Option(panel, _section, _option) => panel_id == panel,
            Self::Everything => true,
        }
    }

    // TODO: this could be more efficient by storing a single string for the filterkey, with indices...?
    pub fn matches_section(&self, panel_id: &str, section_id: &str) -> bool {
        match self {
            Self::Panel(panel) => panel_id == panel,
            Self::Section(panel, section) => panel_id == panel && section_id == section,
            Self::Option(panel, section, _option) => panel_id == panel && section_id == section,
            Self::Everything => true,
        }
    }

    // TODO: this could be more efficient by storing a single string for the filterkey, with indices...?
    pub fn matches_option(&self, panel_id: &str, section_id: &str, option_id: &str) -> bool {
        match self {
            Self::Panel(panel) => panel_id == panel,
            Self::Section(panel, section) => panel_id == panel && section_id == section,
            Self::Option(panel, section, option) => {
                panel_id == panel && section_id == section && option_id == option
            }
            Self::Everything => true,
        }
    }
}

impl std::fmt::Display for FilterKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Panel(p) => write!(f, "{}", p),
            Self::Section(p, s) => write!(f, "{}.{}", p, s),
            Self::Option(p, s, o) => write!(f, "{}.{}.{}", p, s, o),
            Self::Everything => write!(f, "EVERYTHING"),
        }
    }
}

impl FromStr for FilterKey {
    type Err = ConfigPanelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "" {
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
            FilterKey::Panel("panel".to_string()),
            FilterKey::from_str("panel").unwrap(),
        );
    }

    #[test]
    fn valid_section() {
        assert_eq!(
            FilterKey::Section("panel".to_string(), "section".to_string()),
            FilterKey::from_str("panel.section").unwrap(),
        );
    }

    #[test]
    fn valid_option() {
        assert_eq!(
            FilterKey::Option(
                "panel".to_string(),
                "section".to_string(),
                "option".to_string()
            ),
            FilterKey::from_str("panel.section.option").unwrap(),
        );
    }

    #[test]
    fn invalid_empty_filter_key() {
        assert_eq!(
            ConfigPanelError::FilterKeyNone,
            FilterKey::from_str("").unwrap_err(),
        );
    }

    #[test]
    fn invalid_too_deep_filter_key() {
        assert_eq!(
            ConfigPanelError::FilterKeyTooDeep {
                filter_key: "a.b.c.d.e".to_string()
            },
            FilterKey::from_str("a.b.c.d.e").unwrap_err(),
        );
    }

    #[test]
    fn panel_matches_panel() {
        let filter = FilterKey::Panel("foo".to_string());
        assert_eq!(filter.matches_panel("foo"), true,);
        assert_eq!(filter.matches_panel("Foo"), false,);
        assert_eq!(filter.matches_panel("bar"), false,);
    }

    #[test]
    fn panel_matches_section() {
        let filter = FilterKey::Panel("foo".to_string());
        assert_eq!(filter.matches_section("foo", "bar"), true,);
        assert_eq!(filter.matches_section("Foo", "bar"), false,);
        assert_eq!(filter.matches_section("bar", "foo"), false,);
    }

    #[test]
    fn panel_matches_option() {
        let filter = FilterKey::Panel("foo".to_string());
        assert_eq!(filter.matches_option("foo", "bar", "baz"), true,);
        assert_eq!(filter.matches_option("Foo", "bar", "baz"), false,);
        assert_eq!(filter.matches_option("bar", "bar", "baz"), false,);
    }

    #[test]
    fn section_matches_panel() {
        let filter = FilterKey::Section("foo".to_string(), "bar".to_string());
        assert_eq!(filter.matches_panel("foo"), true,);
        assert_eq!(filter.matches_panel("Foo"), false,);
        assert_eq!(filter.matches_panel("bar"), false,);
    }

    #[test]
    fn section_matches_section() {
        let filter = FilterKey::Section("foo".to_string(), "bar".to_string());
        assert_eq!(filter.matches_section("foo", "bar"), true,);
        assert_eq!(filter.matches_section("Foo", "bar"), false,);
        assert_eq!(filter.matches_section("bar", "foo"), false,);
    }

    #[test]
    fn section_matches_option() {
        let filter = FilterKey::Section("foo".to_string(), "bar".to_string());
        assert_eq!(filter.matches_option("foo", "bar", "baz"), true,);
        assert_eq!(filter.matches_option("Foo", "bar", "baz"), false,);
        assert_eq!(filter.matches_option("bar", "foo", "baz"), false,);
    }

    #[test]
    fn option_matches_panel() {
        let filter = FilterKey::Option("foo".to_string(), "bar".to_string(), "baz".to_string());
        assert_eq!(filter.matches_panel("foo"), true,);
        assert_eq!(filter.matches_panel("Foo"), false,);
        assert_eq!(filter.matches_panel("bar"), false,);
    }

    #[test]
    fn option_matches_section() {
        let filter = FilterKey::Option("foo".to_string(), "bar".to_string(), "baz".to_string());
        assert_eq!(filter.matches_section("foo", "bar"), true,);
        assert_eq!(filter.matches_section("Foo", "bar"), false,);
        assert_eq!(filter.matches_section("bar", "foo"), false,);
    }

    #[test]
    fn option_matches_option() {
        let filter = FilterKey::Option("foo".to_string(), "bar".to_string(), "baz".to_string());
        assert_eq!(filter.matches_option("foo", "bar", "baz"), true,);
        assert_eq!(filter.matches_option("Foo", "bar", "baz"), false,);
        assert_eq!(filter.matches_option("bar", "foo", "baz"), false,);
    }
}

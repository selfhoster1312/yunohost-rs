use std::str::FromStr;

use super::error::ConfigPanelError;

/// A panel, section or option in a [`ConfigPanel`](super::ConfigPanel).
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
    Panel(String),
    Section(String, String),
    Option(String, String, String),
}

impl std::fmt::Display for FilterKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Panel(p) => write!(f, "{}", p),
            Self::Section(p, s) => write!(f, "{}.{}", p, s),
            Self::Option(p, s, o) => write!(f, "{}.{}.{}", p, s, o),
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
            Err(ConfigPanelError::FilterKeyNone),
            FilterKey::from_str(""),
        );
    }

    #[test]
    fn invalid_too_deep_filter_key() {
        assert_eq!(
            Err(ConfigPanelError::FilterKeyTooDeep {
                filter_key: "a.b.c.d.e".to_string()
            }),
            FilterKey::from_str("a.b.c.d.e"),
        );
    }
}

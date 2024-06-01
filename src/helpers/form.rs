use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum::{Display, EnumString};

use crate::helpers::distro::{debian_version, DebianRelease};

#[derive(
    Copy, Clone, Debug, EnumString, Display, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord,
)]
#[strum(serialize_all = "snake_case")]
pub enum OptionType {
    // display
    DisplayText,
    Markdown,
    Alert,
    // action
    Button,
    // text
    String,
    Text,
    Password,
    Color,
    // numeric
    Number,
    Range,
    // boolean
    Boolean,
    // time
    Date,
    Time,
    // location
    Email,
    Path,
    Url,
    // file
    File,
    // choice
    Select,
    Tags,
    // entity
    Domain,
    App,
    User,
    Group,
}

impl OptionType {
    pub fn to_option_type(&self) -> Box<dyn OptionTypeInterface> {
        match self {
            Self::DisplayText => Box::new(DisplayTextOption),
            Self::Markdown => Box::new(MarkdownOption),
            Self::Alert => Box::new(AlertOption),
            Self::Button => Box::new(ButtonOption),
            Self::String => Box::new(TextOption),
            Self::Text => Box::new(TextOption),
            Self::Password => Box::new(PasswordOption),
            Self::Color => Box::new(ColorOption),
            Self::Number => Box::new(NumberOption),
            Self::Range => Box::new(NumberOption),
            Self::Boolean => Box::new(BooleanOption),
            Self::Date => Box::new(DateOption),
            Self::Time => Box::new(TimeOption),
            Self::Email => Box::new(EmailOption),
            Self::Path => Box::new(PathOption),
            Self::Url => Box::new(UrlOption),
            Self::File => Box::new(FileOption),
            Self::Select => Box::new(SelectOption),
            Self::Tags => Box::new(TagsOption),
            Self::Domain => Box::new(DomainOption),
            Self::App => Box::new(AppOption),
            Self::User => Box::new(UserOption),
            Self::Group => Box::new(GroupOption),
        }
    }
}

pub trait OptionTypeInterface {
    /// Whether the actual value should be hidden in output (eg. password/secret)
    fn hide_user_input_in_prompt(&self) -> bool;
    /// Normalization takes any toml::Value and turns it into a properly-typed Value.
    /// This process happens in classic view when requesting a single entry.
    fn normalize(&self, val: &Value) -> Option<Value>;
    /// Humanization takes the normalized value and formats it for output.
    /// This process happens in classic view when requesting multiple values in a broader filter key.
    fn humanize(&self, val: &Value) -> Option<String>;
    /// Defines some extra fields to add to default values in full mode.
    /// The extra fields depend on Debian release
    fn full_extra_fields(
        &self,
        option_id: &str,
        release: DebianRelease,
    ) -> Option<Vec<(String, Value)>>;
}

pub struct DisplayTextOption;
impl OptionTypeInterface for DisplayTextOption {
    fn hide_user_input_in_prompt(&self) -> bool {
        false
    }

    fn normalize(&self, _val: &Value) -> Option<Value> {
        None
    }

    fn humanize(&self, _val: &Value) -> Option<String> {
        None
    }

    fn full_extra_fields(
        &self,
        _option_id: &str,
        _release: DebianRelease,
    ) -> Option<Vec<(String, Value)>> {
        None
    }
}

pub struct MarkdownOption;
impl OptionTypeInterface for MarkdownOption {
    fn hide_user_input_in_prompt(&self) -> bool {
        false
    }

    fn normalize(&self, _val: &Value) -> Option<Value> {
        None
    }

    fn humanize(&self, _val: &Value) -> Option<String> {
        None
    }

    fn full_extra_fields(
        &self,
        _option_id: &str,
        _release: DebianRelease,
    ) -> Option<Vec<(String, Value)>> {
        None
    }
}

pub struct AlertOption;
impl OptionTypeInterface for AlertOption {
    fn hide_user_input_in_prompt(&self) -> bool {
        false
    }

    fn normalize(&self, _val: &Value) -> Option<Value> {
        None
    }

    fn humanize(&self, _val: &Value) -> Option<String> {
        None
    }

    fn full_extra_fields(
        &self,
        _option_id: &str,
        _release: DebianRelease,
    ) -> Option<Vec<(String, Value)>> {
        None
    }
}

pub struct ButtonOption;
impl OptionTypeInterface for ButtonOption {
    fn hide_user_input_in_prompt(&self) -> bool {
        false
    }

    fn normalize(&self, _val: &Value) -> Option<Value> {
        None
    }

    fn humanize(&self, _val: &Value) -> Option<String> {
        None
    }

    fn full_extra_fields(
        &self,
        _option_id: &str,
        _release: DebianRelease,
    ) -> Option<Vec<(String, Value)>> {
        None
    }
}

pub struct TextOption;
impl OptionTypeInterface for TextOption {
    fn hide_user_input_in_prompt(&self) -> bool {
        false
    }

    fn normalize(&self, val: &Value) -> Option<Value> {
        match debian_version().unwrap() {
            DebianRelease::Bookworm => {
                if let Some(s) = val.as_str() {
                    if s == "" {
                        return Some(Value::Null);
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn humanize(&self, _val: &Value) -> Option<String> {
        None
    }

    fn full_extra_fields(
        &self,
        _option_id: &str,
        _release: DebianRelease,
    ) -> Option<Vec<(String, Value)>> {
        None
    }
}

pub struct PasswordOption;
impl OptionTypeInterface for PasswordOption {
    fn hide_user_input_in_prompt(&self) -> bool {
        true
    }

    fn normalize(&self, val: &Value) -> Option<Value> {
        match debian_version().unwrap() {
            DebianRelease::Bookworm => {
                if let Some(s) = val.as_str() {
                    if s == "" {
                        return Some(Value::Null);
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn humanize(&self, _val: &Value) -> Option<String> {
        None
    }

    fn full_extra_fields(
        &self,
        _option_id: &str,
        release: DebianRelease,
    ) -> Option<Vec<(String, Value)>> {
        match release {
            DebianRelease::Bookworm => Some(vec![("redact".to_string(), Value::Bool(true))]),
            _ => None,
        }
    }
}

pub struct ColorOption;
impl OptionTypeInterface for ColorOption {
    fn hide_user_input_in_prompt(&self) -> bool {
        false
    }

    fn normalize(&self, _val: &Value) -> Option<Value> {
        None
    }

    fn humanize(&self, _val: &Value) -> Option<String> {
        None
    }

    fn full_extra_fields(
        &self,
        _option_id: &str,
        _release: DebianRelease,
    ) -> Option<Vec<(String, Value)>> {
        None
    }
}

pub struct NumberOption;
impl OptionTypeInterface for NumberOption {
    fn hide_user_input_in_prompt(&self) -> bool {
        false
    }

    fn normalize(&self, _val: &Value) -> Option<Value> {
        None
    }

    fn humanize(&self, val: &Value) -> Option<String> {
        if let Some(n) = val.as_u64() {
            return Some(n.to_string());
        }
        panic!();
    }

    fn full_extra_fields(
        &self,
        _option_id: &str,
        _release: DebianRelease,
    ) -> Option<Vec<(String, Value)>> {
        None
    }
}

pub struct BooleanOption;
impl OptionTypeInterface for BooleanOption {
    fn hide_user_input_in_prompt(&self) -> bool {
        false
    }

    fn normalize(&self, val: &Value) -> Option<Value> {
        let b = if let Some(b) = val.as_bool() {
            if b {
                1
            } else {
                0
            }
        } else if let Some(s) = val.as_str() {
            match s {
                "1" | "yes" | "y" | "true" | "t" | "on" => 1,
                "0" | "no" | "n" | "false" | "f" | "off" => 0,
                _ => {
                    panic!("THIS IS SUPER DUPER WRONG");
                }
            }
        } else {
            panic!("THIS IS WRONG!");
        };
        Some(Value::Number(b.into()))
    }

    fn humanize(&self, val: &Value) -> Option<String> {
        if self.normalize(val).unwrap().as_u64().unwrap() == 1 {
            Some("yes".to_string())
        } else {
            Some("no".to_string())
        }
    }

    fn full_extra_fields(
        &self,
        _option_id: &str,
        release: DebianRelease,
    ) -> Option<Vec<(String, Value)>> {
        match release {
            DebianRelease::Bookworm => Some(vec![
                ("yes".to_string(), Value::Number(1.into())),
                ("no".to_string(), Value::Number(0.into())),
            ]),
            _ => None,
        }
    }
}

pub struct DateOption;
impl OptionTypeInterface for DateOption {
    fn hide_user_input_in_prompt(&self) -> bool {
        false
    }

    fn normalize(&self, _val: &Value) -> Option<Value> {
        None
    }

    fn humanize(&self, _val: &Value) -> Option<String> {
        None
    }

    fn full_extra_fields(
        &self,
        _option_id: &str,
        _release: DebianRelease,
    ) -> Option<Vec<(String, Value)>> {
        None
    }
}

pub struct TimeOption;
impl OptionTypeInterface for TimeOption {
    fn hide_user_input_in_prompt(&self) -> bool {
        false
    }

    fn normalize(&self, _val: &Value) -> Option<Value> {
        None
    }

    fn humanize(&self, _val: &Value) -> Option<String> {
        None
    }

    fn full_extra_fields(
        &self,
        _option_id: &str,
        _release: DebianRelease,
    ) -> Option<Vec<(String, Value)>> {
        None
    }
}

pub struct EmailOption;
impl OptionTypeInterface for EmailOption {
    fn hide_user_input_in_prompt(&self) -> bool {
        false
    }

    fn normalize(&self, _val: &Value) -> Option<Value> {
        None
    }

    fn humanize(&self, _val: &Value) -> Option<String> {
        None
    }

    fn full_extra_fields(
        &self,
        _option_id: &str,
        _release: DebianRelease,
    ) -> Option<Vec<(String, Value)>> {
        None
    }
}

pub struct PathOption;
impl OptionTypeInterface for PathOption {
    fn hide_user_input_in_prompt(&self) -> bool {
        false
    }

    fn normalize(&self, _val: &Value) -> Option<Value> {
        None
    }

    fn humanize(&self, _val: &Value) -> Option<String> {
        None
    }

    fn full_extra_fields(
        &self,
        _option_id: &str,
        _release: DebianRelease,
    ) -> Option<Vec<(String, Value)>> {
        None
    }
}

pub struct UrlOption;
impl OptionTypeInterface for UrlOption {
    fn hide_user_input_in_prompt(&self) -> bool {
        false
    }

    fn normalize(&self, _val: &Value) -> Option<Value> {
        None
    }

    fn humanize(&self, _val: &Value) -> Option<String> {
        None
    }

    fn full_extra_fields(
        &self,
        _option_id: &str,
        _release: DebianRelease,
    ) -> Option<Vec<(String, Value)>> {
        None
    }
}

pub struct FileOption;
impl OptionTypeInterface for FileOption {
    fn hide_user_input_in_prompt(&self) -> bool {
        false
    }

    fn normalize(&self, _val: &Value) -> Option<Value> {
        None
    }

    fn humanize(&self, _val: &Value) -> Option<String> {
        None
    }

    fn full_extra_fields(
        &self,
        _option_id: &str,
        _release: DebianRelease,
    ) -> Option<Vec<(String, Value)>> {
        None
    }
}

pub struct SelectOption;
impl OptionTypeInterface for SelectOption {
    fn hide_user_input_in_prompt(&self) -> bool {
        false
    }

    fn normalize(&self, _val: &Value) -> Option<Value> {
        None
    }

    fn humanize(&self, _val: &Value) -> Option<String> {
        None
    }

    fn full_extra_fields(
        &self,
        option_id: &str,
        _release: DebianRelease,
    ) -> Option<Vec<(String, Value)>> {
        // TODO: actually read_dir the themes from /usr/share/ssowat/portal/assets/themes
        // TODO: Probably no longer the case on bookworm
        // `portal_theme` is a special case...
        if option_id == "portal_theme" {
            Some(vec![(
                "choices".to_string(),
                serde_json::to_value(vec!["unsplash", "vapor", "light", "default", "clouds"])
                    .unwrap(),
            )])
        } else {
            None
        }
    }
}

pub struct TagsOption;
impl OptionTypeInterface for TagsOption {
    fn hide_user_input_in_prompt(&self) -> bool {
        false
    }

    fn normalize(&self, _val: &Value) -> Option<Value> {
        None
    }

    fn humanize(&self, _val: &Value) -> Option<String> {
        None
    }

    fn full_extra_fields(
        &self,
        _option_id: &str,
        release: DebianRelease,
    ) -> Option<Vec<(String, Value)>> {
        match release {
            DebianRelease::Bullseye => Some(vec![("choices".to_string(), Value::Null)]),
            _ => None,
        }
    }
}

pub struct DomainOption;
impl OptionTypeInterface for DomainOption {
    fn hide_user_input_in_prompt(&self) -> bool {
        false
    }

    fn normalize(&self, val: &Value) -> Option<Value> {
        let s = if let Some(s) = val.as_str() {
            s.trim_start_matches("https://")
                .trim_start_matches("http://")
                .trim_end_matches('/')
                .to_string()
        } else {
            panic!(
                "Called DomainOption::normalize with a non-string;y _value: {:?}",
                val
            )
        };

        Some(Value::String(s))
    }

    fn humanize(&self, _val: &Value) -> Option<String> {
        None
    }

    fn full_extra_fields(
        &self,
        _option_id: &str,
        _release: DebianRelease,
    ) -> Option<Vec<(String, Value)>> {
        None
    }
}

pub struct AppOption;
impl OptionTypeInterface for AppOption {
    fn hide_user_input_in_prompt(&self) -> bool {
        false
    }

    fn normalize(&self, _val: &Value) -> Option<Value> {
        None
    }

    fn humanize(&self, _val: &Value) -> Option<String> {
        None
    }

    fn full_extra_fields(
        &self,
        _option_id: &str,
        _release: DebianRelease,
    ) -> Option<Vec<(String, Value)>> {
        None
    }
}

pub struct UserOption;
impl OptionTypeInterface for UserOption {
    fn hide_user_input_in_prompt(&self) -> bool {
        false
    }

    fn normalize(&self, _val: &Value) -> Option<Value> {
        None
    }

    fn humanize(&self, _val: &Value) -> Option<String> {
        None
    }

    fn full_extra_fields(
        &self,
        _option_id: &str,
        _release: DebianRelease,
    ) -> Option<Vec<(String, Value)>> {
        None
    }
}

pub struct GroupOption;
impl OptionTypeInterface for GroupOption {
    fn hide_user_input_in_prompt(&self) -> bool {
        false
    }

    fn normalize(&self, _val: &Value) -> Option<Value> {
        None
    }

    fn humanize(&self, _val: &Value) -> Option<String> {
        None
    }

    fn full_extra_fields(
        &self,
        _option_id: &str,
        _release: DebianRelease,
    ) -> Option<Vec<(String, Value)>> {
        None
    }
}

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use toml::Value;

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

// TODO: what does this do? normalize... and ?
// What is the difference between normalize and humanize?

pub trait OptionTypeInterface {
    /// Whether the actual value should be hidden in output (eg. password/secret)
    fn hide_user_input_in_prompt(&self) -> bool;
    /// Normalization takes any toml::Value and turns it into a properly-typed Value. Fallible process
    fn normalize(&self, val: &Value) -> Option<Value>;
    /// Humanization takes the normalized value and formats it for output.
    fn humanize(&self, val: &Value) -> Option<String>;
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
}

pub struct TextOption;
impl OptionTypeInterface for TextOption {
    fn hide_user_input_in_prompt(&self) -> bool {
        false
    }

    fn normalize(&self, _val: &Value) -> Option<Value> {
        None
    }

    fn humanize(&self, _val: &Value) -> Option<String> {
        None
    }
}

pub struct PasswordOption;
impl OptionTypeInterface for PasswordOption {
    fn hide_user_input_in_prompt(&self) -> bool {
        true
    }

    fn normalize(&self, _val: &Value) -> Option<Value> {
        None
    }

    fn humanize(&self, _val: &Value) -> Option<String> {
        None
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
        if let Some(n) = val.as_integer() {
            return Some(n.to_string());
        }
        panic!();
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
        Some(Value::Integer(b))
    }

    fn humanize(&self, val: &Value) -> Option<String> {
        if self.normalize(val).unwrap().as_integer().unwrap() == 1 {
            Some("yes".to_string())
        } else {
            Some("no".to_string())
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
}

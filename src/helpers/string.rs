//! [`String`] and [`&str`]-related helpers.
//! 
//! Not a lot here yet...

/// Capitalize the first letter of the given stringy value, returning a new string.
///
/// Example:
///
/// ```rust
/// // Outputs: Hello, world!
/// println("{}", capitalize("hello, world!"));
/// # assert_eq!("Hello, world!".to_string(), capitalize("hello, world!"));
/// ```
pub fn capitalize<T: AsRef<str>>(string: T) -> String {
    let mut chars = string.as_ref().chars();
    match chars.next() {
        None => String::new(),
        Some(first_letter) => first_letter.to_uppercase().chain(chars).collect(),
    }
}

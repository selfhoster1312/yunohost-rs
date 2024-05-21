pub fn capitalize<T: AsRef<str>>(string: T) -> String {
    let mut chars = string.as_ref().chars();
    match chars.next() {
        None => String::new(),
        Some(first_letter) => first_letter.to_uppercase().chain(chars).collect(),
    }
}

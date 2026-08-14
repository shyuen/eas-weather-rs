pub fn capitalize_first_lowercase_rest(s: &str) -> String {
    // Convert the entire string to lowercase first
    let lowercase_s = s.to_lowercase();

    // Get an iterator over the characters of the lowercase string
    let mut chars = lowercase_s.chars();

    // Take the first character, convert it to uppercase, and chain the rest of the characters
    match chars.next() {
        None => String::new(), // Return an empty string if the input is empty
        Some(first_char) => {
            first_char
                .to_uppercase() // Convert the first char to uppercase
                .chain(chars) // Chain the rest of the (lowercase) characters
                .collect() // Collect the result into a new String
        }
    }
}

/// Serializes a value that implements `Display` as a string using its `to_string()` method.
/// Useful for hiding sensitive information (e.g., database connection strings) when serializing to formats like JSON.
pub fn serialize_with_display<S, K>(conn_string: &K, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    K: std::fmt::Display,
{
    if &conn_string.to_string() == "None" {
        return serializer.serialize_none();
    }

    serializer.serialize_str(&conn_string.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct Wrapper<'a> {
        #[serde(serialize_with = "serialize_with_display")]
        value: &'a Wrapped,
    }

    #[derive(Debug)]
    enum Wrapped {
        Some(String),
        None,
    }

    impl std::fmt::Display for Wrapped {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Wrapped::Some(s) => write!(f, "{}", s),
                Wrapped::None => write!(f, "None"),
            }
        }
    }

    #[test]
    fn test_capitalize_first_lowercase_rest() {
        assert_eq!(
            capitalize_first_lowercase_rest("hELLo WORld"),
            "Hello world"
        );
        assert_eq!(capitalize_first_lowercase_rest("RUST"), "Rust");
        assert_eq!(capitalize_first_lowercase_rest("rust"), "Rust");
        assert_eq!(capitalize_first_lowercase_rest("r"), "R");
        assert_eq!(capitalize_first_lowercase_rest(""), "");
    }

    #[test]
    fn serialize_with_display_writes_display_string() {
        let wrapped = Wrapped::Some("mysql://user@db/app".to_string());
        let json = serde_json::to_string(&Wrapper { value: &wrapped }).unwrap();
        assert_eq!(json, r#"{"value":"mysql://user@db/app"}"#);
    }

    #[test]
    fn serialize_with_display_maps_none_to_null() {
        let wrapped = Wrapped::None;
        let json = serde_json::to_string(&Wrapper { value: &wrapped }).unwrap();
        assert_eq!(json, r#"{"value":null}"#);
    }
}

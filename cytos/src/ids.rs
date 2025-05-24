//! Handles the creation and management of unique identifiers (IDs) within the application.
//!
//! This module provides a macro to define type-safe ID structs and helper functions for
//! converting between numerical IDs and their base36 string representations for
//! serialization and external use.

use super::Result;
use serde::de::Error as DeError;
use serde::ser::Error as SerError;

/// Macro to create a type-safe ID struct.
///
/// This macro generates a struct with a `u64` inner value and implements necessary traits
/// for equality, cloning, hashing, ordering, and importantly, `serde::Serialize` and
/// `serde::Deserialize`. Serialization and deserialization are handled using a base36
/// string representation of the inner `u64` value.
macro_rules! create_ids {
    ($struct_name:ident) => {
        #[derive(PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
        pub struct $struct_name(pub u64);

        impl serde::Serialize for $struct_name {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let value = format_radix(self.0, 36)
                    .map_err(|v| <S as serde::Serializer>::Error::custom(v))?;

                serializer.serialize_str(&value)
            }
        }

        impl<'de> serde::Deserialize<'de> for $struct_name {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;

                let value = u64::from_str_radix(&value, 36)
                    .map_err(|v| <D as serde::Deserializer<'de>>::Error::custom(v))?;
                Ok($struct_name(value))
            }
        }

        impl std::fmt::Debug for $struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match format_radix(self.0, 36) {
                    Ok(v) => write!(f, "{}", v),
                    Err(e) => write!(f, "Error formatting ID: {}", e),
                }
            }
        }

        impl std::fmt::Display for $struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match format_radix(self.0, 36) {
                    Ok(v) => write!(f, "{}", v),
                    Err(e) => write!(f, "Error formatting ID: {}", e),
                }
            }
        }

        impl From<String> for $struct_name {
            fn from(value: String) -> Self {
                let value = serde_json::to_string(&value).unwrap();
                serde_json::from_str(&value).unwrap()
            }
        }
    };
}

/// Converts a `u64` number into its base36 string representation.
///
/// This is primarily used for external representation and serialization of IDs.
///
/// # Arguments
///
/// * `x` - The `u64` number to convert.
///
/// # Errors
///
/// Will return `Err` if the number cannot be converted to a string in base36.
pub fn id_number_to_string(x: u64) -> Result<String> {
    format_radix(x, 36)
}

/// Converts a base36 string representation of an ID back into a `u64` number.
///
/// This is primarily used for parsing external representations or during deserialization.
///
/// # Arguments
///
/// * `value` - The base36 string to convert.
///
/// # Errors
///
/// Will return `Err` if the string is not a valid base36 representation or cannot be
/// converted to a `u64`.
pub fn id_string_to_number(value: &str) -> Result<u64> {
    u64::from_str_radix(value, 36).map_err(std::convert::Into::into)
}

/// Formats a `u64` number into a string representation using the specified radix.
///
/// This function repeatedly divides the number by the radix and appends the remainder's
/// digit character to a result string until the number becomes zero.
///
/// # Arguments
///
/// * `x` - The `u64` number to format.
/// * `radix` - The base (radix) to use for formatting (e.g., 10 for decimal, 36 for base36).
///
/// # Errors
///
/// Returns `Err` if the number cannot be converted to a string in the given `radix`,
/// which might happen if a digit character cannot be obtained for a remainder.
fn format_radix(mut x: u64, radix: u32) -> Result<String> {
    let mut result = vec![];
    let r = u64::from(radix);
    loop {
        let m = x % r;
        x /= r;

        result.push(std::char::from_digit(u32::try_from(m)?, radix).ok_or("cannot convert")?);
        if x == 0 {
            break;
        }
    }
    Ok(result.into_iter().rev().collect())
}

create_ids!(GraphId);
create_ids!(NodeId);
create_ids!(ParamId);

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_format_radix() {
        assert_eq!(format_radix(120, 10).unwrap(), "120".to_owned());
        assert_eq!(format_radix(8, 2).unwrap(), "1000".to_owned());
        assert_eq!(format_radix(35, 36).unwrap(), "z".to_owned());
        assert_eq!(format_radix(36, 36).unwrap(), "10".to_owned());
        assert_eq!(format_radix(123456, 36).unwrap(), "2n9c".to_owned());
    }

    #[test]
    fn test_id_number_to_string() {
        assert_eq!(id_number_to_string(0).unwrap(), "0".to_owned());
        assert_eq!(id_number_to_string(1).unwrap(), "1".to_owned());
        assert_eq!(id_number_to_string(9).unwrap(), "9".to_owned());
        assert_eq!(id_number_to_string(10).unwrap(), "a".to_owned());
        assert_eq!(id_number_to_string(35).unwrap(), "z".to_owned());
        assert_eq!(id_number_to_string(36).unwrap(), "10".to_owned());
        assert_eq!(id_number_to_string(123456).unwrap(), "2n9c".to_owned());
    }

    #[test]
    fn test_id_string_to_number() {
        assert_eq!(id_string_to_number("0").unwrap(), 0);
        assert_eq!(id_string_to_number("1").unwrap(), 1);
        assert_eq!(id_string_to_number("9").unwrap(), 9);
        assert_eq!(id_string_to_number("a").unwrap(), 10);
        assert_eq!(id_string_to_number("z").unwrap(), 35);
        assert_eq!(id_string_to_number("10").unwrap(), 36);
        assert_eq!(id_string_to_number("2n9c").unwrap(), 123456);

        // Test invalid base36 strings
        assert!(id_string_to_number(".").is_err());
        assert!(id_string_to_number("-1").is_err());
        assert!(id_string_to_number("!invalid").is_err());
    }

    #[test]
    fn test_graph_id() {
        let g = GraphId(1);
        let v = format!("{}", g);
        assert_eq!(v, "1".to_owned());

        let g2 = GraphId(123456);
        let v2 = format!("{}", g2);
        assert_eq!(v2, "2n9c".to_owned());
    }

    #[test]
    fn test_graph_id_serde() {
        let original_id = GraphId(123456);
        let serialized = serde_json::to_string(&original_id).unwrap();
        // Base36 representation of 123456 is "2N9C"
        assert_eq!(serialized, r#""2n9c""#);

        let deserialized_id: GraphId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized_id, original_id);

        // Test deserialization of another value
        let serialized_zero = r#""0""#;
        let deserialized_zero: GraphId = serde_json::from_str(serialized_zero).unwrap();
        assert_eq!(deserialized_zero, GraphId(0));

        let serialized_single_char = r#""F""#; // F is 15 in base36
        let deserialized_single_char: GraphId =
            serde_json::from_str(serialized_single_char).unwrap();
        assert_eq!(deserialized_single_char, GraphId(15));

        // Test deserialization of invalid base36 string
        let invalid_serialized = r#""!invalid""#;
        let result: std::result::Result<GraphId, _> = serde_json::from_str(invalid_serialized);
        assert!(result.is_err());

        // Test deserialization of incorrect JSON format
        let incorrect_json = r#"123456"#; // Should be a string, not a number
        let result: std::result::Result<GraphId, _> = serde_json::from_str(incorrect_json);
        assert!(result.is_err());
    }
}

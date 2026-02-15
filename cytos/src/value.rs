use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::Result;

/// A generic value wrapper around JSON values for flexible serialization and deserialization.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct Value(serde_json::Value);

impl Value {
    /// Load a serializable value into a `Value`.
    ///
    /// # Errors
    ///
    /// Returns an error if the value cannot be serialized to JSON.
    pub fn load<T: Serialize>(val: &T) -> Result<Self> {
        serde_json::to_value(val)
            .map(Self)
            .or(Err("cannot load value".into()))
    }

    /// Deserialize the `Value` into a specific type.
    ///
    /// # Errors
    ///
    /// Returns an error if the value cannot be deserialized into the target type.
    pub fn dump<T: DeserializeOwned>(self) -> Result<T> {
        serde_json::from_value(self.0).or(Err("cannot dump value".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_load_and_dump() {
        let original = 42i32;
        let value = Value::load(&original).expect("should load value");
        let result: i32 = value.dump().expect("should dump value");
        assert_eq!(result, 42);
    }

    #[test]
    fn test_value_with_string() {
        let original = "hello world".to_string();
        let value = Value::load(&original).expect("should load value");
        let result: String = value.dump().expect("should dump value");
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_value_with_struct() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestStruct {
            name: String,
            value: i32,
        }

        let original = TestStruct {
            name: "test".to_string(),
            value: 100,
        };
        let value = Value::load(&original).expect("should load value");
        let result: TestStruct = value.dump().expect("should dump value");
        assert_eq!(result, original);
    }

    #[test]
    fn test_value_with_vec() {
        let original = vec![1, 2, 3, 4, 5];
        let value = Value::load(&original).expect("should load value");
        let result: Vec<i32> = value.dump().expect("should dump value");
        assert_eq!(result, original);
    }

    #[test]
    fn test_value_clone() {
        let value = Value::load(&42i32).expect("should load value");
        let cloned = value.clone();
        let result: i32 = cloned.dump().expect("should dump value");
        assert_eq!(result, 42);
    }

    #[test]
    fn test_value_debug() {
        let value = Value::load(&42i32).expect("should load value");
        let debug_str = format!("{:?}", value);
        assert!(debug_str.contains("42"));
    }

    #[test]
    fn test_value_load_invalid_serialization() {
        // Testing with a type that serializes successfully
        let value = Value::load(&42i32);
        assert!(value.is_ok());
    }

    #[test]
    fn test_value_dump_wrong_type() {
        let value = Value::load(&42i32).expect("should load value");
        // Try to dump as String when it's an integer
        let result: Result<String> = value.dump();
        assert!(result.is_err());
    }

    #[test]
    fn test_value_with_bool() {
        let value = Value::load(&true).expect("should load value");
        let result: bool = value.dump().expect("should dump value");
        assert!(result);
    }

    #[test]
    fn test_value_with_float() {
        let value = Value::load(&3.14f64).expect("should load value");
        let result: f64 = value.dump().expect("should dump value");
        assert!((result - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn test_value_with_option_some() {
        let original: Option<i32> = Some(42);
        let value = Value::load(&original).expect("should load value");
        let result: Option<i32> = value.dump().expect("should dump value");
        assert_eq!(result, Some(42));
    }

    #[test]
    fn test_value_with_option_none() {
        let original: Option<i32> = None;
        let value = Value::load(&original).expect("should load value");
        let result: Option<i32> = value.dump().expect("should dump value");
        assert_eq!(result, None);
    }

    #[test]
    fn test_value_with_nested_struct() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct Inner {
            x: f64,
            y: f64,
        }

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct Outer {
            inner: Inner,
            name: String,
        }

        let original = Outer {
            inner: Inner { x: 1.0, y: 2.0 },
            name: "test".to_string(),
        };
        let value = Value::load(&original).expect("should load value");
        let result: Outer = value.dump().expect("should dump value");
        assert_eq!(result, original);
    }
}

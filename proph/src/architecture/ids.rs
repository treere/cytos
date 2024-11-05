//! Index module

use serde::de::Error;

/// Macro to create an index struct
macro_rules! create_ids {
    ($struct_name:ident) => {
        #[derive(PartialEq, Eq, Clone, Copy, Hash)]
        pub struct $struct_name(pub u64);

        impl serde::Serialize for $struct_name {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let value = format_radix(self.0, 36);

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
                write!(f, "{}", format_radix(self.0, 36))
            }
        }

        impl std::fmt::Display for $struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", format_radix(self.0, 36))
            }
        }
    };
}

/// Format an u64 using a given radix
fn format_radix(mut x: u64, radix: u32) -> String {
    let mut result = vec![];
    let r = u64::from(radix);
    loop {
        let m = x % r;
        x /= r;

        // will panic if you use a bad radix (< 2 or > 36).
        result.push(std::char::from_digit(u32::try_from(m).unwrap(), radix).unwrap());
        if x == 0 {
            break;
        }
    }
    result.into_iter().rev().collect()
}

create_ids!(GraphId);
create_ids!(NodeId);
create_ids!(ParamId);

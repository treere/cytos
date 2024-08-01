use serde::de::Error;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

macro_rules! create_ids {
    ($struct_name:ident) => {
        #[derive(PartialEq, Eq, Clone, Copy, Hash)]
        pub struct $struct_name(pub u64);

        impl Serialize for $struct_name {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let value = format_radix(self.0, 36);

                serializer.serialize_str(&value)
            }
        }

        impl<'de> Deserialize<'de> for $struct_name {
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

        impl Display for $struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", format_radix(self.0, 36))
            }
        }
    };
}

fn format_radix(mut x: u64, radix: u64) -> String {
    let mut result = vec![];

    loop {
        let m = x % radix;
        x /= radix;

        // will panic if you use a bad radix (< 2 or > 36).
        result.push(std::char::from_digit(m as u32, radix as u32).unwrap());
        if x == 0 {
            break;
        }
    }
    result.into_iter().rev().collect()
}

create_ids!(GraphId);
create_ids!(NodeId);
create_ids!(ParamId);

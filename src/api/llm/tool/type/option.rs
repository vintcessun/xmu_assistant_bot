use serde::{Deserialize, Deserializer, Serialize};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub struct LlmOption<T>(pub Option<T>);

impl<T> Deref for LlmOption<T> {
    type Target = Option<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for LlmOption<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<Option<T>> for LlmOption<T> {
    fn from(opt: Option<T>) -> Self {
        LlmOption(opt)
    }
}

impl<T> From<LlmOption<T>> for Option<T> {
    fn from(lo: LlmOption<T>) -> Self {
        lo.0
    }
}

impl<T: PartialEq> PartialEq<Option<T>> for LlmOption<T> {
    fn eq(&self, other: &Option<T>) -> bool {
        &self.0 == other
    }
}

impl<T> LlmOption<T> {
    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }

    pub fn unwrap_or(self, default: T) -> T {
        self.0.unwrap_or(default)
    }
}

impl<T: Copy> LlmOption<T> {
    pub fn get(&self) -> Option<T> {
        self.0
    }
}

impl<'de, T> Deserialize<'de> for LlmOption<T>
where
    T: std::str::FromStr + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = Option::<String>::deserialize(deserializer)?;

        match s {
            None => Ok(LlmOption(None)),
            Some(val) => {
                let trimmed = val.trim();
                //空值全家桶
                match trimmed.to_lowercase().as_str() {
                    "" | "null" | "none" | "nil" | "undefined" | "n/a" | "nan" | "null_main"
                    | "空" | "空值" => Ok(LlmOption(None)),
                    _ => match trimmed.parse::<T>() {
                        Ok(parsed) => Ok(LlmOption(Some(parsed))),
                        Err(_) => Ok(LlmOption(None)),
                    },
                }
            }
        }
    }
}

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(untagged)]
pub enum UpdateField<T> {
    Value(T),
    SetNull,
    #[serde(skip)]
    Ignore,
}

impl<T> UpdateField<T> {
    pub fn is_ignore(&self) -> bool {
        matches!(self, Self::Ignore)
    }
    
    pub fn not_ignore(&self) -> bool {
        !Self::is_ignore(self)
    }

    pub fn value(self) -> Option<T> {
        match self {
            Self::Value(v) => Some(v),
            _ => None,
        }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> UpdateField<U> {
        match self {
            Self::Value(v) => UpdateField::Value(f(v)),
            Self::SetNull => UpdateField::SetNull,
            Self::Ignore => UpdateField::Ignore,
        }
    }
}

impl<T> Default for UpdateField<T> {
    fn default() -> Self {
        Self::Ignore
    }
}
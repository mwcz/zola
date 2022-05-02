use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkCheckerLevel {
    #[serde(rename = "error")]
    ErrorLevel,
    #[serde(rename = "warn")]
    WarnLevel,
}

impl Default for LinkCheckerLevel {
    fn default() -> Self {
        Self::ErrorLevel
    }
}

impl LinkCheckerLevel {
    pub fn log_prefix(&self) -> &str {
        match self {
            LinkCheckerLevel::ErrorLevel => "Error: ",
            LinkCheckerLevel::WarnLevel => "Warning: ",
        }
    }
}

impl Display for LinkCheckerLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinkCheckerLevel::ErrorLevel => f.write_str("error"),
            LinkCheckerLevel::WarnLevel => f.write_str("warn"),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LinkChecker {
    /// Skip link checking for these URL prefixes
    pub skip_prefixes: Vec<String>,
    /// Skip anchor checking for these URL prefixes
    pub skip_anchor_prefixes: Vec<String>,
    /// Emit either "error" or "warn" for broken internal links (including anchor links).
    pub internal_level: LinkCheckerLevel,
    /// Emit either "error" or "warn" for broken external links (including anchor links).
    pub external_level: LinkCheckerLevel,
}

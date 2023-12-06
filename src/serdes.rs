use anyhow::{bail, Result};
use log::error;

#[derive(Debug)]
pub enum Rename {
    Lowercase,
    CamelCase,
    SnakeCase
}

impl Rename {
    pub fn from_str(name: &str) -> Result<Self> {
        match name.trim_matches('"') {
            "lowercase" => Ok(Self::Lowercase),
            "camelCase" => Ok(Self::CamelCase),
            "snake_case" => Ok(Self::SnakeCase),
            _ => {
                error!("rename not support {}", name);
                bail!("rename not support {}", name);
            }
        }
    }
}

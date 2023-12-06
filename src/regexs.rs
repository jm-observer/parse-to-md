use crate::serdes::Rename;
use anyhow::Result;
use regex::Regex;

pub struct Regexs {
    pub rx_rename:  Regex,
    pub rx_tag:     Regex,
    pub rx_content: Regex
}

impl Regexs {
    pub fn init() -> Result<Self> {
        let rx_rename = Regex::new("rename_all = \"([^\"]*)\"")?;
        let rx_tag = Regex::new("tag = \"([^\"]*)\"")?;
        let rx_content = Regex::new("content = \"([^\"]*)\"")?;
        Ok(Self {
            rx_rename,
            rx_tag,
            rx_content
        })
    }

    pub fn get_tag(&self, content: &str) -> Option<String> {
        let Some(caps) = self.rx_tag.captures(content) else {
            return None;
        };
        let Some(rename) = caps.get(1) else {
            return None;
        };
        Some(rename.as_str().to_string())
    }

    pub fn get_content(&self, content: &str) -> Option<String> {
        let Some(caps) = self.rx_content.captures(content) else {
            return None;
        };
        let Some(content) = caps.get(1) else {
            return None;
        };
        Some(content.as_str().to_string())
    }

    pub fn get_rename_all(&self, content: &str) -> Option<Rename> {
        let Some(caps) = self.rx_rename.captures(content) else {
            return None;
        };
        let Some(rename) = caps.get(1) else {
            return None;
        };
        let Ok(name) = Rename::from_str(rename.as_str()) else {
            return None;
        };
        Some(name)
    }
}

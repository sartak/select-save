use anyhow::Result;
use regex::Regex;
use serde::Deserialize;
use std::{borrow::Cow, collections::HashMap, path::Path, sync::OnceLock};
use tracing::{debug, error};

pub struct Extractor {
    config: Config,
}

#[derive(Deserialize, Debug)]
struct Config {
    #[serde(flatten)]
    types: HashMap<String, Vec<String>>,
}

impl Extractor {
    pub fn new(filename: &Path) -> Result<Self> {
        let file = std::fs::File::open(filename)?;
        let reader = std::io::BufReader::new(file);
        let config = serde_json::from_reader(reader)?;

        Ok(Self { config })
    }

    pub fn extract(&self, filename: &Path) -> Result<Vec<String>> {
        let Some(patterns) = filename
            .extension()
            .and_then(|e| e.to_str().and_then(|e| self.config.types.get(e)))
        else {
            return Ok(Vec::new());
        };

        let content = std::fs::read(filename)?;

        Ok(patterns
            .iter()
            .map(|p| self.extract_pattern(&content, p).to_string())
            .collect::<Vec<_>>())
    }

    fn extract_pattern<'a>(&self, content: &[u8], pattern: &'a str) -> Cow<'a, str> {
        static RE: OnceLock<Regex> = OnceLock::new();
        let re = RE.get_or_init(|| Regex::new(r"\{([^}]+)}").unwrap());
        re.replace_all(pattern, |caps: &regex::Captures| {
            let pattern = &caps[1];
            debug!("Found subpattern: {pattern}");
            self.extract_subpattern(content, pattern)
                .unwrap_or("??".to_string())
        })
    }

    fn extract_subpattern(&self, content: &[u8], pattern: &str) -> Option<String> {
        let mut result: i64 = 0;

        let (pattern, format) = pattern.split_once(':').unwrap_or((pattern, ""));
        let addresses = pattern
            .splitn(2, '~')
            .map(|address| {
                let address = address.trim_start_matches("0x");
                usize::from_str_radix(address, 16)
            })
            .collect::<Result<Vec<_>, _>>()
            .ok()?;

        let start = addresses.first()?;
        let end = if addresses.len() == 1 {
            addresses.first()?
        } else {
            addresses.get(1)?
        };

        for index in (*start..=*end).rev() {
            let byte = content.get(index)?;
            result <<= 8;
            result += *byte as i64;
        }

        let result = if format.is_empty() {
            format!("{result}")
        } else {
            let zero_pad = format.starts_with('0');
            let pad = format.parse::<usize>();
            let Ok(pad) = pad else {
                error!("Invalid format specifier: {format}");
                return None;
            };

            if zero_pad {
                format!("{result:00$}", pad)
            } else {
                format!("{result:0$}", pad)
            }
        };

        debug!("{pattern} -> {result}");
        Some(result)
    }
}

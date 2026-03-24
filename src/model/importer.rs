use crate::model::{NormalizedPath, ToNormalizedPath};
use serde::Deserialize;
use serde_json::Value;
use std::error::Error;

#[derive(Debug, Clone)]
pub enum ImportFormat {
    Waffle,
    UVL,
}

impl<S: Into<String>> From<S> for ImportFormat {
    fn from(value: S) -> Self {
        let real = value.into();
        match real.to_lowercase().as_str() {
            "waffle" => ImportFormat::Waffle,
            "uvl" => ImportFormat::UVL,
            _ => unreachable!("Importer does not support format '{}'", real),
        }
    }
}

pub trait FormatParser {
    fn parse(&self, data: &str) -> Result<Vec<NormalizedPath>, Box<dyn Error>>;
}

pub struct ModelParser {
    parser: Box<dyn FormatParser>,
}

impl ModelParser {
    pub fn new(format: &ImportFormat) -> ModelParser {
        let parser = match format {
            ImportFormat::Waffle => WaffleProductParser,
            _ => todo!()
        };
        ModelParser {
            parser: Box::new(parser),
        }
    }
    pub fn import(&self, data: &str) -> Result<Vec<NormalizedPath>, Box<dyn Error>> {
        Ok(self.parser.parse(data)?)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WaffleSchema {

}

pub struct WaffleProductParser;

impl FormatParser for WaffleProductParser {
    fn parse(&self, data: &str) -> Result<Vec<NormalizedPath>, Box<dyn Error>> {
        fn parse_recursive(value: Value) -> Result<Vec<NormalizedPath>, Box<dyn Error>> {
            let map = value.as_object();
            if let Some(map) = map {
                let mut paths: Vec<NormalizedPath> = Vec::new();
                for (key, value) in map.iter() {
                    let path = key.to_normalized_path();
                    paths.push(path.clone());
                    let rec_paths = parse_recursive(value.clone())?;
                    for p in rec_paths {
                        paths.push(path.clone() + p);
                    }
                };
                Ok(paths)
            } else {
                Err("Waffle product malformed: could not create map".into())
            }
        }

        let schema = serde_json::from_str::<Value>(data)?;
        parse_recursive(schema)
    }
}

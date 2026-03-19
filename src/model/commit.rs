use serde::Deserialize;

pub trait CommitMetadata where Self: Sized {
    fn header() -> String;
    fn from_json<S: Into<String>>(content: S) -> serde_json::error::Result<Self>;
    fn to_json(&self) -> serde_json::error::Result<String>;
    fn to_commit_message(&self) -> serde_json::error::Result<String> {
        let base = Self::header() + "\n";
        let serialized = self.to_json()?;
        Ok(base + serialized.as_str())
    }
}

pub struct Base {}

impl CommitMetadata for Base {
    fn header() -> String {
        "".to_string()
    }

    fn from_json<S: Into<String>>(_content: S) -> serde_json::Result<Self> {
        Ok(Self {})
    }

    fn to_json(&self) -> serde_json::Result<String> {
        Ok("".to_string())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Commit {
    hash: String,
    message: String,
}

impl PartialEq for Commit {
    fn eq(&self, other: &Self) -> bool {
        other.hash == self.hash
    }

    fn ne(&self, other: &Self) -> bool {
        other.hash != self.hash
    }
}

impl Commit {
    pub fn new<S1: Into<String>, S2: Into<String>>(
        hash: S1,
        message: S2,
    ) -> Self {
        Self {
            hash: hash.into(),
            message: message.into(),
        }
    }
    fn get_hash(&self) -> &String {
        &self.hash
    }
    fn get_message(&self) -> &String {
        &self.message
    }
    fn try_get_metadata<M: CommitMetadata>(&self) -> serde_json::error::Result<M> {
        let to_parse = self
            .get_message()
            .split(M::header().as_str())
            .collect::<Vec<&str>>()
            .get(1)
            .cloned()
            .unwrap();
        M::from_json(to_parse)
    }
}

use crate::core::git::error::GitError;
use crate::core::model::ToNormalizedPath;
use crate::core::model::git::GitInterface;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub const METADATA_SEPARATOR: &str = "---metadata---";

#[derive(Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct CommitHash {
    full_hash: String,
}

impl Display for CommitHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.get_short_hash())
    }
}

impl CommitHash {
    pub fn new<S: Into<String>>(full_hash: S) -> Self {
        let full = full_hash.into();
        if full.len() < 8 {
            panic!("Commit hash must be at least 8 characters long");
        }
        CommitHash { full_hash: full }
    }
    pub fn get_full_hash(&self) -> &String {
        &self.full_hash
    }
    pub fn get_short_hash(&self) -> String {
        self.full_hash[0..8].to_string()
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct CommitTag {
    tag: String,
    full_path: String,
}

impl CommitTag {
    pub fn new<S: Into<String>>(full_path: S) -> Self {
        let full_path = full_path.into();
        let normalized = full_path.to_normalized_path();
        let tag = normalized.last().unwrap().to_string();
        CommitTag { tag, full_path }
    }
    pub fn get_full_path(&self) -> &String {
        &self.full_path
    }
    pub fn get_tag(&self) -> &String {
        &self.tag
    }
}

pub trait CommitMetadata
where
    Self: Sized,
{
    fn header() -> String;
    fn from_json<S: Into<String>>(content: S) -> serde_json::error::Result<Self>;
    fn to_json(&self) -> serde_json::error::Result<String>;
    fn to_commit_message(&self) -> serde_json::error::Result<String> {
        let base = format!("{METADATA_SEPARATOR}\n{}\n", Self::header());
        let serialized = self.to_json()?;
        Ok(base + serialized.as_str())
    }
    fn from_commit_message<S: Into<String>>(content: S) -> Option<serde_json::error::Result<Self>> {
        let message = content.into();
        if !message.contains(&Self::header()) {
            None
        } else {
            let to_parse = message
                .split(Self::header().as_str())
                .collect::<Vec<&str>>()[1];
            Some(Self::from_json(to_parse))
        }
    }
}

pub struct CommitMetadataContainer {
    metadata: String,
}

impl CommitMetadataContainer {
    pub fn new(metadata: &impl CommitMetadata) -> Result<Self, serde_json::Error> {
        Ok(Self {
            metadata: metadata.to_commit_message()?,
        })
    }

    pub fn get_metadata(&self) -> &String {
        &self.metadata
    }
}

#[derive(Debug, Clone)]
pub struct Commit {
    hash: CommitHash,
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
    pub fn new<S: Into<String>>(hash: CommitHash, message: S) -> Self {
        Self {
            hash,
            message: message.into(),
        }
    }
    pub fn get_hash(&self) -> &CommitHash {
        &self.hash
    }
    pub fn get_message(&self) -> &String {
        &self.message
    }
    pub fn get_metadata(&self) -> Vec<String> {
        let extracted = self
            .get_message()
            .split(METADATA_SEPARATOR)
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        if extracted.len() > 1 {
            extracted[1..].to_vec()
        } else {
            vec![]
        }
    }
}

pub struct CommitIterator<'a> {
    hashes: Vec<String>,
    git: &'a GitInterface,
    current_position: usize,
}

impl<'a> CommitIterator<'a> {
    pub fn new(hashes: Vec<String>, git: &'a GitInterface) -> Self {
        Self {
            hashes,
            git,
            current_position: 0,
        }
    }
}

impl<'a> Iterator for CommitIterator<'a> {
    type Item = Result<Commit, GitError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_position < self.hashes.len() {
            let hash = CommitHash::new(self.hashes.get(self.current_position).unwrap());
            let commit = self.git.get_commit_from_hash(&hash);
            self.current_position += 1;
            match commit {
                Ok(commit) => Some(Ok(commit)),
                Err(err) => Some(Err(err)),
            }
        } else {
            None
        }
    }
}

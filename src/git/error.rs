use crate::model::ModelError;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;

#[derive(Debug, Clone)]
pub struct GitInterfaceError {
    msg: String,
}
impl GitInterfaceError {
    pub fn new(msg: &str) -> GitInterfaceError {
        GitInterfaceError {
            msg: msg.to_string(),
        }
    }
}
impl Display for GitInterfaceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}
impl Error for GitInterfaceError {}

#[derive(Debug)]
pub enum GitError {
    Io(io::Error),
    GitInterface(GitInterfaceError),
    Model(ModelError),
    SerdeJson(serde_json::Error),
}
impl Display for GitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GitError::Io(err) => err.fmt(f),
            GitError::GitInterface(err) => err.fmt(f),
            GitError::Model(err) => err.fmt(f),
            GitError::SerdeJson(err) => err.fmt(f),
        }
    }
}
impl Error for GitError {}
impl From<io::Error> for GitError {
    fn from(err: io::Error) -> GitError {
        GitError::Io(err)
    }
}
impl From<GitInterfaceError> for GitError {
    fn from(value: GitInterfaceError) -> Self {
        GitError::GitInterface(value)
    }
}
impl From<ModelError> for GitError {
    fn from(value: ModelError) -> Self {
        GitError::Model(value)
    }
}
impl From<serde_json::Error> for GitError {
    fn from(err: serde_json::Error) -> Self {
        GitError::SerdeJson(err)
    }
}

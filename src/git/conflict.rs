use crate::git::error::GitError;
use crate::git::interface::GitInterface;
use crate::model::QualifiedPath;
use colored::Colorize;
use itertools::Itertools;
use std::fmt::Display;

#[derive(Debug)]
pub enum ConflictStatistic {
    Success(Vec<QualifiedPath>),
    Conflict(Vec<QualifiedPath>, usize),
    Error(Vec<QualifiedPath>, GitError),
}

impl PartialEq for ConflictStatistic {
    fn eq(&self, other: &Self) -> bool {
        match other {
            Self::Success(other_paths) => match self {
                Self::Success(self_paths) => other_paths == self_paths,
                _ => false,
            },
            Self::Conflict(other_paths, other_index) => match self {
                Self::Conflict(self_paths, self_index) => {
                    other_paths == self_paths && other_index == self_index
                }
                _ => false,
            },
            Self::Error(other_paths, _) => match self {
                Self::Error(self_paths, _) => other_paths == self_paths,
                _ => false,
            },
        }
    }
}

impl Display for ConflictStatistic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn format(paths: &Vec<QualifiedPath>, fail_at: Option<&usize>, error: bool) -> String {
            match error {
                true => paths
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(" <- ")
                    .red()
                    .to_string(),
                false => paths
                    .iter()
                    .enumerate()
                    .map(|(index, p)| match fail_at {
                        Some(fail_at) => {
                            if &index < fail_at {
                                p.to_string().green().to_string()
                            } else if &index == fail_at {
                                p.to_string().red().to_string()
                            } else {
                                p.to_string().strikethrough().to_string()
                            }
                        }
                        None => p.to_string().blue().to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(" <- "),
            }
        }
        let formatted = match self {
            ConflictStatistic::Success(paths) => {
                format!("{} {}", format(paths, None, false), "OK".green())
            }
            ConflictStatistic::Conflict(paths, index) => {
                format!("{} {}", format(paths, Some(index), false), "CONFLICT".red())
            }
            ConflictStatistic::Error(paths, error) => {
                format!(
                    "{} {}:\n{}",
                    format(paths, None, true),
                    "ERROR".red(),
                    error.to_string().red()
                )
            }
        };
        f.write_str(formatted.as_str())
    }
}
impl Into<String> for ConflictStatistic {
    fn into(self) -> String {
        self.to_string()
    }
}
impl Into<String> for &ConflictStatistic {
    fn into(self) -> String {
        self.to_string()
    }
}

pub struct ConflictStatistics {
    ok: Vec<ConflictStatistic>,
    conflict: Vec<ConflictStatistic>,
    error: Vec<ConflictStatistic>,
}

impl ConflictStatistics {
    pub fn new() -> Self {
        Self {
            ok: vec![],
            conflict: vec![],
            error: vec![],
        }
    }
    pub fn from_iter<T: Iterator<Item = ConflictStatistic>>(statistics: T) -> Self {
        let mut new = Self::new();
        for statistic in statistics {
            new.push(statistic);
        }
        new
    }
    pub fn push(&mut self, statistic: ConflictStatistic) {
        match statistic {
            ConflictStatistic::Success(_) => self.ok.push(statistic),
            ConflictStatistic::Conflict(_, _) => self.conflict.push(statistic),
            ConflictStatistic::Error(_, _) => self.error.push(statistic),
        }
    }
    pub fn iter_all(&self) -> impl Iterator<Item = &ConflictStatistic> {
        self.iter_ok()
            .chain(self.iter_conflicts())
            .chain(self.iter_errors())
    }
    pub fn iter_ok(&self) -> impl Iterator<Item = &ConflictStatistic> {
        self.ok.iter()
    }
    pub fn iter_conflicts(&self) -> impl Iterator<Item = &ConflictStatistic> {
        self.conflict.iter()
    }
    pub fn iter_errors(&self) -> impl Iterator<Item = &ConflictStatistic> {
        self.error.iter()
    }
    pub fn n_ok(&self) -> usize {
        self.ok.len()
    }
    pub fn n_conflict(&self) -> usize {
        self.conflict.len()
    }
    pub fn n_errors(&self) -> usize {
        self.error.len()
    }
    pub fn contains(&self, statistic: &ConflictStatistic) -> bool {
        self.ok.contains(statistic)
            || self.conflict.contains(statistic)
            || self.error.contains(statistic)
    }
}

impl FromIterator<ConflictStatistic> for ConflictStatistics {
    fn from_iter<T: IntoIterator<Item = ConflictStatistic>>(iter: T) -> Self {
        Self::from_iter(iter.into_iter())
    }
}
pub struct ConflictChecker<'a> {
    interface: &'a GitInterface,
}

impl<'a> ConflictChecker<'a> {
    pub fn new(interface: &'a GitInterface) -> Self {
        Self { interface }
    }

    pub fn check_n_to_n_permutations(
        &self,
        paths: Vec<QualifiedPath>,
        k: usize,
    ) -> Result<impl Iterator<Item = ConflictStatistic>, GitError> {
        let iterator = paths.into_iter().permutations(k).map(|perm| {
            let statistic = self.check_chain(&perm);
            self.build_statistic(perm, statistic)
        });
        Ok(iterator)
    }

    pub fn check_1_to_n_permutations(
        &self,
        source: &QualifiedPath,
        targets: Vec<QualifiedPath>,
        k: usize,
    ) -> Result<impl Iterator<Item = ConflictStatistic>, GitError> {
        let iterator = targets.into_iter().permutations(k).map(move |target| {
            let mut to_check: Vec<QualifiedPath> = vec![];
            to_check.push(source.clone());
            to_check.extend(target);
            let statistic = self.check_chain(&to_check);
            self.build_statistic(to_check, statistic)
        });
        Ok(iterator)
    }

    fn check_chain(
        &self,
        chain: &Vec<QualifiedPath>,
    ) -> Result<Option<usize>, GitError> {
        if chain.len() < 2 {
            panic!("Chain has to contain at least 2 paths")
        }
        let mut failed_at: Option<usize> = None;
        let current_path = self.interface.get_current_qualified_path()?;
        let base = &chain[0];
        self.interface.checkout(base)?;
        let temporary = QualifiedPath::from("tmp");
        self.interface.create_branch_no_mut(&temporary)?;
        self.interface.checkout_raw(&temporary)?;
        for (index, path) in chain[1..].iter().enumerate() {
            let success = self.interface.merge(&vec![path.clone()])?.status.success();
            if !success {
                self.interface.abort_merge()?;
                failed_at = Some(index);
                break;
            }
        }
        self.interface.checkout(&current_path)?;
        self.interface.delete_branch(&temporary)?;
        Ok(failed_at)
    }

    fn build_statistic(
        &self,
        paths: Vec<QualifiedPath>,
        result: Result<Option<usize>, GitError>,
    ) -> ConflictStatistic {
        match result {
            Ok(stat) => match stat {
                None => ConflictStatistic::Success(paths),
                Some(value) => ConflictStatistic::Conflict(paths, value),
            },
            Err(e) => ConflictStatistic::Error(paths, e),
        }
    }
}

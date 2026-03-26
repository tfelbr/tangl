use crate::git::error::GitError;
use crate::git::interface::GitInterface;
use crate::logging::TanglLogger;
use crate::model::{AnyHasBranch, NodePath, NormalizedPath, ToNormalizedPath, ToNormalizedPaths};
use colored::Colorize;
use itertools::Itertools;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MergeSuccess {
    path: NormalizedPath,
}
impl MergeSuccess {
    pub fn new(path: NormalizedPath) -> Self {
        Self { path }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MergePending {
    path: NormalizedPath,
}
impl MergePending {
    pub fn new(path: NormalizedPath) -> Self {
        Self { path }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MergeConflict {
    path: NormalizedPath,
}
impl MergeConflict {
    pub fn new(path: NormalizedPath) -> Self {
        Self { path }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MergeStatistic {
    Base(NormalizedPath),
    Success(MergeSuccess),
    UpToDate(NormalizedPath),
    Conflict(MergeConflict),
    Merging(MergePending),
    Aborted(NormalizedPath),
}

impl MergeStatistic {
    pub fn get_path(&self) -> &NormalizedPath {
        match self {
            Self::Base(path) | Self::Aborted(path) | Self::UpToDate(path) => path,
            Self::Success(success) => &success.path,
            Self::Merging(pending) => &pending.path,
            Self::Conflict(conflict) => &conflict.path,
        }
    }
}

impl Display for MergeStatistic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value: String = match self {
            Self::Base(path) => {
                format!("{} ({})", path.formatted(false, true), "Base")
            }
            Self::Success(success) => {
                format!("{} ({})", success.path.formatted(true, true), "Ok".green())
            }
            Self::UpToDate(path) => {
                format!("{} ({})", path.formatted(true, true), "Up to date".green())
            }
            Self::Conflict(conflict) => {
                format!(
                    "{} ({})",
                    conflict.path.formatted(true, true),
                    "Conflict".red()
                )
            }
            Self::Merging(pending) => {
                format!(
                    "{} ({})",
                    pending.path.to_string().blue(),
                    "Merging".yellow()
                )
            }
            Self::Aborted(path) => {
                format!("{} ({})", path.to_string().blue(), "Aborted".red())
            }
        };
        f.write_str(value.as_str())
    }
}

impl ToNormalizedPaths for Vec<MergeStatistic> {
    fn to_normalized_paths(&self) -> Vec<NormalizedPath> {
        self.iter().map(|s| s.get_path().clone()).collect()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MergeChainStatistic {
    chain: Vec<MergeStatistic>,
    n_merged: usize,
    n_up_to_date: usize,
    n_conflict: usize,
}

impl MergeChainStatistic {
    pub fn new() -> Self {
        Self {
            chain: vec![],
            n_merged: 0,
            n_up_to_date: 0,
            n_conflict: 0,
        }
    }

    fn add_to_internal_counters(&mut self, stat: &MergeStatistic) {
        match &stat {
            MergeStatistic::Success(_) => self.n_merged += 1,
            MergeStatistic::UpToDate(_) => self.n_up_to_date += 1,
            MergeStatistic::Conflict(_) => self.n_conflict += 1,
            _ => {}
        }
    }

    fn subtract_from_internal_counters(&mut self, stat: &MergeStatistic) {
        match &stat {
            MergeStatistic::Success(_) => self.n_merged -= 1,
            MergeStatistic::UpToDate(_) => self.n_up_to_date -= 1,
            MergeStatistic::Conflict(_) => self.n_conflict -= 1,
            _ => {}
        }
    }
    pub fn push(&mut self, stat: MergeStatistic) {
        if self.chain.is_empty() {
            match stat {
                MergeStatistic::Base(_) => {}
                _ => panic!("First item in MergeChainStatistic must be a base"),
            }
        }
        self.add_to_internal_counters(&stat);
        self.chain.push(stat);
    }
    pub fn fill(&mut self, stats: Vec<MergeStatistic>) {
        for stat in stats {
            self.chain.push(stat)
        }
    }
    pub fn insert(&mut self, index: usize, stat: MergeStatistic) {
        self.add_to_internal_counters(&stat);
        self.chain.insert(index, stat);
    }
    pub fn remove(&mut self, index: usize) -> MergeStatistic {
        let statistic = self.chain.remove(index);
        self.subtract_from_internal_counters(&statistic);
        statistic
    }
    pub fn get(&self, index: usize) -> Option<&MergeStatistic> {
        self.chain.get(index)
    }
    pub fn replace(&mut self, index: usize, stat: MergeStatistic) {
        self.remove(index);
        self.insert(index, stat);
    }
    pub fn get_chain(&self) -> &Vec<MergeStatistic> {
        &self.chain
    }
    pub fn iter_except_base(&self) -> impl Iterator<Item = &MergeStatistic> {
        self.chain
            .iter()
            .enumerate()
            .filter_map(|(i, s)| if i != 0 { Some(s) } else { None })
    }
    pub fn get_n_merged(&self) -> usize {
        self.n_merged
    }
    pub fn get_n_up_to_date(&self) -> usize {
        self.n_up_to_date
    }
    pub fn all_up_to_date(&self) -> bool {
        if self.chain.is_empty() || self.chain.len() == 1 {
            true
        } else {
            self.n_up_to_date == self.chain.len() - 1
        }
    }
    pub fn len(&self) -> usize {
        self.chain.len()
    }
    pub fn is_empty(&self) -> bool {
        self.chain.is_empty()
    }
    pub fn get_n_conflict(&self) -> usize {
        self.n_conflict
    }
    pub fn contains_conflicts(&self) -> bool {
        self.n_conflict > 0
    }
    pub fn display_as_path(&self) -> String {
        self.chain.iter().map(|stat| stat.to_string()).join(" <- ")
    }
    pub fn display_as_list(&self) -> impl Iterator<Item = String> {
        self.chain.iter().map(|stat| match stat {
            MergeStatistic::Base(_) => stat.to_string(),
            _ => format!(" <- {}", stat),
        })
    }
}


pub struct MergeChainStatistics {
    statistics: Vec<MergeChainStatistic>,
    total_successes: usize,
    total_conflicts: usize,
}

impl MergeChainStatistics {
    pub fn new() -> Self {
        Self {
            statistics: vec![],
            total_successes: 0,
            total_conflicts: 0,
        }
    }
    pub fn fill_from_iter<T: Iterator<Item = MergeChainStatistic>>(&mut self, statistics: T) {
        for statistic in statistics {
            self.push(statistic);
        }
    }
    pub fn push(&mut self, statistic: MergeChainStatistic) {
        self.total_successes += statistic.n_merged;
        self.total_conflicts += statistic.n_conflict;
        self.statistics.push(statistic);
    }
    pub fn iter_all(&self) -> impl Iterator<Item = &MergeChainStatistic> {
        self.statistics.iter()
    }
    pub fn iter_conflicts(&self) -> impl Iterator<Item = &MergeChainStatistic> {
        self.statistics.iter().filter(|s| s.contains_conflicts())
    }
    pub fn n_ok(&self) -> usize {
        self.total_successes
    }
    pub fn n_conflicts(&self) -> usize {
        self.total_conflicts
    }
}

impl FromIterator<MergeChainStatistic> for MergeChainStatistics {
    fn from_iter<T: IntoIterator<Item = MergeChainStatistic>>(iter: T) -> Self {
        let mut new = Self::new();
        new.fill_from_iter(iter.into_iter());
        new
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MergeStatisticWeight {
    Simple,
}

impl MergeStatisticWeight {
    pub fn get_weight(&self, statistic: &MergeStatistic) -> i32 {
        match self {
            Self::Simple => {
                match statistic {
                    MergeStatistic::Base(_) => 0,
                    MergeStatistic::UpToDate(_) => 1,
                    MergeStatistic::Success(_) => 0,
                    MergeStatistic::Conflict(_) => -1,
                    MergeStatistic::Merging(_) => 0,
                    MergeStatistic::Aborted(_) => -10,
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MergeStatistics {
    statistics: Vec<MergeStatistic>,
    weights: MergeStatisticWeight,
}

impl PartialOrd for MergeStatistics {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let my_weights = self.accumulate_weights();
        let their_weights = other.accumulate_weights();
        Some(my_weights.cmp(&their_weights))
    }
}

impl Ord for MergeStatistics {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl MergeStatistics {
    pub fn new(weights: MergeStatisticWeight) -> Self {
        Self { statistics: vec![], weights }
    }
    pub fn push(&mut self, statistic: MergeStatistic) {
        self.statistics.push(statistic);
    }
    pub fn accumulate_weights(&self) -> i32 {
        let mut sum = 0;
        for s in &self.statistics {
            sum += self.weights.get_weight(s)
        };
        sum
    }
    pub fn get_lowest(&self) -> &MergeStatistic {
        self.statistics.iter().min_by(|a, b| {
            self.weights.get_weight(a).cmp(&self.weights.get_weight(b))
        }).unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct ConflictChecker<'a> {
    interface: &'a GitInterface,
}

impl<'a> ConflictChecker<'a> {
    pub fn new(interface: &'a GitInterface) -> Self {
        Self { interface }
    }

    pub fn check_k_permutations(
        &self,
        paths: &Vec<NodePath<AnyHasBranch>>,
        k: usize,
    ) -> impl Iterator<Item = Result<MergeChainStatistic, GitError>> {
        let iterator = paths
            .iter()
            .permutations(k)
            .map(|perm| self.check_chain(&perm));
        iterator
    }

    pub fn check_permutations_against_base(
        &self,
        targets: &Vec<NodePath<AnyHasBranch>>,
        base: &NodePath<AnyHasBranch>,
        k: usize,
    ) -> impl Iterator<Item = Result<MergeChainStatistic, GitError>> {
        let iterator = targets.iter().permutations(k).map(|target| {
            let mut to_check: Vec<&NodePath<AnyHasBranch>> = vec![];
            to_check.push(base);
            to_check.extend(target);
            self.check_chain(&to_check)
        });
        iterator
    }

    pub fn check_by_order(
        &self,
        paths: &Vec<NodePath<AnyHasBranch>>,
    ) -> Result<MergeChainStatistic, GitError> {
        let chain: Vec<&NodePath<AnyHasBranch>> = paths.iter().collect();
        self.check_chain(&chain)
    }

    pub fn check_n_against_permutations(
        &self,
        n: &'a Vec<NodePath<AnyHasBranch>>,
        against: &'a Vec<NodePath<AnyHasBranch>>,
        k: &'a usize,
    ) -> impl Iterator<Item = Result<MergeChainStatistic, GitError>> {
        // I don't know why, but k has to be borrowed here
        let iterator = n
            .iter()
            .map(|path| {
                against
                    .iter()
                    .combinations(*k)
                    .map(|mut combination| {
                        combination.push(path);
                        combination
                            .iter()
                            .permutations(*k + 1)
                            .map(|permutations| {
                                let dereferenced = permutations
                                    .iter()
                                    .map(|permutation| **permutation)
                                    .collect::<Vec<_>>();
                                self.check_chain(&dereferenced)
                            })
                            .collect::<Vec<_>>()
                    })
                    .flatten()
            })
            .flatten();
        iterator
    }

    pub fn clean_up(&mut self) {}

    fn check_chain(
        &self,
        chain: &Vec<&NodePath<AnyHasBranch>>,
    ) -> Result<MergeChainStatistic, GitError> {
        if chain.len() < 2 {
            panic!("Chain has to contain at least 2 paths")
        }
        let mut chain_statistic = MergeChainStatistic::new();
        let current_path = self
            .interface
            .assert_current_node_path::<AnyHasBranch>()
            .unwrap();
        let base = chain[0];
        chain_statistic.push(MergeStatistic::Base(base.to_normalized_path()));
        self.interface.checkout(base)?;
        let temporary = NormalizedPath::from("tmp");
        self.interface.create_branch_no_mut(&temporary)?;
        self.interface.checkout_raw(&temporary)?;
        let mut skip = false;
        for path in chain[1..].iter() {
            if skip {
                chain_statistic.push(MergeStatistic::Aborted(path.to_normalized_path()));
            } else {
                let (statistic, _) = self.interface.merge(path)?;
                if statistic.contains_conflicts() {
                    self.interface.abort_merge()?;
                    skip = true;
                }
                chain_statistic.push(statistic.get(1).unwrap().clone());
            }
        }
        self.interface.checkout(&current_path)?;
        self.interface.delete_branch_no_mut(&temporary)?;
        Ok(chain_statistic)
    }
}

#[derive(Debug, Clone)]
pub struct Conflict2DMatrix {
    matrix: HashMap<NormalizedPath, HashMap<NormalizedPath, MergeStatistic>>,
}

impl Conflict2DMatrix {
    pub fn new(statistics: &MergeChainStatistics) -> Self {
        let mut matrix: HashMap<NormalizedPath, HashMap<NormalizedPath, MergeStatistic>> =
            HashMap::new();
        for chain in statistics.iter_all() {
            if chain.len() > 2 {
                panic!("Matrix only supports 2 dimensions")
            }
            let base = chain.get(0).unwrap();
            let second = chain.get(1).unwrap();
            if !matrix.contains_key(base.get_path()) {
                matrix.insert(base.get_path().clone(), HashMap::new());
            }
            matrix
                .get_mut(base.get_path())
                .unwrap()
                .insert(second.get_path().clone(), second.clone());
        }
        Self { matrix }
    }

    pub fn predict_conflicts(&self, order: &Vec<NormalizedPath>) -> MergeChainStatistic {
        let base = order.get(0).unwrap().clone();
        let mut final_path = vec![(base, MergeStatistics::new(MergeStatisticWeight::Simple))];
        for path in order[1..].iter() {
            let voters = final_path.iter().map(|(k, _)| k.clone()).collect();
            let votes = self.calculate_votes(&voters, &vec![path.clone()]);
            let vote = votes.get(&path).unwrap();
            final_path.push((path.clone(), vote.clone()));
        }
        self.statistics_from_votes(&final_path)
    }

    pub fn estimate_best_path(&self, base_path: &NormalizedPath) -> MergeChainStatistic {
        let mut missing: Vec<NormalizedPath> = self.matrix.keys().cloned().collect();
        let start = base_path.clone();
        missing.retain(|k| k != base_path);
        let mut final_path = vec![(start, MergeStatistics::new(MergeStatisticWeight::Simple))];
        while missing.len() > 0 {
            let voters = final_path.iter().map(|(k, _)| k.clone()).collect();
            let votes = Self::reverse_votes(self.calculate_votes(&voters, &missing));
            let max_vote = votes.keys().max().unwrap();
            let max_candidates = &votes[&max_vote];
            let winner = match max_candidates.len() {
                0 => {
                    panic!("Empty candidates should not be possible")
                }
                1 => max_candidates[0].clone(),
                _ => {
                    let start = max_candidates[0].clone();
                    let compatibility = self.calculate_forward_compatibility(&start, &missing);
                    let mut highest_compatible = (start, compatibility);
                    for candidate in max_candidates[1..].iter() {
                        let compatibility =
                            self.calculate_forward_compatibility(&candidate, &missing);
                        if compatibility > highest_compatible.1 {
                            highest_compatible = (candidate.clone(), compatibility);
                        }
                    }
                    highest_compatible.0
                }
            };
            let index: usize = missing
                .iter()
                .enumerate()
                .find_map(|(index, e)| if e == &winner { Some(index) } else { None })
                .unwrap();
            missing.remove(index);
            final_path.push((winner, max_vote.clone()));
        }
        self.statistics_from_votes(&final_path)
    }

    fn calculate_forward_compatibility(
        &self,
        element: &NormalizedPath,
        missing: &Vec<NormalizedPath>,
    ) -> MergeStatistics {
        let table = &self.matrix[element];
        let mut statistics = MergeStatistics::new(MergeStatisticWeight::Simple);
        for statistic in table
            .iter()
            .filter_map(|(k, v)| if missing.contains(k) { Some(v.clone()) } else { None }) {
            statistics.push(statistic)
        };
        statistics
    }

    fn calculate_votes(
        &self,
        voters: &Vec<NormalizedPath>,
        targets: &Vec<NormalizedPath>,
    ) -> HashMap<NormalizedPath, MergeStatistics> {
        let mut votes: HashMap<NormalizedPath, MergeStatistics> = HashMap::new();
        for candidate in targets.iter() {
            let mut statistics = MergeStatistics::new(MergeStatisticWeight::Simple);
            for p in voters.iter() {
                let statistic = self.matrix[p].get(candidate).unwrap();
                statistics.push(statistic.clone());
            }
            votes.insert(candidate.clone(), statistics);
        }
        votes
    }

    fn reverse_votes(votes: HashMap<NormalizedPath, MergeStatistics>) -> HashMap<MergeStatistics, Vec<NormalizedPath>> {
        let mut reversed: HashMap<MergeStatistics, Vec<NormalizedPath>> = HashMap::new();
        for (path, vote) in votes.iter() {
            if reversed.contains_key(vote) {
                reversed.get_mut(vote).unwrap().push(path.clone());
            } else {
                reversed.insert(vote.clone(), vec![path.clone()]);
            }
        }
        reversed
    }

    fn statistics_from_votes(&self, votes: &Vec<(NormalizedPath, MergeStatistics)>) -> MergeChainStatistic {
        let mut chain_statistic = MergeChainStatistic::new();
        for (index, (path, vote)) in votes.iter().enumerate() {
            let statistic = if index == 0 {
                MergeStatistic::Base(path.clone())
            } else {
                vote.get_lowest().clone()
            };
            chain_statistic.push(statistic);
        }
        chain_statistic
    }
}

pub struct ConflictAnalyzer<'a> {
    checker: ConflictChecker<'a>,
    logger: &'a TanglLogger,
}

impl<'a> ConflictAnalyzer<'a> {
    pub fn new(checker: ConflictChecker<'a>, logger: &'a TanglLogger) -> Self {
        Self { checker, logger }
    }

    pub fn calculate_2d_heuristics_matrix_with_merge_base(
        &mut self,
        paths: &Vec<NodePath<AnyHasBranch>>,
        base: &NodePath<AnyHasBranch>,
    ) -> Result<Conflict2DMatrix, GitError> {
        let mut statistics = MergeChainStatistics::new();

        let mut conflicting_with_base: Vec<NormalizedPath> = vec![];
        self.logger.debug("Checking against base pairwise");
        for s in self.checker.check_permutations_against_base(paths, base, 1) {
            let result = s?;
            self.logger.debug(result.display_as_path());
            statistics.push(result.clone());
            if result.contains_conflicts() {
                conflicting_with_base.push(result.get_chain().get(1).unwrap().get_path().clone());
            }
        }
        let to_test_with_base: Vec<NodePath<AnyHasBranch>> = paths
            .iter()
            .filter(|path| !conflicting_with_base.contains(&path.to_normalized_path()))
            .cloned()
            .collect();
        let _to_test_without_base: Vec<NodePath<AnyHasBranch>> = paths
            .iter()
            .filter(|path| conflicting_with_base.contains(&path.to_normalized_path()))
            .cloned()
            .collect();

        self.logger.debug("Checking successful against base");
        for with_base in self
            .checker
            .check_permutations_against_base(&to_test_with_base, &base, 2)
        {
            let mut result = with_base?;
            self.logger.debug(result.display_as_path());
            result.remove(0);
            let second = result.remove(0);
            let new = MergeStatistic::Base(second.get_path().clone());
            result.insert(0, new);
            statistics.push(result);
        }
        self.logger.debug("Checking conflicting without base");
        // TODO
        self.checker.clean_up();
        let matrix = Conflict2DMatrix::new(&statistics);
        Ok(matrix)
    }
}

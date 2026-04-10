use crate::cli::*;
use crate::git::interface::GitInterface;
use crate::model::*;
use crate::spl::DerivationMetadata;
use clap::{Arg, Command};
use std::error::Error;
use itertools::Itertools;
use crate::git::conflict::MergeResult;

const COMMIT: &str = "commit";
const FEATURE: &str = "feature";

fn find_features_that_contain_files<'a>(
    commit: &'a CommitHash,
    features: &'a Vec<NodePath<ConcreteFeature>>,
    git_interface: &GitInterface,
) -> Result<Vec<&'a NodePath<ConcreteFeature>>, Box<dyn Error>> {
    let files_of_commit = git_interface.get_files_changed_by_commit(commit)?;
    let all = features
        .iter()
        .filter(|feature| {
            let files_of_feature = git_interface.get_files_managed_by_branch(feature).unwrap();
            let mut found_all = true;
            for file in files_of_commit.iter() {
                if !files_of_feature.contains(file) {
                    found_all = false
                }
            }
            found_all
        })
        .collect();
    Ok(all)
}

#[derive(Clone, Debug)]
pub struct UntieCommand;

impl CommandDefinition for UntieCommand {
    fn build_command(&self) -> Command {
        Command::new("untie")
            .about("Untie commit from product and merge back into feature")
            .disable_help_subcommand(true)
            .arg(
                Arg::new(COMMIT)
                    .default_value("HEAD")
                    .help("Specific commit to untie"),
            )
            .arg(Arg::new(FEATURE).help("Feature to untie to"))
    }
}

impl CommandInterface for UntieCommand {
    fn run_command(&self, context: &mut CommandContext) -> Result<(), Box<dyn Error>> {
        let product = context.git.assert_current_node_path::<ConcreteProduct>()?;
        let target_commit = context
            .arg_helper
            .get_argument_value::<String>(COMMIT)
            .unwrap();
        let maybe_feature = context.arg_helper.get_argument_value::<String>(FEATURE);

        let commit: Commit = match target_commit.as_str() {
            "HEAD" => context.git.get_commit(&product)?,
            _ => context
                .git
                .get_commit_from_hash(&CommitHash::new(target_commit))?,
        };
        let metadata = commit.get_metadata();
        let maybe_derivation_data = metadata
            .iter()
            .find_map(|data| DerivationMetadata::from_commit_message(data));
        if maybe_derivation_data.is_none() {
            return Err(format!(
                "fatal: commit '{}' does not contain derivation metadata pointer",
                commit.get_hash().get_short_hash()
            )
            .into());
        }
        let derivation_data = maybe_derivation_data.unwrap()?;
        if derivation_data.get_data().is_some() {
            return Err(format!(
                "fatal: commit '{}' is a dedicated derivation commit and cannot be untied",
                commit.get_hash().get_short_hash()
            )
            .into());
        }
        let pointer = derivation_data.get_previous();
        let previous = context.git.get_commit_from_hash(pointer)?;
        let full_metadata = previous
            .get_metadata()
            .iter()
            .find_map(|data| DerivationMetadata::from_commit_message(data))
            .unwrap()?;
        let state = full_metadata.get_data().unwrap();
        let all_features = context.git.assert_paths::<ConcreteFeature>(state.get_total())?;

        let found = find_features_that_contain_files(commit.get_hash(), &all_features, &context.git)?;
        let feature: NodePath<ConcreteFeature> = match maybe_feature {
            Some(feature) => context
                .git
                .assert_path(&feature.to_normalized_path())?,
            None => match found.len() {
                0 => return Err(
                    "There are no features matching all changed files. Please choose one manually."
                        .into(),
                ),
                1 => all_features.get(0).unwrap().clone(),
                _ => {
                    let all = found
                        .iter()
                        .map(|feature| feature.formatted(true))
                        .join("\n  ");
                    return Err(format!(
                        "There are multiple potential untie targets. Please choose one manually.\nThe following features are candidates:\n  {all}",
                    ).into());
                }
            },
        };

        let mut product_with_commit = product.clone();
        product_with_commit.update_version(PointsTo::Commit(commit.get_hash().clone()));
        context.logger.info(format!("Untying commit to {}", feature.formatted(true)));
        context.git.checkout(&feature)?;
        let (result, _) = context.git.cherry_pick::<ConcreteFeature, _>(product_with_commit, true)?;
        if result.contains_up_to_date() {
            context.logger.info("Skipped: feature is up-to-date.");
            context.git.checkout(&product)?;
        } else if result.contains_conflicts() {
            context.logger.info("CONFLICT; Please fix all conflicts, then commit your changes.");
        } else if result.contains_errors() {
            context.git.checkout(&product)?;
            let error = match result.get(0).unwrap().get_stat() {
                MergeResult::Error(error) => error,
                _ => unreachable!(),
            };
            return Err(format!("Cannot untie: {error}").into())
        } else {
            context.logger.info("Please review the changes made and commit them afterwards.");
        }
        Ok(())
    }
}

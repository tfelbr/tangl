use crate::cli::completion::*;
use crate::cli::*;
use crate::model::{BranchAble, QualifiedPath};
use clap::{Arg, Command};
use colored::Colorize;
use std::error::Error;

#[derive(Clone, Debug)]
pub struct CheckoutCommand;
impl CommandDefinition for CheckoutCommand {
    fn build_command(&self) -> Command {
        Command::new("checkout")
            .about("Switch branches")
            .disable_help_subcommand(true)
            .arg(Arg::new("branch").required(true))
    }
}
impl CommandInterface for CheckoutCommand {
    fn run_command(&self, context: &mut CommandContext) -> Result<(), Box<dyn Error>> {
        let branch = context
            .arg_helper
            .get_argument_value::<String>("branch")
            .unwrap();
        let full_target = context.git.get_current_qualified_path()? + QualifiedPath::from(branch);
        let maybe_node_path = context.git.get_model().get_node_path(&full_target);
        if let Some(node_path) = maybe_node_path {
            let maybe_concrete = node_path.try_as_concrete_type::<BranchAble>();
            if let Some(concrete_branch) = maybe_concrete {
                let result = context.git.checkout(&concrete_branch)?;
                context.log_from_output(&result);
                Ok(())
            } else {
                Err(format!(
                    "fatal: path {} is no valid target for checkout",
                    full_target.to_string().red()
                )
                .into())
            }
        } else {
            Err(format!(
                "fatal: cannot checkout path {}: does not exist",
                full_target.to_string().red()
            )
            .into())
        }
    }
    fn shell_complete(
        &self,
        completion_helper: CompletionHelper,
        context: &mut CommandContext,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let maybe_editing = completion_helper.currently_editing();
        if maybe_editing.is_none() {
            return Ok(vec![]);
        }
        let all_branches = context.git.get_model().get_qualified_paths_with_branches();
        let result = match maybe_editing.unwrap().get_id().as_str() {
            "branch" => completion_helper.complete_qualified_paths(
                context.git.get_current_qualified_path()?,
                all_branches.iter().map(|path| path.clone()),
            ),
            _ => vec![],
        };
        Ok(result)
    }
}

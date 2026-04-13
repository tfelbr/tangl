use crate::cli::*;
use clap::{Arg, Command};
use std::error::Error;

const REPOSITORY: &str = "repository";

#[derive(Clone, Debug)]
pub struct CloneCommand;

impl CommandDefinition for CloneCommand {
    fn build_command(&self) -> Command {
        Command::new("clone")
            .about("Clone a repository")
            .disable_help_subcommand(true)
            .arg(
                Arg::new(REPOSITORY)
                    .help("Repository url"),
            )
    }
}

impl CommandInterface for CloneCommand {
    fn run_command(&self, context: &mut CommandContext) -> Result<(), Box<dyn Error>> {
        let repo = context.arg_helper.get_argument_value::<String>(REPOSITORY).unwrap();
        context.git.clone_repo(repo)?;
        Ok(())
    }
}

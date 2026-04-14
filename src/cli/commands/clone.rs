use crate::cli::*;
use clap::{Arg, ArgAction, Command};
use std::error::Error;

const REPOSITORY: &str = "repository";
const TRACK: &str = "track";

#[derive(Clone, Debug)]
pub struct CloneCommand;

impl CommandDefinition for CloneCommand {
    fn build_command(&self) -> Command {
        Command::new("clone")
            .about("Clone a repository")
            .disable_help_subcommand(true)
            .arg_required_else_help(true)
            .args(vec![
                Arg::new(REPOSITORY)
                    .help("Repository url"),
                Arg::new(TRACK)
                    .long(TRACK)
                    .action(ArgAction::SetTrue)
                    .help("Setup tracking for all branches. Must be located within the repository."),
            ])
    }
}

impl CommandInterface for CloneCommand {
    fn run_command(&self, context: &mut CommandContext) -> Result<(), Box<dyn Error>> {
        let repo = context.arg_helper.get_argument_value::<String>(REPOSITORY);
        let track = context.arg_helper.get_argument_value::<bool>(TRACK).unwrap();

        if track {
            let remotes = context.git.get_remote_branches()?;
            for remote in remotes {
                if !remote.contains("HEAD") {
                    let local = remote.replace("origin/", "");
                    match context.git.track_branch(&local, &remote) {
                        Ok(_) => {}
                        Err(_) => {}
                    };
                }
            }
        } else {
            context.git.clone_repo(repo.unwrap())?;
        }
        Ok(())
    }
}

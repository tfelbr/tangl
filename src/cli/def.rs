use crate::cli::ArgHelper;
use crate::cli::completion::CompletionHelper;
use crate::git::interface::GitInterface;
use crate::logging::TanglLogger;
use crate::model::ImportFormat;
use clap::{ArgMatches, Command};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub struct CommandMap {
    pub clap_command: Command,
    pub command: Box<dyn CommandInterface>,
    pub children: Vec<CommandMap>,
}

impl CommandMap {
    pub fn new(command: Box<dyn CommandImpl>) -> CommandMap {
        let mut children: Vec<CommandMap> = Vec::new();
        let clap_command = command.build_command().subcommands(
            command
                .get_subcommands()
                .into_iter()
                .map(|c| {
                    let sub_command = c.build_command();
                    children.push(CommandMap::new(c));
                    sub_command
                })
                .collect::<Vec<Command>>(),
        );
        CommandMap {
            clap_command,
            command,
            children,
        }
    }
    pub fn find_child(&self, name: &str) -> Option<&CommandMap> {
        self.children
            .iter()
            .find(|child| child.clap_command.get_name() == name)
    }
    pub fn find_current_child(&self, matches: &ArgMatches) -> Option<&CommandMap> {
        match matches.subcommand() {
            Some((name, sub_matches)) => {
                let maybe_child = self.find_child(name);
                if maybe_child.is_some() {
                    let child_result = maybe_child.unwrap().find_current_child(sub_matches);
                    if child_result.is_some() {
                        child_result
                    } else {
                        Some(self)
                    }
                } else {
                    Some(self)
                }
            }
            _ => Some(self),
        }
    }
    pub fn find_children_by_prefix(&self, prefix: &str) -> Vec<&CommandMap> {
        self.children
            .iter()
            .filter(|child| child.clap_command.get_name().starts_with(prefix))
            .collect()
    }
}

#[derive(Debug)]
pub struct CommandContext<'a> {
    pub current_command: &'a CommandMap,
    pub root_command: &'a CommandMap,
    pub git: GitInterface,
    pub logger: TanglLogger,
    pub arg_helper: ArgHelper,
    pub import_format: ImportFormat,
}

impl CommandContext<'_> {
    pub fn new<'a>(
        current_command: &'a CommandMap,
        root_command: &'a CommandMap,
        git: GitInterface,
        logger: TanglLogger,
        arg_helper: ArgHelper,
        import_format: ImportFormat,
    ) -> CommandContext<'a> {
        CommandContext {
            current_command,
            root_command,
            git,
            logger,
            arg_helper,
            import_format,
        }
    }
}

pub trait CommandDefinition: Debug {
    fn build_command(&self) -> Command;
    fn get_subcommands(&self) -> Vec<Box<dyn CommandImpl>> {
        Vec::new()
    }
}

pub trait CommandInterface: Debug {
    fn run_command(&self, _context: &mut CommandContext) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    fn shell_complete(
        &self,
        _completion_helper: CompletionHelper,
        _context: &mut CommandContext,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        Ok(Vec::new())
    }
}

pub trait CommandImpl: CommandDefinition + CommandInterface + Debug {}
impl<T: CommandDefinition + CommandInterface + Debug> CommandImpl for T {}

#[derive(Debug, Clone)]
pub struct CommandError {
    msg: String,
}
impl CommandError {
    pub fn new(msg: &str) -> CommandError {
        CommandError {
            msg: msg.to_string(),
        }
    }
}
impl Display for CommandError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}
impl Error for CommandError {}

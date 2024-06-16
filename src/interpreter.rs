use anyhow::{Result, anyhow};

use crate::commands;

pub struct Interpreter {
    
}

impl Interpreter {
    pub fn respond(self: &Self, cmd: commands::Command) -> Result<commands::Command> {
        match cmd {
            commands::Command::PING => Ok(commands::Command::PONG),
            commands::Command::ECHO(x) => Ok(commands::Command::ECHO(x)),
            x => Err(anyhow!("unexpected command: {:?}", x))
        }
    }

    pub(crate) fn new() -> Interpreter {
        Interpreter{}
    }
}

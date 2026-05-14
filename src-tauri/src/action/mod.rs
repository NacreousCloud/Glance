pub mod launch_app;
pub mod open_url;
pub mod run_shell;

use crate::settings::Action;
use anyhow::Result;

pub struct ActionRunner;

impl ActionRunner {
    /// Execute an action. For shell actions that require confirmation, the
    /// confirm step must be handled BEFORE calling this (frontend
    /// ConfirmDialog gates the call). This method does not prompt.
    pub fn execute(action: &Action) -> Result<()> {
        match action {
            Action::LaunchApp { path } => launch_app::run(path),
            Action::OpenUrl { url } => open_url::run(url),
            Action::RunShell { command, args, .. } => run_shell::run(command, args),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_branches_compile() {
        // Cheap smoke: enum variants reach their handlers.
        let _ = ActionRunner::execute as fn(&Action) -> Result<()>;
    }
}

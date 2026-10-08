pub mod launch_app;
pub mod open_url;
mod process;
pub mod run_shell;

use crate::settings::Action;
use anyhow::Result;

pub struct ActionRunner;

impl ActionRunner {
    /// Execute the same snapshot shown in the native prompt. Cancellation
    /// is a successful no-op, not an action failure.
    pub fn execute_with_confirmation(
        action: &Action,
        confirm: impl FnOnce(&str) -> bool,
    ) -> Result<bool> {
        if let Action::RunShell {
            command,
            args,
            confirm: true,
        } = action
        {
            let message = format!(
                "Run this command?\n\nProgram: {command}\nArguments: {}",
                serde_json::to_string(args)?
            );
            if !confirm(&message) {
                return Ok(false);
            }
        }
        Self::execute(action)?;
        Ok(true)
    }

    fn execute(action: &Action) -> Result<()> {
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
    fn cancelling_confirmation_does_not_execute() {
        let action = Action::RunShell {
            command: "glance-nonexistent-command-for-test".into(),
            args: vec!["argument with spaces".into()],
            confirm: true,
        };
        let executed = ActionRunner::execute_with_confirmation(&action, |message| {
            assert!(message.contains("glance-nonexistent-command-for-test"));
            assert!(message.contains("argument with spaces"));
            false
        })
        .unwrap();
        assert!(!executed);
        assert!(ActionRunner::execute_with_confirmation(&action, |_| true).is_err());
    }

    #[test]
    fn dispatch_branches_compile() {
        // Cheap smoke: enum variants reach their handlers.
        let _ = ActionRunner::execute as fn(&Action) -> Result<()>;
    }
}

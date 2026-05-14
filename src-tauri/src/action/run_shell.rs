use anyhow::{Context, Result};
use std::process::Command;

pub fn run(command: &str, args: &[String]) -> Result<()> {
    if command.is_empty() {
        anyhow::bail!("shell command cannot be empty");
    }
    Command::new(command)
        .args(args)
        .status()
        .with_context(|| format!("failed to spawn `{}`", command))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_command_errors() {
        let err = run("", &[]).unwrap_err();
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn true_command_runs() {
        run("/usr/bin/true", &[]).unwrap();
    }
}

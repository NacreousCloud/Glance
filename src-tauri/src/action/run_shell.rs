use anyhow::Result;
use std::process::Command;

pub fn run(command: &str, args: &[String]) -> Result<()> {
    if command.is_empty() {
        anyhow::bail!("shell command cannot be empty");
    }
    super::process::run(Command::new(command).args(args))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(unix)]
    fn nonzero_exit_reports_status_and_stderr() {
        let error = run(
            "/bin/sh",
            &["-c".into(), "printf 'permission denied' >&2; exit 7".into()],
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains('7'), "{error}");
        assert!(error.contains("permission denied"), "{error}");
    }

    #[test]
    #[cfg(windows)]
    fn nonzero_exit_reports_status_and_stderr() {
        let error = run(
            "cmd",
            &["/c".into(), "echo permission denied >&2 & exit /b 7".into()],
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains('7'), "{error}");
        assert!(error.contains("permission denied"), "{error}");
    }

    #[test]
    fn empty_command_errors() {
        let err = run("", &[]).unwrap_err();
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    #[cfg(unix)]
    fn true_command_runs() {
        run("/usr/bin/true", &[]).unwrap();
    }

    #[test]
    #[cfg(windows)]
    fn cmd_command_runs() {
        // `cmd /c exit 0` — Windows equivalent of `/usr/bin/true`.
        run("cmd", &["/c".into(), "exit".into(), "0".into()]).unwrap();
    }
}

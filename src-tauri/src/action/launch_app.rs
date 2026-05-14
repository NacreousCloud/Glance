use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

pub fn run(path: &str) -> Result<()> {
    let p = PathBuf::from(path);
    if !p.exists() {
        anyhow::bail!("app path does not exist: {}", path);
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&p)
            .status()
            .with_context(|| format!("failed to spawn `open` for {}", path))?;
    }

    #[cfg(target_os = "windows")]
    {
        // `cmd /c start "" <path>` resolves .lnk, .exe, and registered
        // file associations the same way Explorer does. The empty quoted
        // argument is the window title that `start` insists on consuming
        // when the first quoted token is the path.
        Command::new("cmd")
            .args(["/c", "start", ""])
            .arg(&p)
            .status()
            .with_context(|| format!("failed to spawn via `cmd /c start` for {}", path))?;
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Command::new("xdg-open")
            .arg(&p)
            .status()
            .with_context(|| format!("failed to spawn `xdg-open` for {}", path))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_path_errors() {
        let err = run("/nonexistent/app.app").unwrap_err();
        assert!(err.to_string().contains("does not exist"));
    }
}

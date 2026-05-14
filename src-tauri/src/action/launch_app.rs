use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

pub fn run(path: &str) -> Result<()> {
    let p = PathBuf::from(path);
    if !p.exists() {
        anyhow::bail!("app path does not exist: {}", path);
    }
    Command::new("open")
        .arg(&p)
        .status()
        .with_context(|| format!("failed to spawn `open` for {}", path))?;
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

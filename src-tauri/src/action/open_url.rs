use anyhow::{Context, Result};
use std::process::Command;

pub fn run(url: &str) -> Result<()> {
    if !url.starts_with("http://") && !url.starts_with("https://") && !url.contains("://") {
        anyhow::bail!("url must have a scheme: {}", url);
    }
    Command::new("open")
        .arg(url)
        .status()
        .with_context(|| format!("failed to spawn `open` for {}", url))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_schemeless() {
        let err = run("example.com").unwrap_err();
        assert!(err.to_string().contains("scheme"));
    }
}

use anyhow::{Context, Result};

pub fn run(url: &str) -> Result<()> {
    if !url.starts_with("http://") && !url.starts_with("https://") && !url.contains("://") {
        anyhow::bail!("url must have a scheme: {}", url);
    }
    tauri_plugin_opener::open_url(url, None::<&str>)
        .with_context(|| format!("failed to open URL: {url}"))?;
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

use anyhow::{Context, Result};
use std::io::{Read, Seek, SeekFrom};
use std::process::{Command, Stdio};

/// Keep output out of the app's terminal; retain a bounded error excerpt.
/// A temporary file avoids pipe backpressure and unbounded in-memory output.
pub(super) fn run(command: &mut Command) -> Result<()> {
    let program = command.get_program().to_string_lossy().into_owned();
    let mut stderr = tempfile::tempfile().context("cannot capture command errors")?;
    let status = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(stderr.try_clone()?)
        .status()
        .with_context(|| format!("failed to spawn `{program}`"))?;
    if !status.success() {
        stderr.seek(SeekFrom::Start(0))?;
        let mut bytes = Vec::new();
        stderr.take(8193).read_to_end(&mut bytes)?;
        let truncated = bytes.len() > 8192;
        bytes.truncate(8192);
        let detail = String::from_utf8_lossy(&bytes);
        anyhow::bail!(
            "`{program}` failed ({status})\n{}{}",
            detail.trim(),
            if truncated {
                "\n[stderr truncated]"
            } else {
                ""
            }
        );
    }
    Ok(())
}

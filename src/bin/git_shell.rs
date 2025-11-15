use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, anyhow};

/// Default root directory for all bare repos.
/// Can be overridden with PAASTEL_GIT_ROOT.
const DEFAULT_GIT_ROOT: &str = "/var/lib/paastel/git";

fn main() -> Result<()> {
    // 1) Read the original SSH command
    //    When called by sshd, this comes from SSH_ORIGINAL_COMMAND.
    //    For local testing, you can pass it as the first CLI argument.
    let original_command = env::var("SSH_ORIGINAL_COMMAND")
        .ok()
        .or_else(|| env::args().nth(1))
        .ok_or_else(|| {
            anyhow!(
                "Missing SSH_ORIGINAL_COMMAND and no CLI argument provided"
            )
        })?;

    // 2) Parse it: we expect "git-receive-pack 'path.git'" or "git-upload-pack 'path.git'"
    let (git_cmd, repo_path_raw) = parse_git_command(&original_command)?;

    // 3) Sanitize and build the full repo path
    let root = env::var("PAASTEL_GIT_ROOT")
        .unwrap_or_else(|_| DEFAULT_GIT_ROOT.to_string());
    let repo_rel = sanitize_repo_path(&repo_path_raw)?;
    let repo_full = Path::new(&root).join(repo_rel);

    // Ensure parent directories exist
    if let Some(parent) = repo_full.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!("Failed to create parent dir {}", parent.display())
        })?;
    }

    // 4) If it's a receive-pack and the repo doesn't exist yet, init it as a bare repo
    if git_cmd == "git-receive-pack" && !repo_full.exists() {
        init_bare_repo(&repo_full)?;
    }

    // 5) Delegate to the real git-* command, wiring stdin/stdout/stderr
    let status = Command::new(git_cmd)
        .arg(repo_full.to_str().ok_or_else(|| anyhow!("Invalid repo path"))?)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("Failed to spawn {}", git_cmd))?;

    if !status.success() {
        return Err(anyhow!(
            "{} exited with status code: {}",
            git_cmd,
            status
        ));
    }

    Ok(())
}

/// Parse commands like:
/// - "git-receive-pack '/kovi/devsecops/site-estatico.git'"
/// - "git-upload-pack \"/kovi/devsecops/site-estatico.git\""
fn parse_git_command(cmd: &str) -> Result<(&'static str, String)> {
    let cmd = cmd.trim();

    for candidate in
        ["git-receive-pack", "git-upload-pack", "git-upload-archive"]
    {
        let prefix = format!("{candidate} ");
        if let Some(rest) = cmd.strip_prefix(&prefix) {
            let repo = rest.trim();
            // strip single or double quotes around the path if present
            let repo = repo
                .strip_prefix('\'')
                .and_then(|s| s.strip_suffix('\''))
                .or_else(|| {
                    repo.strip_prefix('"').and_then(|s| s.strip_suffix('"'))
                })
                .unwrap_or(repo);

            if repo.is_empty() {
                return Err(anyhow!(
                    "Missing repository path in command: {cmd}"
                ));
            }

            return Ok((candidate, repo.to_string()));
        }
    }

    Err(anyhow!("Unsupported git command: {cmd}"))
}

/// Very small sanitization for the repository path.
/// We do not allow path traversal ("..") and strip leading slashes.
/// This function returns a relative path to be appended to the GIT_ROOT.
fn sanitize_repo_path(raw: &str) -> Result<PathBuf> {
    if raw.contains("..") {
        return Err(anyhow!(
            "Repository path cannot contain '..' (got: {raw})"
        ));
    }

    let trimmed = raw.trim_start_matches('/');
    if trimmed.is_empty() {
        return Err(anyhow!("Invalid repository path: {raw}"));
    }

    Ok(PathBuf::from(trimmed))
}

/// Initialize a bare git repository at the given path.
///
/// Equivalent to: `git init --bare /var/lib/paastel/git/kovi/devsecops/app.git`
fn init_bare_repo(path: &Path) -> Result<()> {
    println!("Initializing bare repository at {}", path.display());

    let status = Command::new("git")
        .arg("init")
        .arg("--bare")
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to run `git init --bare`")?;

    if !status.success() {
        return Err(anyhow!("`git init --bare` failed with status: {status}"));
    }

    Ok(())
}

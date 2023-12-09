use std::{
    ffi::OsStr,
    io,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    str::ParseBoolError,
    string::FromUtf8Error,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitCmdError {
    #[error("failed to check if directory is git repo: {0}")]
    IsRepositoryUtf8Error(#[from] FromUtf8Error),

    #[error("failed to check if directory is git repo: {0}")]
    GetRemoteError(#[source] FromUtf8Error),

    #[error("failed to check if directory is git repo: {0}")]
    IsRepositoryBoolParseError(#[from] ParseBoolError),

    #[error("failed to check if directory is git repo: {0}")]
    IsRepositoryIo(#[source] io::Error),

    #[error("failed to clone: {0}")]
    Clone(#[source] io::Error),
}

pub struct Git;
impl Git {
    pub fn clone(uri: &str, to_path: PathBuf) -> Result<Output, GitCmdError> {
        wrap_cmd(
            "git",
            [
                "clone".to_string(),
                uri.to_string(),
                to_path.to_string_lossy().to_string(),
            ],
        )
        .map_err(GitCmdError::Clone)
    }

    pub fn get_remote_url<P>(repo_path: &P) -> Result<Option<String>, GitCmdError>
    where
        P: AsRef<Path>,
    {
        let output = wrap_cmd_dir("git", ["remote", "get-url", "origin"], repo_path)
            .map_err(GitCmdError::IsRepositoryIo)?;

        let remote = String::from_utf8(output.stdout)
            .map_err(GitCmdError::GetRemoteError)?
            .trim()
            .to_string();
        if remote.is_empty() {
            return Ok(None);
        }

        Ok(Some(remote))
    }

    pub fn is_inside_worktree<P>(repo_path: &P) -> bool
    where
        P: AsRef<Path>,
    {
        if let Ok(output) = wrap_cmd_dir("git", ["rev-parse", "--is-inside-work-tree"], repo_path) {
            if let Ok(is_git_worktree) = String::from_utf8(output.stdout) {
                if let Ok(parsed) = is_git_worktree.trim().parse::<bool>() {
                    return parsed;
                }
            }
        }
        false
    }
}

fn wrap_cmd<I, S>(cmd: &str, args: I) -> io::Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = pipe_io(Command::new(cmd).args(args))
        .spawn()?
        .wait_with_output()?;

    log_output(&output);

    Ok(output)
}

fn wrap_cmd_dir<I, S, P>(cmd: &str, args: I, path: P) -> io::Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
    P: AsRef<Path>,
{
    let output = pipe_io(Command::new(cmd).args(args).current_dir(path))
        .spawn()?
        .wait_with_output()?;

    log_output(&output);

    Ok(output)
}

pub fn pipe_io(cmd: &mut Command) -> &mut Command {
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped())
}

pub fn log_output(output: &Output) {
    // Use log crate to allow verbosity flag to control wrapped command logs.
    if output.status.success() && !output.stdout.is_empty() {
        log::info!("{}", String::from_utf8_lossy(&output.stdout).trim());
    } else if !output.stderr.is_empty() {
        log::warn!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }
}

use git_url_parse::GitUrl;
use std::{
    ffi::OsStr,
    io,
    path::Path,
    process::{Command, Output, Stdio},
    str::{FromStr, ParseBoolError},
    string::FromUtf8Error,
};
use thiserror::Error;

use crate::git_uri::GitUri;

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

    #[error("failed to initalize directory as git repo: {0}")]
    InitError(#[source] io::Error),

    #[error("failed to clone: {0}")]
    Clone(#[source] io::Error),

    #[error("failed parsing git url: {0}")]
    ParseUriError(#[source] <GitUrl as FromStr>::Err),
}

const GIT_COMMAND: &str = "git";

pub struct Git;

impl Git {
    pub fn clone(uri: &str, to_path: &Path) -> Result<Output, GitCmdError> {
        wrap_cmd(
            GIT_COMMAND,
            [
                "clone".to_string(),
                uri.to_string(),
                to_path.to_string_lossy().to_string(),
            ],
        )
        .map_err(GitCmdError::Clone)
    }

    pub fn status<P>(repo_path: &P) -> Result<Option<String>, GitCmdError>
    where
        P: AsRef<Path>,
    {
        let output = wrap_cmd_dir(GIT_COMMAND, ["status"], repo_path)
            .map_err(GitCmdError::IsRepositoryIo)?;

        let status = String::from_utf8(output.stdout)
            .map_err(GitCmdError::GetRemoteError)?
            .trim()
            .to_string();
        if status.is_empty() {
            return Ok(None);
        }

        Ok(Some(status))
    }

    pub fn log<P>(repo_path: &P) -> Result<Option<String>, GitCmdError>
    where
        P: AsRef<Path>,
    {
        let output =
            wrap_cmd_dir(GIT_COMMAND, ["log"], repo_path).map_err(GitCmdError::IsRepositoryIo)?;

        let log = String::from_utf8(output.stdout)
            .map_err(GitCmdError::GetRemoteError)?
            .trim()
            .to_string();
        if log.is_empty() {
            return Ok(None);
        }

        Ok(Some(log))
    }

    pub fn init(path: &Path) -> Result<(), GitCmdError> {
        let _ = wrap_cmd_dir("git", ["init"], path).map_err(GitCmdError::InitError)?;
        Ok(())
    }

    pub fn add_remote(
        remote_name: &str,
        remote_url: &str,
        repo_path: &Path,
    ) -> Result<(), GitCmdError> {
        let _ = wrap_cmd_dir("git", ["remote", "add", remote_name, remote_url], repo_path)
            .map_err(GitCmdError::InitError)?;
        Ok(())
    }

    pub fn get_remote_url<P>(
        remote_name: &str,
        repo_path: &P,
    ) -> Result<Option<String>, GitCmdError>
    where
        P: AsRef<Path>,
    {
        let output = wrap_cmd_dir("git", ["remote", "get-url", remote_name], repo_path)
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

    pub fn parse_uri(url: &str) -> Result<GitUri, GitCmdError> {
        Ok(GitUri::from(GitUrl::parse(url).map_err(GitCmdError::ParseUriError)?))
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

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::Result;

    use assert_fs::*;
    use predicates::prelude::*;
    use rstest::{fixture, rstest};

    use super::Git;

    #[fixture]
    fn temp_directory_fs() -> TempDir {
        // Arrange
        TempDir::new().expect("should be able to make temp dir")
    }

    #[fixture]
    fn temp_repo_fs(temp_directory_fs: TempDir) -> TempDir {
        // Arrange
        Git::init(temp_directory_fs.path()).expect("git repo should init in temp dir");
        temp_directory_fs
    }

    #[rstest]
    fn should_init_directory_as_git_repo(temp_directory_fs: TempDir) -> Result<()> {
        // Arrange / Act
        Git::init(temp_directory_fs.path())?;

        // Assert
        assert!(
            predicate::path::exists().eval(&temp_directory_fs.path().join(".git").join("hooks"))
        );
        assert!(predicate::path::exists().eval(&temp_directory_fs.path().join(".git").join("info")));
        assert!(
            predicate::path::exists().eval(&temp_directory_fs.path().join(".git").join("objects"))
        );
        assert!(predicate::path::exists().eval(&temp_directory_fs.path().join(".git").join("refs")));
        assert!(predicate::path::exists()
            .eval(&temp_directory_fs.path().join(".git").join("description")));
        assert!(
            predicate::path::exists().eval(&temp_directory_fs.path().join(".git").join("config"))
        );
        assert!(predicate::path::exists().eval(&temp_directory_fs.path().join(".git").join("HEAD")));

        Ok(())
    }

    #[rstest]
    fn should_add_remote_and_get_it_from_repo(temp_repo_fs: TempDir) -> Result<()> {
        // Arrange
        let remote = "git@github.com:test_user/test_repo1.git";

        // Act
        Git::add_remote("origin", remote, temp_repo_fs.path())?;
        let config_content = fs::read_to_string(temp_repo_fs.path().join(".git").join("config"))?;
        let found_remote = Git::get_remote_url("origin", &temp_repo_fs.path())?;

        // Assert
        assert!(config_content.contains(&format!("[remote \"origin\"]\n\turl = {remote}")));
        assert_eq!(found_remote, Some(remote.to_string()));

        Ok(())
    }
}

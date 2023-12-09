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
pub enum GitRepoError {
    #[error("there was an error while cloning {ssh_uri} to {repo_path}: {source}")]
    CloneError {
        ssh_uri: String,
        repo_path: String,
        #[source]
        source: anyhow::Error,
    },

    #[error("failed to parse string into boolean: {0}")]
    RepoCheckBoolParseError(#[from] ParseBoolError),

    #[error("failed to convert git command output from utf8 into string: {0}")]
    RepoCheckUtf8Error(#[from] FromUtf8Error),

    #[error("failed to run git command: {source}")]
    GitCommandError {
        #[source]
        source: io::Error,
    },
}

pub struct GitRepo {
    root_path: PathBuf,
    ssh_uri: Option<String>,
}

impl GitRepo {
    pub fn from_ssh_uri(ssh_uri: &str, to_path: PathBuf) -> Result<GitRepo, GitRepoError> {
        Git::clone(ssh_uri, to_path.clone())?;
        Ok(GitRepo {
            root_path: to_path,
            ssh_uri: Some(ssh_uri.to_string()),
        })
    }

    pub fn from_existing(repo_path: &PathBuf) -> Result<GitRepo, GitRepoError> {
        if Git::is_inside_worktree(repo_path)? {
            let ssh_uri = Some("".to_string());
            Ok(GitRepo {
                root_path: repo_path.clone(),
                ssh_uri,
            })
        } else {
            todo!()
        }
    }
}

pub struct Git;
impl Git {
    pub fn clone(uri: &str, to_path: PathBuf) -> Result<Output, GitRepoError> {
        wrap_cmd(
            "git",
            [
                "clone".to_string(),
                uri.to_string(),
                to_path.to_string_lossy().to_string(),
            ],
        )
    }

    pub fn is_inside_worktree<P>(repo_path: P) -> Result<bool, GitRepoError>
    where
        P: AsRef<Path>,
    {
        let output = wrap_cmd_dir(
            "git",
            ["rev-parse".to_string(), "--is-inside-worktree".to_string()],
            repo_path,
        )?;

        let is_git_worktree = String::from_utf8(output.stdout)
            .map_err(GitRepoError::RepoCheckUtf8Error)?
            .parse::<bool>()
            .map_err(GitRepoError::RepoCheckBoolParseError)?;

        Ok(is_git_worktree)
    }
}

fn wrap_cmd<I, S>(cmd: &str, args: I) -> Result<Output, GitRepoError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = pipe_io(Command::new(cmd).args(args))?;

    log_output(&output);

    Ok(output)
}

fn wrap_cmd_dir<I, S, P>(cmd: &str, args: I, path: P) -> Result<Output, GitRepoError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
    P: AsRef<Path>,
{
    let output = pipe_io(Command::new(cmd).args(args).current_dir(path))?;

    log_output(&output);

    Ok(output)
}

pub fn pipe_io(cmd: &mut Command) -> Result<Output, GitRepoError> {
    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| GitRepoError::GitCommandError { source: e })?
        .wait_with_output()
        .map_err(|e| GitRepoError::GitCommandError { source: e })
}

pub fn log_output(output: &Output) {
    // Use log crate to allow verbosity flag to control wrapped command logs.
    if output.status.success() && !output.stdout.is_empty() {
        log::info!("{}", String::from_utf8_lossy(&output.stdout).trim());
    } else if !output.stderr.is_empty() {
        log::warn!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        panic::{catch_unwind, UnwindSafe},
        path::Path,
        str::FromStr,
    };

    use super::*;

    const REPO_CLONE_DIR: &str = "/tmp/git_repo_tests";

    #[test]
    fn test_ssh_clone_git_repo() {
        run_test(|| {
            // Arrange
            let path = &format!("{}/test_ssh_clone", REPO_CLONE_DIR);

            // Act
            let repo = GitRepo::from_ssh_uri(
                "git@github.com:pitoniak32/git_repo.git",
                PathBuf::from_str(path).expect("should not fail"),
            )
            .expect("should not fail");

            // Assert
            assert_eq!(
                repo.ssh_uri,
                Some("git@github.com:pitoniak32/git_repo.git".to_string())
            );
            assert!(Path::exists(&repo.root_path));
        })
    }

    #[test]
    fn test_https_clone_git_repo() {
        run_test(|| {
            // Arrange
            let path = &format!("{}/test_https_clone", REPO_CLONE_DIR);

            // Act
            let repo = GitRepo::from_ssh_uri(
                "https://github.com/pitoniak32/git_repo.git",
                PathBuf::from_str(path).expect("should not fail"),
            )
            .expect("should not fail");

            // Assert
            assert_eq!(
                repo.ssh_uri,
                Some("https://github.com/pitoniak32/git_repo.git".to_string())
            );
            assert!(Path::exists(&repo.root_path));
        })
    }

    fn teardown() -> io::Result<()> {
        fs::remove_dir_all(REPO_CLONE_DIR)?;
        Ok(())
    }

    fn run_test<T>(test: T)
    where
        T: FnOnce() + UnwindSafe,
    {
        let result = catch_unwind(test);

        teardown().expect("teardown process should not fail");

        assert!(result.is_ok())
    }
}

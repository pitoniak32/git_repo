use git_url_parse::Scheme;
use std::{
    fs, io,
    path::{Path, PathBuf},
    string::FromUtf8Error,
};
use thiserror::Error;

use crate::git::{Git, GitCmdError};

#[derive(Error, Debug)]
pub enum GitRepoError {
    #[error("there was an error while cloning {remote_url} to {repo_path}: {source}")]
    CloneError {
        remote_url: String,
        repo_path: String,
        #[source]
        source: GitCmdError,
    },

    #[error("failed to parse string into boolean: {0}")]
    RepoCheck(#[from] GitCmdError),

    #[error("failed to expand provided repo path: {0}")]
    RepoPathExpansionError(#[from] io::Error),

    #[error("failed to convert git command output from utf8 into string: {0}")]
    RepoCheckUtf8Error(#[from] FromUtf8Error),

    #[error("failed to clone git repo into {0}. this path is already a git repo.")]
    AlreadyExistsError(String),

    #[error("failed to clone git repo with uri {0}. invalid ssh uri.")]
    InvalidGitSshUri(String),
}

#[derive(Debug)]
pub struct GitRepo {
    pub root_path: PathBuf,
    pub remote_url: Option<String>,
}

impl GitRepo {
    pub fn from_ssh_uri(ssh_uri: &str, to_path: &PathBuf) -> Result<GitRepo, GitRepoError> {
        assert!(
            !to_path.to_string_lossy().to_string().contains('~'),
            "repo_path must be absoloute or relative, ~ is not supported"
        );
        if !to_path.exists() {
            fs::create_dir_all(to_path)?;
        }
        let expanded_path = &to_path
            .canonicalize()
            .map_err(GitRepoError::RepoPathExpansionError)?;

        if Git::is_inside_worktree(&expanded_path) {
            return Err(GitRepoError::AlreadyExistsError(
                expanded_path.to_string_lossy().to_string(),
            ));
        }

        Git::clone(ssh_uri, to_path.clone()).map_err(|e| GitRepoError::CloneError {
            remote_url: ssh_uri.to_string(),
            repo_path: expanded_path.to_string_lossy().to_string(),
            source: e,
        })?;

        GitRepo::from_existing(to_path)
    }

    /// Will remove the contents of the `to_path` before cloning
    pub fn from_ssh_uri_force(ssh_uri: &str, to_path: &PathBuf) -> Result<GitRepo, GitRepoError> {
        assert!(
            !to_path.to_string_lossy().to_string().contains('~'),
            "repo_path must be absoloute or relative, ~ is not supported"
        );

        if !to_path.exists() {
            fs::create_dir_all(to_path)?;
        } else {
            fs::remove_dir_all(to_path)?;
            fs::create_dir_all(to_path)?;
        }

        let expanded_path = &to_path
            .canonicalize()
            .map_err(GitRepoError::RepoPathExpansionError)?;

        Git::clone(ssh_uri, to_path.clone()).map_err(|e| GitRepoError::CloneError {
            remote_url: ssh_uri.to_string(),
            repo_path: expanded_path.to_string_lossy().to_string(),
            source: e,
        })?;

        GitRepo::from_existing(to_path)
    }

    pub fn from_ssh_uri_multi(
        ssh_uris: &[&str],
        to_root_path: &Path,
    ) -> Vec<Result<GitRepo, GitRepoError>> {
        let mut repo_results = vec![];
        for ssh_uri in ssh_uris {
            if let Ok(parsed_uri) = Git::parse_url(ssh_uri) {
                if parsed_uri.scheme == Scheme::GitSsh || parsed_uri.scheme == Scheme::Ssh {
                    repo_results.push(GitRepo::from_ssh_uri(
                        ssh_uri,
                        &to_root_path.join(parsed_uri.name),
                    ));
                } else {
                    repo_results.push(Err(GitRepoError::InvalidGitSshUri(ssh_uri.to_string())));
                }
            } else {
                repo_results.push(Err(GitRepoError::InvalidGitSshUri(ssh_uri.to_string())));
            }
        }
        repo_results
    }

    /// Sets remote_url to value of `origin`.
    pub fn from_existing(repo_path: &PathBuf) -> Result<GitRepo, GitRepoError> {
        assert!(
            !repo_path.to_string_lossy().to_string().contains('~'),
            "repo_path must be absoloute or relative, ~ is not supported"
        );
        let expanded_path =
            std::fs::canonicalize(repo_path).map_err(GitRepoError::RepoPathExpansionError)?;

        if Git::is_inside_worktree(&expanded_path) {
            Ok(GitRepo {
                root_path: expanded_path.clone(),
                remote_url: Git::get_remote_url(&expanded_path)?,
            })
        } else {
            todo!()
        }
    }
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
                &PathBuf::from_str(path).expect("should not fail"),
            )
            .expect("should not fail");

            // Assert
            assert_eq!(
                repo.remote_url,
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
                &PathBuf::from_str(path).expect("should not fail"),
            )
            .expect("should not fail");

            // Assert
            assert_eq!(
                repo.remote_url,
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

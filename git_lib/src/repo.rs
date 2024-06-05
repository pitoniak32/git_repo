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

    #[error("failed to clone git repo with url {0}. invalid remote url.")]
    InvalidGitRemoteUrl(String),
}

#[derive(Debug)]
pub struct GitRepo {
    pub root_path: PathBuf,
    pub remote_url: Option<String>,
}

impl GitRepo {
    pub fn from_url(remote_url: &str, to_path: &Path) -> Result<GitRepo, GitRepoError> {
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

        Git::clone(remote_url, to_path).map_err(|e| GitRepoError::CloneError {
            remote_url: remote_url.to_string(),
            repo_path: expanded_path.to_string_lossy().to_string(),
            source: e,
        })?;

        GitRepo::from_existing(to_path)
    }

    /// Will remove the contents of the `to_path` before cloning
    pub fn from_url_force(remote_url: &str, to_path: &PathBuf) -> Result<GitRepo, GitRepoError> {
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

        Git::clone(remote_url, to_path).map_err(|e| GitRepoError::CloneError {
            remote_url: remote_url.to_string(),
            repo_path: expanded_path.to_string_lossy().to_string(),
            source: e,
        })?;

        GitRepo::from_existing(to_path)
    }

    pub fn from_url_multi(
        remote_urls: &[&str],
        to_root_path: &Path,
    ) -> Vec<Result<GitRepo, GitRepoError>> {
        let mut repo_results = vec![];
        for remote_url in remote_urls {
            if let Ok(parsed_uri) = Git::parse_uri(remote_url) {
                repo_results.push(GitRepo::from_url(
                    remote_url,
                    &to_root_path.join(parsed_uri.name),
                ));
            } else {
                repo_results.push(Err(GitRepoError::InvalidGitRemoteUrl(
                    remote_url.to_string(),
                )));
            }
        }
        repo_results
    }

    /// Sets remote_url to value of `origin`.
    pub fn from_existing(repo_path: &Path) -> Result<GitRepo, GitRepoError> {
        assert!(
            !repo_path.to_string_lossy().to_string().contains('~'),
            "repo_path must be absoloute or relative, ~ is not supported"
        );
        let expanded_path =
            std::fs::canonicalize(repo_path).map_err(GitRepoError::RepoPathExpansionError)?;

        if Git::is_inside_worktree(&expanded_path) {
            Ok(GitRepo {
                root_path: expanded_path.clone(),
                remote_url: Git::get_remote_url("origin", &expanded_path)?,
            })
        } else {
            todo!()
        }
    }
}

#[cfg(test)]
mod tests {

    use std::path::Path;

    use super::*;

    use assert_fs::*;

    use rstest::{fixture, rstest};

    // const REPO_CLONE_SSH: &str = "git@github.com:pitoniak32/git_repo.git";
    const REPO_CLONE_HTTPS: &str = "https://github.com/pitoniak32/git_repo.git";

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

    // #[rstest]
    // fn should_clone_into_directory(temp_directory_fs: TempDir) -> Result<()> {
    //     // Arrange / Act
    //     let repo = GitRepo::from_ssh_uri(REPO_CLONE_SSH, &temp_directory_fs.path())
    //         .expect("should not fail");
    //
    //     // Assert
    //     assert_eq!(
    //         repo.remote_url,
    //         Some(REPO_CLONE_SSH.to_string())
    //     );
    //     assert!(Path::exists(&repo.root_path));
    //
    //     Ok(())
    // }
    //
    // #[rstest]
    // fn test_ssh_clone_git_repo(temp_directory_fs: TempDir) {
    //     // Act
    //     let repo = GitRepo::from_ssh_uri(REPO_CLONE_SSH, temp_directory_fs.path())
    //         .expect("should not fail");
    //
    //     // Assert
    //     assert_eq!(
    //         repo.remote_url,
    //         Some(REPO_CLONE_SSH.to_string())
    //     );
    //     assert!(Path::exists(&repo.root_path));
    // }

    #[rstest]
    fn test_https_clone_git_repo(temp_directory_fs: TempDir) {
        // Arrange / Act
        let repo =
            GitRepo::from_url(REPO_CLONE_HTTPS, temp_directory_fs.path()).expect("should not fail");

        // Assert
        assert_eq!(repo.remote_url, Some(REPO_CLONE_HTTPS.to_string()));
        assert!(Path::exists(&repo.root_path));
    }

    #[rstest]
    fn test_https_clone_multi_git_repo(temp_directory_fs: TempDir) {
        // Arrange
        let remote_urls = [
            REPO_CLONE_HTTPS,
            "https://github.com/pitoniak32/actions.git",
        ];

        // Act
        GitRepo::from_url_multi(&remote_urls, temp_directory_fs.path());

        // Assert
        assert!(Path::exists(&temp_directory_fs.path().join("git_repo")));
        assert!(Path::exists(&temp_directory_fs.path().join("actions")));
    }
}

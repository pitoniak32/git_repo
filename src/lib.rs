use anyhow::Result;
use std::{
    path::PathBuf,
    process::{Command, Output, Stdio},
};

pub struct GitRepo {
    root_path: PathBuf,
    ssh_uri: Option<String>,
}

impl GitRepo {
    pub fn from_ssh_uri(ssh_uri: &str, to_path: PathBuf) -> Result<GitRepo> {
        let _ = Git::clone(ssh_uri, to_path.clone());
        Ok(GitRepo {
            root_path: to_path,
            ssh_uri: Some(ssh_uri.to_string()),
        })
    }
}

pub struct Git;
impl Git {
    pub fn clone(uri: &str, to_path: PathBuf) -> Result<Output> {
        wrap_command(Command::new("git").args([
            "clone".to_string(),
            uri.to_string(),
            to_path.to_string_lossy().to_string(),
        ]))
    }
}

fn wrap_command(command: &mut Command) -> Result<Output> {
    let output = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?
        .wait_with_output()?;

    // Use log crate to allow verbosity flag to control wrapped command logs.
    if output.status.success() && !output.stdout.is_empty() {
        log::info!("{}", String::from_utf8_lossy(&output.stdout).trim());
    } else if !output.stderr.is_empty() {
        log::warn!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }

    Ok(output)
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

    fn teardown() -> Result<()> {
        fs::remove_dir_all(REPO_CLONE_DIR)?;
        Ok(())
    }

    fn run_test<T>(test: T) -> ()
    where
        T: FnOnce() -> () + UnwindSafe,
    {
        let result = catch_unwind(|| test());

        teardown().expect("teardown process should not fail");

        assert!(result.is_ok())
    }
}

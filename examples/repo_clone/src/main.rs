use anyhow::Result;
use git_lib::repo::GitRepo;
use std::path::PathBuf;

fn main() -> Result<()> {
    let cloned = GitRepo::from_ssh_uri_force(
        "git@github.com:pitoniak32/git_repo.git",
        &PathBuf::from("/tmp/git_repo"),
    )?;

    dbg!(cloned);

    let existing = GitRepo::from_existing(&PathBuf::from("/tmp/git_repo"))?;

    dbg!(existing);

    Ok(())
}

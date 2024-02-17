use anyhow::Result;
use git_lib::repo::GitRepo;
use std::path::PathBuf;

fn main() -> Result<()> {
    let urls = [
        "git@github.com:pitoniak32/git_repo.git",
        "git@github.com:pitoniak32/mukduk.git",
    ];

    let cloned = GitRepo::from_url_multi(&urls, &PathBuf::from("/tmp/tester"));

    dbg!(cloned);

    Ok(())
}

use anyhow::Result;
use assert_fs::{
    prelude::{PathChild, PathCreateDir},
    TempDir,
};
use git_lib::{git::Git, repo::GitRepo};

fn main() -> Result<()> {
    // If you would like to inspect the temp dir you can set this to true.
    // (YOU WILL BE RESPONISBLE FOR THE CLEANUP)
    let should_persist_temp_dir = false;

    // Setup for cloning into a temp directory.
    let tmp_dir = TempDir::new()
        .expect("temp dir should be created")
        .into_persistent_if(should_persist_temp_dir);

    // Add or remove clone urls.
    let urls = ["https://github.com/pitoniak32/git_repo.git"];

    let cloned_results = GitRepo::from_url_multi(&urls, tmp_dir.path());

    for result in cloned_results {
        let cloned = result?;

        dbg!(&cloned);
    }

    let existing_repo = GitRepo::from_existing(&tmp_dir.path().join("git_repo"))?;
    dbg!(&existing_repo);

    let new_repo_path = tmp_dir.child("new_repo");
    new_repo_path.create_dir_all()?;

    Git::init(&new_repo_path)?;

    Git::add_remote(
        "origin",
        "git@github.com:test_user/test_repo1",
        &new_repo_path,
    )?;

    let remote = Git::get_remote_url("origin", &new_repo_path)?;
    dbg!(&remote);

    let parsed_remote = Git::parse_uri(&remote.expect("should be some"))?;
    dbg!(&parsed_remote);

    // Explicitly cleaning the temp dir.
    // (It gets cleaned up when dropped, just being sure for this example)
    tmp_dir.close().expect("temp dir should be closed");

    Ok(())
}

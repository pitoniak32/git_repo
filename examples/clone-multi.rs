use anyhow::Result;
use assert_fs::TempDir;
use git_lib::repo::GitRepo;

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

    let cloned = GitRepo::from_url_multi(&urls, tmp_dir.path());

    dbg!(&cloned);

    // Explicitly cleaning the temp dir.
    // (It gets cleaned up when dropped, just being sure for this example)
    tmp_dir.close().expect("temp dir should be closed");

    Ok(())
}

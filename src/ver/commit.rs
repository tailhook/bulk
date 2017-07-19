use std::error::Error;
use std::path::Path;
use std::io::{stderr, Write};

use git2::{Repository, Status, Index};

use config::{Config};
use version::{Version};


pub fn check_status(cfg: &Config, dir: &Path)
    -> Result<Repository, Box<Error>>
{
    let repo = Repository::open(".")?;

    // check config
    let git_config = repo.config()?;
    git_config.get_entry("user.name")?;
    git_config.get_entry("user.email")?;

    // check status of files
    let mut ok = true;
    for item in &cfg.versions {
        for filename in item.file.iter().chain(&item.files) {
            let path = dir.join(filename);
            // git don't like ./ paths
            let git_path = path.strip_prefix(".").unwrap_or(&path);
            let status = repo.status_file(git_path)?;
            if status != Status::empty() {
                writeln!(&mut stderr(), "File {:?} is dirty", filename).ok();
                ok = false;
            }
        }
    }
    if !ok {
        return Err(format!("all files with version number must be unchanged \
            before bumping version").into());
    }

    Ok(repo)
}

pub fn commit_version(cfg: &Config, dir: &Path, repo: &mut Repository,
    ver: &Version<String>, dry_run: bool)
    -> Result<(), Box<Error>>
{
    let mut file_index = repo.index()?;
    // use temporary index so that we don't bother with things in user's index
    let mut index = Index::new()?;
    let head = repo.head()?;
    let head_oid = head.resolve()?.target()
        .ok_or(format!("can't resolve head"))?;
    let head_commit = repo.find_commit(head_oid)?;
    let head_tree = repo.find_tree(head_commit.tree_id())?;
    index.read_tree(&head_tree)?;
    repo.set_index(&mut index);
    for item in &cfg.versions {
        for filename in item.file.iter().chain(&item.files) {
            let path = dir.join(filename);
            // git don't like ./ paths
            let git_path = path.strip_prefix(".").unwrap_or(&path);
            index.add_path(&git_path)?;
        }
    }
    if !dry_run {
        let tree_oid = index.write_tree()?;
        let sig = repo.signature()?;
        let tree = repo.find_tree(tree_oid)?;
        let oid = repo.commit(Some("HEAD"), &sig, &sig,
            &format!("Version bumped to v{}", ver.num()),
            &tree, &[&head_commit])?;
        println!("Commited as {}", oid);
        let commit_ob = repo.find_object(oid, None)?;

        // then update user's index
        repo.set_index(&mut file_index);
        for item in &cfg.versions {
            for filename in item.file.iter().chain(&item.files) {
                let path = dir.join(filename);
                // git don't like ./ paths
                let git_path = path.strip_prefix(".").unwrap_or(&path);
                file_index.add_path(&git_path)?;
            }
        }
        file_index.write()?;

        repo.tag(&format!("v{}", ver.num()),
            &commit_ob, &sig,
            &format!("Version v{}", ver.num()),
            false)?;
        println!("Created tag v{}", ver.num());
        println!("To push tag run:");
        println!("  git push --atomic origin HEAD v{}", ver.num());
    }
    Ok(())
}

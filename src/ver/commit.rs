use std::collections::{VecDeque, HashMap};
use std::env;
use std::error::Error;
use std::io::{self, stderr, Write, BufWriter};
use std::io::{Seek, SeekFrom, BufReader, BufRead};
use std::path::Path;
use std::process::Command;

use git2::{Repository, Status, Index, Commit};
use tempfile::{NamedTempFile, NamedTempFileOptions};

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

fn message_file(repo: &Repository, ver: &Version<String>, commit: Commit)
    -> Result<NamedTempFile, Box<Error>>
{
    let mut file = NamedTempFileOptions::new()
        .suffix(".TAG_COMMIT")
        .create()?;
    {
        let mut buf = BufWriter::new(&mut file);
        writeln!(&mut buf, "Version v{}: ", ver.num())?;
        writeln!(&mut buf, "#")?;
        writeln!(&mut buf, "# Write a message for tag:")?;
        writeln!(&mut buf, "#   v{}", ver.num())?;
        writeln!(&mut buf, "# Lines starting with '#' will be ignored.")?;
        writeln!(&mut buf, "#")?;
        writeln!(&mut buf, "# Log:")?;

        let tag_names = repo.tag_names(Some("v*"))?;
        let tags = tag_names.iter()
            .filter_map(|name| name)
            .filter_map(|name|
                repo.refname_to_id(&format!("refs/tags/{}", name)).ok()
                .and_then(|oid| repo.find_tag(oid).ok())
                .map(|tag| (tag.target_id(), name)))
            .collect::<HashMap<_, _>>();
        println!("TAGS {:?}", tags);

        let mut queue = VecDeque::new();
        queue.push_back(commit);
        for _ in 0..100 {
            let commit = match queue.pop_front() {
                Some(x) => x,
                None => break,
            };
            let msg = commit.message()
                .and_then(|x| x.lines().next())
                .unwrap_or("<invalid message>");
            if let Some(tag_name) = tags.get(&commit.id()) {
                writeln!(&mut buf, "#   {:0.8} [tag: {}] {}",
                    commit.id(), tag_name, msg)?;
                break;
            } else {
                writeln!(&mut buf, "#   {:0.8} {}", commit.id(), msg)?;
            }
            for pid in commit.parent_ids() {
                queue.push_back(repo.find_commit(pid)?);
            }
        }
    }
    Ok(file)
}

fn spawn_editor(file_name: &Path) -> Result<(), Box<Error>> {
    if let Some(editor) = env::var_os("VISUAL") {
        let mut cmd = Command::new(editor);
        cmd.arg(file_name);
        match cmd.status() {
            Ok(s) if s.success() => return Ok(()),
            Ok(s) => return Err(format!("editor exited with {}", s).into()),
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
            }
            Err(e) => return Err(e.into()),
        }
    }
    if let Some(editor) = env::var_os("EDITOR") {
        let mut cmd = Command::new(editor);
        cmd.arg(file_name);
        match cmd.status() {
            Ok(s) if s.success() => return Ok(()),
            Ok(s) => return Err(format!("editor exited with {}", s).into()),
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
            }
            Err(e) => return Err(e.into()),
        }
    }
    let mut cmd = Command::new("vim");
    cmd.arg(file_name);
    match cmd.status() {
        Ok(s) if s.success() => return Ok(()),
        Ok(s) => return Err(format!("vim exited with {}", s).into()),
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
        }
        Err(e) => return Err(e.into()),
    }
    let mut cmd = Command::new("vi");
    cmd.arg(file_name);
    match cmd.status() {
        Ok(s) if s.success() => return Ok(()),
        Ok(s) => return Err(format!("vi exited with {}", s).into()),
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
        }
        Err(e) => return Err(e.into()),
    }
    let mut cmd = Command::new("nano");
    cmd.arg(file_name);
    match cmd.status() {
        Ok(s) if s.success() => return Ok(()),
        Ok(s) => return Err(format!("nano exited with {}", s).into()),
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
        }
        Err(e) => return Err(e.into()),
    }
    Err(format!("no editor found").into())
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

        let commit = repo.find_commit(oid)?;
        let mut message_file = message_file(repo, ver, commit)?;
        spawn_editor(message_file.path())?;
        message_file.seek(SeekFrom::Start(0))?;
        let mut message = String::with_capacity(512);
        for line in BufReader::new(message_file).lines() {
            let line = line?;
            if !line.starts_with("#") {
                message.push_str(line.trim_right());
                message.push('\n');
            }
        }
        if message.trim() == "" {
            return Err("tag description is empty, \
                aborting tag creation.".into())
        }

        repo.tag(&format!("v{}", ver.num()),
            &commit_ob, &sig,
            &message.trim(),
            false)?;
        println!("Created tag v{}", ver.num());
        println!("To push tag run:");
        println!("  git push --atomic origin HEAD v{}", ver.num());
    }
    Ok(())
}

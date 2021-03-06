use std::str;
use std::path::Path;
use std::process::{Command, Stdio, Output};
use std::sync::mpsc::Receiver;

use conf::{Conf, Project};
use error::{Reason, Result, Error};

type BytesResult = Result<Vec<u8>>;

pub struct Executor {
    conf: Conf,
    rx: Receiver<String>,
}

impl Executor {
    pub fn new(conf: Conf, rx: Receiver<String>) -> Executor {
        Executor {
            conf: conf,
            rx: rx,
        }
    }

    pub fn start(&self) {
        loop {
            match self.rx.recv() {
                Ok(repo) => self.run(&repo),
                Err(err) => error!("Error occurred while reading updates: {}", err),
            }
        }
    }

    pub fn run(&self, repo: &str) {
        match self.update_repo(repo) {
            Ok(_) => (),
            Err(err) => error!("Failed to update repository {}: {}", repo, err),
        }
    }

    fn update_repo(&self, repo: &str) -> Result<()> {
        let project = try!(self.get_project(repo));
        update_project(&self.conf.location, &self.conf.gitpath, project)
    }

    fn get_project(&self, repo: &str) -> Result<&Project> {
        self.conf
            .get_project(repo)
            .ok_or(Error::app(Reason::InvalidRepository, "No repository found"))
    }
}

fn update_project(location: &str, git: &str, project: &Project) -> Result<()> {
    let path_buf = Path::new(location).join(&project.id);
    let path = path_buf.as_path();

    let has_changed = try!(update_repo(git, &path, project));

    if has_changed {
        let _ = try!(run_from_str(&project.command, path));
        info!("Repository {} updated successfully", &project.repo);
        Ok(())
    } else {
        info!("No changes in repository. Skipping update command.");
        Ok(())
    }
}

fn update_repo(git: &str, path: &Path, project: &Project) -> Result<bool> {
    if path.exists() {
        info!("Local repo exists: updating");
        let _ = try!(git_checkout(git, path, &project.branch));
        let _ = try!(git_remote_update(git, path));
        let has_changed = try!(git_remote_changed(git, path));
        let _ = try!(git_pull(git, path));
        Ok(has_changed)
    } else {
        info!("No local repo found: cloning");
        let _ = try!(git_clone(git, path, &project.repo));
        let _ = try!(git_checkout(git, path, &project.branch));
        Ok(true)
    }
}

fn git_clone(git: &str, path: &Path, project: &str) -> BytesResult {
    let path_s = try!(path.to_str().ok_or(Error::app(Reason::InvalidPath, "Invalid project path")));
    info!("Cloning project {} to {}", project, path_s);
    run(Command::new(git)
            .arg("clone")
            .arg(github_url(project))
            .arg(path_s),
        "git clone")
}

fn github_url(project: &str) -> String {
    format!("https://github.com/{}.git", project)
}

fn git_checkout(git: &str, path: &Path, branch: &str) -> BytesResult {
    info!("Checking out branch {} in {}", branch, path_str(path));
    run(Command::new(git)
            .arg("checkout")
            .arg(branch)
            .current_dir(path),
        "git checkout")
}

fn path_str(path: &Path) -> &str {
    path.to_str().unwrap_or("[unprintable path]")
}

fn git_remote_update(git: &str, path: &Path) -> BytesResult {
    info!("Updating remotes in {}", path_str(path));
    run(Command::new(git)
            .arg("remote")
            .arg("update")
            .current_dir(path),
        "git remote update")
}

fn git_pull(git: &str, path: &Path) -> BytesResult {
    info!("Pulling changes in {}", path_str(path));
    run(Command::new(git).arg("pull").current_dir(path), "git pull")
}

fn git_remote_changed(git: &str, path: &Path) -> Result<bool> {
    let local = try!(run(Command::new(git)
                             .arg("rev-parse")
                             .arg("@")
                             .current_dir(path),
                         "git rev-parse"));
    let remote = try!(run(Command::new(git)
                              .arg("rev-parse")
                              .arg("@{u}")
                              .current_dir(path),
                          "git rev-parse"));
    Ok(local != remote)
}

fn run_from_str(command: &str, path: &Path) -> BytesResult {
    info!("Running update command {} in {}", command, path_str(path));
    run(Command::new(command).current_dir(path), command)
}

fn run(command: &mut Command, name: &str) -> BytesResult {
    command.stdin(Stdio::null())
           .output()
           .map_err(Error::from)
           .and_then(|out| non_zero_to_error(name, out))
}

fn non_zero_to_error(cmd: &str, out: Output) -> BytesResult {
    if out.status.success() {
        Ok(out.stdout)
    } else {
        Err(output_to_error(cmd, out))
    }
}

fn output_to_error(cmd: &str, out: Output) -> Error {
    let text = str::from_utf8(&out.stderr).unwrap_or("[invalid string]");
    let msg = format!("Command {} exited with status {}: {}",
                      cmd,
                      out.status,
                      text);
    Error::app(Reason::CommandFailed, msg)
}

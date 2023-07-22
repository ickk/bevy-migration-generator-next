use anyhow::bail;
use colored::Colorize;
use derivative::Derivative;
use git2::Repository;
use std::path::PathBuf;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct GitClient {
    #[derivative(Debug = "ignore")]
    _github_token: String,
    github_username: String,
    repo_local_path: PathBuf,
    repo_remote_url: String,
    #[derivative(Debug = "ignore")]
    repo: Repository,
}

impl GitClient {
    /// when `clone` is true, this attempts to clone from the remote if the
    /// repository doesn't exist
    pub fn new(
        github_token: String,
        github_username: String,
        local_path: PathBuf,
        remote_url: String,
        clone: bool,
    ) -> anyhow::Result<Self> {
        let repo = match Repository::open(local_path.clone()) {
            Ok(repo) => {
                println!("{}", "opened repository".green());
                repo
            }
            Err(_) => {
                let local_path_str = local_path.to_str().unwrap();
                if clone {
                    println!(
                        "{rep} {local_path_str} {notfound} {remote_url} {to} {local_path_str}",
                        rep = "repository".yellow(),
                        notfound = "not found.\nCloning".yellow(),
                        to = "to".yellow(),
                    );
                    match Repository::clone(&remote_url, &local_path) {
                        Ok(repo) => repo,
                        Err(e) => {
                            bail!(format!("failed to clone: {}", e).red())
                        }
                    }
                } else {
                    println!(
                        "{}",
                        "Try --clone if you would like to clone it from the remote".yellow()
                    );
                    bail!(format!("No git repository found at {local_path_str}").red())
                }
            }
        };

        Ok(GitClient {
            _github_token: github_token,
            github_username,
            repo_local_path: local_path,
            repo_remote_url: remote_url,
            repo,
        })
    }

    pub fn create_single_file_commit(
        &self,
        path: impl Into<PathBuf>,
        message: &str,
    ) -> anyhow::Result<()> {
        let mut index = self.repo.index()?;
        index.add_path(&path.into())?;
        let sig = self.repo.signature()?;
        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;
        let head = self.repo.head()?.peel_to_commit()?;
        self.repo
            .commit(Some("HEAD"), &sig, &sig, message, &tree, &[&head])?;

        Ok(())
    }
}

use anyhow::bail;
use colored::Colorize;
use config::Config;
use regex::Regex;
use std::{collections::HashMap, path::PathBuf};

const CONFIG_FILE: &str = "relgen.toml";
const ENV_CONFIG_PREFIX: &str = "RELGEN";
const SOURCE_REPO: &str = "source_repo";
const MIGRATION_NOTES_LOCAL_PATH: &str = "migration_notes_local_path";
const MIGRATION_NOTES_REPO: &str = "migration_notes_repo";
const PROJECT_PREFIX: &str = "project_prefix";

const CARGO_MANIFEST_DIR: &str = "CARGO_MANIFEST_DIR";

pub struct Settings {
    /// The github path for the source repo
    ///
    /// formated as "<organisation>/<repo>" or "<username>/<repo>".
    /// note that the domain prefix is not included.
    pub source_repo: String,

    /// The github path for the migration_notes repo
    ///
    /// formated as "<organisation>/<repo>" or "<username>/<repo>".
    /// note that the domain prefix is not included.
    pub migration_notes_repo: String,

    /// The local path to the migration_notes git repository
    ///
    /// Should be a canonicalised path on disk, i.e.
    /// "/home/username/projects/migration_notes"
    /// or "\\?C:\Users\username\projects\migration_notes"
    pub migration_notes_local_path: PathBuf,

    pub project_prefix: Option<String>,
}

pub fn get_settings() -> anyhow::Result<Settings> {
    let mut settings = {
        Config::builder()
            .add_source(config::File::with_name(CONFIG_FILE))
            .add_source(config::Environment::with_prefix(ENV_CONFIG_PREFIX))
            .build()?
            .try_deserialize::<HashMap<String, String>>()?
    };

    let Some(source_repo) = settings.remove(SOURCE_REPO) else {
        bail!(format!("No {SOURCE_REPO} specified").red());
    };
    let source_repo = clean_repo_str(source_repo)?;

    let Some(migration_notes_repo) =
        settings.remove(MIGRATION_NOTES_REPO) else {
            bail!(format!("No {MIGRATION_NOTES_REPO} specified").red());
        };
    let migration_notes_repo = clean_repo_str(migration_notes_repo)?;

    let Some(migration_notes_local_path) =
        settings.remove(MIGRATION_NOTES_LOCAL_PATH) else {
            bail!(format!("No {MIGRATION_NOTES_LOCAL_PATH} specified").red());
        };
    let migration_notes_local_path = path_from_str(&migration_notes_local_path)?;

    let project_prefix = settings.remove(PROJECT_PREFIX);
    if let Some(ref project_prefix) = project_prefix {
        let re = Regex::new(r"^[\w][\w.-]*$").unwrap();
        if !re.is_match(project_prefix) {
            bail!(format!("{PROJECT_PREFIX} contains is invalid").red());
        }
    }

    Ok(Settings {
        source_repo,
        migration_notes_repo,
        migration_notes_local_path,
        project_prefix,
    })
}

// Truncates the leading "github.com" and trailing slash from a string
// containing the url to a github repository
fn clean_repo_str(mut repo_str: String) -> anyhow::Result<String> {
    let github_regex = Regex::new(r"github.com/(?<repo>.+?)(.git)?/?$").unwrap();
    let repo_regex = Regex::new(r"^((?:[a-zA-Z])(?:[-]?[a-zA-Z\d])*[-]?)/([\w.-]+)$").unwrap();

    let mat = github_regex.captures(&repo_str);
    let repo_str = match mat {
        Some(mat) => mat.get(1).unwrap().as_str().to_owned(),
        None => {
            if repo_str.ends_with('/') {
                repo_str.pop();
            }
            if repo_str.ends_with(".git") {
                repo_str.truncate(repo_str.len() - 4);
            }
            repo_str
        }
    };

    if repo_regex.is_match(&repo_str) {
        Ok(repo_str)
    } else {
        bail!(format!("{repo_str} is not a valid github repository name").red())
    }
}

/// Get a PathBuf from a &str, and canonicalise it
///
/// A relative path is canonicalised relative to the CARGO_MANIFEST_DIR
fn path_from_str(path: &str) -> anyhow::Result<PathBuf> {
    let path = if Regex::new(r"^/|^[A-Z]:[/\\]").unwrap().is_match(path) {
        PathBuf::from(path)
    } else {
        let Ok(manifest_dir) = &std::env::var(CARGO_MANIFEST_DIR) else {
            bail!(format!("{CARGO_MANIFEST_DIR} not found").red())
        };
        [manifest_dir, path].iter().collect()
    };

    // We can't call canonicalize on directories that don't exist, so just
    // canonicalise as much as we can.
    let mut p = PathBuf::new();
    for parent in path.ancestors().collect::<Vec<_>>().iter().rev() {
        match parent.canonicalize() {
            Ok(_) => {
                p = parent.to_path_buf();
                continue;
            }
            Err(_) => break,
        }
    }
    let tail = path.strip_prefix(&p)?;
    p = p.canonicalize()?;
    p.push(tail);

    Ok(p)
}

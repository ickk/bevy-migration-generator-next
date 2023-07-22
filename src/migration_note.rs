use crate::{
    get_merged_prs, get_settings, git_client, github_client, helpers::get_pr_areas, markdown,
};
use anyhow::bail;
use colored::Colorize;
use regex::Regex;
use std::{fs::File, io::Write, path::PathBuf};

const GITHUB_TOKEN: &str = "GITHUB_TOKEN";
const GITHUB_USERNAME: &str = "GITHUB_USERNAME";

pub fn migration_notes_command(
    release: String,
    create_release: bool,
    no_create_commit: bool,
    clone: bool,
    from: String,
    to: String,
) -> anyhow::Result<()> {
    let settings = get_settings()?;

    let client = github_client::GithubClient::new(
        std::env::var(GITHUB_TOKEN).expect("GITHUB_TOKEN not found"),
        settings.source_repo.clone(),
    );

    let migration_notes_repo = format!("https://github.com/{}", settings.migration_notes_repo);
    let git_client = git_client::GitClient::new(
        std::env::var(GITHUB_TOKEN).expect("GITHUB_TOKEN not found"),
        std::env::var(GITHUB_USERNAME).expect("GITHUB_USERNAME not found"),
        settings.migration_notes_local_path.clone(),
        migration_notes_repo,
        clone,
    )?;

    let base_path = settings.migration_notes_local_path;
    let mut release_path = base_path.clone();
    if let Some(ref project_prefix) = settings.project_prefix {
        release_path.push(project_prefix);
    }
    release_path.push(&release);
    if !release_path.exists() {
        if create_release {
            std::fs::create_dir(release_path)?;
        } else {
            bail!(format!(
                "The release folder {path} was not found. Use --create-release to create it.",
                path = release_path.to_str().unwrap()
            )
            .red());
        }
    }

    let mut release_path = PathBuf::new();
    if let Some(ref project_prefix) = settings.project_prefix {
        release_path.push(project_prefix);
    }
    let re = Regex::new(r"^[\w][\w.-]*$").unwrap();
    if !re.is_match(&release) {
        bail!(format!(r#"The release name "{release}" is invalid"#).red());
    }
    release_path.push(&release);

    for (pr, _, _) in get_merged_prs(&client, &from, &to, Some("C-Breaking-Change"))? {
        println!("creating note for #{}", pr.number);
        create_migration_note_file(
            &pr,
            &settings.source_repo,
            &base_path,
            &release_path,
            !no_create_commit,
            Some(&git_client),
        )?
    }

    Ok(())
}

fn create_migration_note_file(
    pr: &github_client::GithubIssuesResponse,
    source_repo_name: &str,
    base_path: impl Into<PathBuf>,
    release_path: impl Into<PathBuf>,
    create_commit: bool,
    git_client: Option<&git_client::GitClient>,
) -> anyhow::Result<()> {
    let mut repo_path: PathBuf = release_path.into();
    repo_path.push(format!("{}.md", pr.number));

    let mut absolute_path: PathBuf = base_path.into();
    absolute_path.push(&repo_path);

    let mut file = File::create(&absolute_path)?;
    write_migration_note(pr, &mut file)?;

    if create_commit {
        let git_client = git_client.expect("Can't create commit when git client is none");
        let mut message = format!(
            "Create migration note for {source_repo_name}#{pr_number}",
            pr_number = pr.number
        );
        message.push_str(&format!(
            "\n\nCo-authored-by: {} <{}+{}@users.noreply.github.com>",
            pr.user.login,
            pr.user.id,
            pr.user.login.to_lowercase()
        ));

        git_client.create_single_file_commit(&repo_path, &message)?;
    }

    Ok(())
}

fn write_migration_note(
    pr: &github_client::GithubIssuesResponse,
    file: &mut impl Write,
) -> anyhow::Result<()> {
    // write front-matter
    writeln!(file, r#"+++"#)?;
    writeln!(file, r#"pr = {}"#, pr.number)?;
    writeln!(file, r#"title = "{}""#, pr.title)?;
    writeln!(file, r#"close_date = "{}""#, pr.closed_at)?;
    let areas = get_pr_areas(pr);
    if !areas.is_empty() {
        writeln!(
            file,
            r#"areas = [{areas}]""#,
            areas = areas
                .iter()
                .map(|s| format!(r#""{s}""#))
                .collect::<Vec<_>>()
                .join(", ")
        )?;
    }
    writeln!(file, r#"+++"#)?;
    // write body
    markdown::write_markdown_section(pr.body.as_ref().unwrap(), "migration guide", file, true)?;
    Ok(())
}

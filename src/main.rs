mod git_client;
mod github_client;
mod helpers;
mod markdown;
mod migration_note;
mod settings;

use crate::helpers::get_merged_prs;
use crate::migration_note::migration_notes_command;
use crate::settings::get_settings;
use clap::{Parser as ClapParser, Subcommand};

/// Generates markdown files used for a bevy releases.
///
/// Migration Guide:
/// * Gets all PRs with the `C-Breaking-Change` label and that were merged by bors.
/// * For each PR:
///     * Generate the title with a link to the relevant PR and
///     * Generate the migration guide section. This parses the markdown and generates valid makrdown that should pass markdownlint rules.
///
/// Release notes:
/// * Gets all PRs merged by bors
/// * Collect each author of closed PRs (Should this just list all contributors?)
/// * Sort each PR per area label
/// * Generate the list of merge PR
///
/// Requires a valid GITHUB_TOKEN environment variable, you can use a .env file or use your prefered method of passing env arguments.
///
/// Example used to generate for 0.9:
/// cargo run -- migration-guide --from v0.9.0 --to main --title "0.9 to 0.10" --weight 6
/// cargo run -- release-note --from v0.9.0 --to main
/// cargo run -- release-note-website --from bd4f611f7576c55739b466c6f0039e8421dab57e --to HEAD
#[derive(ClapParser)]
#[command(author, version, about)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    MigrationNotes {
        /// release-name
        #[arg(short, long)]
        release: String,

        /// The name of the branch / tag to start from
        #[arg(long)]
        from: String,

        /// The name of the branch / tag to end on
        #[arg(long)]
        to: String,

        /// Create the release folder in the migration_notes repository if it
        /// doesn't already exist
        #[arg(long)]
        create_release: bool,

        /// Clone the migration_notes repository if it doesn't already exist
        #[arg(long)]
        clone: bool,

        #[arg(long)]
        no_create_commit: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    let args = Args::parse();

    match args.command {
        Commands::MigrationNotes {
            release,
            create_release,
            no_create_commit,
            clone,
            from,
            to,
        } => migration_notes_command(release, create_release, no_create_commit, clone, from, to),
    }?;

    Ok(())
}

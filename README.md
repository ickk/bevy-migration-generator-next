# Release Generator

A semi-automatic migration note generator tool for the bevy project.

Original code adapted from `bevy-website/generate-release`

## Setup

You can set up a Github Personal-Access-Token in your user settings
https://github.com/settings/tokens

You can either specify your PAT & github username as environment variables, or
by creating a `.env` file
```sh
GITHUB_USERNAME = "username"
GITHUB_TOKEN = "PAT"
```

Then complete the `relgen.toml` file with the following information
```toml
source_repo = "https://github.com/bevyengine/bevy"
migration_notes_repo = "https://github.com/ickk/migration_notes"
migration_notes_local_path = "./migration_notes"
```

## Usage

```
cargo run -- migration_notes --release 0.11 --from v0.11.0 --to main
```

Git tags, SHAs or other references can be specified in the `--from` & `--to`
arguments. The value of `--release` specifies the subfolder where the notes
will be created.

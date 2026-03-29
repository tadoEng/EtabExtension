use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use ext_api::AppContext;
use ext_api::checkout::{CheckoutConflict, CheckoutConflictResolution};
use ext_api::stash::StashPopConflict;

use crate::args::Cli;
use crate::output::OutputFormat;

pub mod branch;
pub mod checkout;
pub mod commit;
pub mod diff;
pub mod init;
pub mod log;
pub mod show;
pub mod stash;
pub mod status;
pub mod switch;

pub(crate) fn to_absolute(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

pub(crate) fn ctx_from(global_project_path: Option<&PathBuf>) -> Result<AppContext> {
    if let Some(path) = global_project_path {
        AppContext::new(&to_absolute(path)?)
    } else {
        AppContext::from_cwd()
    }
}

pub(crate) fn prompt_line(prompt: &str) -> Result<String> {
    print!("{prompt}");
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

pub(crate) fn prompt_onedrive_continue() -> Result<bool> {
    println!("⚠ OneDrive path detected");
    println!("  ETABS model files can be corrupted by sync conflicts.");
    println!("  [c] Continue anyway  [x] Cancel");
    Ok(prompt_line("> ")?.eq_ignore_ascii_case("c"))
}

pub(crate) fn prompt_checkout_conflict(
    conflict: &CheckoutConflict,
) -> Result<Option<CheckoutConflictResolution>> {
    println!("{conflict}");
    if conflict.stash_exists {
        println!("⚠ A stash already exists for this branch and would need overwrite later.");
    }
    let choice = prompt_line("> ")?;
    match choice.to_ascii_lowercase().as_str() {
        "c" => {
            let message = prompt_line("Commit message: ")?;
            if message.is_empty() {
                bail!("Commit message cannot be empty");
            }
            Ok(Some(CheckoutConflictResolution::CommitFirst { message }))
        }
        "s" => Ok(Some(CheckoutConflictResolution::Stash)),
        "d" => Ok(Some(CheckoutConflictResolution::Discard)),
        "x" => Ok(None),
        _ => bail!("Invalid choice: {choice}"),
    }
}

pub(crate) fn prompt_stash_overwrite() -> Result<bool> {
    println!("⚠ Stash already exists for this branch");
    println!("  [o] Overwrite  [x] Cancel");
    Ok(prompt_line("> ")?.eq_ignore_ascii_case("o"))
}

pub(crate) fn prompt_stash_pop_conflict(conflict: &StashPopConflict) -> Result<bool> {
    println!("{conflict}");
    Ok(prompt_line("> ")?.eq_ignore_ascii_case("o"))
}

pub fn output_format(cli: &Cli) -> OutputFormat {
    if cli.json {
        OutputFormat::Json
    } else if cli.shell {
        OutputFormat::Shell
    } else {
        OutputFormat::Human
    }
}

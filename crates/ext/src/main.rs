use anyhow::{Context, Result};
use clap::Parser;

mod args;
mod commands;
mod output;

use args::{Cli, Command, EtabsSubcommand};
use output::OutputChannel;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let out = OutputChannel::new(commands::output_format(&cli));

    match cli.command {
        Command::Init(args) => commands::init::execute(&out, cli.project_path.as_ref(), args).await,
        Command::Status(args) => {
            commands::status::execute(&out, cli.project_path.as_ref(), args).await
        }
        Command::Commit(args) => {
            commands::commit::execute(&out, cli.project_path.as_ref(), args).await
        }
        Command::Analyze(args) => {
            commands::analyze::execute(&out, cli.project_path.as_ref(), args).await
        }
        Command::Log(args) => commands::log::execute(&out, cli.project_path.as_ref(), args).await,
        Command::Show(args) => commands::show::execute(&out, cli.project_path.as_ref(), args).await,
        Command::Branch(args) => {
            commands::branch::execute(&out, cli.project_path.as_ref(), args).await
        }
        Command::Switch(args) => {
            commands::switch::execute(&out, cli.project_path.as_ref(), args).await
        }
        Command::Checkout(args) => {
            commands::checkout::execute(&out, cli.project_path.as_ref(), args).await
        }
        Command::Stash(args) => {
            commands::stash::execute(&out, cli.project_path.as_ref(), args).await
        }
        Command::Diff(args) => commands::diff::execute(&out, cli.project_path.as_ref(), args).await,
        Command::Etabs(args) => match args.command {
            EtabsSubcommand::Open(args) => {
                commands::etabs_open::execute(&out, cli.project_path.as_ref(), args).await
            }
            EtabsSubcommand::Close(args) => {
                commands::etabs_close::execute(&out, cli.project_path.as_ref(), args).await
            }
            EtabsSubcommand::Status => {
                commands::etabs_status::execute(&out, cli.project_path.as_ref()).await
            }
            EtabsSubcommand::Unlock => {
                commands::etabs_unlock::execute(&out, cli.project_path.as_ref()).await
            }
            EtabsSubcommand::Recover => {
                commands::etabs_recover::execute(&out, cli.project_path.as_ref()).await
            }
        },
    }
    .with_context(|| "Command failed".to_string())
}

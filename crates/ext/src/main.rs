// ext — CLI binary
//
// RULE (agents.md): every command handler must be under 20 lines.
// Zero business logic lives here. All operations go through ext-api.
//
// Structure:
//   parse args (clap)
//   build AppContext
//   call ext-api::<command>()
//   format output via OutputChannel

mod args;
mod commands;
mod utils;
mod tui;
mod setup;
mod trace;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    // TODO Week 1: wire up clap subcommands, tracing init, AppContext construction
    // For now: verify the binary compiles.
    println!("ext — ETABS extension CLI (stub)");
    println!("Run `ext --help` once commands are wired up.");
    Ok(())
}

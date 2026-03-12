// ext — CLI binary
//
// RULE (agents.md): every command handler must be under 20 lines.
// Zero business logic lives here. All operations go through ext-api.
//
// Current bootstrap stage:
//   keep the binary minimal until command wiring is implemented.
//   Legacy modules from the previous scaffold are
//   intentionally not compiled right now.

use anyhow::Result;

fn main() -> Result<()> {
    // TODO Week 1: wire up clap subcommands, tracing init, AppContext construction
    // For now: verify the binary compiles.
    println!("ext — ETABS extension CLI (stub)");
    println!("Run `ext --help` once commands are wired up.");
    Ok(())
}

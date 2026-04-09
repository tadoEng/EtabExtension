use anyhow::Result;
use ext_api::{get_config, list_config, set_config};

use crate::args::{ConfigArgs, ConfigSubcommand};
use crate::output::OutputChannel;

use super::ctx_from;

pub async fn execute(
    out: &OutputChannel,
    global_project_path: Option<&std::path::PathBuf>,
    args: ConfigArgs,
) -> Result<()> {
    let ctx = ctx_from(global_project_path)?;

    match args.command {
        ConfigSubcommand::List => {
            let result = list_config(&ctx)?;
            if out.is_human() {
                for entry in &result.entries {
                    println!(
                        "[{}] {} = {}",
                        entry.scope,
                        entry.key,
                        format_value(&entry.value)
                    );
                }
            } else if out.is_shell() {
                for entry in &result.entries {
                    println!("{}", entry.key);
                }
            }
            out.json_value(&result)?;
        }
        ConfigSubcommand::Get(args) => {
            let entry = get_config(&ctx, &args.key)?;
            if out.is_human() {
                println!(
                    "[{}] {} = {}",
                    entry.scope,
                    entry.key,
                    format_value(&entry.value)
                );
            } else if out.is_shell() {
                println!("{}", format_value(&entry.value));
            }
            out.json_value(&entry)?;
        }
        ConfigSubcommand::Set(args) => {
            let result = set_config(&ctx, &args.key, &args.value)?;
            if out.is_human() {
                println!(
                    "✓ Updated {} config: {} = {}",
                    result.entry.scope,
                    result.entry.key,
                    format_value(&result.entry.value)
                );
            } else if out.is_shell() {
                println!("{}", format_value(&result.entry.value));
            }
            out.json_value(&result)?;
        }
    }

    Ok(())
}

fn format_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(text) => format!("\"{text}\""),
        _ => value.to_string(),
    }
}

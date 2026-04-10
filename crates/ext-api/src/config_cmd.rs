use anyhow::{Context, Result, bail};
use ext_db::config::Config;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::path::PathBuf;

use crate::context::AppContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigEntry {
    pub key: String,
    pub scope: String,
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigListResult {
    pub entries: Vec<ConfigEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigSetResult {
    pub entry: ConfigEntry,
    pub shared_path: PathBuf,
    pub local_path: PathBuf,
}

pub fn list_config(ctx: &AppContext) -> Result<ConfigListResult> {
    let config = Config::load(&ctx.project_root)?;
    let schema = config_schema()?;
    let current = serde_json::to_value(&config).context("Serialize current config")?;
    let mut entries = Vec::new();
    flatten_entries(&schema, &current, None, &mut entries)?;
    entries.sort_by(|left, right| left.key.cmp(&right.key));
    Ok(ConfigListResult { entries })
}

pub fn get_config(ctx: &AppContext, key: &str) -> Result<ConfigEntry> {
    let config = Config::load(&ctx.project_root)?;
    let schema = config_schema()?;
    let canonical_path = resolve_path(&schema, key)?;
    let current = serde_json::to_value(&config).context("Serialize current config")?;
    let value = get_path_value(&current, &canonical_path)?
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Unknown config key: {key}"))?;

    Ok(ConfigEntry {
        key: display_key(&canonical_path),
        scope: scope_for_path(&canonical_path).to_string(),
        value,
    })
}

pub fn set_config(ctx: &AppContext, key: &str, raw_value: &str) -> Result<ConfigSetResult> {
    let mut config = Config::load(&ctx.project_root)?;
    let schema = config_schema()?;
    let canonical_path = resolve_path(&schema, key)?;
    let parsed_value = parse_cli_value(raw_value)?;
    let mut current = serde_json::to_value(&config).context("Serialize current config")?;

    set_path_value(&mut current, &canonical_path, parsed_value)?;
    config = serde_json::from_value(current).context("Deserialize updated config")?;

    match scope_for_path(&canonical_path) {
        "shared" => Config::write_shared(&ctx.project_root, &config)?,
        "local" => Config::write_local(&ctx.project_root, &config)?,
        other => bail!("Unknown config scope '{other}'"),
    }

    let entry = get_config(ctx, &canonical_path.join("."))?;
    let config_dir = Config::config_dir(&ctx.project_root);
    Ok(ConfigSetResult {
        entry,
        shared_path: config_dir.join(ext_db::config::CONFIG_FILE),
        local_path: config_dir.join(ext_db::config::CONFIG_LOCAL_FILE),
    })
}

fn config_schema() -> Result<Value> {
    serde_json::to_value(Config::default()).context("Serialize config schema")
}

fn flatten_entries(
    schema: &Value,
    current: &Value,
    prefix: Option<&str>,
    entries: &mut Vec<ConfigEntry>,
) -> Result<()> {
    match schema {
        Value::Object(object) => {
            for (key, nested_schema) in object {
                let next_prefix = match prefix {
                    Some(prefix) => format!("{prefix}.{key}"),
                    None => key.clone(),
                };
                let nested_current = match current {
                    Value::Object(current_object) => {
                        current_object.get(key).unwrap_or(&Value::Null)
                    }
                    _ => &Value::Null,
                };
                flatten_entries(nested_schema, nested_current, Some(&next_prefix), entries)?;
            }
        }
        leaf => {
            let key = prefix.unwrap_or_default().to_string();
            entries.push(ConfigEntry {
                key: display_key(
                    &key.split('.')
                        .map(|segment| segment.to_string())
                        .collect::<Vec<_>>(),
                ),
                scope: scope_for_key(&key).to_string(),
                value: if current.is_null() {
                    leaf.clone()
                } else {
                    current.clone()
                },
            });
        }
    }
    Ok(())
}

fn resolve_path(schema: &Value, raw_key: &str) -> Result<Vec<String>> {
    let mut current = schema;
    let mut resolved = Vec::new();

    for segment in raw_key.split('.') {
        let object = current
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("Config key '{raw_key}' descends into a non-object"))?;
        let actual = resolve_segment(object, segment).ok_or_else(|| {
            anyhow::anyhow!("Unknown config key segment '{segment}' in '{raw_key}'")
        })?;
        resolved.push(actual.clone());
        current = object
            .get(&actual)
            .ok_or_else(|| anyhow::anyhow!("Unknown config key: {raw_key}"))?;
    }

    Ok(resolved)
}

fn resolve_segment(object: &Map<String, Value>, segment: &str) -> Option<String> {
    let normalized = normalize_segment(segment);
    object
        .keys()
        .find(|candidate| normalize_segment(candidate) == normalized)
        .cloned()
}

fn normalize_segment(segment: &str) -> String {
    segment
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

fn display_key(path: &[String]) -> String {
    path.iter()
        .map(|segment| display_segment(segment))
        .collect::<Vec<_>>()
        .join(".")
}

fn display_segment(segment: &str) -> String {
    let mut rendered = String::new();
    let mut previous_was_lower_or_digit = false;

    for ch in segment.chars() {
        if ch == '_' || ch == '-' {
            if !rendered.ends_with('-') {
                rendered.push('-');
            }
            previous_was_lower_or_digit = false;
            continue;
        }

        if ch.is_ascii_uppercase() && previous_was_lower_or_digit && !rendered.ends_with('-') {
            rendered.push('-');
        }

        rendered.push(ch.to_ascii_lowercase());
        previous_was_lower_or_digit = ch.is_ascii_lowercase() || ch.is_ascii_digit();
    }

    rendered
}

fn scope_for_key(key: &str) -> &'static str {
    let path = key.split('.').collect::<Vec<_>>();
    scope_for_path(&path.iter().map(|part| part.to_string()).collect::<Vec<_>>())
}

fn scope_for_path(path: &[String]) -> &'static str {
    let Some(root) = path.first().map(|segment| normalize_segment(segment)) else {
        return "shared";
    };

    match root.as_str() {
        "extract" | "calc" => "shared",
        "llm" | "git" | "paths" | "onedrive" => "local",
        "project" => match path.get(1).map(|segment| normalize_segment(segment)) {
            Some(ref segment) if segment == "name" => "shared",
            _ => "local",
        },
        _ => "shared",
    }
}

fn parse_cli_value(raw: &str) -> Result<Value> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(Value::String(String::new()));
    }

    let wrapped = format!("value = {trimmed}");
    if let Ok(table) = toml::from_str::<toml::Table>(&wrapped) {
        if let Some(value) = table.get("value") {
            return toml_to_json(value);
        }
    }

    Ok(Value::String(trimmed.to_string()))
}

fn toml_to_json(value: &toml::Value) -> Result<Value> {
    Ok(match value {
        toml::Value::String(text) => Value::String(text.clone()),
        toml::Value::Integer(number) => Value::Number((*number).into()),
        toml::Value::Float(number) => serde_json::Number::from_f64(*number)
            .map(Value::Number)
            .ok_or_else(|| anyhow::anyhow!("Invalid floating-point config value"))?,
        toml::Value::Boolean(flag) => Value::Bool(*flag),
        toml::Value::Datetime(datetime) => Value::String(datetime.to_string()),
        toml::Value::Array(array) => {
            Value::Array(array.iter().map(toml_to_json).collect::<Result<Vec<_>>>()?)
        }
        toml::Value::Table(table) => Value::Object(
            table
                .iter()
                .map(|(key, value)| Ok((key.clone(), toml_to_json(value)?)))
                .collect::<Result<Map<String, Value>>>()?,
        ),
    })
}

fn get_path_value<'a>(value: &'a Value, path: &[String]) -> Result<Option<&'a Value>> {
    let mut current = value;
    for segment in path {
        let object = match current {
            Value::Object(object) => object,
            _ => bail!("Config key '{}' descends into a non-object", path.join(".")),
        };
        current = object.get(segment).unwrap_or(&Value::Null);
    }
    Ok(Some(current))
}

fn set_path_value(value: &mut Value, path: &[String], replacement: Value) -> Result<()> {
    let mut current = value;
    for segment in &path[..path.len().saturating_sub(1)] {
        let object = current.as_object_mut().ok_or_else(|| {
            anyhow::anyhow!("Config key '{}' descends into a non-object", path.join("."))
        })?;
        current = object
            .get_mut(segment)
            .ok_or_else(|| anyhow::anyhow!("Unknown config key '{}'", path.join(".")))?;
    }

    let object = current.as_object_mut().ok_or_else(|| {
        anyhow::anyhow!("Config key '{}' descends into a non-object", path.join("."))
    })?;
    let leaf = path
        .last()
        .ok_or_else(|| anyhow::anyhow!("Config key cannot be empty"))?;
    object.insert(leaf.clone(), replacement);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::AppContext;
    use ext_db::config::{CONFIG_DIR, CONFIG_FILE, CONFIG_LOCAL_FILE};
    use tempfile::tempdir;

    fn test_ctx() -> (tempfile::TempDir, AppContext) {
        let dir = tempdir().unwrap();
        let ctx = AppContext::for_test(dir.path().to_path_buf(), Config::default());
        (dir, ctx)
    }

    #[test]
    fn list_config_includes_shared_and_local_keys() {
        let (_dir, ctx) = test_ctx();
        let result = list_config(&ctx).unwrap();

        assert!(
            result
                .entries
                .iter()
                .any(|entry| entry.key == "project.name")
        );
        assert!(
            result
                .entries
                .iter()
                .any(|entry| entry.key == "project.sidecar-path")
        );
        assert!(
            result
                .entries
                .iter()
                .any(|entry| entry.key == "extract.units")
        );
        assert!(result.entries.iter().any(|entry| entry.key == "calc.code"));
        assert!(
            result
                .entries
                .iter()
                .any(|entry| entry.key == "paths.one-drive-dir")
        );
    }

    #[test]
    fn set_config_routes_shared_and_local_values_to_separate_files() {
        let (dir, ctx) = test_ctx();

        let shared = set_config(&ctx, "project.name", "\"Proof Tower\"").unwrap();
        assert_eq!(shared.entry.scope, "shared");
        assert_eq!(shared.entry.value, Value::String("Proof Tower".to_string()));

        let local = set_config(&ctx, "project.sidecar-path", "\"C:/tools/etab-cli.exe\"").unwrap();
        assert_eq!(local.entry.scope, "local");
        assert_eq!(
            local.entry.value,
            Value::String("C:/tools/etab-cli.exe".to_string())
        );

        let config_dir = dir.path().join(CONFIG_DIR);
        let shared_text = std::fs::read_to_string(config_dir.join(CONFIG_FILE)).unwrap();
        let local_text = std::fs::read_to_string(config_dir.join(CONFIG_LOCAL_FILE)).unwrap();

        assert!(shared_text.contains("name = \"Proof Tower\""));
        assert!(!shared_text.contains("sidecar-path"));
        assert!(local_text.contains("sidecar-path = \"C:/tools/etab-cli.exe\""));
        assert!(!local_text.contains("name = \"Proof Tower\""));
    }

    #[test]
    fn get_config_accepts_alias_style_keys() {
        let (_dir, ctx) = test_ctx();
        set_config(&ctx, "paths.oneDriveDir", "\"D:/Reports\"").unwrap();

        let entry = get_config(&ctx, "paths.one-drive-dir").unwrap();
        assert_eq!(entry.scope, "local");
        assert_eq!(entry.key, "paths.one-drive-dir");
        assert_eq!(entry.value, Value::String("D:/Reports".to_string()));
    }
}

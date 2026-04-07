use anyhow::{Context, Result, anyhow};
use polars::prelude::{AnyValue, Column};

pub mod base_reactions;
pub mod group_assignments;
pub mod joint_drift;
pub mod material_by_story;
pub mod material_props;
pub mod modal;
pub mod pier_forces;
pub mod pier_section;
pub mod story_def;
pub mod story_forces;

pub(crate) fn required_string(column: &Column, idx: usize, name: &str) -> Result<String> {
    let value = column
        .get(idx)
        .with_context(|| format!("Failed to read {name} at row {idx}"))?;

    match value {
        AnyValue::Null => Err(anyhow!("Missing {name} at row {idx}")),
        AnyValue::String(v) => Ok(v.trim().to_string()),
        AnyValue::StringOwned(v) => Ok(v.as_str().trim().to_string()),
        _ => Ok(value.to_string()),
    }
}

pub(crate) fn required_f64(column: &Column, idx: usize, name: &str) -> Result<f64> {
    let value = column
        .get(idx)
        .with_context(|| format!("Failed to read {name} at row {idx}"))?;

    match value {
        AnyValue::Null => Err(anyhow!("Missing {name} at row {idx}")),
        AnyValue::Float64(v) => Ok(v),
        AnyValue::Float32(v) => Ok(v as f64),
        AnyValue::Int8(v) => Ok(v as f64),
        AnyValue::Int16(v) => Ok(v as f64),
        AnyValue::Int32(v) => Ok(v as f64),
        AnyValue::Int64(v) => Ok(v as f64),
        AnyValue::UInt8(v) => Ok(v as f64),
        AnyValue::UInt16(v) => Ok(v as f64),
        AnyValue::UInt32(v) => Ok(v as f64),
        AnyValue::UInt64(v) => Ok(v as f64),
        AnyValue::String(v) => parse_f64(v, idx, name),
        AnyValue::StringOwned(v) => parse_f64(v.as_str(), idx, name),
        _ => parse_f64(&value.to_string(), idx, name),
    }
}

pub(crate) fn optional_f64(column: &Column, idx: usize, name: &str) -> Result<Option<f64>> {
    let value = column
        .get(idx)
        .with_context(|| format!("Failed to read {name} at row {idx}"))?;

    match value {
        AnyValue::Null => Ok(None),
        AnyValue::String(v) if v.trim().is_empty() => Ok(None),
        AnyValue::StringOwned(v) if v.as_str().trim().is_empty() => Ok(None),
        _ => required_f64(column, idx, name).map(Some),
    }
}

pub(crate) fn required_i64(column: &Column, idx: usize, name: &str) -> Result<i64> {
    let value = column
        .get(idx)
        .with_context(|| format!("Failed to read {name} at row {idx}"))?;

    match value {
        AnyValue::Null => Err(anyhow!("Missing {name} at row {idx}")),
        AnyValue::Int8(v) => Ok(i64::from(v)),
        AnyValue::Int16(v) => Ok(i64::from(v)),
        AnyValue::Int32(v) => Ok(i64::from(v)),
        AnyValue::Int64(v) => Ok(v),
        AnyValue::UInt8(v) => Ok(i64::from(v)),
        AnyValue::UInt16(v) => Ok(i64::from(v)),
        AnyValue::UInt32(v) => Ok(i64::from(v)),
        AnyValue::UInt64(v) => i64::try_from(v)
            .with_context(|| format!("Value too large for {name} at row {idx}: {v}")),
        AnyValue::Float32(v) => parse_i64(&v.to_string(), idx, name),
        AnyValue::Float64(v) => parse_i64(&v.to_string(), idx, name),
        AnyValue::String(v) => parse_i64(v, idx, name),
        AnyValue::StringOwned(v) => parse_i64(v.as_str(), idx, name),
        _ => parse_i64(&value.to_string(), idx, name),
    }
}

fn parse_f64(raw: &str, idx: usize, name: &str) -> Result<f64> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("Missing {name} at row {idx}"));
    }

    trimmed
        .parse::<f64>()
        .with_context(|| format!("Failed to parse {name} as f64 at row {idx}: {trimmed}"))
}

fn parse_i64(raw: &str, idx: usize, name: &str) -> Result<i64> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("Missing {name} at row {idx}"));
    }

    if let Ok(value) = trimmed.parse::<i64>() {
        return Ok(value);
    }

    let value = trimmed
        .parse::<f64>()
        .with_context(|| format!("Failed to parse {name} as i64 at row {idx}: {trimmed}"))?;
    if value.fract() == 0.0 {
        Ok(value as i64)
    } else {
        Err(anyhow!(
            "Non-integer value for {name} at row {idx}: {trimmed}"
        ))
    }
}

pub(super) fn fmt_float(value: f64) -> String {
    format!("{value:.3}")
}

pub(super) fn fmt_percent(value: f64) -> String {
    format!("{:.1}%", value * 100.0)
}

pub(super) fn fmt_with_unit(value: f64, unit: &str) -> String {
    format!("{} {}", fmt_float(value), unit)
}

pub(super) fn wrap_load_case_label(value: &str) -> String {
    const SOFT_WRAP: char = '\u{200B}';
    let mut out = String::with_capacity(value.len() + 16);
    for ch in value.chars() {
        out.push(ch);
        if matches!(ch, '_' | '+' | '-' | '/' | ':' | ')') {
            out.push(SOFT_WRAP);
        }
    }
    out
}

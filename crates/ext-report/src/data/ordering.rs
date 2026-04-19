pub(super) fn ordered_unique(iter: impl Iterator<Item = String>) -> Vec<String> {
    let mut out = Vec::new();
    for value in iter {
        if !out.contains(&value) {
            out.push(value);
        }
    }
    out
}

pub(super) fn is_default_pier_label(label: &str) -> bool {
    let trimmed = label.trim();
    trimmed.is_empty() || trimmed == "0"
}

pub(super) fn compare_pier_labels(left: &str, right: &str) -> std::cmp::Ordering {
    let left_key = pier_label_key(left);
    let right_key = pier_label_key(right);
    left_key
        .0
        .cmp(&right_key.0)
        .then_with(|| left_key.1.cmp(&right_key.1))
        .then_with(|| natural_cmp(left, right))
}

fn pier_label_key(label: &str) -> (u8, u32) {
    if let Some(num) = parse_prefixed_number(label, "PX") {
        return (0, num);
    }
    if let Some(num) = parse_prefixed_number(label, "PY") {
        return (1, num);
    }
    (2, u32::MAX)
}

fn parse_prefixed_number(label: &str, prefix: &str) -> Option<u32> {
    let upper = label.trim().to_ascii_uppercase();
    if !upper.starts_with(prefix) {
        return None;
    }
    let suffix = &upper[prefix.len()..];
    if suffix.is_empty() || !suffix.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    suffix.parse::<u32>().ok()
}

fn natural_cmp(left: &str, right: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    let mut li = left.chars().peekable();
    let mut ri = right.chars().peekable();

    loop {
        match (li.peek(), ri.peek()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(lc), Some(rc)) if lc.is_ascii_digit() && rc.is_ascii_digit() => {
                let mut l_num = String::new();
                let mut r_num = String::new();
                while let Some(ch) = li.peek() {
                    if ch.is_ascii_digit() {
                        l_num.push(*ch);
                        li.next();
                    } else {
                        break;
                    }
                }
                while let Some(ch) = ri.peek() {
                    if ch.is_ascii_digit() {
                        r_num.push(*ch);
                        ri.next();
                    } else {
                        break;
                    }
                }
                let l_val = l_num.parse::<u64>().unwrap_or(0);
                let r_val = r_num.parse::<u64>().unwrap_or(0);
                match l_val.cmp(&r_val) {
                    Ordering::Equal => continue,
                    other => return other,
                }
            }
            (Some(_), Some(_)) => {
                let l = li.next().unwrap().to_ascii_lowercase();
                let r = ri.next().unwrap().to_ascii_lowercase();
                match l.cmp(&r) {
                    Ordering::Equal => continue,
                    other => return other,
                }
            }
        }
    }
}

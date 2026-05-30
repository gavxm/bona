use chrono::{DateTime, Utc};

/// OSC 8 hyperlink for terminals that support it.
pub fn hyperlink(url: &str, text: &str) -> String {
    format!("\x1b]8;;{url}\x1b\\{text}\x1b]8;;\x1b\\")
}

/// Format an ISO 8601 timestamp as "Mon YYYY (N years/months ago)".
pub fn format_date(iso: &str) -> String {
    let Ok(dt) = DateTime::parse_from_rfc3339(iso) else {
        return iso.to_string();
    };
    let now = Utc::now();
    let age = now.signed_duration_since(dt.to_utc());
    let days = age.num_days();

    let month = dt.format("%b %Y");
    let ago = if days >= 365 {
        let years = days / 365;
        format!("{years} year{}", if years == 1 { "" } else { "s" })
    } else if days >= 30 {
        let months = days / 30;
        format!("{months} month{}", if months == 1 { "" } else { "s" })
    } else {
        format!("{days} day{}", if days == 1 { "" } else { "s" })
    };

    format!("{month} ({ago} ago)")
}

/// Format a number with comma separators.
pub fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Wrap text at word boundaries to fit within `width` columns.
pub fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
        } else if current.len() + 1 + word.len() > width {
            lines.push(current);
            current = word.to_string();
        } else {
            current.push(' ');
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

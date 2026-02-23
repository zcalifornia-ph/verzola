#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NegotiatedGroupClassification {
    Pq,
    Classical,
    #[default]
    None,
}

impl NegotiatedGroupClassification {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pq => "pq",
            Self::Classical => "classical",
            Self::None => "none",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NegotiatedGroupMetadata {
    pub raw: String,
    pub negotiated_group: Option<String>,
    pub normalized_group: Option<String>,
    pub classification: NegotiatedGroupClassification,
}

impl NegotiatedGroupMetadata {
    pub fn parse(raw: &str) -> Self {
        parse_negotiated_group_metadata(raw)
    }
}

pub fn parse_negotiated_group_metadata(raw: &str) -> NegotiatedGroupMetadata {
    let negotiated_group = extract_negotiated_group_value(raw);
    let normalized_group = negotiated_group
        .as_deref()
        .and_then(normalize_group_name);
    let classification = classify_normalized_group(normalized_group.as_deref());

    NegotiatedGroupMetadata {
        raw: raw.to_string(),
        negotiated_group,
        normalized_group,
        classification,
    }
}

pub fn classify_negotiated_group(group_name: Option<&str>) -> NegotiatedGroupClassification {
    classify_normalized_group(group_name.and_then(normalize_group_name).as_deref())
}

const GROUP_KEYS: [&str; 6] = [
    "negotiated_group",
    "named_group",
    "key_exchange_group",
    "key_share_group",
    "group",
    "curve",
];

fn extract_negotiated_group_value(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    for (index, token) in tokens.iter().enumerate() {
        if let Some((key, value)) = split_token_key_value(token) {
            if is_group_key(key) {
                if let Some(group) = clean_group_value(value) {
                    return Some(group);
                }

                if let Some(next) = next_group_value_token(&tokens, index) {
                    return Some(next);
                }
            }
        }

        let key_only = normalize_key_token(token);
        if is_group_key(&key_only) {
            if let Some(next) = next_group_value_token(&tokens, index) {
                return Some(next);
            }
        }
    }

    if tokens.len() == 1 {
        if split_token_key_value(tokens[0]).is_some() {
            return None;
        }
        return clean_group_value(tokens[0]);
    }

    None
}

fn next_group_value_token(tokens: &[&str], index: usize) -> Option<String> {
    tokens
        .iter()
        .skip(index + 1)
        .find_map(|token| clean_group_value(token))
}

fn split_token_key_value(token: &str) -> Option<(&str, &str)> {
    let separator_index = token.find(['=', ':'])?;
    let (key, remainder) = token.split_at(separator_index);
    let value = remainder.get(1..)?;
    Some((key, value))
}

fn normalize_key_token(token: &str) -> String {
    token
        .trim_matches(|c: char| {
            c.is_ascii_whitespace()
                || matches!(c, '"' | '\'' | '`' | '[' | ']' | '(' | ')' | '{' | '}' | ',' | ';')
        })
        .trim_end_matches(['=', ':'])
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

fn is_group_key(key: &str) -> bool {
    GROUP_KEYS.contains(&key)
}

fn clean_group_value(value: &str) -> Option<String> {
    let cleaned = value
        .trim()
        .trim_matches(|c: char| {
            c.is_ascii_whitespace()
                || matches!(
                    c,
                    '"' | '\'' | '`' | '[' | ']' | '(' | ')' | '{' | '}' | ',' | ';'
                )
        });

    if cleaned.is_empty() {
        None
    } else if matches!(cleaned, "=" | ":") {
        None
    } else {
        Some(cleaned.to_string())
    }
}

fn normalize_group_name(group_name: &str) -> Option<String> {
    let normalized: String = group_name
        .trim()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect();

    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn classify_normalized_group(normalized_group: Option<&str>) -> NegotiatedGroupClassification {
    let Some(group) = normalized_group else {
        return NegotiatedGroupClassification::None;
    };

    if matches!(group, "none" | "na" | "n/a" | "unknown") {
        return NegotiatedGroupClassification::None;
    }

    if contains_any(
        group,
        &[
            "mlkem",
            "kyber",
            "frodo",
            "ntru",
            "saber",
            "bike",
            "hqc",
            "sntrup",
            "mceliece",
            "xwing",
        ],
    ) {
        return NegotiatedGroupClassification::Pq;
    }

    if is_classical_group(group) {
        return NegotiatedGroupClassification::Classical;
    }

    NegotiatedGroupClassification::None
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn is_classical_group(group: &str) -> bool {
    matches!(
        group,
        "x25519"
            | "curve25519"
            | "x448"
            | "curve448"
            | "secp256r1"
            | "prime256v1"
            | "secp384r1"
            | "secp521r1"
            | "p256"
            | "p384"
            | "p521"
            | "brainpoolp256r1"
            | "brainpoolp384r1"
            | "brainpoolp512r1"
            | "ffdhe2048"
            | "ffdhe3072"
            | "ffdhe4096"
            | "ffdhe6144"
            | "ffdhe8192"
    )
}

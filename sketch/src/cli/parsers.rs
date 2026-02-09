use std::str::FromStr;

use serde_json::Value;

pub fn parse_key_value_pairs<'a>(
	name: &'a str,
	s: &'a str,
) -> Result<Vec<(&'a str, &'a str)>, String> {
	let mut pairs: Vec<(&str, &str)> = Vec::new();

	for part in s.split(',') {
		let trimmed_part = part.trim();
		if trimmed_part.is_empty() {
			continue;
		}

		let (key, val) = parse_single_key_value_pair(name, trimmed_part)?;

		pairs.push((key, val));
	}

	Ok(pairs)
}

pub fn parse_single_key_value_pair<'a>(
	name: &'a str,
	trimmed_part: &'a str,
) -> Result<(&'a str, &'a str), String> {
	let pair: Vec<&str> = trimmed_part.split('=').collect();

	if pair.len() == 2 {
		let key = pair[0].trim();
		let val = pair[1].trim();

		Ok((key, val))
	} else {
		Err(format!(
			"Invalid key-value pair format for {name}. Only key-value pairs with '=' between them are allowed"
		))
	}
}

pub fn parse_serializable_key_value_pair(s: &str) -> Result<(String, Value), String> {
	let (key, val) = parse_single_key_value_pair("context", s)?;

	let parsed_val: Value = Value::from_str(val)
		.map_err(|e| format!("Could not map the value '{val}' to a serde-compatible value: {e}"))?;

	Ok((key.to_string(), parsed_val))
}

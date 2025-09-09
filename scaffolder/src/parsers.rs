use std::collections::BTreeSet;

use indexmap::IndexSet;

pub fn parse_btreeset_from_csv(s: &str) -> Result<BTreeSet<String>, String> {
  Ok(
    s.split(',')
      .map(|item| item.trim().to_string())
      .filter(|item| !item.is_empty())
      .collect(),
  )
}

pub fn parse_indexset_from_csv(s: &str) -> Result<IndexSet<String>, String> {
  Ok(
    s.split(',')
      .map(|item| item.trim().to_string())
      .filter(|item| !item.is_empty())
      .collect(),
  )
}

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

    let pair: Vec<&str> = trimmed_part.split('=').collect();

    if pair.len() == 2 {
      let key = pair[0].trim();
      let val = pair[1].trim();

      pairs.push((key, val));
    } else {
      return Err(format!("Invalid key-value pair format for {}. Only key-value pairs with '=' between them are allowed", name));
    }
  }

  Ok(pairs)
}

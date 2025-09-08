pub fn strip_trailing_comma<T: std::fmt::Display>(
  s: T,
  _: &dyn askama::Values,
) -> askama::Result<String> {
  let mut s = s.to_string();
  let last_non_whitespace_idx_byte = s
    .char_indices()
    .rev()
    .find(|(_, c)| !c.is_whitespace())
    .map(|(idx, _)| idx);

  if let Some(idx) = last_non_whitespace_idx_byte {
    let char_at_idx = s[idx..].chars().next();

    if char_at_idx == Some(',') {
      s.replace_range(idx..idx + ','.len_utf8(), "");
    }
  };

  Ok(s)
}

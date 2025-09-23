#[cfg(feature = "uuid")]
pub(crate) fn tera_uuid(
  _: &std::collections::HashMap<String, tera::Value>,
) -> Result<tera::Value, tera::Error> {
  Ok(uuid::Uuid::new_v4().to_string().into())
}

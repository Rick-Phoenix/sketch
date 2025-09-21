macro_rules! write_template {
  ($output:expr, $no_overwrite:expr, $data:expr, $suffix:expr) => {
    let path = $output.join($suffix);
    let mut file = crate::fs::open_file_if_overwriting($no_overwrite, &path)?;

    $data
      .write_into(&mut file)
      .map_err(|e| GenError::WriteError {
        path: path.clone(),
        source: e,
      })?;
  };
}

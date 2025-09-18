macro_rules! get_contributors {
  ($data:ident, $config:ident, $list_name:ident) => {
    $data.$list_name = $data
      .$list_name
      .into_iter()
      .map(|c| -> Result<Person, GenError> {
        match c {
          Person::Id(name) => Ok(Person::Data(
            $config
              .people
              .get(&name)
              .ok_or(GenError::Custom(format!(
                "Typescript individual with id `{}` not found",
                name
              )))?
              .clone(),
          )),
          Person::Data(person) => Ok(Person::Data(person)),
        }
      })
      .collect::<Result<std::collections::BTreeSet<Person>, GenError>>()?
  };
}

macro_rules! write_file {
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

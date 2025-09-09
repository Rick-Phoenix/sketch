macro_rules! get_contributors {
  ($data:ident, $config:ident, $list_name:ident) => {
    $data.$list_name = $data
      .$list_name
      .into_iter()
      .map(|c| -> Result<Person, GenError> {
        match c {
          Person::Workspace(name) => Ok(Person::Data(
            $config
              .people
              .get(&name)
              .ok_or(GenError::PersonNotFound { name })?
              .clone(),
          )),
          Person::Data(person) => Ok(Person::Data(person)),
        }
      })
      .collect::<Result<std::collections::BTreeSet<Person>, GenError>>()?
  };
}

macro_rules! write_file {
  ($output:expr, $overwrite:expr, $data:expr, $suffix:expr) => {
    let path = $output.join($suffix);
    let mut file = if $overwrite {
      File::create(&path).map_err(|e| GenError::FileCreation {
        path: path.clone(),
        source: e,
      })?
    } else {
      File::create_new(&path).map_err(|e| match e.kind() {
        std::io::ErrorKind::AlreadyExists => GenError::FileExists { path: path.clone() },
        _ => GenError::WriteError {
          path: path.clone(),
          source: e,
        },
      })?
    };

    $data
      .write_into(&mut file)
      .map_err(|e| GenError::WriteError {
        path: path.clone(),
        source: e,
      })?;
  };
}

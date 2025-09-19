pub(crate) mod custom_templating;

use serde_json::Value;

pub(crate) fn render_yaml_val(val: &Value, indent: usize) -> String {
  let mut string = String::new();

  match val {
    Value::Null => {
      string.push_str("null");
    }
    Value::Bool(v) => {
      string.push_str(&v.to_string());
    }
    Value::Number(v) => {
      string.push_str(&v.to_string());
    }
    Value::String(v) => {
      string.push('\'');
      string.push_str(v);
      string.push('\'');
    }
    Value::Array(values) => {
      string.push('\n');
      for value in values.iter() {
        let rendered_val = render_yaml_val(value, indent + 1);
        let stripped_val = if matches!(value, Value::Object(_)) {
          rendered_val.trim_start()
        } else {
          rendered_val.as_str()
        };

        string.push_str(&format!("{}- {}\n", "  ".repeat(indent + 1), stripped_val));
      }
    }
    Value::Object(map) => {
      string.push('\n');
      for (key, value) in map.iter() {
        string.push_str(&format!(
          "{}{}: {}\n",
          "  ".repeat(indent + 1),
          key,
          render_yaml_val(value, indent + 1)
        ));
      }
    }
  };

  string
}

#[cfg(test)]
pub(crate) fn convert_btreemap_to_json<T>(map: std::collections::BTreeMap<String, T>) -> Value
where
  T: Into<Value>,
{
  map.into_iter().collect()
}

macro_rules! generic_error {
  ($message:literal $(, $($rest:tt),*)?) => {
    GenError::Custom(format!($message $(, $($rest),*)?))
  };
}

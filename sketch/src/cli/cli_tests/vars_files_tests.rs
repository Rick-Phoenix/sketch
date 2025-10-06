use pretty_assertions::assert_eq;

#[tokio::test]
async fn vars_files() -> Result<(), Box<dyn std::error::Error>> {
  for ext in ["yaml", "json", "toml"] {
    let mut bin = get_bin!();

    let cmd = bin
      .args([
        &format!("--vars-{ext}"),
        &format!("tests/vars_files/vars.{ext}"),
        "render",
        "--content",
        "{{ myvar }}",
        "--stdout",
      ])
      .output()?;

    let output = String::from_utf8_lossy(&cmd.stdout);

    assert_eq!(output, "15\n");
  }

  Ok(())
}

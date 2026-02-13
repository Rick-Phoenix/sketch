use super::*;

#[tokio::test]
async fn vars_files() -> Result<(), Box<dyn std::error::Error>> {
	for ext in ["yaml", "json", "toml"] {
		let mut bin = get_bin!();

		let cmd = bin
			.args([
				"--ignore-config",
				"--vars-file",
				&format!("tests/vars_files/vars.{ext}"),
				"render",
				"--content",
				"{{ myvar }}",
			])
			.output()?;

		let output = String::from_utf8_lossy(&cmd.stdout);

		pretty_assert_eq!(output, "15\n");
	}

	Ok(())
}

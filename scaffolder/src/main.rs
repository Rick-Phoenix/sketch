#![allow(clippy::result_large_err)]

use scaffolder::GenError;

#[tokio::main]
async fn main() -> Result<(), GenError> {
  scaffolder::cli::start_cli().await?;

  Ok(())
}

#![allow(clippy::result_large_err)]

use sketch_it::GenError;

#[tokio::main]
async fn main() -> Result<(), GenError> {
  sketch_it::cli::main_entrypoint().await?;

  Ok(())
}

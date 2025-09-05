use scaffolder::{build_repo, GenError};

#[tokio::main]
async fn main() -> Result<(), GenError> {
  build_repo().await
}

use scaffolder::build_repo;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  build_repo().await
}

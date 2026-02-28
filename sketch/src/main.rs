#![allow(clippy::result_large_err)]

use clap::{CommandFactory, error::ErrorKind};
use sketch_it::cli::Cli;

#[tokio::main]
async fn main() {
	match sketch_it::cli::main_entrypoint().await {
		Ok(()) => {}
		Err(e) => {
			let mut cmd = Cli::command();

			cmd.error(ErrorKind::InvalidValue, e).exit();
		}
	}
}

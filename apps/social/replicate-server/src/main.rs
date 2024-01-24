use clap::Parser;
use replicate_server::Args;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
	let args = Args::parse();
	replicate_server::main(args).await
}

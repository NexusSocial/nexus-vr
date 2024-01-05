use clap::Parser;
use social_server2::Args;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
	let args = Args::parse();
	social_server2::main(args).await
}

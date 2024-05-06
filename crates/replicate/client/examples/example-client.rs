use clap::Parser;
use color_eyre::{eyre::WrapErr, Result};
use replicate_client::{instance::Instance, manager::Manager};
use replicate_common::{
	data_model::State,
	did::{AuthenticationAttestation, Did, DidPrivateKey},
};
use tracing::info;
use tracing_subscriber::{filter::LevelFilter, EnvFilter};
use url::Url;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about)]
pub struct Args {
	#[clap(long)]
	url: Url,
	#[clap(long)]
	username: String,
}

#[tokio::main]
async fn main() -> Result<()> {
	color_eyre::install()?;

	let env_filter = EnvFilter::builder()
		.with_default_directive(LevelFilter::INFO.into())
		.from_env_lossy();

	tracing_subscriber::fmt()
		.with_target(true)
		.with_level(true)
		.with_env_filter(env_filter)
		.init();

	let args = Args::parse();

	let did = Did(args.username);
	let did_private_key = DidPrivateKey;

	let auth_attest = AuthenticationAttestation::new(did, &did_private_key);

	let mut manager = Manager::connect(args.url, &auth_attest)
		.await
		.wrap_err("failed to connect to manager")?;
	info!("Connected to manager!");

	let instance_id = manager
		.instance_create()
		.await
		.wrap_err("failed to create instance")?;

	let instance_url = manager
		.instance_url(instance_id)
		.await
		.wrap_err("failed to get instance url")?;
	info!("Got instance {instance_id} at: {instance_url}");

	let (mut instance, net_task) = Instance::connect(instance_url, auth_attest)
		.await
		.wrap_err("failed to connect to instance")?;
	info!("Connected to instance!");

	let dm = instance.data_model_mut();
	let e1 = dm.spawn(bytes::Bytes::from_static(&[0]));
	assert_eq!(dm.get(e1).unwrap()[0], 0, "state mismatched at spawn");

	for i in 0..10u8 {
		dm.update(e1, bytes::Bytes::from(vec![i])).unwrap();
		let state: &State = dm.get(e1).unwrap();
		assert_eq!(state[0], i, "state mismatched at iteration {i}");
	}

	net_task
		.handle
		.await
		.wrap_err("net task died unexpectedly")?
		.wrap_err("net task returned an error")
}

use bevy::prelude::*;
use bevy_vrm::VrmBundle;
use lightyear::_reexport::{ShouldBeInterpolated, ShouldBePredicted};
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use social_common::shared::*;
use social_common::*;
use std::f32::consts::PI;
use std::net::{Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::time::Duration;

#[derive(Resource, Clone, Copy)]
pub struct MyClientPlugin {
	pub client_id: ClientId,
	pub client_port: u16,
	pub server_port: u16,
	pub transport: Transports,
}

impl Plugin for MyClientPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(PlayerClientId(self.client_id));

		let server_addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), self.server_port);
		let auth = Authentication::Manual {
			server_addr,
			client_id: self.client_id as ClientId,
			private_key: KEY,
			protocol_id: PROTOCOL_ID,
		};
		let client_addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), self.client_port);
		let link_conditioner = LinkConditionerConfig {
			incoming_latency: Duration::from_millis(100),
			incoming_jitter: Duration::from_millis(5),
			incoming_loss: 0.001,
		};
		let transport = match self.transport {
			Transports::Udp => TransportConfig::UdpSocket(client_addr),
			// Transports::Webtransport => TransportConfig::WebTransportClient {
			// 	client_addr,
			// 	server_addr,
			// },
		};
		let io = Io::from_config(
			&IoConfig::from_transport(transport).with_conditioner(link_conditioner),
		);
		let config = ClientConfig {
			shared: shared_config().clone(),
			input: InputConfig::default(),
			netcode: Default::default(),
			ping: PingConfig::default(),
			sync: SyncConfig::default(),
			prediction: PredictionConfig::default(),
			// we are sending updates every frame (60fps), let's add a delay of 6 network-ticks
			interpolation: InterpolationConfig::default().with_delay(
				InterpolationDelay::default().with_send_interval_ratio(2.0),
			),
			// .with_delay(InterpolationDelay::Ratio(2.0)),
		};
		let plugin_config = PluginConfig::new(config, io, protocol(), auth);
		app.add_plugins(ClientPlugin::new(plugin_config));
		app.add_plugins(shared::SharedPlugin);
		app.insert_resource(self.clone());
		app.add_systems(Startup, init);
		app.add_systems(
			FixedUpdate,
			buffer_input.in_set(InputSystemSet::BufferInputs),
		);
		app.add_systems(FixedUpdate, movement.in_set(FixedUpdateSet::Main));
		app.add_systems(
			Update,
			(
				receive_message1,
				handle_predicted_spawn,
				handle_interpolated_spawn,
				log,
				on_avatar_url_add,
				on_avatar_url_changed,
				change_pos,
				pos_added,
			),
		);
	}
}

#[derive(Resource)]
pub struct PlayerClientId(pub(crate) u64);

pub fn on_avatar_url_add(
	mut query: Query<
		(&PlayerId, &mut PlayerAvatarUrl),
		(Added<PlayerAvatarUrl>, With<Predicted>),
	>,
	player_client_id: Res<PlayerClientId>,
	mut client: ResMut<Client<MyProtocol>>,
) {
	for (player_id, mut player_avatar_url) in query.iter_mut() {
		if player_id.0 == player_client_id.0 {
			if player_avatar_url.0.is_none() {
				client.buffer_send::<Channel1, _>(Message1("https://vipe.mypinata.cloud/ipfs/QmU7QeqqVMgnMtCAqZBpAYKSwgcjD4gnx4pxFNY9LqA7KQ/default_398.vrm".to_string())).unwrap();
				//player_avatar_url.0.replace("https://vipe.mypinata.cloud/ipfs/QmU7QeqqVMgnMtCAqZBpAYKSwgcjD4gnx4pxFNY9LqA7KQ/default_398.vrm".to_string());
			}
		}
	}
}

pub fn on_avatar_url_changed(
	mut commands: Commands,
	assets: Res<AssetServer>,
	mut query: Query<
		(Entity, &PlayerAvatarUrl),
		(Changed<PlayerAvatarUrl>, With<Predicted>),
	>,
) {
	for (entity, url) in query.iter() {
		let url = match url.0.as_ref() {
			None => continue,
			Some(url) => url.as_str(),
		};
		let mut transform = Transform::from_xyz(0.0, -1.0, -4.0);
		transform.rotate_y(PI);

		commands.entity(entity).insert(VrmBundle {
			vrm: assets.load(url.to_string()),
			scene_bundle: SceneBundle {
				transform,
				..default()
			},
		});
	}
}

pub fn change_pos(
	mut query: Query<(&PlayerPosition, &mut Transform), Changed<PlayerPosition>>,
) {
	for (player_pos, mut transform) in query.iter_mut() {
		transform.translation = player_pos.0;
	}
}

pub fn pos_added(
	mut query: Query<(&PlayerPosition, &mut Transform), Added<PlayerPosition>>,
) {
	for (player_pos, mut transform) in query.iter_mut() {
		transform.translation = player_pos.0;
	}
}

// Startup system for the client
pub(crate) fn init(
	mut commands: Commands,
	mut client: ResMut<Client<MyProtocol>>,
	plugin: Res<MyClientPlugin>,
) {
	//commands.spawn(Camera2dBundle::default());
	commands.spawn(TextBundle::from_section(
		format!("Client {}", plugin.client_id),
		TextStyle {
			font_size: 30.0,
			color: Color::WHITE,
			..default()
		},
	));
	client.connect();
	// client.set_base_relative_speed(0.001);
}

// System that reads from peripherals and adds inputs to the buffer
pub(crate) fn buffer_input(
	mut client: ResMut<Client<MyProtocol>>,
	keypress: Res<Input<KeyCode>>,
) {
	let mut input = social_common::Direction {
		up: false,
		down: false,
		left: false,
		right: false,
	};
	if keypress.pressed(KeyCode::W) || keypress.pressed(KeyCode::Up) {
		input.up = true;
	}
	if keypress.pressed(KeyCode::S) || keypress.pressed(KeyCode::Down) {
		input.down = true;
	}
	if keypress.pressed(KeyCode::A) || keypress.pressed(KeyCode::Left) {
		input.left = true;
	}
	if keypress.pressed(KeyCode::D) || keypress.pressed(KeyCode::Right) {
		input.right = true;
	}
	if keypress.pressed(KeyCode::Delete) {
		// currently, inputs is an enum and we can only add one input per tick
		return client.add_input(Inputs::Delete);
	}
	if keypress.pressed(KeyCode::Space) {
		return client.add_input(Inputs::Spawn);
	}
	// TODO: should we only send an input if it's not all NIL?
	// info!("Sending input: {:?} on tick: {:?}", &input, client.tick());
	if !input.is_none() {
		info!(client_tick = ?client.tick(), input = ?&input, "Sending input");
		client.add_input(Inputs::Direction(input));
	}
}

// The client input only gets applied to predicted entities that we own
// This works because we only predict the user's controlled entity.
// If we were predicting more entities, we would have to only apply movement to the player owned one.
pub(crate) fn movement(
	// TODO: maybe make prediction mode a separate component!!!
	mut position_query: Query<&mut PlayerPosition, With<Predicted>>,
	mut input_reader: EventReader<InputEvent<Inputs>>,
) {
	// if we are not doing prediction, no need to read inputs
	if PlayerPosition::mode() != ComponentSyncMode::Full {
		return;
	}
	for input in input_reader.read() {
		if let Some(input) = input.input() {
			info!(?input, "read input");
			for mut position in position_query.iter_mut() {
				social_common::shared::shared_movement_behaviour(&mut position, input);
			}
		}
	}
}

// System to receive messages on the client
pub(crate) fn receive_message1(mut reader: EventReader<MessageEvent<Message1>>) {
	/*for event in reader.read() {
		info!("Received message: {:?}", event.message());
	}*/
}

// When the predicted copy of the client-owned entity is spawned, do stuff
// - assign it a different saturation
// - keep track of it in the Global resource
pub(crate) fn handle_predicted_spawn(
	mut predicted: Query<&mut PlayerColor, Added<Predicted>>,
) {
	for mut color in predicted.iter_mut() {
		color.0.set_s(0.2);
	}
}

pub(crate) fn log(
	client: Res<Client<MyProtocol>>,
	confirmed: Query<&PlayerPosition, With<Confirmed>>,
	predicted: Query<&PlayerPosition, (With<Predicted>, Without<Confirmed>)>,
	mut interp_event: EventReader<ComponentInsertEvent<ShouldBeInterpolated>>,
	mut predict_event: EventReader<ComponentInsertEvent<ShouldBePredicted>>,
) {
	let server_tick = client.latest_received_server_tick();
	for confirmed_pos in confirmed.iter() {
		debug!(?server_tick, "Confirmed position: {:?}", confirmed_pos);
	}
	let client_tick = client.tick();
	for predicted_pos in predicted.iter() {
		debug!(?client_tick, "Predicted position: {:?}", predicted_pos);
	}
	for event in interp_event.read() {
		info!("Interpolated event: {:?}", event.entity());
	}
	for event in predict_event.read() {
		info!("Predicted event: {:?}", event.entity());
	}
}

// When the predicted copy of the client-owned entity is spawned, do stuff
// - assign it a different saturation
// - keep track of it in the Global resource
pub(crate) fn handle_interpolated_spawn(
	mut interpolated: Query<&mut PlayerColor, Added<Interpolated>>,
) {
	for mut color in interpolated.iter_mut() {
		color.0.set_s(0.2);
	}
}

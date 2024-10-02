use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy_u8_assets::{U8AssetPlugin, U8AssetWriter};

pub fn main() {
	App::new()
		.add_plugins(U8AssetPlugin)
		.add_plugins(DefaultPlugins.set(AssetPlugin {
			meta_check: AssetMetaCheck::Never,
			..AssetPlugin::default()
		}))
		.add_systems(Startup, setup)
		.run();
}

fn setup(
	mut commands: Commands,
	mut writer: ResMut<U8AssetWriter>,
	asset_server: Res<AssetServer>,
) {
	let bytes =
		std::fs::read("/home/malek/RustroverProjects/nexus-vr/assets/Grass_01.png")
			.unwrap();
	writer.write("grass.png", &[]);
	commands.spawn(Camera2dBundle::default());
	commands.spawn(SpriteBundle {
		texture: asset_server.load("u8://grass.png"),
		..default()
	});
	writer.write_all("grass.png", bytes);
}

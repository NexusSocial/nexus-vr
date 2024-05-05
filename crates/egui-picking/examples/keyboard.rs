use bevy::{
	prelude::*,
	render::render_resource::{Extent3d, TextureUsages},
};
use bevy::input::keyboard::KeyboardInput;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiRenderToTexture};
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_egui_keyboard::draw_keyboard;
use egui_picking::{PickabelEguiPlugin, WorldSpaceUI};

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_plugins(DefaultPickingPlugins)
		.add_plugins(EguiPlugin)
		.add_plugins(PickabelEguiPlugin)
		.insert_resource(AmbientLight {
			color: Color::WHITE,
			brightness: 1.,
		})
		.add_systems(Startup, setup_worldspace)
		// Systems that create Egui widgets should be run during the `CoreSet::Update` set,
		// or after the `EguiSet::BeginFrame` system (which belongs to the `CoreSet::PreUpdate` set).
		.add_systems(Update, (update_screenspace, update_worldspace))
		.run()
}
fn setup_worldspace(
	mut images: ResMut<Assets<Image>>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	mut commands: Commands,
) {
	let output_texture = images.add({
		let size = Extent3d {
			width: 512,
			height: 512,
			depth_or_array_layers: 1,
		};
		let mut output_texture = Image {
			// You should use `0` so that the pixels are transparent.
			data: vec![0; (size.width * size.height * 4) as usize],
			..default()
		};
		output_texture.texture_descriptor.usage |= TextureUsages::RENDER_ATTACHMENT;
		output_texture.texture_descriptor.size = size;
		output_texture
	});
	commands.spawn((
		PbrBundle {
			mesh: meshes.add(Plane3d::new(Vec3::Y).mesh().size(1.0, 1.0)),
			material: materials.add(StandardMaterial {
				base_color: Color::WHITE,
				base_color_texture: Some(Handle::clone(&output_texture)),
				// Remove this if you want it to use the world's lighting.
				unlit: true,

				..default()
			}),
			transform: Transform::looking_at(
				Transform::IDENTITY,
				Vec3::Y,
				Vec3::splat(1.5),
			),
			..default()
		},
		WorldSpaceUI::new(output_texture, 1.0, 1.0),
	));
	let output_texture = images.add({
		let size = Extent3d {
			width: 512,
			height: 512,
			depth_or_array_layers: 1,
		};
		let mut output_texture = Image {
			// You should use `0` so that the pixels are transparent.
			data: vec![0; (size.width * size.height * 4) as usize],
			..default()
		};
		output_texture.texture_descriptor.usage |= TextureUsages::RENDER_ATTACHMENT;
		output_texture.texture_descriptor.size = size;
		output_texture
	});
	commands.spawn((
		PbrBundle {
			mesh: meshes.add(Plane3d::new(Vec3::Y).mesh().size(1.0, 1.0)),
			material: materials.add(StandardMaterial {
				base_color: Color::WHITE,
				base_color_texture: Some(Handle::clone(&output_texture)),
				// Remove this if you want it to use the world's lighting.
				unlit: true,

				..default()
			}),
			transform: Transform::looking_at(
				Transform::IDENTITY,
				Vec3::Y,
				Vec3::splat(1.5),
			).with_translation(Vec3::new(1.5, 0.0, 0.0)),
			..default()
		},
		WorldSpaceUI::new(output_texture, 1.0, 1.0),
		KeyboardBoi,
	));
	commands.spawn(Camera3dBundle {
		transform: Transform::from_xyz(1.5, 1.5, 1.5)
			.looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
		..default()
	});
}

fn update_screenspace(mut contexts: EguiContexts, mut buf: Local<String>) {
	egui::Window::new("Screenspace UI").show(contexts.ctx_mut(), |ui| {
		ui.label("I'm rendering to screenspace!");
		let buf: &mut String = &mut buf;
		ui.text_edit_singleline(buf);
	});
}

#[derive(Component)]
pub struct KeyboardBoi;

fn update_worldspace(
	mut contexts: Query<&mut bevy_egui::EguiContext, (With<EguiRenderToTexture>, Without<KeyboardBoi>)>,
	mut contexts2: Query<&mut bevy_egui::EguiContext, With<KeyboardBoi>>,
	window: Query<Entity, With<PrimaryWindow>>, mut just_pressed: Local<Option<KeyCode>>, mut event_writer: EventWriter<KeyboardInput>,
	mut previously_pressed: Local<Option<KeyCode>>,
	mut buf: Local<String>,
) {
	for mut ctx in contexts2.iter_mut() {
		egui::Window::new("other_ui").show(ctx.get_mut(), |ui| {
			draw_keyboard(ui, window.get_single().unwrap(), &mut previously_pressed, &mut event_writer);
		});
	}

	for mut ctx in contexts.iter_mut() {
		egui::Window::new("Worldspace UI").show(ctx.get_mut(), |ui| {
			ui.label("I'm rendering to a texture in worldspace!");
			let buf: &mut String = &mut buf;
			ui.text_edit_singleline(buf);
		});
	}
}

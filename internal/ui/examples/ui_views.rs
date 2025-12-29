use bevy::{
	DefaultPlugins,
	app::{App, AppExit, Startup},
	asset::{Assets, Handle},
	camera::{
		Camera, Camera3d, ClearColorConfig, RenderTarget, visibility::RenderLayers,
	},
	color::Color,
	ecs::{
		name::Name,
		system::{Commands, ResMut},
	},
	image::Image,
	light::PointLight,
	math::{
		Quat, Vec2, Vec3,
		primitives::{Circle, Cuboid, Plane3d},
	},
	mesh::{Mesh, Mesh3d},
	pbr::{MeshMaterial3d, StandardMaterial},
	render::{
		alpha::AlphaMode,
		render_resource::{Extent3d, TextureUsages},
	},
	transform::components::Transform,
	utils::default,
};
use bevy_egui::{EguiGlobalSettings, EguiPlugin, PrimaryEguiContext};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use nx_ui::{NexusUiPlugin, views::LoginUi};

fn main() -> AppExit {
	color_eyre::install().unwrap();

	App::new()
		.add_plugins(DefaultPlugins)
		.add_plugins(EguiPlugin::default())
		.add_plugins(WorldInspectorPlugin::new())
		.add_plugins(NexusUiPlugin)
		.add_systems(Startup, setup)
		.run()
}

fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	mut images: ResMut<Assets<Image>>,
	mut egui_global_settings: ResMut<EguiGlobalSettings>,
) {
	// Disable the automatic creation of a primary context to set it up manually for the camera we need.
	egui_global_settings.auto_create_primary_context = false;

	let image = images.add({
		let size = Extent3d {
			width: 256,
			height: 256,
			depth_or_array_layers: 1,
		};
		let mut image = Image {
			// You should use `0` so that the pixels are transparent.
			data: Some(vec![0; (size.width * size.height * 4) as usize]),
			..default()
		};
		image.texture_descriptor.usage |= TextureUsages::RENDER_ATTACHMENT;
		image.texture_descriptor.size = size;
		image
	});
	let cube_material = materials.add(StandardMaterial {
		base_color: Color::WHITE,
		base_color_texture: Some(Handle::clone(&image)),
		alpha_mode: AlphaMode::Blend,
		// Remove this if you want it to use the world's lighting.
		unlit: true,
		..default()
	});

	// circular base
	commands.spawn((
		Mesh3d(meshes.add(Circle::new(4.0))),
		MeshMaterial3d(materials.add(Color::WHITE)),
		Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
	));
	// plane
	commands.spawn((
		Mesh3d(meshes.add(Plane3d::new(Vec3::new(-1., 1., 1.), Vec2::new(1., 1.)))),
		MeshMaterial3d(Handle::clone(&cube_material)),
		Transform::from_xyz(1.0, 1.5, 1.0),
	));
	// cube
	commands.spawn((
		Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
		MeshMaterial3d(cube_material),
		Transform::from_xyz(0.0, 0.5, 0.0),
	));
	// light
	commands.spawn((
		PointLight {
			shadows_enabled: true,
			..default()
		},
		Transform::from_xyz(4.0, 8.0, 4.0),
	));
	// camera
	commands.spawn((
		LoginUi,
		Camera3d::default(),
		Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
		PrimaryEguiContext,
		Camera {
			clear_color: ClearColorConfig::Custom(Color::linear_rgb(0.64, 0.925, 1.)),
			..default()
		},
	));

	commands.spawn((
		Name::new("Login UI"),
		LoginUi,
		Camera3d::default(),
		RenderLayers::none(),
		Camera {
			target: RenderTarget::Image(image.into()),
			clear_color: ClearColorConfig::None,
			..default()
		},
	));
}

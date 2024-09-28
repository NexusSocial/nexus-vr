use bevy::prelude::*;
use bevy_suis::{
	raymarching::{
		raymarch_fields, RaymarchDefaultStepSize, RaymarchHitDistance,
		RaymarchMaxIterations,
	},
	xr::HandSide,
	xr_controllers::XrControllerInputMethodData,
	Field, InputMethod, PointerInputMethod, SuisPreUpdateSets,
};

use crate::draw_lasers::Laser;

pub struct LaserInputMethodPlugin;

/// 10km since that is the hardcoded max distance for bevy-suis raymarching
const DEFAULT_LASER_LENGHT: u32 = 10_000;

impl Plugin for LaserInputMethodPlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		app.add_systems(
			PreUpdate,
			update_things.after(SuisPreUpdateSets::InputMethodCapturing),
		);
		app.add_systems(PostStartup, setup_methods);
		app.add_systems(
			PreUpdate,
			update_rays.in_set(SuisPreUpdateSets::UpdateInputMethods),
		);
	}
}

fn update_rays(
	laser: Query<&Laser>,
	mut query: Query<(&Lazer, &mut PointerInputMethod, &GlobalTransform)>,
) {
	for (lazer, mut ray, global_transform) in &mut query {
		let laser = laser.get(lazer.0).unwrap();
		ray.0.origin = global_transform.translation();
		ray.0.direction = Dir3::new_unchecked(
			(laser.ray.get_point(laser.length) - global_transform.translation())
				.normalize(),
		);
	}
}

fn update_things(
	mut laser: Query<&mut Laser>,
	query: Query<(
		&Lazer,
		&InputMethod,
		&PointerInputMethod,
		&GlobalTransform,
		Option<&RaymarchMaxIterations>,
		Option<&RaymarchHitDistance>,
		Option<&RaymarchDefaultStepSize>,
	)>,
	handler_query: Query<(Entity, &Field, &GlobalTransform)>,
) {
	for (
		lazer,
		method,
		ray,
		location,
		max_iterations,
		hit_distance,
		default_step_size,
	) in query.iter()
	{
		let mut laser = laser.get_mut(lazer.0).unwrap();
		let distance = if let Some(handler) = method.captured_by {
			let Ok(handler) = handler_query.get(handler) else {
				warn!("invalid handler, how?");
				continue;
			};
			raymarch_fields(
				&ray.0,
				vec![handler],
				max_iterations.unwrap_or(&default()),
				hit_distance.unwrap_or(&default()),
				default_step_size.unwrap_or(&default()),
			)
			.first()
			// Option::<(&Vec3, Entity)>::None
			.map(|(pos, _)| pos.distance(ray.0.origin))
			.unwrap_or(DEFAULT_LASER_LENGHT as f32)
		} else {
			DEFAULT_LASER_LENGHT as f32
		};
		info!("distance: {distance}");
		let transform = location.compute_transform();

		laser.ray.origin = transform.translation;
		laser.ray.direction = transform.forward();
		laser.length = distance;
	}
}

#[derive(Component)]
struct Lazer(Entity);

fn setup_methods(
	mut cmds: Commands,
	query: Query<(Entity, &HandSide), With<XrControllerInputMethodData>>,
) {
	let lazer_left = cmds
		.spawn(Laser::new())
		.insert(SpatialBundle::default())
		.id();
	let lazer_right = cmds
		.spawn(Laser::new())
		.insert(SpatialBundle::default())
		.id();
	let left = cmds
		.spawn((
			XrControllerInputMethodData::default(),
			SpatialBundle::default(),
			Lazer(lazer_left),
			PointerInputMethod(Ray3d::new(Vec3::ZERO, Vec3::NEG_Z)),
			HandSide::Left,
		))
		.id();
	let right = cmds
		.spawn((
			XrControllerInputMethodData::default(),
			SpatialBundle::default(),
			Lazer(lazer_right),
			PointerInputMethod(Ray3d::new(Vec3::ZERO, Vec3::NEG_Z)),
			HandSide::Right,
		))
		.id();
	// not super clean but hopefully avoids depending on bevy_mod_*xr
	for (e, side) in &query {
		cmds.entity(e).push_children(&[match side {
			HandSide::Left => left,
			HandSide::Right => right,
		}]);
	}
}

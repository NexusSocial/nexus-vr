use bevy::prelude::*;
use bevy_mod_raycast::prelude::*;
use bevy_oxr::{
	xr_init::xr_only,
	xr_input::{
		interactions::XRRayInteractor,
		trackers::{AimPose, OpenXRTrackingRoot},
	},
};
use bevy_picking_core::{backend::prelude::*, pointer::InputMove};

use crate::XrPickLastPointerTransform;

pub mod prelude {}

#[derive(Resource, Default)]
pub struct XrRaycastSettings {}

#[derive(Clone)]
pub struct XrRaycastBackend;
impl Plugin for XrRaycastBackend {
	fn build(&self, app: &mut App) {
		app.init_resource::<XrRaycastSettings>().add_systems(
			PreUpdate,
			update_hits.in_set(PickSet::Backend).run_if(xr_only()),
		);
	}
}

// #[allow(clippy::too_many_arguments)]
/// Raycasts into the scene using [`PointerLocation`]s, then outputs
/// [`PointerHits`].
pub fn update_hits(
	mut pointers: Query<
		(
			Entity,
			&PointerId,
			&AimPose,
			&mut XrPickLastPointerTransform,
			&PointerLocation,
		),
		With<XRRayInteractor>,
	>,
	root: Query<&Transform, With<OpenXRTrackingRoot>>,
	pickables: Query<&Pickable>,
	mut raycast: Raycast,
	mut output_events: EventWriter<PointerHits>,
	mut move_events: EventWriter<InputMove>,
) {
	let t = match root.get_single() {
		Ok(t) => *t,
		Err(_) => Transform::IDENTITY,
	};
	for (pointer_entity, pointer_id, aim_pose, mut last_hit, location) in &mut pointers
	{
		let transform = t.mul_transform(aim_pose.0);
		if last_hit.0 != transform {
			move_events.send(InputMove {
				pointer_id: *pointer_id,
				location: location.clone().location.unwrap(),
				delta: Vec2::ZERO,
			})
		}
		last_hit.0 = transform;

		let ray = Ray3d::new(transform.translation, transform.forward());
		let settings = RaycastSettings {
			visibility: RaycastVisibility::MustBeVisible,
			filter: &|e| pickables.get(e).is_ok(),
			early_exit_test: &|entity_hit| {
				pickables
					.get(entity_hit)
					.is_ok_and(|pickable| pickable.should_block_lower)
			},
		};
		let picks = raycast
			.cast_ray(ray, &settings)
			.iter()
			.map(|(entity, hit)| {
				let hit_data = HitData::new(
					pointer_entity,
					hit.distance(),
					Some(hit.position()),
					Some(hit.normal()),
				);
				(*entity, hit_data)
			})
			.collect::<Vec<_>>();
		if !picks.is_empty() {
			output_events.send(PointerHits::new(*pointer_id, picks, 0f32));
		}
	}
}

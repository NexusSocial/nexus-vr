use bevy::prelude::*;
use bevy_mod_raycast::prelude::*;
use bevy_oxr::xr_input::interactions::XRRayInteractor;
use bevy_picking_core::backend::prelude::*;

pub mod prelude {}

#[derive(Resource, Default)]
pub struct XrRaycastSettings {}

#[derive(Clone)]
pub struct XrRaycastBackend;
impl Plugin for XrRaycastBackend {
	fn build(&self, app: &mut App) {
		app.init_resource::<XrRaycastSettings>()
			.add_systems(PreUpdate, update_hits.in_set(PickSet::Backend));
	}
}

/// Raycasts into the scene using [`PointerLocation`]s, then outputs
/// [`PointerHits`].
pub fn update_hits(
	pointers: Query<(Entity, &PointerId, &GlobalTransform), With<XRRayInteractor>>,
	pickables: Query<&Pickable>,
	// backend_settings: Res<XrRaycastSettings>,
	mut raycast: Raycast,
	mut output_events: EventWriter<PointerHits>,
) {
	for (pointer_entity, pointer_id, pointer_transform) in &pointers {
		let ray = Ray3d::from_transform(pointer_transform.compute_matrix());
		let settings = RaycastSettings {
			visibility: RaycastVisibility::MustBeVisible,
			filter: &|_| true,
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

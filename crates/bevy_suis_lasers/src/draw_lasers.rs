use bevy::prelude::*;
pub struct LaserPlugin;

impl Plugin for LaserPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Update, update_laser);
		app.add_systems(Startup, setup);
	}
}

#[derive(Component)]
pub struct Laser {
	pub ray: Ray3d,
	pub current_dir: Dir3,
	pub length: f32,
}

#[derive(Component)]
pub struct LaserSegment;

#[derive(Resource)]
struct LaserSegmentStuff {
	mesh: Handle<Mesh>,
	material: Handle<StandardMaterial>,
}

impl Laser {
	pub fn new() -> Laser {
		Laser {
			ray: Ray3d::new(Vec3::ZERO, Vec3::NEG_Z),
			current_dir: Dir3::NEG_Z,
			length: 1.0,
		}
	}
}

const TOTAL_LENGTH: f32 = SEGMENT_LENGTH + GAP_LENGTH;

const SEGMENT_LENGTH: f32 = 0.05;
const GAP_LENGTH: f32 = 0.03;
const INTERPOLATION_SPEED: f32 = 5.0;

fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	let capsule_mesh = meshes.add(Mesh::from(Capsule3d::new(0.005, SEGMENT_LENGTH)));

	let mat = StandardMaterial {
		base_color: Color::srgb(8.0, 0.2, 0.2),
		emissive: LinearRgba::rgb(8.0, 0.0, 0.0),
		perceptual_roughness: 0.0,
		metallic: 0.0,
		reflectance: 0.0,
		diffuse_transmission: 0.0,
		specular_transmission: 0.0,
		thickness: 0.0,
		ior: 0.0,
		attenuation_distance: 0.0,
		clearcoat: 0.0,
		clearcoat_perceptual_roughness: 0.0,
		anisotropy_strength: 0.0,
		anisotropy_rotation: 0.0,
		unlit: true,
		..default()
	};

	let capsule_material = materials.add(mat);

	commands.insert_resource(LaserSegmentStuff {
		mesh: capsule_mesh,
		material: capsule_material,
	});
}
const K: u32 = 20; // Adjust every 10 segments

fn update_laser(
	mut commands: Commands,
	mut laser_query: Query<(Entity, &mut Laser)>,
	laser_segment_stuff: Res<LaserSegmentStuff>,
	time: Res<Time>,
) {
	for (entity, mut laser) in laser_query.iter_mut() {
		// Remove existing laser segments
		commands.entity(entity).despawn_descendants();

		laser.current_dir = laser.current_dir.slerp(
			laser.ray.direction,
			time.delta_seconds() * INTERPOLATION_SPEED,
		);

		// Calculate length
		let length = laser.length;

		// Avoid division by zero
		if length <= 0.0 {
			continue;
		}

		// Direction from start to current_end
		let laser_direction = *laser.current_dir;

		let mut position_along = 0.0;
		let mut segment_count = 0;
		let mut level = 0;
		let mut segment_length = SEGMENT_LENGTH;
		let mut gap_length = GAP_LENGTH;

		while position_along < length {
			// Adjust segment length and gap length every K segments
			if segment_count > 0 && segment_count % K == 0 {
				level += 1;
				segment_length *= 2.0;
				gap_length *= 2.0;
			}

			// Check if adding this segment exceeds the laser length
			if position_along + segment_length / 2.0 > length {
				break;
			}

			// Compute segment position
			let segment_position =
				laser.ray.origin + laser_direction * (position_along + segment_length / 2.0);

			// Compute rotation to align with laser_direction
			let rotation = Quat::from_rotation_arc(Vec3::Y, laser_direction);

			// Scale the segment mesh along Y to represent the new length
			let mut transform = Transform {
				translation: segment_position,
				rotation,
				scale: Vec3::new(1.0, segment_length / SEGMENT_LENGTH, 1.0),
				..Default::default()
			};

			let bundle = PbrBundle {
				mesh: laser_segment_stuff.mesh.clone_weak(),
				material: laser_segment_stuff.material.clone_weak(),
				transform,
				..Default::default()
			};
			let child = commands.spawn(bundle).id();
			commands.entity(entity).add_child(child);

			// Update position_along
			position_along += segment_length + gap_length;
			segment_count += 1;
		}
	}
}

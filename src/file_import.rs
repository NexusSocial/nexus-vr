use crate::MainWindow;
use bevy::log::info;
use bevy::math::EulerRot::XYZ;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use bevy_spatial_egui::SpawnSpatialEguiWindowCommand;
use egui_aesthetix::Aesthetix;
use std::ops::{Add, Sub};
use std::path::Path;
use std::sync::Arc;

pub struct FileImportPlugin;
impl Plugin for FileImportPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Update, (draw_ui, read_dropped_files));
	}
}

#[derive(Copy, Clone, Component, Debug)]
pub enum FileType {
	Vrm,
	Gltf,
	Png,
}

#[derive(Component)]
struct FileImportWindow {
	path: String,
}

fn draw_ui(
	mut commands: Commands,
	mut query: Query<(Entity, &mut EguiContext, &FileImportWindow)>,
) {
	for (entity, mut ctx, file_import_window) in &mut query {
		let ctx: &mut EguiContext = &mut ctx;
		ctx.get_mut().set_style(
			Arc::new(egui_aesthetix::themes::TokyoNightStorm).custom_style(),
		);
		egui::panel::CentralPanel::default().show(ctx.get_mut(), |ui| {
			ui.heading(format!("Path: {}", file_import_window.path));
			if ui.button("png").clicked() {

            }
            if ui.button("worldspace model").clicked() {

            }
            if ui.button("switch avatar").clicked() {

            }
            ui.add_space(10.0);
            if ui.button("cancel").clicked() {
                commands.entity(entity).despawn_recursive();
            }
		});
	}
}

fn read_dropped_files(
	mut commands: Commands,
	mut events: EventReader<FileDragAndDrop>,
	camera: Query<&GlobalTransform, With<Camera>>,
) {
	let Ok(camera) = camera.get_single() else {
		return;
	};
	for event in events.read() {
		if let FileDragAndDrop::DroppedFile { path_buf, .. } = event {
			#[cfg(target_family = "wasm")]
			let path = String::from(path_buf.to_str().unwrap());
			#[cfg(not(target_family = "wasm"))]
			let path = bevy::asset::AssetPath::from_path(path_buf.as_path());

			let window = commands
				.spawn(FileImportWindow {
					path: path.to_string(),
				})
				.id();

			let mut rotation = camera.to_scale_rotation_translation().1;
			rotation = rotation.mul_quat(Quat::from_euler(
				XYZ,
				0.0,
				180.0f32.to_radians(),
				0.0,
			));
			let mut position = camera.translation();
			position = position.add(camera.compute_transform().forward() * 1.5);

			commands.push(SpawnSpatialEguiWindowCommand {
				target_entity: Some(window),
				position,
				rotation,
				resolution: UVec2::splat(1024),
				height: 1.0,
				unlit: true,
			});

			info!("DroppedFile: {}", path);

			//event_writer.send(NewLocalAvatar(path.to_string()));
		}
	}
}

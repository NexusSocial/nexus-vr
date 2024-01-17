use bevy::prelude::*;
use bevy_egui::egui;
use social_networking::data_model::PlayerAvatarUrl;

use crate::controllers::KeyboardController;
#[derive(Resource)]
pub struct Avatars {
	avis: &'static [AvatarData],
}

#[derive(Clone, Copy)]
pub struct AvatarData {
	name: &'static str,
	url: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct AvatarSwitchingUI;

pub struct AvatarSwitcherPlugin;
impl Plugin for AvatarSwitcherPlugin {
	fn build(&self, app: &mut App) {
		let avis = &[
			AvatarData {
				name: "016",
				url: "https://cloudcafe-executables.s3.us-west-2.amazonaws.com/016.vrm",
			},
			AvatarData {
				name: "015",
				url: "https://cloudcafe-executables.s3.us-west-2.amazonaws.com/015.vrm",
			},
			// Currentlly broken
			// AvatarData {
			// 	name: "010",
			// 	url: "https://cloudcafe-executables.s3.us-west-2.amazonaws.com/010.vrm",
			// },
			// Currentlly broken
			// AvatarData {
			// 	name: "009",
			// 	url: "https://cloudcafe-executables.s3.us-west-2.amazonaws.com/009.vrm",
			// },
			// Currentlly broken
			// AvatarData {
			// 	name: "MF Robot",
			// 	url: "https://cloudcafe-executables.s3.us-west-2.amazonaws.com/MF_Robot.vrm",
			// },
			AvatarData {
				name: "Avatar Sample F",
				url: "https://cloudcafe-executables.s3.us-west-2.amazonaws.com/AvatarSampleF.vrm",
			},
			AvatarData {
				name: "Avatar Sample G",
				url: "https://cloudcafe-executables.s3.us-west-2.amazonaws.com/AvatarSampleG.vrm",
			},
			AvatarData {
				name: "Hair Sample Male",
				url: "https://cloudcafe-executables.s3.us-west-2.amazonaws.com/HairSampleMale.vrm",
			},
			AvatarData {
				name: "Malek",
				url: "https://cloudcafe-executables.s3.us-west-2.amazonaws.com/malek.vrm",
			},
		];
		app.insert_resource(Avatars { avis });
		app.add_systems(Update, update_switching_ui);
	}
}
fn update_switching_ui(
	mut contexts: Query<&mut bevy_egui::EguiContext, With<AvatarSwitchingUI>>,
	local_avi: Query<&crate::avatars::DmEntity, With<KeyboardController>>,
	mut avatar_urls: Query<&mut PlayerAvatarUrl>,
	avatars: Res<Avatars>,
) {
	for mut ctx in contexts.iter_mut() {
		egui::CentralPanel::default().show(ctx.get_mut(), |ui| {
			ui.heading("Available Avatars");
			ui.horizontal_wrapped(|ui| {
				for a in avatars.avis.iter() {
					if ui.button(a.name).clicked() {
						info!("Switching To Avatar: {}", a.name);
						for avi in local_avi.iter() {
							avatar_urls
								.get_mut(avi.0)
								.expect("expected dm entity to contain avatar url")
								.0 = a.url.to_string();
						}
					}
				}
			});
		});
	}
}

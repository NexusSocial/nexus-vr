use bevy::prelude::*;
use bevy_egui::egui;

use crate::{avatars::assign::AssignAvatar, controllers::KeyboardController};
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
				url: "https://cdn.discordapp.com/attachments/1190761425396830369/1190761747162873947/016.vrm",
			},
			AvatarData {
				name: "015",
				url: "https://cdn.discordapp.com/attachments/1190761425396830369/1190761746869276794/015.vrm",
			},
			AvatarData {
				name: "010",
				url: "https://cdn.discordapp.com/attachments/1190761425396830369/1190761746445643827/010.vrm",
			},
			AvatarData {
				name: "009",
				url: "https://cdn.discordapp.com/attachments/1190761425396830369/1190761746126880778/009.vrm",
			},
			AvatarData {
				name: "008",
				url: "https://cdn.discordapp.com/attachments/1190761425396830369/1190761746126880778/009.vrm",
			},
			AvatarData {
				name: "MF Robot",
				url: "https://cdn.discordapp.com/attachments/1190761425396830369/1190761745426428005/MF_Robot.vrm",
			},
			AvatarData {
				name: "Avatar Sample F",
				url: "https://cdn.discordapp.com/attachments/1190761425396830369/1190761747456471130/AvatarSampleF.vrm",
			},
			AvatarData {
				name: "Avatar Sample G",
				url: "https://cdn.discordapp.com/attachments/1190761425396830369/1190761747775246386/AvatarSampleG.vrm",
			},
			AvatarData {
				name: "Hair Sample Male",
				url: "https://cdn.discordapp.com/attachments/1190761425396830369/1190761748119175278/HairSampleMale.vrm",
			},
			AvatarData {
				name: "Malek",
				url: "https://cdn.discordapp.com/attachments/1190761425396830369/1190863195418677359/malek.vrm",
			},
		];
		app.insert_resource(Avatars { avis });
		app.add_systems(Update, update_switching_ui);
	}
}
fn update_switching_ui(
	mut contexts: Query<&mut bevy_egui::EguiContext, With<AvatarSwitchingUI>>,
	mut assign_avi_evts: EventWriter<AssignAvatar>,
	local_avi: Query<Entity, With<KeyboardController>>,
	avatars: Res<Avatars>,
) {
	for mut ctx in contexts.iter_mut() {
		egui::CentralPanel::default().show(ctx.get_mut(), |ui| {
			ui.heading("Available Avatars");
			ui.horizontal_wrapped(|ui| {
				for a in avatars.avis.iter() {
					if ui.button(a.name).clicked() {
						info!("Switching To Avatar: {}", a.name);
						assign_avi_evts.send(AssignAvatar {
							avi_url: a.url.to_string(),
							avi_entity: local_avi.get_single().unwrap(),
						})
					}
				}
			});
		});
	}
}

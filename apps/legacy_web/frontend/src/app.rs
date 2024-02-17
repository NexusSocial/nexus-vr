use std::io::Cursor;
use std::sync::{Arc, Mutex};

// use tokio::runtime::Runtime;
// use tokio::sync::mpsc::Sender;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
	// Example stuff:
	#[serde(skip)]
	label: String,

	#[serde(skip)] // This how you opt-out of serialization of a field
	value: f32,

	#[serde(skip)]
	dropped_file: Option<egui::DroppedFile>,

	username: Option<String>,

	temp_username: String,

	avatars: Arc<Mutex<Vec<String>>>,
}

impl Default for TemplateApp {
	fn default() -> Self {
		Self {
			// Example stuff:
			label: "Hello World!".to_owned(),
			value: 2.7,
			dropped_file: None,
			username: None,
			temp_username: "".to_string(),
			avatars: Arc::new(Mutex::new(vec![])),
		}
	}
}

impl TemplateApp {
	/// Called once before the first frame.
	pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
		// This is also where you can customize the look and feel of egui using
		// `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

		// Load previous app state (if any).
		// Note that you must enable the `persistence` feature for this to work.
		if let Some(storage) = cc.storage {
			return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
		}

		Default::default()
	}

	pub fn refresh_avatar_list(&self, ctx: &egui::Context) -> impl FnOnce() {
		let url = "http://127.0.0.1:3000".to_string();
		let ctx = ctx.clone();
		let this_avatars = self.avatars.clone();
		let username = self.username.as_ref().unwrap().clone();
		let request = ehttp::Request::get(format!("{}/get_avatars/{}", url, username));
		move || {
			ehttp::fetch(request, move |response| {
				if let Ok(value) = response {
					let avatars = value.json::<Vec<String>>().unwrap();
					*this_avatars.lock().unwrap() = avatars;
				}
				ctx.request_repaint();
			});
		}
	}
}

impl eframe::App for TemplateApp {
	/// Called by the frame work to save state before shutdown.
	fn save(&mut self, storage: &mut dyn eframe::Storage) {
		eframe::set_value(storage, eframe::APP_KEY, self);
	}

	/// Called each time the UI needs repainting, which may be many times per second.
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		// Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
		// For inspiration and more examples, go to https://emilk.github.io/egui

		egui::CentralPanel::default().show(ctx, |ui| {
			let url = "http://127.0.0.1:3000".to_string();
			if self.username.is_none() {
				ui.text_edit_singleline(&mut self.temp_username);
				if ui.button("login").clicked() {
					self.username.replace(self.temp_username.clone());
				}
				return;
			}
			ui.label(format!("logged in as: {}", self.username.as_ref().unwrap()));
			if ui.button("logout").clicked() {
				self.username.take();
				self.temp_username = "".to_string();
				return;
			}
			ui.indent("avatars", |ui| {
				for avatar in self.avatars.lock().unwrap().iter() {
					ui.add_space(2.0);
					ui.horizontal(|ui| {
						ui.label(avatar);
						if ui.button("delete").clicked() {
							let request = ehttp::Request::get(format!(
								"{}/delete_avatar/{}/{}",
								url,
								self.username.as_ref().unwrap(),
								avatar
							));
							let ctx = ctx.clone();
							let refresh_avatar = self.refresh_avatar_list(&ctx);
							ehttp::fetch(request, move |_| {
								ctx.request_repaint();
								refresh_avatar();
							});
						}
					});
					if ui.button("download").clicked() {
						use wasm_bindgen::JsCast;

						let win = web_sys::window().unwrap();
						let doc = win.document().unwrap();

						let link = doc.create_element("a").unwrap();
						link.set_attribute(
							"href",
							&format!(
								"{}/get_avatar/{}/{}",
								url,
								self.username.as_ref().unwrap(),
								avatar
							),
						)
						.unwrap();

						let link: web_sys::HtmlAnchorElement =
							web_sys::HtmlAnchorElement::unchecked_from_js(link.into());
						link.click();
					}
					ui.add_space(2.0);
				}
			});

			match self.dropped_file.take() {
				None => {
					ui.label("drag and drop avatars to upload");
				}
				Some(dropped_file) => {
					ui.label(&dropped_file.name);
					if ui.button("Upload Avatar").clicked() {
						let ctx = ctx.clone();
						let mut bytes = Cursor::new(dropped_file.bytes.unwrap());
						let multipart = ehttp::multipart::MultipartBuilder::new()
							.add_stream(
								&mut bytes,
								&dropped_file.name,
								Some(&dropped_file.name),
								None,
							)
							.unwrap();
						let request = ehttp::Request::multipart(
							format!(
								"{}/upload/{}",
								url,
								self.username.as_ref().unwrap()
							),
							multipart,
						);
						let refresh_avatars = self.refresh_avatar_list(&ctx);
						ehttp::fetch(request, move |_part| {
							ctx.request_repaint();
							refresh_avatars();
						});
					} else {
						self.dropped_file.replace(dropped_file);
					}
				}
			}
		});

		if self.username.is_some() {
			preview_files_being_dropped(ctx);
			ctx.input_mut(|i| {
				while !i.raw.dropped_files.is_empty() {
					self.dropped_file.replace(i.raw.dropped_files.remove(0));
				}
			});
		}
	}
}

/// Preview hovering files:
fn preview_files_being_dropped(ctx: &egui::Context) {
	use egui::*;
	use std::fmt::Write as _;

	if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
		let text = ctx.input(|i| {
			let mut text = "Dropping files:\n".to_owned();
			for file in &i.raw.hovered_files {
				if let Some(path) = &file.path {
					write!(text, "\n{}", path.display()).ok();
				} else if !file.mime.is_empty() {
					write!(text, "\n{}", file.mime).ok();
				} else {
					text += "\n???";
				}
			}
			text
		});

		let painter = ctx.layer_painter(LayerId::new(
			Order::Foreground,
			Id::new("file_drop_target"),
		));

		let screen_rect = ctx.screen_rect();
		painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
		painter.text(
			screen_rect.center(),
			Align2::CENTER_CENTER,
			text,
			TextStyle::Heading.resolve(&ctx.style()),
			Color32::WHITE,
		);
	}
}

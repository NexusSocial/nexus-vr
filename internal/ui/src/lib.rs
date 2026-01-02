//! Nexus UI Crate

mod schedule;

use std::{fmt::Debug, marker::PhantomData};

use bevy::{
	app::{App, Plugin},
	camera::Camera3d,
	ecs::{
		component::{Component, Mutable},
		lifecycle::HookContext,
		name::Name,
		query::{With, Without},
		system::Single,
		world::DeferredWorld,
	},
	log::warn,
};
use bevy_egui::{
	EguiContext, EguiMultipassSchedule, EguiPrimaryContextPass, PrimaryEguiContext,
};
use egui::WidgetText;

use crate::schedule::UiViewSchedule;

/// Requried by any component that implements [`UiViewT`].
#[derive(Debug, Default, Component)]
#[require(Camera3d, UiViewTitle)]
pub struct UiView;

/// the core egui logic used to render the view.
pub trait UiViewT: Component + 'static + Send + Sync + Debug {
	fn render_ui(&mut self, ui: &mut egui::Ui);
}

/// Becomes the title of the egui Window.
#[derive(Debug, Component, Clone)]
#[component(on_add = name_from_title)]
pub struct UiViewTitle(pub WidgetText);

impl UiViewTitle {
	pub fn new(text: impl Into<WidgetText>) -> Self {
		Self(text.into())
	}
}

impl Default for UiViewTitle {
	fn default() -> Self {
		Self(WidgetText::from("UI View"))
	}
}

fn name_from_title(mut world: DeferredWorld, ctx: HookContext) {
	let Some(title) = world.get::<UiViewTitle>(ctx.entity) else {
		unreachable!("we only run this system in response to a UiViewTitle being added")
	};
	let text = Name::from(title.0.text());
	world.commands().entity(ctx.entity).insert_if_new(text);
}

/// Add this to your app to enable UI for a particular [`UiViewT`].
#[derive(Debug)]
pub struct UiViewPlugin<T>(PhantomData<T>);

impl<T> Default for UiViewPlugin<T> {
	fn default() -> Self {
		Self(PhantomData)
	}
}

impl<T: UiViewT<Mutability = Mutable>> Plugin for UiViewPlugin<T> {
	fn build(&self, app: &mut App) {
		if !app.is_plugin_added::<bevy_egui::EguiPlugin>() {
			// TODO: Check if we should panic instead, or if it can work without
			// bevy_egui
			warn!(
				"bevy_egui plugin is not enabled, so {:?} might be inert",
				self
			);
		}

		app.add_systems(UiViewSchedule::<T>::default(), worldspace_render::<T>);
		app.add_systems(EguiPrimaryContextPass, screenspace_render::<T>);
		let world = app.world_mut();
		world.try_register_required_components::<T, UiView>().ok();
		world
			.register_component_hooks::<T>()
			.on_insert(insert_schedule_if_not_primary::<T>);
	}
}

fn insert_schedule_if_not_primary<T: UiViewT>(
	mut world: DeferredWorld,
	ctx: HookContext,
) {
	let e = ctx.entity;
	if world.get::<PrimaryEguiContext>(e).is_some() {
		return;
	}
	world
		.commands()
		.entity(ctx.entity)
		.insert(EguiMultipassSchedule::new(UiViewSchedule::<T>::default()));
}

fn screenspace_render<T: UiViewT<Mutability = Mutable>>(
	query: Single<(&mut EguiContext, &mut T, &UiViewTitle), With<PrimaryEguiContext>>,
) {
	let (mut ctx, mut view, title) = query.into_inner();
	render::<T>(ctx.get_mut(), &mut view, title);
}

fn worldspace_render<T: UiViewT<Mutability = Mutable>>(
	query: Single<
		(&mut EguiContext, &mut T, &UiViewTitle),
		Without<PrimaryEguiContext>,
	>,
) {
	let (mut ctx, mut view, title) = query.into_inner();
	render::<T>(ctx.get_mut(), &mut view, title);
}

fn render<T: UiViewT>(ctx: &mut egui::Context, view: &mut T, title: &UiViewTitle) {
	egui::Window::new(title.0.clone()).show(ctx, |ui| view.render_ui(ui));
}

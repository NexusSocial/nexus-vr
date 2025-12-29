use bevy::{
	app::App,
	ecs::{
		component::Component,
		entity::Entity,
		lifecycle::HookContext,
		query::{With, Without},
		schedule::ScheduleLabel,
		system::Single,
		world::DeferredWorld,
	},
};
use bevy_egui::{
	EguiContext, EguiMultipassSchedule, EguiPrimaryContextPass, PrimaryEguiContext,
};
use egui::Context;

// Should it be a resource?
#[derive(Debug, Default, Component)]
#[component(on_insert = insert_schedule_if_not_primary)]
pub struct LoginUi;

fn insert_schedule_if_not_primary(mut world: DeferredWorld, ctx: HookContext) {
	let e = ctx.entity;
	if world.get::<PrimaryEguiContext>(e).is_some() {
		return;
	}
	world
		.commands()
		.entity(ctx.entity)
		.insert(EguiMultipassSchedule::new(LoginUiSchedule));
}

#[derive(Debug, Default, Clone, Hash, Eq, PartialEq, ScheduleLabel)]
pub struct LoginUiSchedule;

pub(super) fn add_systems(app: &mut App) {
	app.add_systems(LoginUiSchedule, worldspace_render);
	app.add_systems(EguiPrimaryContextPass, screenspace_render);
}

fn screenspace_render(
	query: Single<
		(Entity, &mut EguiContext),
		(With<LoginUi>, With<PrimaryEguiContext>),
	>,
) {
	let (e, mut ctx) = query.into_inner();
	render(e, ctx.get_mut());
}

fn worldspace_render(
	query: Single<(Entity, &mut EguiContext), Without<PrimaryEguiContext>>,
) {
	let (e, mut ctx) = query.into_inner();
	render(e, ctx.get_mut());
}

fn render(e: Entity, ctx: &mut Context) {
	egui::Window::new(format!("Login UI: {}", e)).show(ctx, |ui| {
		ui.horizontal(|ui| {
			ui.label("Foobar");
		});
	});
}

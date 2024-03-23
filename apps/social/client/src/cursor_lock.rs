use std::marker::PhantomData;

use bevy::{
	input::common_conditions::input_just_pressed,
	prelude::*,
	window::{CursorGrabMode, PrimaryWindow},
};
use bevy_oxr::xr_init::XrStatus;
use bevy_schminput::mouse::{motion::MouseMotionAction, MouseBindings};

pub struct CursorLockingPlugin;
impl Plugin for CursorLockingPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Startup, init_cursor_lock);
		app.add_systems(
			Update,
			toggle_mouse_lock.run_if(input_just_pressed(KeyCode::Tab)),
		);
		app.add_systems(
			Update,
			apply_mouse_lock_toggle.run_if(resource_exists_and_changed::<MouseLocked>),
		);
		app.add_systems(
			Update,
			cursor_recenter
				.run_if(|res: Option<Res<MouseLocked>>| res.is_some_and(|v| **v)),
		);
	}
}

fn init_cursor_lock(mut cmds: Commands, xr_status: Option<Res<XrStatus>>) {
	if xr_status.is_some_and(|e| *e == XrStatus::Enabled) {
		cmds.insert_resource(MouseLocked(false));
	} else {
		cmds.insert_resource(MouseLocked(true));
	}
}

fn cursor_recenter(mut q_windows: Query<&mut Window, With<PrimaryWindow>>) {
	let mut primary_window = match q_windows.get_single_mut() {
		Ok(v) => v,
		Err(_) => return,
	};
	let center = Vec2::new(primary_window.width() / 2.0, primary_window.height() / 2.0);
	primary_window.set_cursor_position(Some(center));
}

fn apply_mouse_lock_toggle(
	res: Res<MouseLocked>,
	mut q_windows: Query<&mut Window, With<PrimaryWindow>>,
) {
	for mut w in &mut q_windows {
		if **res {
			w.cursor.grab_mode = CursorGrabMode::Locked;
			w.cursor.visible = false;
		} else {
			w.cursor.grab_mode = CursorGrabMode::None;
			w.cursor.visible = true;
		}
	}
}

fn toggle_mouse_lock(mut res: ResMut<MouseLocked>) {
	**res = !**res;
}

#[derive(Default)]
pub struct LockingMouseActionPlugin<T: MouseMotionAction + Component + Default>(
	PhantomData<T>,
);

impl<T: MouseMotionAction + Component + Default> Plugin
	for LockingMouseActionPlugin<T>
{
	fn build(&self, app: &mut App) {
		app.add_systems(
			Update,
			toggle_mouse_input_on_lock::<T>
				.run_if(resource_exists_and_changed::<MouseLocked>),
		);
	}
}

pub fn toggle_mouse_input_on_lock<T: MouseMotionAction + Component + Default>(
	mut mouse_bindings: ResMut<MouseBindings>,
	locked: Res<MouseLocked>,
) {
	// This works because behind the scenes it's static per type for most things
	if !locked.0 {
		mouse_bindings.drop_motion_binding(&T::default());
	} else {
		mouse_bindings.add_motion_binding(&T::default());
	}
}

#[derive(Clone, Copy, Component, Default, Debug, Reflect)]
struct LastMouseSens<T> {
	x: f32,
	y: f32,
	_phantom_data: PhantomData<T>,
}

#[derive(Deref, DerefMut, Default, Reflect, Clone, Copy, Resource)]
pub struct MouseLocked(bool);

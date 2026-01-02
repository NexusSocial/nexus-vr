use std::marker::PhantomData;

use bevy::ecs::schedule::ScheduleLabel;
use std::hash::Hash;

#[derive(Debug, ScheduleLabel)]
pub(crate) struct UiViewSchedule<T>(PhantomData<T>);

impl<T> Hash for UiViewSchedule<T> {
	fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {}
}

impl<T> PartialEq for UiViewSchedule<T> {
	fn eq(&self, _other: &Self) -> bool {
		true
	}
}

impl<T> Eq for UiViewSchedule<T> {}

impl<T> Default for UiViewSchedule<T> {
	fn default() -> Self {
		Self(PhantomData)
	}
}

impl<T> Clone for UiViewSchedule<T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T> Copy for UiViewSchedule<T> {}

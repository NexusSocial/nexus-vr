use crate::networking::{Connection, ConnectionTrait, ReliableMessage};
use bevy::prelude::*;
use bevy_matchbox::prelude::MultipleChannels;
use bevy_matchbox::MatchboxSocket;
use futures_channel::mpsc::{channel, Receiver, SendError, Sender};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, Event)]
pub enum FilePart {
	Start { uuid: Uuid, len: usize },
	Part(Uuid, Vec<u8>),
	Done(Uuid),
}

impl FilePart {
	pub fn uuid(&self) -> &Uuid {
		match self {
			FilePart::Start { uuid, .. } => uuid,
			FilePart::Part(uuid, _) => uuid,
			FilePart::Done(uuid) => uuid,
		}
	}
}

#[derive(Component)]
pub struct InProgressFile {
	len: usize,
	uuid: Uuid,
	data: Vec<u8>,
	finished: bool,
}

pub struct FileSharingPlugin;

impl Plugin for FileSharingPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(FileParts::default());
		let (tx, rx) = channel(100);
		app.insert_resource(P2pFileRx(rx));
		app.insert_resource(P2pFileSender(tx));
		app.add_systems(
			Update,
			handle_file_part
				.run_if(resource_exists::<MatchboxSocket<MultipleChannels>>),
		);
		app.add_systems(
			Update,
			send_parts_of_file
				.run_if(resource_exists::<MatchboxSocket<MultipleChannels>>),
		);
	}
}

#[derive(Resource)]
struct P2pFileRx(Receiver<(Uuid, Vec<u8>)>);

#[derive(Resource)]
pub struct P2pFileSender(Sender<(Uuid, Vec<u8>)>);

fn send_parts_of_file(
	mut p2p_rx: ResMut<P2pFileRx>,
	mut local: Local<HashMap<Uuid, Vec<Vec<u8>>>>,
	mut connection: Connection,
) {
	while let Ok(Some((uuid, bytes))) = p2p_rx.0.try_next() {
		let mut chunks = vec![];
		let len = bytes.len();
		for chunk in bytes.chunks(256) {
			chunks.push(chunk.to_vec());
		}
		local.insert(uuid.clone(), chunks);
		while let Err(e) = connection
			.send_all(&ReliableMessage::FilePart(FilePart::Start { uuid, len }))
		{
			if e.first().unwrap().1.is_disconnected() {
				return;
			}
		}
	}

	let mut list_of_empty = vec![];

	for (uuid, mut chunks) in local.iter_mut() {
		if let Some(chunk) = chunks.pop() {
			let message =
				ReliableMessage::FilePart(FilePart::Part(uuid.clone(), chunk));
			while let Err(e) = connection.send_all(&message) {
				if e.first().unwrap().1.is_disconnected() {
					return;
				}
			}
		} else {
			list_of_empty.push(uuid.clone());
		}
	}

	for uuid in list_of_empty {
		local.remove(&uuid);
		while let Err(e) =
			connection.send_all(&ReliableMessage::FilePart(FilePart::Done(uuid)))
		{
			if e.first().unwrap().1.is_disconnected() {
				return;
			}
		}
	}
}

#[derive(Resource, Default)]
pub struct FileParts(pub Vec<FilePart>);

fn handle_file_part(
	mut commands: Commands,
	mut file_parts_vec: ResMut<FileParts>,
	mut file_parts: Query<&mut InProgressFile>,
) {
	for file_part in file_parts_vec.0.drain(0..) {
		match file_part {
			FilePart::Start { uuid, len } => {
				commands.spawn(InProgressFile {
					len,
					uuid,
					data: vec![],
					finished: false,
				});
			}
			FilePart::Part(uuid, mut data) => {
				for mut existing_part in file_parts.iter_mut() {
					if existing_part.uuid == uuid {
						existing_part.data.append(&mut data);
						break;
					}
				}
			}
			FilePart::Done(uuid) => {
				for mut existing_part in file_parts.iter_mut() {
					if existing_part.uuid == uuid {
						existing_part.finished = true;
						break;
					}
				}
			}
		}
	}
}

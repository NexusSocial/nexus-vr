use bevy::asset::io::ErasedAssetReader;
use bevy::asset::io::{
	AssetReader, AssetReaderError, AssetSource, AssetSourceId, PathStream, Reader,
};
pub use bevy::prelude::*;
use flume::{Receiver, Sender};
use futures_io::{AsyncRead, AsyncSeek};
use futures_locks::RwLock;
use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::Poll;
use thiserror::Error;

pub struct U8AssetPlugin;

impl Plugin for U8AssetPlugin {
	fn build(&self, app: &mut App) {
		let (asset_registry, asset_writer) = U8AssetRegistry::new();
		app.register_asset_source(
			AssetSourceId::new(Some("u8")),
			AssetSource::build().with_reader(move || Box::new(asset_registry.clone())),
		);
		app.insert_resource(asset_writer);
	}
}

// Enum to represent the state of asset data
enum AssetDataState {
	Pending,
	Ready(Vec<u8>),
	Done,
}

// Enum to hold either a Receiver or a Vec<u8>
enum AssetSourceEnum {
	Stream(Receiver<AssetDataState>),
	Data(Vec<u8>),
}

#[derive(Clone)]
pub struct U8AssetRegistry {
	rx: Receiver<(PathBuf, Receiver<AssetDataState>)>,
	assets: RwLock<HashMap<PathBuf, AssetSourceEnum>>,
}

#[derive(Resource)]
pub struct U8AssetWriter {
	writers: HashMap<PathBuf, Sender<AssetDataState>>,
	tx: Sender<(PathBuf, Receiver<AssetDataState>)>,
}

impl U8AssetRegistry {
	pub fn new() -> (Self, U8AssetWriter) {
		let (tx, rx) = flume::bounded(100);
		(
			Self {
				rx,
				assets: RwLock::new(HashMap::new()),
			},
			U8AssetWriter {
				writers: HashMap::new(),
				tx,
			},
		)
	}

	pub async fn add_stream(
		&mut self,
		path: PathBuf,
		receiver: Receiver<AssetDataState>,
	) {
		self.assets
			.write()
			.await
			.insert(path, AssetSourceEnum::Stream(receiver));
	}

	pub async fn add_data(&mut self, path: PathBuf, data: Vec<u8>) {
		self.assets
			.write()
			.await
			.insert(path, AssetSourceEnum::Data(data));
	}
}

impl U8AssetWriter {
	fn send(&mut self, path: impl Into<PathBuf>, asset_data_state: AssetDataState) {
		let path = path.into();
		if !self.writers.contains_key(&path) {
			let (tx, rx) = flume::unbounded();
			self.tx.send((path.clone(), rx)).unwrap();
			self.writers.insert(path.clone(), tx);
		}
		self.writers
			.get_mut(&path)
			.unwrap()
			.send(asset_data_state)
			.unwrap();
	}
	pub fn write(&mut self, path: impl Into<PathBuf>, data: impl AsRef<[u8]>) {
		self.send(path, AssetDataState::Ready(data.as_ref().to_vec()));
	}
	pub fn finish(&mut self, path: impl Into<PathBuf>) {
		self.send(path, AssetDataState::Done);
	}
	pub fn write_all(&mut self, path: impl Into<PathBuf>, data: impl AsRef<[u8]>) {
		let path = path.into();
		self.write(&path, data);
		self.finish(path);
	}
}

#[derive(Error, Debug)]
enum U8AssetReaderError {
	#[error("Seek is not supported when embeded")]
	SeekNotSupported,
}

#[derive(Default, Debug, Clone)]
pub struct DataReader(pub Cursor<Vec<u8>>);

impl AsyncRead for DataReader {
	fn poll_read(
		mut self: Pin<&mut Self>,
		_: &mut std::task::Context<'_>,
		buf: &mut [u8],
	) -> Poll<futures_io::Result<usize>> {
		use std::io::Read;
		let read = self.as_mut().0.read(buf);
		Poll::Ready(read)
	}
}

impl AsyncSeek for DataReader {
	fn poll_seek(
		self: Pin<&mut Self>,
		_: &mut std::task::Context<'_>,
		_pos: futures_io::SeekFrom,
	) -> Poll<futures_io::Result<u64>> {
		Poll::Ready(Err(futures_io::Error::new(
			futures_io::ErrorKind::Other,
			U8AssetReaderError::SeekNotSupported,
		)))
	}
}

impl AssetReader for U8AssetRegistry {
	async fn read<'a>(
		&'a self,
		path: &'a Path,
	) -> Result<Box<Reader<'a>>, AssetReaderError> {
		for thing in self.rx.try_recv() {
			self.assets
				.write()
				.await
				.insert(thing.0, AssetSourceEnum::Stream(thing.1));
		}
		let data = match self.assets.read().await.get(path) {
			Some(AssetSourceEnum::Data(data)) => {
				return Ok(Box::new(DataReader(Cursor::new(data.clone()))))
			}
			Some(AssetSourceEnum::Stream(receiver)) => {
				let mut receiver = receiver.clone();
				let mut final_data = Vec::new();

				loop {
					if let Ok(state) = receiver.recv_async().await {
						match state {
							AssetDataState::Pending => continue,
							AssetDataState::Ready(data) => final_data.extend(data),
							AssetDataState::Done => break,
						}
					}
				}

				if final_data.is_empty() {
					return Err(AssetReaderError::NotFound(path.to_path_buf()));
				} else {
					final_data
				}
			}
			None => return Err(AssetReaderError::NotFound(path.to_path_buf())),
		};

		self.assets
			.write()
			.await
			.insert(path.to_path_buf(), AssetSourceEnum::Data(data.clone()));
		Ok(Box::new(DataReader(Cursor::new(data))))
	}

	async fn read_meta<'a>(
		&'a self,
		path: &'a Path,
	) -> Result<Box<Reader<'a>>, AssetReaderError> {
		Err(AssetReaderError::NotFound(path.to_path_buf()))
	}

	async fn read_directory<'a>(
		&'a self,
		path: &'a Path,
	) -> Result<Box<PathStream>, AssetReaderError> {
		Err(AssetReaderError::NotFound(path.to_path_buf()))
	}

	async fn is_directory<'a>(
		&'a self,
		path: &'a Path,
	) -> Result<bool, AssetReaderError> {
		Ok(false)
	}
}

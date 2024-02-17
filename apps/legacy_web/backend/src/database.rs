use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Database {
	path: Arc<Mutex<PathBuf>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
	pub users: HashMap<String, User>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
	pub username: String,
	pub avatars: HashMap<String, PathBuf>,
}

impl User {
	pub fn new(username: String) -> Self {
		Self {
			username,
			avatars: HashMap::default(),
		}
	}
}

impl Database {
	pub fn new(path: PathBuf) -> Self {
		if fs::metadata(&path).is_err() {
			let mut dir_path = path.clone();
			dir_path.pop();
			fs::create_dir_all(&dir_path).unwrap();
			File::create(&path).unwrap();
			std::fs::write(
				path.as_path(),
				serde_json::to_string(&Data {
					users: Default::default(),
				})
				.unwrap(),
			)
			.unwrap();
		}

		Self {
			path: Arc::new(Mutex::new(path)),
		}
	}
	pub fn transaction(&mut self, transaction: impl FnOnce(&mut Data)) {
		let path = self.path.lock().unwrap();
		let data = std::fs::read_to_string(path.as_path()).unwrap();
		let mut data: Data = serde_json::from_str(&data).unwrap();
		transaction(&mut data);
		let data = serde_json::to_string(&data).unwrap();
		std::fs::write(path.as_path(), data).unwrap();
	}
}

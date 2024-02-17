mod database;

use crate::database::{Data, Database, User};
use axum::body::Body;
use axum::extract::{DefaultBodyLimit, Path};
use axum::http::{HeaderMap, Request, Response};
use axum::response::IntoResponse;
use axum::{
	extract::Multipart,
	http::StatusCode,
	routing::{get, post},
	Extension, Json, Router,
};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;
use tracing_subscriber::{
	prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt,
};

#[tokio::main]
async fn main() {
	let proj_dirs = ProjectDirs::from("com", "NexusSocial", "NexusVR").unwrap();

	let db_file = proj_dirs.data_dir().join("my_db.db");

	let database = Database::new(db_file);

	let app = Router::new()
		// route for testing if api is running correctly
		.route("/", get(|| async move { "welcome to Image upload api" }))
		//route for uploading image or any file
		.route("/upload/:username", post(upload_file))
		.route("/get_avatars/:username", get(get_avatars))
		.route("/delete_avatar/:username/:avatar_name", get(delete_avatar))
		.route("/get_avatar/:username/:avatar_name", get(get_avatar_file))
		// set your cors config
		.layer(Extension(database))
		.layer(Extension(proj_dirs))
		.layer(DefaultBodyLimit::disable())
		.layer(CorsLayer::permissive());

	let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));
	tracing::debug!("starting server on port: {}", addr.port());
	let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
	axum::serve(listener, app).await.unwrap();
}

#[axum_macros::debug_handler]
async fn upload_file(
	Extension(mut db): Extension<Database>,
	Extension(proj_dirs): Extension<ProjectDirs>,
	Path(username): Path<String>,
	mut files: Multipart,
) {
	ensure_user_in_db(&mut db, &username).await;

	let avatar_path = proj_dirs.data_dir().join("avatars").join(username.clone());
	tokio::fs::create_dir_all(&avatar_path)
		.await
		.expect("unable to create avatar dir");

	while let Some(file) = files.next_field().await.unwrap() {
		// this is the name which is sent in formdata from frontend or whoever called the api, i am
		// using it as category, we can get the filename from file data
		let category = file.name().unwrap().to_string();
		// name of the file with extention
		let name = file.file_name().unwrap().to_string();
		// file data
		let data = file.bytes().await.unwrap();

		let file_path = avatar_path.clone().join(name.clone());

		println!("writing: {:?}", file_path);

		tokio::fs::write(file_path.clone(), data).await.unwrap();

		db.transaction(|data| {
			let user = data.users.get_mut(&username).unwrap();
			user.avatars.insert(name.clone(), file_path);
		});
	}
}

async fn get_avatar_file(
	Extension(mut db): Extension<Database>,
	Path((username, avatar_name)): Path<(String, String)>,
	headers: HeaderMap,
) -> impl IntoResponse {
	let mut path = None;
	db.transaction(|data| {
		path.replace(
			data.users
				.get(&username)
				.unwrap()
				.avatars
				.get(&avatar_name)
				.unwrap()
				.clone(),
		);
	});

	let path = path.unwrap();

	let mut req = Request::new(Body::empty());
	*req.headers_mut() = headers;

	tower_http::services::ServeFile::new(path)
		.try_call(req)
		.await
		.unwrap()
}

async fn get_avatars(
	Extension(mut db): Extension<Database>,
	Path(username): Path<String>,
) -> Json<Vec<String>> {
	let mut names = Arc::new(Mutex::new(None));
	db.transaction(|data| {
		let names2 = data
			.users
			.get(&username)
			.unwrap()
			.avatars
			.keys()
			.map(|key| key.clone())
			.collect::<Vec<_>>();
		names.lock().unwrap().replace(names2);
	});
	let names = names.lock().unwrap().clone().unwrap();
	Json(names)
}

async fn delete_avatar(
	Extension(mut db): Extension<Database>,
	Path((username, avatar_name)): Path<(String, String)>,
) {
	db.transaction(|data| {
		let mut user = data.users.get_mut(&username).unwrap();
		let path = user.avatars.remove(&avatar_name).unwrap();
		std::fs::remove_file(path).unwrap();
	});
}

async fn ensure_user_in_db(db: &mut Database, username: &str) {
	db.transaction(|data| {
		if data.users.get(username).is_none() {
			println!("creating user: {}", username);
			data.users
				.insert(username.to_owned(), User::new(username.to_owned()));
		}
	});
}

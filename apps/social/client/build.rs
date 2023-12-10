use std::env;
use std::path::PathBuf;

#[cfg(target_os = "macos")]
fn main() {
	let mut lib_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
	lib_dir.pop();
	lib_dir.pop();
	lib_dir.pop();
	let lib_dir = lib_dir.join("assets/lib");
	let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

	let loader_path = lib_dir.join("libopenxr_loader.dylib");
	println!("cargo:rerun-if-changed={}", loader_path.display());
	std::fs::copy(loader_path, out_dir.join("libopenxr_loader.dylib"))
		.expect("Failed to copy loader to target dir");
}

#[cfg(not(target_os = "macos"))]
fn main() {}

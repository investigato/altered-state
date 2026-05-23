use std::env;
use std::path::Path;
use fs_extra::dir::{copy, CopyOptions};

fn main() {

	println!("cargo:rerun-if-changed=wwwroot");

	let out_dir = env::var("OUT_DIR").unwrap(); // e.g., target/debug/build/...
	// To copy to the actual executable folder (target/debug or target/release):
	let target_dir = Path::new(&out_dir);

	let mut options = CopyOptions::new();
	options.overwrite = true;

	copy("wwwroot", &target_dir, &options).expect("Failed to copy wwwroot");
	copy("config.json", &out_dir, &options).expect("Failed to copy config.json");
}

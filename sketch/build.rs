use std::env;

fn main() {
	for name in [
		"CARGO_CFG_TARGET_FAMILY",
		"CARGO_CFG_TARGET_OS",
		"CARGO_CFG_TARGET_ARCH",
	] {
		let value = env::var(name).unwrap();
		println!("cargo:rustc-env={name}={value}");
	}
}

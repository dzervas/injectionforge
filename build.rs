use std::env;
#[cfg(feature = "dotnet")]
use csbindgen;

fn main() {
	println!("cargo:rerun-if-env-changed=FRIDA_CODE");
	println!("cargo:rerun-if-env-changed=DLL_PROXY");

	if let Ok(lib_path) = env::var("DLL_PROXY") {
		println!("cargo:rerun-if-changed={}", &lib_path);
		use goblin::Object::{self, PE};

		let path = std::path::Path::new(&lib_path);
		let lib_filename = path.file_name().unwrap().to_str().unwrap();

		let lib_bytes = std::fs::read(path).expect(format!("Failed to open given library file {}", &lib_filename).as_str());
		let object = Object::parse(&lib_bytes).expect(format!("Failed to parse given libary file {}", &lib_filename).as_str());

		let (exports, lib_name): (Vec<&str>, String) = match object {
			PE(o) =>
				(o.exports
					.iter()
					.map(|e| e.name.unwrap())
					.collect(),
				o.name.unwrap().replace(".dll", "")),
			_ => {
				println!("Only DLL files are supported");
				std::process::exit(1);
			},
		};

		for e in exports.iter() {
			println!("cargo:warning=Exported function: {} => {}-orig.{}", e, lib_name, e);
			println!("cargo:rustc-link-arg=/export:{}={}-orig.{}", e, lib_name, e);
		}
		println!("cargo:warning=Expected library name: {}-orig.dll", lib_name);
		println!("cargo:rustc-env=LIB_NAME={}-orig.dll", lib_name);
	}

	#[cfg(feature = "dotnet")]
	{
		let lib_path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs");
		let csharp_file = concat!(env!("CARGO_MANIFEST_DIR"), "/dotnet/NativeMethods.g.cs");
		csbindgen::Builder::default()
			.input_extern_file(lib_path)
			.csharp_dll_name("deepfreeze")
			.generate_csharp_file(csharp_file)
			.unwrap();
	}
}

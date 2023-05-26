use std::env;

fn main() {
	println!("cargo:rerun-if-env-changed=FRIDA_CODE");
	println!("cargo:rerun-if-env-changed=FRIDA_CODE_FILE");
	println!("cargo:rerun-if-env-changed=DLL_PROXY");

	if let Ok(code_file) = env::var("FRIDA_CODE_FILE") {
		env::set_var("FRIDA_CODE", &std::fs::read_to_string(&code_file).unwrap());
		println!("cargo:warning=Using code from file: {}", &code_file);
	} else if env::var("FRIDA_CODE").is_ok() {
		println!("cargo:warning=Using code from environment variable: FRIDA_CODE");
	} else {
		println!("Please set FRIDA_CODE or FRIDA_CODE_FILE environment variable");
		std::process::exit(1);
	}

	if let Ok(lib_path) = env::var("DLL_PROXY") {
		use goblin::Object::{self, PE};

		let path = std::path::Path::new(&lib_path);
		let lib_filename = path.file_name().unwrap().to_str().unwrap();

		let lib_bytes = std::fs::read(path).expect(format!("Failed to open given library file {}", &lib_filename).as_str());
		let object = Object::parse(&lib_bytes).expect(format!("Failed to parse given libary file {}", &lib_filename).as_str());

		let (exports, lib_name): (Vec<&str>, String) = match object {
			PE(o) =>
				(o.exports
					.iter()
					.map(|e| e.name.unwrap().clone())
					.collect(),
				o.name.unwrap().replace(".dll", "")),
			_ => {
				println!("Only DLL files are supported");
				std::process::exit(1);
			},
		};

		for e in exports.iter() {
			println!("cargo:warning=Exported function: {}", e);
			// println!("cargo:rustc-link-lib=dylib={}-orig", lib_name);
			println!("cargo:rustc-link-arg=/export:{}={}-orig.{}", e, lib_name, e);
		}
		println!("cargo:warning=Expected library name: {}-orig.dll", lib_name);
	}
}

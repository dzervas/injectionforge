use std::env;

fn main() {
	println!("cargo:rerun-if-env-changed=FRIDA_CODE");
	println!("cargo:rerun-if-env-changed=FRIDA_CODE_FILE");
	println!("cargo:rerun-if-env-changed=LIB_PROXY");

	if let Ok(code_file) = env::var("FRIDA_CODE_FILE") {
		env::set_var("FRIDA_CODE", &std::fs::read_to_string(&code_file).unwrap());
		println!("cargo:warning=Using code from file: {}", &code_file);
	} else if env::var("FRIDA_CODE").is_ok() {
		println!("cargo:warning=Using code from environment variable: FRIDA_CODE");
	} else {
		println!("Please set FRIDA_CODE or FRIDA_CODE_FILE environment variable");
		std::process::exit(1);
	}

	if let Ok(lib_path) = env::var("LIB_PROXY") {
		use goblin::Object::{self, Elf, PE, Mach, Archive, Unknown};
		// use goblin::mach::{MultiArch, MachO};

		let path = std::path::Path::new(&lib_path);
		let lib_name = path.file_name().unwrap().to_str().unwrap();

		let lib_bytes = std::fs::read(path).expect(format!("Failed to open given library file {}", &lib_name).as_str());
		let object = Object::parse(&lib_bytes).expect(format!("Failed to parse given libary file {}", &lib_name).as_str());

		let exports: Vec<&str> = match object {
			Elf(o) =>
				o.dynsyms
					.iter()
					.filter(|e| !e.is_import())
					.map(|e| o.dynstrtab.get_at(e.st_name).unwrap())
					.collect(),
			PE(o) =>
				o.exports
					.iter()
					.map(|e| e.name.unwrap().clone())
					.collect(),
			Mach(_o) => {
				println!("Mach binaries are not supported yet");
				std::process::exit(1);
			},
			Archive(_o) => {
				println!("Archive files are not supported");
				std::process::exit(1);
			},
			Unknown(_o) => {
				println!("The file you provided is of unknown format");
				std::process::exit(1);
			},
		};

		for e in exports.iter() {
			println!("cargo:warning=Exported function: {}", e);
			// println!("cargo:rustc-link-lib=dylib=orig.{}", lib_name);
			println!("cargo:rustc-link-arg=/export:{}=orig.{}.{}", e, lib_name, e);
		}
	}
}

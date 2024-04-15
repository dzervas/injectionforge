use serde::Deserialize;

use std::env;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Hook {
	pub name: String,
	pub args: Vec<String>,
	pub ret: String,
	pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
	pub dll_proxy: Option<String>,
	pub frida_code: Option<String>,
	pub frida_code_file: Option<String>,
	pub target_process: Option<String>,
	pub target_spawn: Option<String>,

	pub hooks: Vec<Hook>,
}

impl Default for Config {
	fn default() -> Self {
		println!("cargo:rerun-if-env-changed=FRIDA_CODE");
		println!("cargo:rerun-if-env-changed=FRIDA_CODE_FILE");
		println!("cargo:rerun-if-env-changed=DLL_PROXY");
		println!("cargo:rerun-if-env-changed=TARGET_PROCESS");
		println!("cargo:rerun-if-env-changed=TARGET_SPAWN");

		Self {
			dll_proxy      : env::var("DLL_PROXY").ok(),
			frida_code     : env::var("FRIDA_CODE").ok(),
			frida_code_file: env::var("FRIDA_CODE_FILE").ok(),
			target_process : env::var("TARGET_PROCESS").ok(),
			target_spawn   : env::var("TARGET_SPAWN").ok(),

			hooks: Vec::new(),
		}
	}
}

impl Config {
	pub fn get_frida_code(&self) -> String {
		if self.frida_code.is_some() && self.frida_code_file.is_some() {
			panic!("Both frida_code and frida_code_file set. Please set one of them.");
		}

		let code = if let Some(frida_code) = &self.frida_code {
			frida_code.clone()
		} else if let Some(frida_code_file) = &self.frida_code_file {
			std::fs::read_to_string(frida_code_file).expect("Failed to read frida_code_file")
		} else {
			panic!("No frida_code or frida_code_file set. Please set one of them.");
		};

		code.replace("\n", "\\n")
	}

	pub fn populate_env(&self) {
		if let Some(target_process) = &self.target_process {
			println!("cargo:rustc-env=TARGET_PROCESS={target_process}");
		}

		if let Some(target_spawn) = &self.target_spawn {
			println!("cargo:rustc-env=TARGET_SPAWN={target_spawn}");
		}

		println!(r#"cargo:rustc-env=FRIDA_CODE={}"#, self.get_frida_code());
	}
}

fn main() {
	println!("cargo:rerun-if-env-changed=CONFIG_FILE");

	let config = if let Ok(config_file) = env::var("CONFIG_FILE") {
		let config_file = std::fs::read_to_string(config_file).expect("Failed to read CONFIG_FILE");
		toml::from_str::<Config>(&config_file).expect("Failed to parse CONFIG_FILE")
	} else {
		Config::default()
	};

	config.populate_env();

	let Some(lib_path) = config.dll_proxy else {
		println!("cargo:warning=No dll_proxy set, the resulting library has to be manually injected or compiled into the target binary/process");
		return;
	};

	dll_proxy_linker_flags(&lib_path, config.hooks);
}

fn dll_proxy_linker_flags(lib_path: &str, _hooks: Vec<Hook>) {
	use goblin::Object;

	println!("cargo:rerun-if-changed={lib_path}");

	let path = Path::new(&lib_path);
	let lib_filename = path.file_name().unwrap().to_str().unwrap();

	let lib_bytes = std::fs::read(path).expect(format!("Failed to open given library file {lib_filename}").as_str());
	let object = Object::parse(&lib_bytes).expect(format!("Failed to parse given libary file {lib_filename}").as_str());

	let (exports, lib_name): (Vec<&str>, String) = match object {
		Object::PE(o) => {
			(o.exports
				.iter()
				.map(|e| e.name.unwrap())
				.collect(),
			o.name.expect("Couldn't read the name of the DLL. Is it a .NET DLL? It's not supported").replace(".dll", ""))
		}
		Object::Elf(o) => {
			(o.dynsyms
				.iter()
				.filter(|e| e.is_function() && !e.is_import())
				.map(|e| o.dynstrtab.get_at(e.st_name).unwrap())
				.collect(),
			// o.soname.expect("Couldn't read the name of the SO.").replace(".so", ""))
			lib_filename.replace(".so", ""))
		},
		// Object::Mach(goblin::mach::Mach::Binary(o)) => {
		// 	(o.dynsyms
		// 		.iter()
		// 		.filter(|e| e.is_function() && !e.is_import())
		// 		.map(|e| o.dynstrtab.get_at(e.st_name).unwrap())
		// 		.collect(),
		// 	o.name.expect("Couldn't read the name of the DLL. Is it a .NET DLL? It's not supported").replace(".dll", ""))
		// },
		_ => {
			println!("Only PE (.dll) and ELF (.so) files are supported in their respective target platforms.");
			std::process::exit(1);
		},
	};

	if build_target::target_os() == Ok(build_target::Os::Windows) {
		for e in exports.iter() {
			println!("cargo:warning=Exported function: {e} => {lib_name}-orig.{e}");
			println!("cargo:rustc-link-arg=/export:{e}={lib_name}-orig.{e}");
		}
	// } else {
		// let link_lib_name = if lib_name.starts_with("lib") {
		// 	lib_name.replacen("lib", "", 1)
		// } else {
		// 	lib_name.clone()
		// };

		// let lib_dir = path.parent().unwrap().to_str().unwrap();

		// let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
		// std::fs::copy("test.x", out.join("test.x")).unwrap();
		// println!("cargo:rustc-link-search={}", out.display());

		// println!("cargo:rustc-link-lib={link_lib_name}");
		// println!("cargo:rustc-link-search={lib_dir}");
		// println!("cargo:rustc-link-arg=-l{link_lib_name}");
		// println!("cargo:rustc-link-arg=-Wl,--no-as-needed");
		// println!("cargo:rustc-link-arg=-Wl,--just-symbols={lib_path}");

		// for e in exports.iter() {
		// 	println!("cargo:warning=Re-exporting function: {e} => {lib_name}-orig.{e}");
		// 	println!("cargo:rustc-link-arg=-Wl,--wrap={e}");
		// }
	}

	println!("cargo:warning=Expected library name: {lib_name}-orig.dll");
	println!("cargo:rustc-env=LIB_NAME={lib_name}-orig.dll");
}

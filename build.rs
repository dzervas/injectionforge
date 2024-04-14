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
	pub target_exec: Option<String>,
	pub target_process: Option<String>,

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
			target_exec    : env::var("TARGET_SPAWN").ok(),
			target_process : env::var("TARGET_PROCESS").ok(),

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
}

fn main() {
	println!("cargo:rerun-if-env-changed=CONFIG_FILE");

	let config = if let Ok(config_file) = env::var("CONFIG_FILE") {
		let config_file = std::fs::read_to_string(config_file).expect("Failed to read CONFIG_FILE");
		toml::from_str::<Config>(&config_file).expect("Failed to parse CONFIG_FILE")
	} else {
		Config::default()
	};

	println!(r#"cargo:rustc-env=FRIDA_CODE={}"#, config.get_frida_code());

	let Some(lib_path) = config.dll_proxy else {
		println!("cargo:warning=No dll_proxy set, the resulting library has to be manually injected or compiled into the target binary/process");
		return;
	};

	if build_target::target_os() != Ok(build_target::Os::Windows) {
		panic!("Dll proxying mode is only supported on Windows.");
	}

	dll_proxy_linker_flags(&lib_path, config.hooks);
}

fn dll_proxy_linker_flags(lib_path: &str, _hooks: Vec<Hook>) {
	use goblin::Object;

	println!("cargo:rerun-if-changed={lib_path}");

	let path = Path::new(&lib_path);
	let lib_filename = path.file_name().unwrap().to_str().unwrap();

	let lib_bytes = std::fs::read(path).expect(format!("Failed to open given library file {lib_filename}").as_str());
	let object = Object::parse(&lib_bytes).expect(format!("Failed to parse given libary file {lib_filename}").as_str());

	let Object::PE(pe) = object else {
		panic!("Only PE (.dll) files are supported in this mode.");
	};

	let exports: Vec<&str> = pe.exports.iter().map(|e| e.name.unwrap()).collect();
	let lib_name = pe.name.expect("Couldn't read the name of the DLL. Is it a .NET DLL? It's not supported").replace(".dll", "");

	for e in exports.iter() {
		println!("cargo:warning=Exported function: {e} => {lib_name}-orig.{e}");
		println!("cargo:rustc-link-arg=/export:{e}={lib_name}-orig.{e}");
	}

	println!("cargo:warning=Expected library name: {lib_name}-orig.dll");
	println!("cargo:rustc-env=LIB_NAME={lib_name}-orig.dll");
}

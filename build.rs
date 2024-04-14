use std::env;
use std::path::Path;

fn main() {
	println!("cargo:rerun-if-env-changed=FRIDA_CODE");
	println!("cargo:rerun-if-env-changed=FRIDA_CODE_FILE");
	println!("cargo:rerun-if-env-changed=DLL_PROXY");

	if env::var("FRIDA_CODE").is_err() && env::var("FRIDA_CODE_FILE").is_err() {
		panic!("No FRIDA_CODE or FRIDA_CODE_FILE set. Please set one of them.");
	}

	if env::var("FRIDA_CODE").is_ok() && env::var("FRIDA_CODE_FILE").is_ok() {
		panic!("Both FRIDA_CODE and FRIDA_CODE_FILE set. Please set one of them.");
	}

	if let Ok(frida_code_file) = env::var("FRIDA_CODE_FILE") {
		let frida_code = std::fs::read_to_string(frida_code_file).expect("Failed to read FRIDA_CODE_FILE");
		println!("cargo:rustc-env=FRIDA_CODE={}", frida_code);
	}

	let Ok(lib_path) = env::var("DLL_PROXY") else {
		println!("cargo:warning=No DLL_PROXY set, the resulting library has to be manually injected or compiled into the target binary/process");
		return;
	};

	if build_target::target_os() != Ok(build_target::Os::Windows) {
		panic!("Dll proxying mode is only supported on Windows.");
	}

	use goblin::Object;
	println!("cargo:rerun-if-changed={}", &lib_path);

	let path = Path::new(&lib_path);
	let lib_filename = path.file_name().unwrap().to_str().unwrap();

	let lib_bytes = std::fs::read(path).expect(format!("Failed to open given library file {}", &lib_filename).as_str());
	let object = Object::parse(&lib_bytes).expect(format!("Failed to parse given libary file {}", &lib_filename).as_str());

	let Object::PE(pe) = object else {
		panic!("Only PE (.dll) files are supported in this mode.");
	};

	let exports: Vec<&str> = pe.exports.iter().map(|e| e.name.unwrap()).collect();
	let lib_name = pe.name.expect("Couldn't read the name of the DLL. Is it a .NET DLL? It's not supported").replace(".dll", "");

	for e in exports.iter() {
		println!("cargo:warning=Exported function: {} => {}-orig.{}", e, lib_name, e);
		println!("cargo:rustc-link-arg=/export:{}={}-orig.{}", e, lib_name, e);
	}

	println!("cargo:warning=Expected library name: {}-orig.dll", lib_name);
	println!("cargo:rustc-env=LIB_NAME={}-orig.dll", lib_name);
}

use std::env;
use std::io::Write;
use std::fs::File;
use std::path::Path;
#[cfg(feature = "dotnet")]
use csbindgen;

fn main() {
	println!("cargo:rerun-if-env-changed=FRIDA_CODE");
	println!("cargo:rerun-if-env-changed=DLL_PROXY");

	let Ok(lib_path) = env::var("DLL_PROXY") else {
		return;
	};

	use goblin::Object;
	println!("cargo:rerun-if-changed={}", &lib_path);

	let path = std::path::Path::new(&lib_path);
	let lib_filename = path.file_name().unwrap().to_str().unwrap();

	let lib_bytes = std::fs::read(path).expect(format!("Failed to open given library file {}", &lib_filename).as_str());
	let object = Object::parse(&lib_bytes).expect(format!("Failed to parse given libary file {}", &lib_filename).as_str());

	let (exports, lib_name): (Vec<&str>, String) = match object {
		#[cfg(target_os = "windows")]
		Object::PE(o) => {
			(o.exports
				.iter()
				.map(|e| e.name.unwrap())
				.collect(),
			o.name.expect("Couldn't read the name of the DLL. Is it a .NET DLL? It's not supported").replace(".dll", ""))
		}
		#[cfg(target_os = "linux")]
		Object::Elf(o) => {
			(o.dynsyms
				.iter()
				.filter(|e| e.is_function() && !e.is_import())
				.map(|e| o.dynstrtab.get_at(e.st_name).unwrap())
				.collect(),
			o.soname.expect("Couldn't read the name of the SO.").replace(".so", ""))
		},
		// #[cfg(target_os = "darwin")]
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

	#[cfg(target_os = "windows")]
	for e in exports.iter() {
		println!("cargo:warning=Exported function: {} => {}-orig.{}", e, lib_name, e);
		println!("cargo:rustc-link-arg=/export:{}={}-orig.{}", e, lib_name, e);
	}

	#[cfg(not(target_os = "windows"))]
	{
		let symbols_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("symbols.rs");
		let mut symbols = File::create(&symbols_path).unwrap();
		println!("cargo:rerun-if-changed={:?}", symbols_path);

		writeln!(symbols, "#[allow(dead_code)]").unwrap();
		writeln!(symbols, "#[link(name = \"{}\")]", lib_name.replace("lib", "")).unwrap();
		writeln!(symbols, "extern {{").unwrap();
		for e in exports.iter() {
			println!("cargo:warning=Exported function: {}", e);
			// writeln!(symbols, "\t#[no_mangle]").unwrap();
			writeln!(symbols, "\tpub fn {}();", e).unwrap();
		}
		writeln!(symbols, "}}").unwrap();
	}

	println!("cargo:warning=Expected library name: {}-orig.dll", lib_name);
	println!("cargo:rustc-env=LIB_NAME={}-orig.dll", lib_name);

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

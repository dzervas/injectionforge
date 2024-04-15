pub mod injector;
#[cfg(feature = "frida")]
pub mod frida_handler;

pub use injector::attach_self;

// During testing we compile a debug binary without `test`.
// Enabling `ctor` during testing would hook the test runner and break it.
#[cfg(all(unix, not(test)))]
pub mod loader_unix;

#[cfg(all(windows, not(test)))]
pub mod loader_windows;
#[cfg(all(windows, not(test)))]
pub use loader_windows::DllMain;

#[cfg(test)]
mod tests {
	use pretty_assertions::assert_eq;
	use std::process::Command;

	#[allow(dead_code)]
	fn get_lib_name(name: &str) -> String {
		#[cfg(target_os = "windows")]
		return format!("{}.dll", name);

		#[cfg(target_os = "linux")]
		return format!("lib{}.so", name);

		#[cfg(target_os = "darwin")]
		return format!("lib{}.dylib", name);
	}

	#[test]
	fn test_frida_on_load() {
		let lib_status = Command::new("cargo")
			.arg("build")
			.arg("--lib")
			.arg("--target-dir")
			.arg("target/test_frida_on_load")
			.env("FRIDA_CODE", r#"
const foo = Module.getExportByName(null, "mylib_foo");
Interceptor.replace(foo, new NativeCallback(function () {
	console.log("replaced foo() called");
	return 40;
}, "uint8", []));
			"#)
			.status()
			.unwrap();

		assert!(lib_status.success(), "Failed to build dynamic library");

		let bin_status = Command::new("cargo")
			.arg("run")
			.arg("--manifest-path")
			.arg("tests/mybin/Cargo.toml")
			.arg("--target-dir")
			.arg("target/test_frida_on_load")
			// We're compiling in the same target dir so that frida_deepfreeze_rs is found
			// We'd have to copy it to the target dir otherwise
			.env("RUSTFLAGS", "-C link-arg=-Wl,--no-as-needed -C link-arg=-lfrida_deepfreeze_rs")
			.status()
			.unwrap();

		assert_eq!(bin_status.code().unwrap(), 40, "Failed to replace foo()");
	}

	#[test]
	#[cfg(windows)]
	fn test_frida_dll_proxy() {
		use std::fs;

		let mylib_name = get_lib_name("mylib");
		fs::remove_file(format!("target/test_frida_dll_proxy/debug/deps/{}", mylib_name)).unwrap_or_else(|_| ());

		let mylib_status = Command::new("cargo")
			.arg("build")
			.arg("--lib")
			.arg("--manifest-path")
			.arg("tests/mylib/Cargo.toml")
			.arg("--target-dir")
			.arg("target/test_frida_dll_proxy/mylib")
			.status()
			.unwrap();
		assert!(mylib_status.success(), "Failed to build mylib");

		// fs::copy(format!("target/test_frida_dll_proxy/mylib/debug/{}", get_lib_name("mylib")), format!("target/test_frida_dll_proxy/lib/debug/deps/{}", get_lib_name("mylib-orig"))).expect("Failed to rename original DLL");
		let lib_status = Command::new("cargo")
			.arg("build")
			.arg("--lib")
			.arg("--target-dir")
			.arg("target/test_frida_dll_proxy/lib")
			.env("DLL_PROXY", format!("target/test_frida_dll_proxy/mylib/debug/{mylib_name}"))
			.env("FRIDA_CODE", r#"
				const foo = Module.getExportByName(null, "mylib_foo");
				Interceptor.replace(foo, new NativeCallback(function () {
					console.log("replaced foo() called");
					return 40;
				}, "uint8", []));
			"#)
			.status()
			.unwrap();

		assert!(lib_status.success(), "Failed to build dynamic library");

		let target_dir = "target/test_frida_dll_proxy/mybin/debug/deps/";
		fs::copy(format!("target/test_frida_dll_proxy/mylib/debug/{}", get_lib_name("mylib")), format!("{target_dir}{}", get_lib_name("mylib-orig"))).expect("Failed to rename original DLL");
		fs::copy(format!("target/test_frida_dll_proxy/lib/debug/{}", get_lib_name("frida_deepfreeze_rs")), format!("{target_dir}{}", get_lib_name("mylib"))).expect("Failed to rename deepfreeze DLL");

		let bin_status = Command::new("cargo")
			.arg("run")
			.arg("--manifest-path")
			.arg("tests/mybin/Cargo.toml")
			.arg("--target-dir")
			.arg("target/test_frida_dll_proxy/mybin")
			// .env("RUSTFLAGS", "-C link-arg=-Wl,--no-as-needed -C link-arg=-lmylib")
			.status()
			.unwrap();

		assert_eq!(bin_status.code().unwrap(), 40, "Failed to replace foo()");
	}
}

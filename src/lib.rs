pub mod injector;
#[cfg(feature = "frida")]
pub mod frida_handler;

#[cfg(symbols)]
pub mod symbols;
#[cfg(symbols)]
pub use symbols::*;

pub use injector::attach_self;

#[cfg(all(unix, not(test)))]
use ctor::ctor;

// During testing we compile a debug binary without `test`.
// Enabling `ctor` during testing would hook the test runner and break it.
#[cfg(all(unix, not(test)))]
#[ctor]
fn _start() {
	println!("[+] frida-deepfreeze-rs library injected");
	attach_self();
}

// For some reason ctor doesn't work on Windows - it hangs the process
// during DeviceManager::obtain. DllMain works fine though.
#[cfg(all(windows, not(test)))]
use std::ffi::c_void;
#[cfg(all(windows, not(test)))]
use winapi::um::winnt::DLL_PROCESS_ATTACH;

#[cfg(all(windows, not(test)))]
use winapi::um::libloaderapi::LoadLibraryA;

#[cfg(all(windows, not(test)))]
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "system" fn DllMain(dll_module: *mut c_void, call_reason: u32, _: *mut ()) -> bool {
	match call_reason {
		DLL_PROCESS_ATTACH => {
			println!("[+] frida-deepfreeze-rs DLL injected");

			if let Some(lib_name) = option_env!("LIB_NAME") {
				unsafe { LoadLibraryA(lib_name.as_ptr() as *const i8); }
				println!("[+] Original DLL {} loaded", lib_name);
			}

			attach_self();
		}
		// Maybe we should detach? Is it useful?
		_ => ()
	}

	true
}

#[cfg(test)]
mod tests {
	use pretty_assertions::assert_eq;
	use std::process::Command;
	use std::fs;

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
			.env("RUSTFLAGS", "-C link-arg=-Wl,--no-as-needed -C link-arg=-lfrida_deepfreeze_rs")
			.status()
			.unwrap();

		assert_eq!(bin_status.code().unwrap(), 40, "Failed to replace foo()");
	}

	#[test]
	fn test_frida_dll_proxy() {
		let mylib_name = get_lib_name("mylib");
		fs::remove_file(format!("target/test_frida_dll_proxy/debug/deps/{}", mylib_name)).unwrap_or_else(|_| ());

		let mylib_status = Command::new("cargo")
			.arg("build")
			.arg("--lib")
			.arg("--manifest-path")
			.arg("tests/mylib/Cargo.toml")
			.arg("--target-dir")
			.arg("target/test_frida_dll_proxy")
			.status()
			.unwrap();
		assert!(mylib_status.success(), "Failed to build mylib");

		let lib_status = Command::new("cargo")
			.arg("build")
			.arg("--lib")
			.arg("--target-dir")
			.arg("target/test_frida_dll_proxy")
			.env("DLL_PROXY", format!("target/test_frida_dll_proxy/debug/deps/{}", mylib_name))
			.env("RUSTFLAGS", "-C link-arg=-Wl,--no-as-needed -C link-arg=-lmylib")
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

		let target_dir = "target/test_frida_dll_proxy/debug/deps/";
		fs::rename(format!("{}{}", target_dir, get_lib_name("mylib")), format!("{}{}", target_dir, get_lib_name("mylib-orig"))).expect("Failed to rename original DLL");
		fs::rename(format!("{}{}", target_dir, get_lib_name("frida_deepfreeze_rs")), format!("{}{}", target_dir, get_lib_name("mylib"))).expect("Failed to rename deepfreeze DLL");

		let bin_status = Command::new("cargo")
			.arg("run")
			.arg("--manifest-path")
			.arg("tests/mybin/Cargo.toml")
			.arg("--target-dir")
			.arg("target/test_frida_dll_proxy")
			.status()
			.unwrap();

		assert_eq!(bin_status.code().unwrap(), 40, "Failed to replace foo()");
	}
}

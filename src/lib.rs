pub mod injector;
#[cfg(feature = "frida")]
pub mod frida_handler;
// #[cfg(feature = "dotnet")]
// pub mod cs;
#[cfg(not(windows))]
pub mod symbols;

pub use injector::attach_self;

#[cfg(all(unix, not(test), not(feature = "dotnet")))]
use ctor::ctor;

#[cfg(all(unix, not(test), not(feature = "dotnet")))]
#[ctor]
fn _start() {
	println!("[+] frida-deepfreeze-rs library injected");
	attach_self();
}

// For some reason ctor doesn't work on Windows - it hangs the process
// during DeviceManager::obtain. DllMain works fine though.
#[cfg(all(any(windows, feature = "dotenv"), not(test)))]
use std::ffi::c_void;
#[cfg(all(any(windows, feature = "dotenv"), not(test)))]
use winapi::um::winnt::DLL_PROCESS_ATTACH;

#[cfg(all(any(windows, feature = "dotenv"), not(test)))]
use winapi::um::libloaderapi::LoadLibraryA;

#[cfg(all(any(windows, feature = "dotenv"), not(test)))]
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "system" fn DllMain(dll_module: *mut c_void, call_reason: u32, _: *mut ()) -> bool {
	match call_reason {
		DLL_PROCESS_ATTACH => {
			println!("[+] frida-deepfreeze-rs DLL injected");

			unsafe { LoadLibraryA(env!("LIB_NAME").as_ptr() as *const i8); }
			println!("[+] Original DLL {} loaded", env!("LIB_NAME"));

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
	#[cfg(all(windows, feature = "frida"))]
	use std::fs;

	#[test]
	#[cfg(feature = "frida")]
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
			.expect("Failed to build dynamic library");

		assert!(lib_status.success(), "Failed to build dynamic library");

		let bin_status = Command::new("cargo")
			.arg("run")
			.arg("--manifest-path")
			.arg("tests/mybin/Cargo.toml")
			.arg("--target-dir")
			.arg("target/test_frida_on_load")
			.env("RUSTFLAGS", "-C link-arg=-Wl,--no-as-needed -C link-arg=-lfrida_deepfreeze_rs")
			.status()
			.expect("Failed to build mybin");

		assert_eq!(bin_status.code().unwrap(), 40, "Failed to replace foo()");
	}

	#[test]
	#[cfg(all(windows, feature = "frida"))]
	fn test_frida_on_load() {
		let bin_exec = Command::new("cargo")
			.arg("build")
			.arg("--manifest-path")
			.arg("tests/mybin/Cargo.toml")
			.arg("--target-dir")
			.arg("target/test_frida_on_load");

		let lib_status = Command::new("cargo")
			.arg("build")
			.arg("--lib")
			.arg("--target-dir")
			.arg("target/test_frida_on_load")
			.env("DLL_PROXY", "target/test_frida_on_load/debug/deps/mylib.dll")
			.env("FRIDA_CODE", r#"
				const foo = Module.getExportByName(null, "mylib_foo");
				Interceptor.replace(foo, new NativeCallback(function () {
					console.log("replaced foo() called");
					return 40;
				}, "uint8", []));
			"#)
			.status()
			.expect("Failed to build dynamic library");

		assert!(lib_status.success(), "Failed to build dynamic library");

		fs::rename("target/test_frida_on_load/debug/deps/mylib.dll", "target/test_frida_on_load/debug/mylib-orig.dll").expect("Failed to rename original DLL");
		fs::rename("target/test_frida_on_load/debug/frida_deepfreeze_rs.dll", "target/test_frida_on_load/debug/mylib.dll").expect("Failed to rename deepfreeze DLL");
		let bin_status = bin_exec.status().expect("Failed to build mybin");
		assert_eq!(bin_status.code().unwrap(), 40, "Failed to replace foo()");
	}
}

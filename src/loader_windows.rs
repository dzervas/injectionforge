use std::ffi::c_void;
use winapi::um::winnt::DLL_PROCESS_ATTACH;
use winapi::um::libloaderapi::LoadLibraryA;

// For some reason ctor doesn't work on Windows - it hangs the process
// during DeviceManager::obtain. DllMain works fine though.
// Would be nice to have a single entry point for all platforms.
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

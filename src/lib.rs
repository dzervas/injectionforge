pub mod injector;
#[cfg(feature = "frida")]
pub mod frida_handler;

pub use injector::{attach, attach_self};

#[cfg(all(unix, not(test)))]
use ctor::ctor;

#[cfg(all(unix, not(test)))]
#[ctor]
fn _start() {
	println!("[+] frida-deepfreeze-rs library injected");
	attach_self();
}

// For some reason ctor doesn't work on Windows - it hangs the process
// during DeviceManager::obtain. DllMain works fine though.
#[cfg(windows)]
use std::ffi::c_void;
#[cfg(windows)]
use winapi::um::winnt::DLL_PROCESS_ATTACH;

#[cfg(all(windows, feature = "dll_proxy"))]
use winapi::um::libloaderapi::LoadLibraryA;

#[cfg(all(windows, not(test)))]
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(dll_module: *mut c_void, call_reason: u32, _: *mut ()) -> bool {
	match call_reason {
		DLL_PROCESS_ATTACH => {
			println!("[+] frida-deepfreeze-rs DLL injected");

			#[cfg(feature = "dll_proxy")]
			{
				unsafe { LoadLibraryA(env!("LIB_NAME").as_ptr() as *const i8); }
				println!("[+] Original DLL {} loaded", env!("LIB_NAME"));
			}

			attach_self();
		}
		// Maybe we should detach? Is it useful?
		_ => ()
	}

	true
}

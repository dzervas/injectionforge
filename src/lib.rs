pub mod injector;

pub use injector::{attach, attach_self};

#[cfg(unix)]
use ctor::ctor;

#[cfg(unix)]
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
#[cfg(windows)]
use winapi::um::libloaderapi::LoadLibraryA;

#[cfg(windows)]
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(dll_module: *mut c_void, call_reason: u32, _: *mut ()) -> bool {
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

pub mod injector;

pub use injector::{inject, inject_self};

// #[cfg(unix)]
use ctor::ctor;

// #[cfg(unix)]
#[ctor]
fn _start() {
	println!("[+] frida-deepfreeze-rs SO injected");
	inject_self();
}

/*
#[cfg(windows)]
use std::ptr;
#[cfg(windows)]
use std::ffi::c_void;
#[cfg(windows)]
use winapi::um::libloaderapi::{DllMain, DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH};

#[allow(non_snake_case)]
#[cfg(windows)]
#[no_mangle]
pub extern "system" fn DllMain(hinstDLL: *mut c_void, fdwReason: u32, _: *mut c_void) -> i32 {
	match fdwReason {
		DLL_PROCESS_ATTACH => {
			println!("[+] frida-deepfreeze-rs DLL injected");
			inject_self();
		}
		// DLL_PROCESS_DETACH => {}
		// DLL_THREAD_ATTACH => {}
		// DLL_THREAD_DETACH => {}
		_ => {}
	}

	1
}
*/

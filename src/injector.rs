
#[cfg(all(unix, not(feature = "frida")))]
compile_error!("Only Frida injection is supported for Unix targets");

#[cfg(all(not(feature = "managed_lib"), not(feature = "frida")))]
compile_error!("No injection method is selected - please enable either managed_lib (windows-only) and/or frida feature");

#[cfg(all(not(windows), feature = "managed_lib"))]
compile_error!("Managed library injection is only supported for Windows target");

#[cfg(feature = "frida")]
use crate::frida_handler::attach_pid as frida_attach_pid;

#[no_mangle]
pub extern "C" fn attach(pid: u32) {
	#[cfg(feature = "frida")]
	{
		let frida_code = env!("FRIDA_CODE").to_string();
		std::thread::spawn(move || frida_attach_pid(frida_code, pid));
	}
}

#[no_mangle]
pub extern "C" fn attach_self() {
	println!("[*] Attaching to self");
	attach(0);
}

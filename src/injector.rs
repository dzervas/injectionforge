
#[cfg(all(not(feature = "frida")))]
compile_error!("No injection method is selected - please enable either dotnet (windows-only) and/or frida feature");

#[cfg(feature = "frida")]
use crate::frida_handler::attach_with as frida_attach_with;
use crate::frida_handler::AttachMode;

#[no_mangle]
pub extern "C" fn attach(pid: u32) {
	#[cfg(feature = "frida")]
	{
		let frida_code = env!("FRIDA_CODE").replace("\\n", "\n");
		#[cfg(windows)]
		std::thread::spawn(move || frida_attach_pid(&frida_code, AttachMode::Pid(pid)));
		#[cfg(not(windows))]
		frida_attach_with(&frida_code, AttachMode::Pid(pid));
	}
}

#[no_mangle]
pub extern "C" fn attach_name(name: *const u8, len: usize) {
	let name_str = unsafe {
		let buf = std::slice::from_raw_parts(name, len);
		std::str::from_utf8(buf).expect("Invalid UTF-8 in process name")
	};

	#[cfg(feature = "frida")]
	{
		let frida_code = env!("FRIDA_CODE").replace("\\n", "\n");
		#[cfg(windows)]
		std::thread::spawn(move || frida_attach_with(&frida_code, AttachMode::Name(name_str.to_string())));
		#[cfg(not(windows))]
		frida_attach_with(&frida_code, AttachMode::Name(name_str.to_string()));
	}
}

#[no_mangle]
pub extern "C" fn attach_self() {
	println!("[*] Attaching to self");
	attach(0);
}

#[no_mangle]
pub extern "C" fn spawn(name: *const u8, len: usize) {
	let name_str = unsafe {
		let buf = std::slice::from_raw_parts(name, len);
		std::str::from_utf8(buf).expect("Invalid UTF-8 in spawn name")
	};

	println!("[*] Spawning: {name_str}");

	#[cfg(feature = "frida")]
	{
		let frida_code = env!("FRIDA_CODE").replace("\\n", "\n");
		#[cfg(windows)]
		std::thread::spawn(move || frida_attach_with(&frida_code, AttachMode::Spawn(name_str.to_string())));
		#[cfg(not(windows))]
		frida_attach_with(&frida_code, AttachMode::Spawn(name_str.to_string()));
	}
}

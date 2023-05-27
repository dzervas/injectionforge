use frida::{DeviceManager, Frida, ScriptHandler, ScriptOption, ScriptRuntime};
use lazy_static::lazy_static;

lazy_static! {
	static ref FRIDA: Frida = unsafe { Frida::obtain() };
}

#[no_mangle]
pub fn attach(pid: u32) {
	let frida_code = env!("FRIDA_CODE").to_string();
	println!("[*] Injecting into PID: {}", pid);

	std::thread::spawn(move || {
		let device_manager = DeviceManager::obtain(&FRIDA);
		println!("[*] Device Manager obtained");

		if let Some(device) = device_manager.enumerate_all_devices().first() {
			println!("[*] First device: {}", device.get_name());

			let session = device.attach(pid).unwrap();

			if !session.is_detached() {
				println!("[*] Attached");

				let mut script_option = ScriptOption::new()
					.set_name("frida-deepfreeze-rs")
					.set_runtime(ScriptRuntime::QJS);
				println!("[*] Script {}", frida_code);
				let script = session
					.create_script(&frida_code, &mut script_option)
					.unwrap();

				script.handle_message(&mut Handler).unwrap();

				script.load().unwrap();
				println!("[*] Script loaded");
			}
		} else {
			println!("[!] No device found!");
		};
	});
}

#[no_mangle]
pub fn attach_self() {
	println!("[*] Attaching to self");
	// #[cfg(windows)]
	// attach(std::process::id());
	// #[cfg(unix)]
	attach(0);
}

struct Handler;

impl ScriptHandler for Handler {
	fn on_message(&mut self, message: &str) {
		eprintln!("[<] {message}");
	}
}

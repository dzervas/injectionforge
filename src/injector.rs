use frida::{DeviceManager, Frida, ScriptHandler, ScriptOption, ScriptRuntime};
use lazy_static::lazy_static;

lazy_static! {
	static ref FRIDA: Frida = unsafe { Frida::obtain() };
}

const FRIDA_CODE: &str = env!("FRIDA_CODE", "Please set FRIDA_CODE environment variable");

#[no_mangle]
pub fn inject(pid: u32) {
	let device_manager = DeviceManager::obtain(&FRIDA);

	if let Some(device) = device_manager.enumerate_all_devices().first() {
		println!("[*] First device: {}", device.get_name());

		let session = device.attach(pid).unwrap();

		if !session.is_detached() {
			println!("[*] Attached");

			let mut script_option = ScriptOption::new()
				// .set_name("frida-deepfreeze-rs")
				.set_runtime(ScriptRuntime::QJS);
			let script = session
				.create_script(FRIDA_CODE, &mut script_option)
				.unwrap();

			script.handle_message(&mut Handler).unwrap();

			script.load().unwrap();
			println!("[*] Script loaded");

			script.unload().unwrap();
			println!("[*] Script unloaded");

			session.detach().unwrap();
			println!("[*] Session detached");
		}
	};
}

#[no_mangle]
pub fn inject_self() {
	println!("[*] Attaching to self (pid 0)");
	inject(0);
}

struct Handler;

impl ScriptHandler for Handler {
	fn on_message(&mut self, message: &str) {
		println!("[<] {message}");
	}
}

#![cfg(feature = "frida")]

use frida::{DeviceManager, DeviceType, Frida, ScriptHandler, ScriptOption, ScriptRuntime, SpawnOptions, Message, MessageLogLevel};
use lazy_static::lazy_static;

lazy_static! {
	pub static ref FRIDA: Frida = unsafe { Frida::obtain() };
}

#[derive(Debug)]
pub enum AttachMode {
	Pid(u32),
	Name(String),
	Spawn(String),
}

pub fn attach_with(frida_code: &str, mode: AttachMode) {
	println!("[+] Injecting into: {mode:?}");

	let device_manager = DeviceManager::obtain(&FRIDA);
	println!("[*] Device Manager obtained");

	if let Ok(mut device) = device_manager.get_device_by_type(DeviceType::Local) {
		println!("[*] First device: {}", device.get_name());

		let pid = match mode {
			AttachMode::Pid(pid) => pid,
			AttachMode::Name(ref name) => {
				device.enumerate_processes().iter()
					.find(|p| p.get_name() == name)
					.expect("Process not found")
					.get_pid()
			},
			AttachMode::Spawn(ref name) => device.spawn(name, &SpawnOptions::new()).unwrap(),
		};

		let session = device.attach(pid).unwrap();

		println!("[*] Attached");

		let mut script_option = ScriptOption::new()
			.set_runtime(ScriptRuntime::QJS);
		println!("[*] Script {}", frida_code);
		let mut script = session
			.create_script(frida_code, &mut script_option)
			.unwrap();

		script.handle_message(Handler).unwrap();

		script.load().unwrap();
		println!("[*] Script loaded");

		if let AttachMode::Spawn(_) = mode {
			device.resume(pid).unwrap();
			println!("[*] Resuming spawned process")
		}
	} else {
		eprintln!("[!] No device found!");
	};
}

struct Handler;

impl ScriptHandler for Handler {
	fn on_message(&mut self, message: &Message, _data: Option<Vec<u8>>) {
		if let Message::Log(log_entry) = message {
			match log_entry.level {
				MessageLogLevel::Debug => eprint!("[-] "),
				MessageLogLevel::Info => eprint!("[i] "),
				MessageLogLevel::Warning => eprint!("[!] "),
				MessageLogLevel::Error => eprint!("[X] "),
			}

			eprintln!("{}", log_entry.payload);
			return;
		}

		eprintln!("{:?}", message);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use pretty_assertions::assert_eq;

	#[link(name = "mylib", kind = "dylib")]
	extern {
		fn mylib_foo() -> u8;
	}

	#[test]
	fn test_attach_pid() {
		assert_eq!(10, unsafe { mylib_foo() });

		let frida_script = r#"
			const foo = Module.getExportByName(null, "mylib_foo");
			Interceptor.replace(foo, new NativeCallback(function () {
				console.log("replaced foo() called");
				return 20;
			}, "uint8", []));
		"#;

		attach_with(frida_script, AttachMode::Pid(0));
		assert_eq!(20, unsafe { mylib_foo() });
	}
}

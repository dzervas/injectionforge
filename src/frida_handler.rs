#![cfg(feature = "frida")]
use frida::{DeviceManager, Frida, ScriptHandler, ScriptOption, ScriptRuntime};
use lazy_static::lazy_static;
use serde::Deserialize;

lazy_static! {
	pub static ref FRIDA: Frida = unsafe { Frida::obtain() };
}

pub fn attach_pid(frida_code: String, pid: u32) {
	println!("[+] Injecting into PID: {}", pid);

	let device_manager = DeviceManager::obtain(&FRIDA);
	println!("[*] Device Manager obtained");

	if let Some(device) = device_manager.enumerate_all_devices().first() {
		println!("[*] First device: {}", device.get_name());

		let session = device.attach(pid).unwrap();

		if !session.is_detached() {
			println!("[*] Attached");

			let mut script_option = ScriptOption::new()
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
		eprintln!("[!] No device found!");
	};
}

#[derive(Debug, Deserialize)]
struct LogEntry {
	#[serde(rename = "type")]
	log_type: LogType,
	level: LogLevel,
	payload: String,
}

#[derive(Debug, Deserialize)]
enum LogType {
	#[serde(rename = "log")]
	Log,
}

#[derive(Debug, Deserialize)]
enum LogLevel {
	#[serde(rename = "debug")]
	Debug,
	#[serde(rename = "info")]
	Info,
	#[serde(rename = "warning")]
	Warning,
	#[serde(rename = "error")]
	Error,
}

struct Handler;

impl ScriptHandler for Handler {
	fn on_message(&mut self, message: &str) {
		if let Ok(log_entry) = serde_json::from_str::<LogEntry>(message) {
			match log_entry.log_type {
				LogType::Log => {
					match log_entry.level {
						LogLevel::Debug => eprint!("[-] "),
						LogLevel::Info => eprint!("[i] "),
						LogLevel::Warning => eprint!("[!] "),
						LogLevel::Error => eprint!("[X] "),
					}
				}
			}

			eprintln!("{}", log_entry.payload);
			return;
		}

		eprintln!("{message}");
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

		attach_pid(frida_script.to_string(), 0);
		assert_eq!(20, unsafe { mylib_foo() });
	}
}

#![cfg(feature = "frida")]

use frida::{DeviceManager, DeviceType, Frida, ScriptHandler, ScriptOption, ScriptRuntime, SpawnOptions};
use lazy_static::lazy_static;
use serde::Deserialize;

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
		let script = session
			.create_script(frida_code, &mut script_option)
			.unwrap();

		script.handle_message(&mut Handler).unwrap();

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

		attach_with(frida_script, AttachMode::Pid(0));
		assert_eq!(20, unsafe { mylib_foo() });
	}
}

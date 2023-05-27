use frida::{DeviceManager, Frida, ScriptHandler, ScriptOption, ScriptRuntime};
use serde::Deserialize;
use lazy_static::lazy_static;

lazy_static! {
	static ref FRIDA: Frida = unsafe { Frida::obtain() };
}

#[no_mangle]
pub fn attach(pid: u32) {
	let frida_code = env!("FRIDA_CODE").to_string();
	println!("[+] Injecting into PID: {}", pid);

	std::thread::spawn(move || {
		let device_manager = DeviceManager::obtain(&FRIDA);
		println!("[*] Device Manager obtained");

		if let Some(device) = device_manager.enumerate_all_devices().first() {
			println!("[*] First device: {}", device.get_name());

			let session = device.attach(pid).unwrap();

			if !session.is_detached() {
				println!("[*] Attached");

				let mut script_option = ScriptOption::new()
					// .set_name("frida-deepfreeze-rs")
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
	});
}

#[no_mangle]
pub fn attach_self() {
	println!("[*] Attaching to self");
	attach(0);
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

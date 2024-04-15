pub mod injector;
#[cfg(feature = "frida")]
pub mod frida_handler;

pub use injector::*;

fn main() {
	let args: Vec<String> = std::env::args().collect();

	if args.len() >= 2 {
		if let Ok(pid) = args[1].parse() {
			attach(pid);
			return;
		} else {
			attach_name(args[1].as_ptr(), args[1].len());
			return;
		}
	} else if let Some(spawn_path) = option_env!("TARGET_SPAWN") {
		spawn(spawn_path.as_ptr(), spawn_path.len());
		return;
	} else if let Some(process_name) = option_env!("TARGET_PROCESS") {
		attach_name(process_name.as_ptr(), process_name.len());
		return;
	}

	eprintln!("Usage: {} <PID|Process Name>", args[0]);
}

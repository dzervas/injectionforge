pub mod injector;
pub use injector::attach;

fn main() {
	let args: Vec<String> = std::env::args().collect();

	if args.len() < 2 {
		println!("Usage: {} <PID>", args[0]);
		return;
	}

	let pid: u32 = args[1].parse().unwrap();
	attach(pid);
}

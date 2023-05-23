pub mod injector;

pub use injector::{inject, inject_self};

use ctor::ctor;

#[ctor]
fn _start() {
	println!("[+] frida-deepfreeze-rs SO injected");
	inject_self();
}

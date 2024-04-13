use ctor::ctor;

use crate::injector::attach_self;

#[ctor]
fn _start() {
	println!("[+] frida-deepfreeze-rs library injected");
	attach_self();
}

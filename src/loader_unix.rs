#![cfg(all(unix, not(test)))]
use ctor::ctor;

use crate::injector::attach_self;

#[ctor]
fn _start() {
	println!("[+] InjectionForge library injected");
	attach_self();
}

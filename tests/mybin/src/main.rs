#[link(name = "mylib", kind = "dylib")]
extern {
	fn mylib_foo() -> u8;
	fn mylib_bar() -> u8;
}

fn main() {
	println!("Hello, world!");

	assert_eq!(unsafe { mylib_bar() }, 100);
	std::process::exit(unsafe { mylib_foo() } as i32);
}

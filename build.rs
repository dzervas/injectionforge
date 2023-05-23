use std::env;

fn main() {
	// Set the environment variable
	env::set_var("MY_STRING", "Hello, world!");

	if let Ok(code_file) = env::var("FRIDA_CODE_FILE") {
		env::set_var("FRIDA_CODE", &std::fs::read_to_string(&code_file).unwrap());
		println!("cargo:warning=Using code from file: {}", &code_file);
	} else if env::var("FRIDA_CODE").is_ok() {
		println!("cargo:warning=Using code from environment variable: FRIDA_CODE");
	} else {
		println!("cargo:error=Please set FRIDA_CODE or FRIDA_CODE_FILE environment variable");
		std::process::exit(1);
	}
}

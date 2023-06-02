use cs_eval::{EvalContext, EvalError};

fn cs_inject() {
	// Define your C# code to be compiled
	let csharp_code = r#"
		using System;

		public class Program
		{
			public static void Main()
			{
				Console.WriteLine("Hello, C#!");
			}
		}
	"#;

	// Create an evaluation context
	let mut context = EvalContext::new();

	// Compile and execute the C# code
	match context.eval::<()>(csharp_code) {
		Ok(_) => {
			println!("C# code executed successfully");
		}
		Err(EvalError::CompilationError(err)) => {
			println!("Compilation error: {:?}", err);
		}
		Err(EvalError::ExecutionError(err)) => {
			println!("Execution error: {:?}", err);
		}
	}
}

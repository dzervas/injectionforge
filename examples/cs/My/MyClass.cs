using System;
using OtherNamespace;

namespace MyNamespace {
	public class MyClass {
		// This method will be called by native code inside the target process…
		public static int MyMethod(String pwzArgument) {
			System.Console.WriteLine("Hello World from C# {0}", pwzArgument);
			return 0;
		}

		public static void Main() {
			int my = MyMethod("from Main()");
			System.Console.WriteLine("MyMethod returned {0}", my);
			int other = OtherNamespace.OtherClass.OtherMethod();
			System.Console.WriteLine("OtherMethod returned {0}", other);
		}
	}
}

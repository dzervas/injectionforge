using System;

namespace OtherNamespace {
	public class OtherClass {
		// This method will be called by native code inside the target process…
		public static int OtherMethod() {
			System.Console.WriteLine("Goodbye World from C#");
			return 1;
		}
	}
}

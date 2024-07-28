# InjectionForge

<img align="right" height="300" src=".github/logo.png" alt="InjectionForge logo" />

Have you ever written a frida script this good, that you wanted to make it permanent?
Well, now you can!

InjectionForge is a tool that allows you to convert your frida scripts into
either a standalone executable that when called with a PID injects itself and runs
the script or a shared library that can be somehow injected to a process and runs
the script.

All desktop platforms are supported (Windows, Linux, macOS).

**NOTE**: To cross-compile for Windows you can use [cargo-xwin](https://github.com/rust-cross/cargo-xwin)
with target `x86_64-pc-windows-msvc`.

## Usage

You're gonna have to compile the tool yourself as the frida script gets embedded
at compile time.

You only need a working cargo installation to compile it, it's quite simple.

You can feed your script either as a string using the `FRIDA_CODE` environment
variable or as a file using the `FRIDA_CODE_FILE` environment variable.

### Standalone executable

The standalone executable is the easiest to use. You just run it with a PID and
it will inject itself and run the frida script.

```bash
git clone https://github.com/dzervas/injectionforge
FRIDA_CODE='console.log("Hello world from InjectionForge!")' cargo run --bin standalone -- 1234
```

The binary is located at `target/debug/standalone` (`.exe` for windows).

### Shared library

The shared library is a bit more complicated to use. You have to inject it to
a process using a tool like `LD_PRELOAD` (linux) or `rundll32.exe` (windows).

```bash
git clone https://github.com/dzervas/injectionforge
FRIDA_CODE='console.log("Hello world from InjectionForge!")' cargo build --lib
LD_PRELOAD=target/debug/libfrida_deepfreeze_rs.so cat
# rundll32.exe target/debug/frida_deepfreeze_rs.dll,inject_self 1234 (windows equivalent)
```

The resulting library is located at `target/debug/libfrida_deepfreeze_rs.so`
(`.dll` for windows). You can inject it using your favorite injector.

There are two exported functions that you can call from the library to inject:

```c
void inject(uint32_t pid); // Run the frida script in the process with the given pid
void inject_self(); // Run the frida script in the process that called the function
```

By default (so `DllMain` in windows and `.ctor` on unix), on load the library
will call `inject_self()` so you can just inject it and it will run the script.

### DLL Proxying

There's also the option of generating a DLL ready for DLL Proxying use.
That means that you give the DLL `myawesome.dll` to cargo
(using the `DLL_PROXY` environment variable) and it will generate a DLL
`myawesome.dll` that can replace the original DLL. It will tell the linker
that any functions found during compilation (e.g. functions `foo` and `bar`
exported by the original `myawesome.dll`) should be redirected to `myawesome-orig.dll`

That allows you to make your script completely permanent without having to
run any extra commands.

**NOTE**: This only works on Windows (for now?).

```bash
git clone https://github.com/dzervas/injectionforge
DLL_PROXY='../myawesome.dll' FRIDA_CODE='console.log("Hello world from InjectionForge!")' cargo xwin build --lib --target x86_64-pc-windows-msvc
```

## Android and anti-anti-frida

Since most people ask about Android and anti-anti-frida techniques,
I created some dockerfiles to help with that.

To just wrap a frida script in a shared library that can be injected to an Android
process (or APK repacking):

```bash
git clone https://github.com/dzervas/injectionforge
cd injectionforge
docker build -t injectionforge-android -f Dockerfile.android
docker run -e FRIDA_CODE_FILE=/script.js -v $(pwd)/target:/injectionforge/target -v $(pwd)/myscript.js:/script.js injectionforge-android
```

(be sure to change the path to `myscript.js`)

To use a patched frida to evade some basic anti-frida techniques
(based on [undetected-frida-patches](https://github.com/ultrafunkamsterdam/undetected-frida-patches/)):

```bash
git clone https://github.com/dzervas/injectionforge
cd injectionforge
docker build -t injectionforge-android -f Dockerfile.android
docker build -t injectionforge-android-undetect -f Dockerfile.android-undetect
docker run -e FRIDA_CODE_FILE=/script.js -v $(pwd)/target:/injectionforge/target -v $(pwd)/myscript.js:/script.js injectionforge-android-undetect
```

During the build of `Dockerfile.android` you can pass args to specify the
NDK version and more (check the Dockerfile).

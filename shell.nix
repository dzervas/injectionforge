with import <nixpkgs> {};
mkShell {
  nativeBuildInputs = [
    rustup
    cargo-xwin
    pkg-config
    llvmPackages.libclang
  ];

  # Got from https://discourse.nixos.org/t/setting-up-a-nix-env-that-can-compile-c-libraries/15833
  shellHook = ''
    export LIBCLANG_PATH="${llvmPackages.libclang.lib}/lib";
    export BINDGEN_EXTRA_CLANG_ARGS="$(< ${stdenv.cc}/nix-support/libc-crt1-cflags) \
      $(< ${stdenv.cc}/nix-support/libc-cflags) \
      $(< ${stdenv.cc}/nix-support/cc-cflags) \
      $(< ${stdenv.cc}/nix-support/libcxx-cxxflags) \
      ${
        lib.optionalString stdenv.cc.isClang
        "-idirafter ${stdenv.cc.cc}/lib/clang/${
          lib.getVersion stdenv.cc.cc
        }/include"
      } \
      ${
        lib.optionalString stdenv.cc.isGNU
        "-isystem ${stdenv.cc.cc}/include/c++/${
          lib.getVersion stdenv.cc.cc
        } -isystem ${stdenv.cc.cc}/include/c++/${
          lib.getVersion stdenv.cc.cc
        }/${stdenv.hostPlatform.config} -idirafter ${stdenv.cc.cc}/lib/gcc/${stdenv.hostPlatform.config}/${
          lib.getVersion stdenv.cc.cc
        }/include"
      } \
    "
  '';
}

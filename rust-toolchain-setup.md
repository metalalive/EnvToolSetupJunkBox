# Rust Toolchain Setup
version: 1.75.0

## Build
### prepare
Prerequisite
- `gcc >= 10.3.0`
- `cmake >= 3.13.4`, you can upgrade `cmake` by manually replacing old pre-built binaries with new ones [downloaded from here](https://github.com/Kitware/CMake/)
- `python3`, I used `python 3.12`

Clone repository from Rust official github

Ensure following dependencies exist, they've been in separate github repositories, and might be lost if you get source files by downloading `.tar.gz` compressed package
- `src/tools/cargo`
- `src/llvm-project`
  - note this codebase might be huge, `git` may keep reporting errors for unreliable / slow network , if that happenes to you, the alternative is to  download the compressed bundle `.zip` or `.tar.gz` drectly , however the bundle might miss some source files and you may end up manually copying them from remote git repo.
- `library/stdarch`
- `library/backtrace` 

### start compiling
- generate config template `config.toml`  by running `python3 ./x.py setup`
- modify the `config.toml`
  ```yaml
  # this links to all possible default configurations for different use cases
  # check `src/bootstrap/defaults` for detail
  profile = "dist"
  changelog-seen = 2
  
  [llvm]
  targets = "AArch64;ARM;RISCV;WebAssembly;X86"
  cflags   = "-mno-avx512f  -mno-avx512pf"
  cxxflags = "-mno-avx512f  -mno-avx512pf"

  [target.x86_64-unknown-linux-gnu]
  cc  = "/PATH/TO/gcc/installed/bin/gcc"
  cxx = "/PATH/TO/gcc/installed/bin/g++"
  ar  = "/PATH/TO/gcc/installed/bin/gcc-ar"
  ranlib = "/PATH/TO/gcc/installed/bin/gcc-ranlib"
  linker = "/PATH/TO/gcc/installed/bin/gcc"

  [build]
  build-dir = "my-build"
  python = "python3.12"
  extended = true
  tools = [
     "cargo", # package management
     "clippy", # linter
     "rustfmt", # code formatter
     "rust-analyzer", # for IDE language server
  ]
  verbose = 1
  profiler = false # will cause static library linking error if enable , FIXME

  configure-args = []

  [install]
  prefix = "/PATH/TO/tools/installed"

  [rust]
  ```
- build compilers (stage 0 to stage 2), utilities, and standard libraries
  ```bash
  python3 ./x.py build
  ```
  - depend on the computing capability of your CPU,  it takes about 1-2 hours or might be longer
  - the build process completed when you see the binaries `rustc`, `cargo` generated and the message at the end
    ```
    Finished release [optimized] target(s) in xxxx ...
    ```


### Hardware constraint
If your Intel CPU does NOT support [AVX-512 instruction set](https://en.wikipedia.org/wiki/AVX-512), avoid the build tool from compiling assembly files
- for C source files, add flags like `-mno-avx512f` and `-mno-avx512pf`
- for x86 assembly file, which only happenes in `blake3_avx512_x86-64_unix.S` under the path `/PATH/TO/RUST/COMPILER/REPO/src/llvm-project/llvm/lib/Support/BLAKE3/`, don't add it to source list in `src/llvm-project/llvm/lib/Support/BLAKE3/CMakeLists.txt`, this also works because `blake3_avx512_x86-64_unix.S` is optional.

### TODO
- verify cross-compiler toolchain for AArch64 and RISC-V platforms once built successfully

## Reference
- [Rust Compiler Development Guide](https://rustc-dev-guide.rust-lang.org/building/how-to-build-and-run.html)

# Rust Toolchain Setup
version: 1.75.0

## Workflow
### prepare
Prerequisite
- `gcc >= 10.3.0`
- `cmake >= 3.13.4`, you can upgrade `cmake` by manually replacing old pre-built binaries with new ones [downloaded from here](https://github.com/Kitware/CMake/)
- `python3`, I used `python 3.12`

Clone repository from Rust official github

Ensure following dependencies exist, they've been in separate github repositories, and might be lost if you get source files by downloading `.tar.gz` compressed package
- `src/tools/cargo`
- `src/llvm-project`
  - Note it is a big codebase, `git` may report errors for unreliable / slow network , if that happenes to you, the alternative is to  download the compressed bundle `.zip` or `.tar.gz` directly , however the bundle might miss some source files and you may end up manually copying them from remote git repo.
- `library/stdarch`
- `library/backtrace` 

### configuration
- generate config template `config.toml`  by running `python3 ./x.py setup`
- modify the `config.toml`
  ```toml
  # this links to all possible default configurations for different usage scenarios
  # check `src/bootstrap/defaults` for detail
  profile = "dist"
  
  # please refer to config.example.toml , the change ID should match the rust version
  # (latest PR for the release)
  change-id = 116881
  
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
  docs = false # don't build doc for saving time

  extended = true # more tools to build (specified below)
  tools = [
     "cargo", # package management
     "clippy", # linter
     "rustfmt", # code formatter
     "rust-analyzer", # for IDE language server
  ]
  profiler = false # will cause static library linking error if enable , FIXME

  verbose = 1  # or `2` , very verbose for more error detail

  configure-args = []

  [install]
  prefix = "/PATH/TO/tools/installed"
  sysconfdir = "./conf" # relative path to `install.prefix` above

  [rust]
  channel = "stable" # if it is `nightly`, test will report errors (TODO), recheck and figure out

  ```
### Build
build compilers (stage 0 to stage 2), utilities, and standard libraries
```bash
python3 ./x.py build
```
- depend on hardware computing capability,  the build process takes about 3 to 4 hours or even longer
- the build completes when you see the binaries `rustc`, `cargo` generated and the message at the end
```
Finished release [optimized] target(s) in xxxx ...
```

Run test
```bash
python3 ./x.py  test
```
- it takes about 1 hour to run all 15k test cases
- TODO, figure out how to run specific test case (instead of running all of them every single time)

Finally installation
```bash
python3 ./x.py  install
```
- Be sure you have access permission to installed folder

### Hardware constraint
If your Intel CPU does NOT support [AVX-512 instruction set](https://en.wikipedia.org/wiki/AVX-512), modify the build script by following :
- for C source files, add flags like `-mno-avx512f` and `-mno-avx512pf`
- for x86 assembly file, go to `/PATH/TO/RUST/src/llvm-project/llvm/lib/Support/BLAKE3/`, modify the CMake file `./CMakeLists.txt` by : 
  - discard `blake3_avx512_x86-64_unix.S` from source list in the CMake file.
  - add compile definition `BLAKE3_NO_AVX512` , then `black3` will skip the functions related to `intel avx512` feature in the build.

### TODO
- verify cross-compiler toolchain for AArch64 and RISC-V platforms once built successfully

## Reference
- [Rust Compiler Development Guide](https://rustc-dev-guide.rust-lang.org/building/how-to-build-and-run.html)
- [Linux From Scratch -- Ch.13 programming tool, Rust](https://www.linuxfromscratch.org/blfs/view/svn/general/rust.html)

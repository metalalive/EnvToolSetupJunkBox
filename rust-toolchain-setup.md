
## Build
### prepare
  - clone repository from Rust official github
  - `gcc >= 10.3.0`
  - `cmake >= 3.13.4`, you can upgrade `cmake` by manually replacing old pre-built binaries with new ones [downloaded from here](https://github.com/Kitware/CMake/)
  - `python3`, I used `python 3.9.6`
  - Before `1.68.0` You might need to manually clone some dependency tools
    - `src/tools/rust-installer`
    - `src/tools/cargo` 
  - If your Intel CPU does NOT support [AVX-512 instruction set](https://en.wikipedia.org/wiki/AVX-512), avoid the build tool from compiling assembly files
    - for C source files, add flags like `-mno-avx512f` and `-mno-avx512pf`
    - for x86 assembly file, which only happenes in `blake3_avx512_x86-64_unix.S` under the path `/PATH/TO/RUST/COMPILER/REPO/src/llvm-project/llvm/lib/Support/BLAKE3/`, don't add it to source list in `src/llvm-project/llvm/lib/Support/BLAKE3/CMakeLists.txt`, this also works because `blake3_avx512_x86-64_unix.S` is optional.

### start compiling
- generate config template `config.toml`  by running `python3 ./x.py setup`
- modify the `config.toml`
  ```yaml
  profile = "user1234"
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
  verbose = 1
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

### TODO
- cross-compiler toolchain for AArch64 and RISC-V platforms

## Reference
- [Rust Compiler Development Guide](https://rustc-dev-guide.rust-lang.org/building/how-to-build-and-run.html)

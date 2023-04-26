use std::env;
use std::process::Command;
use std::path::PathBuf;

// compile a few C files placed in this Cargo project, a typical build
// process of a C project may include Makefiles or CMake script, to let
// application simply run the command `make`/`cmake` to build C library.
fn main() {
    let outdir = PathBuf::from(env::var("OUT_DIR").unwrap()); 
    let c_src_dir = PathBuf::from("c_src");
    let src_names = vec!["example" ,"arith"];
    let f1 = |item:&&str| -> (String,String) {
        (format!("{item}.c"), format!("{item}.o")) 
    };
    let f2 = |item:&&str| -> PathBuf {
        let oname = format!("{item}.o");
        outdir.join(oname)
    };
    let pair_iter = src_names.iter().map(f1);
    for (src_name, obj_name) in pair_iter {
        let src_path = c_src_dir.join(src_name);
        let obj_path = outdir.join(obj_name);
        // ---- COmpile option `-fPIC` explanation in GCC doc 10.3 -----
        // If supported for the target machine, emit position-independent code,
        // suitable for dynamic linking and avoiding any limit on the size of
        // the global offset table. This option makes a difference on x86, 
        // AArch64, m68k, PowerPC and SPARC.
        let gcc_out = Command::new("gcc") .arg("-c") .arg("-O0") .arg("-fPIC") .arg("-g")
            .arg("-std=c17") .arg("-o") .arg(&obj_path) .arg(&src_path)
            .output() .expect("Failed to compile C source");
        if !gcc_out.status.success() {
            println!("gcc failed: {}", String::from_utf8_lossy(&gcc_out.stderr));
            std::process::exit(1);
        }
    } // end of loop
    let lib_path = outdir.join("libeexampaul.so");
    let mut linkcmd = Command::new("gcc");
    let mut halfdone = linkcmd .arg("-shared").arg("-o").arg(&lib_path);
    for obj_path in src_names.iter().map(f2) {
        halfdone = halfdone.arg(&obj_path); 
    }
    let gcc_out = halfdone.output() .expect("Failed to link library");
    if !gcc_out.status.success() {
        println!("gcc failed: {}", String::from_utf8_lossy(&gcc_out.stderr));
        std::process::exit(1);
    }
    println!("cargo:rustc-link-search=native={}", outdir.display());
    println!("cargo:rustc-link-lib=dylib=eexampaul");
} // end of main

/*
 *
    let gcc_out = Command::new("gcc")
        .arg("-c") .arg("-O0") .arg("-g") .arg("-o")
        .arg(&c_obj_path) .arg(&c_src_path)
        .output()  .expect("Failed to compile C code");
    let gcc_out = Command::new("gcc-ar")
        .arg("rcs").arg(&lib_path)  .arg(&c_obj_path)
        .output()  .expect("Failed to generate library");
    if !gcc_out.status.success() {
        println!("gcc failed: {}", String::from_utf8_lossy(&gcc_out.stderr));
        std::process::exit(1);
    }
    println!("cargo:rustc-link-lib=static=eexampaul");
 * */

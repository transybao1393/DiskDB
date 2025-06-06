use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to rerun this if the C files change
    println!("cargo:rerun-if-changed=src/native/src/arena.c");
    println!("cargo:rerun-if-changed=src/native/src/parser.c");
    println!("cargo:rerun-if-changed=src/native/src/slab_allocator.c");
    println!("cargo:rerun-if-changed=src/native/src/memory_pool.c");
    println!("cargo:rerun-if-changed=src/native/include/arena.h");
    println!("cargo:rerun-if-changed=src/native/include/parser.h");
    println!("cargo:rerun-if-changed=src/native/include/memory_pool.h");
    
    // Build the C parser and memory library
    cc::Build::new()
        .file("src/native/src/arena.c")
        .file("src/native/src/parser.c")
        .file("src/native/src/slab_allocator.c")
        .file("src/native/src/memory_pool.c")
        .include("src/native/include")
        .opt_level(3)
        .flag_if_supported("-march=native")
        .flag_if_supported("-fPIC")
        .flag_if_supported("-pthread")
        .warnings(true)
        .compile("diskdb_native");
    
    // Tell cargo where to find the library
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=diskdb_native");
    
    // Link pthread for thread-local storage
    println!("cargo:rustc-link-lib=pthread");
}
use std::env;
use std::path::PathBuf;

use cc::Build;
use cmake::Config;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=wrapper.cpp");
    println!("cargo:rerun-if-changed=../CMakeLists.txt");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let dst = Config::new(manifest_dir.parent().unwrap())
        .build_target("chiapos_static")
        .define("BUILD_STATIC_CHIAPOS_LIBRARY", "ON")
        .build();

    let blake3_include_path = dst.join("build").join("_deps").join("blake3-src").join("c");

    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("build").to_str().unwrap()
    );

    println!("cargo:rustc-link-lib=static=chiapos_static");

    Build::new()
        .cpp(true)
        .file(manifest_dir.join("wrapper.cpp"))
        .include(
            manifest_dir
                .parent()
                .unwrap()
                .join("lib")
                .join("include")
                .to_str()
                .unwrap(),
        )
        .include(blake3_include_path.to_str().unwrap())
        .std("c++14")
        .compile("wrapper");

    let bindings = bindgen::Builder::default()
        .header(manifest_dir.join("wrapper.h").to_str().unwrap())
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg(format!(
            "-I{}",
            manifest_dir
                .parent()
                .unwrap()
                .join("lib")
                .join("include")
                .to_str()
                .unwrap()
        ))
        .clang_arg(format!("-I{}", blake3_include_path.to_str().unwrap()))
        .clang_arg("-std=c++14")
        .allowlist_function("something")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

use std::path::PathBuf;

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").expect("Cargo target OS");
    if target_os != "macos" && target_os != "linux" {
        panic!("Shitty 0.0.1 supports macOS and Linux; target OS is {target_os}");
    }
    let sdk = std::env::var("SOKSAK_SHITTY_VT_SDK")
        .ok()
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .expect("SOKSAK_SHITTY_VT_SDK must declare the vterm-c SDK directory");
    let library = sdk.join("lib");
    for archive in ["libshitty_vt.a", "libplt_headless.a", "libstd.a"] {
        let path = library.join(archive);
        assert!(
            path.is_file(),
            "required Shitty SDK archive is missing: {}",
            path.display()
        );
        println!("cargo:rerun-if-changed={}", path.display());
    }
    assert!(
        sdk.join("include/vterm_c.h").is_file(),
        "Shitty SDK header is missing"
    );
    println!("cargo:rustc-link-search=native={}", library.display());
    println!("cargo:rustc-link-lib=static=shitty_vt");
    println!("cargo:rustc-link-lib=static=plt_headless");
    println!("cargo:rustc-link-lib=static=std");
    if target_os == "macos" {
        println!("cargo:rustc-link-lib=dylib=c++");
    } else {
        println!("cargo:rustc-link-lib=dylib=stdc++");
        println!("cargo:rustc-link-lib=dylib=pthread");
        println!("cargo:rustc-link-lib=dylib=xxhash");
    }
    println!("cargo:rerun-if-env-changed=SOKSAK_SHITTY_VT_SDK");
}

use std::path::{Path, PathBuf};

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").expect("Cargo target OS");
    if target_os != "macos" && target_os != "linux" {
        panic!("Shitty supports macOS and Linux; target OS is {target_os}");
    }
    let target = std::env::var("TARGET").expect("Cargo target");
    let root = PathBuf::from(
        std::env::var("SOKSAK_BUILD_DEPENDENCY_ROOT")
            .expect("make build supplies SOKSAK_BUILD_DEPENDENCY_ROOT"),
    );
    assert!(root.is_absolute(), "build dependency root must be absolute");
    assert_eq!(
        std::fs::canonicalize(&root).expect("canonical build dependency root"),
        root,
        "build dependency root must not use a symbolic path"
    );
    let receipt_path = root.join("receipts").join(format!("{target}.json"));
    let receipt: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&receipt_path).expect("read Shitty build dependency receipt"),
    )
    .expect("parse Shitty build dependency receipt");
    assert_eq!(receipt["schema"], "soksak-build-dependency-receipt-v1");
    assert_eq!(receipt["dependency"], "shitty-vt-sdk");
    assert_eq!(receipt["target"], target);
    let tree = receipt["outputs"]
        .as_array()
        .expect("receipt outputs")
        .iter()
        .find(|output| output["type"] == "tree")
        .and_then(|output| output["path"].as_str())
        .expect("Shitty receipt tree output");
    let sdk = root.join(Path::new(tree));
    assert_eq!(
        std::fs::canonicalize(&sdk).expect("canonical Shitty SDK"),
        sdk,
        "Shitty SDK must not use a symbolic path"
    );
    let library = sdk.join("lib");
    for archive in ["libshitty_vt.a", "libplt_headless.a", "libstd.a"] {
        let path = library.join(archive);
        assert!(
            path.is_file(),
            "required Shitty SDK archive is missing: {}",
            path.display()
        );
        assert_eq!(
            std::fs::canonicalize(&path).expect("canonical Shitty SDK archive"),
            path,
            "Shitty SDK archive must not use a symbolic path"
        );
        println!("cargo:rerun-if-changed={}", path.display());
    }
    let header = sdk.join("include/vterm_c.h");
    assert!(header.is_file(), "Shitty SDK header is missing");
    assert_eq!(
        std::fs::canonicalize(&header).expect("canonical Shitty SDK header"),
        header,
        "Shitty SDK header must not use a symbolic path"
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
    println!("cargo:rerun-if-changed={}", header.display());
    println!("cargo:rerun-if-changed={}", receipt_path.display());
    println!("cargo:rerun-if-env-changed=SOKSAK_BUILD_DEPENDENCY_ROOT");
}

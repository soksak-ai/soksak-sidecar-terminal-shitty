use std::{fs, path::PathBuf};

#[test]
fn stage_consumes_the_make_owned_target_artifact() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let script = fs::read_to_string(root.join("stage.sh")).expect("read stage.sh");

    assert!(script.contains("scripts/stage-built.sh"));
    assert!(!script.contains("cargo build"));
    let stage = fs::read_to_string(root.join("scripts/stage-built.sh")).expect("read stage owner");
    assert!(stage.contains("target/$target/release/soksak-sidecar-terminal-shitty"));
    assert!(stage.contains("cmp -s"));
}

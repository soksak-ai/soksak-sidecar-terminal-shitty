use std::{fs, path::PathBuf};

#[test]
fn stage_uses_the_declared_cargo_target_directory() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let script = fs::read_to_string(root.join("stage.sh")).expect("read stage.sh");

    assert!(script.contains("release_dir=release"));
    assert!(script.contains("release_dir=\"$target/release\""));
    assert!(
        script.contains("${CARGO_TARGET_DIR:-target}/$release_dir/soksak-sidecar-terminal-shitty")
    );
}

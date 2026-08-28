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

// The SDK toolchain builds the SDK. A build root that already holds a verified receipt has a built
// SDK, so demanding those tools again refuses work this host can do — and the receipt, not the
// host, is what states which toolchain built it.
#[test]
fn the_sdk_toolchain_is_demanded_only_when_the_sdk_has_to_be_built() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let check = fs::read_to_string(root.join("scripts/check-build-environment.sh"))
        .expect("read environment check");
    assert!(
        check.contains("soksak-validate build-receipt "),
        "the check does not consult the receipt"
    );
    assert!(
        check.contains("SDK_REUSED") || check.contains("sdk=reused"),
        "the check does not report which toolchain it required"
    );

    let makefile = fs::read_to_string(root.join("Makefile")).expect("read Makefile");
    let preflight = makefile
        .split("preflight:")
        .nth(1)
        .and_then(|rest| rest.split("\nprepare:").next())
        .expect("preflight recipe");
    assert!(
        preflight.contains("check-build-environment.sh '$(TARGET)' '$(BUILD_DEPENDENCY_ROOT)'"),
        "preflight does not tell the check where the SDK is: {preflight}"
    );
}

use std::process::Command;

fn fixture_path(name: &str) -> String {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .join("docs/fixtures/rhist")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

#[test]
fn verify_l2_three_cycle_exits_zero() {
    let output = Command::new(env!("CARGO_BIN_EXE_rhist"))
        .args([
            "verify",
            &fixture_path("l2-three-cycle"),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""status":"pass""#));
    assert!(stdout.contains(r#""package_id":"syn-rhist-l2-three-cycle""#));
    assert!(stdout.contains(r#""crosswalk_weight_sum""#));
}

#[test]
fn verify_bad_weights_exits_one() {
    let output = Command::new(env!("CARGO_BIN_EXE_rhist"))
        .args([
            "verify",
            &fixture_path("l1-bad-weights"),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""status":"fail""#));
    assert!(stdout.contains("do not sum to 1"));
}

#[test]
fn verify_can_write_output_file() {
    let tmp = tempfile::tempdir().unwrap();
    let output_path = tmp.path().join("verify.json");
    let output = Command::new(env!("CARGO_BIN_EXE_rhist"))
        .arg("verify")
        .arg(fixture_path("l0-rename"))
        .args(["--format", "json", "--output"])
        .arg(&output_path)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    let text = std::fs::read_to_string(output_path).unwrap();
    assert!(text.contains(r#""status":"pass""#));
    assert!(text.contains(r#""package_id":"syn-rhist-l0-rename""#));
}

#[test]
fn verify_real_ri_fixture_exits_zero() {
    let output = Command::new(env!("CARGO_BIN_EXE_rhist"))
        .args([
            "verify",
            &fixture_path("real-ri-tract-unchanged"),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""status":"pass""#));
    assert!(stdout.contains(r#""package_id":"real-ri-tract-unchanged""#));
}

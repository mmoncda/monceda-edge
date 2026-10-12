use std::process::Command;

fn edge_bin() -> String {
    env!("CARGO_BIN_EXE_monceda-edge").to_string()
}

#[test]
fn version_command_works() {
    let output = Command::new(edge_bin())
        .arg("version")
        .output()
        .expect("failed to run version command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("Monceda Edge v"),
        "unexpected output: {stdout}"
    );
}

#[test]
fn help_lists_doctor_and_config() {
    let output = Command::new(edge_bin())
        .arg("--help")
        .output()
        .expect("failed to run help");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("doctor"));
    assert!(stdout.contains("config"));
}

#[test]
fn config_validate_accepts_local_config() {
    let root = env!("CARGO_MANIFEST_DIR");
    let config = format!("{root}/monceda-edge.toml");

    let output = Command::new(edge_bin())
        .args(["config", "validate", "--config", &config])
        .output()
        .expect("failed to run config validator");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Monceda Edge Config Validator"));
    assert!(stdout.contains("Result: PASS"));
}

#[test]
fn config_validate_accepts_public_staging_config() {
    let root = env!("CARGO_MANIFEST_DIR");
    let config = format!("{root}/deploy/linux/monceda-edge.public-staging.toml");

    let output = Command::new(edge_bin())
        .args(["config", "validate", "--config", &config])
        .output()
        .expect("failed to run public staging validator");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Result: PASS"));
}

#[test]
fn doctor_reports_local_node() {
    let root = env!("CARGO_MANIFEST_DIR");
    let config = format!("{root}/monceda-edge.toml");

    let output = Command::new(edge_bin())
        .args(["doctor", "--config", &config])
        .output()
        .expect("failed to run doctor");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Monceda Edge Doctor"));
    assert!(stdout.contains("Configuration parsed successfully"));
    assert!(stdout.contains("Result: PASS") || stdout.contains("Result: WARN"));
}

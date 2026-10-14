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

#[test]
fn config_show_outputs_effective_config() {
    let root = env!("CARGO_MANIFEST_DIR");
    let config = format!("{root}/monceda-edge.toml");

    let output = Command::new(edge_bin())
        .args(["config", "show", "--config", &config])
        .output()
        .expect("failed to run config show");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Monceda Edge Effective Configuration"));
    assert!(stdout.contains("Server"));
    assert!(stdout.contains("Routes"));
}

#[test]
fn config_show_json_outputs_valid_json() {
    let root = env!("CARGO_MANIFEST_DIR");
    let config = format!("{root}/monceda-edge.toml");

    let output = Command::new(edge_bin())
        .args(["config", "show", "--config", &config, "--json"])
        .output()
        .expect("failed to run config show json");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    let value: serde_json::Value = serde_json::from_str(&stdout).expect("invalid JSON output");

    assert_eq!(value["server"]["host"], "127.0.0.1");

    assert!(value["routes"].is_array());
}

#[test]
fn help_lists_node_command() {
    let output = Command::new(edge_bin())
        .arg("--help")
        .output()
        .expect("failed to run help");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("node"));
}

#[test]
fn node_help_lists_status() {
    let output = Command::new(edge_bin())
        .args(["node", "--help"])
        .output()
        .expect("failed to run node help");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("status"));
}

#[test]
fn node_status_handles_unreachable_node_cleanly() {
    let root = env!("CARGO_MANIFEST_DIR");
    let config = format!("{root}/monceda-edge.toml");

    let output = Command::new(edge_bin())
        .args(["node", "status", "--config", &config])
        .output()
        .expect("failed to run node status");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success()
            || stderr.contains("Monceda Edge is not reachable")
            || stdout.contains("Monceda Edge is running."),
        "stdout: {stdout}\nstderr: {stderr}"
    );
}

#[test]
fn node_help_lists_info() {
    let output = Command::new(edge_bin())
        .args(["node", "--help"])
        .output()
        .expect("failed to run node help");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("info"));
}

#[test]
fn node_info_outputs_effective_node_configuration() {
    let root = env!("CARGO_MANIFEST_DIR");
    let config = format!("{root}/monceda-edge.toml");

    let output = Command::new(edge_bin())
        .args(["node", "info", "--config", &config])
        .output()
        .expect("failed to run node info");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Monceda Edge Node Information"));
    assert!(stdout.contains("Version:"));
    assert!(stdout.contains("Deployment mode:"));
    assert!(stdout.contains("Mode:"));
    assert!(stdout.contains("Endpoint:"));
    assert!(stdout.contains("HTTP:"));
    assert!(stdout.contains("TLS:"));
    assert!(stdout.contains("Proxy:"));
    assert!(stdout.contains("Public node:"));
}

#[test]
fn node_info_json_outputs_valid_json() {
    let root = env!("CARGO_MANIFEST_DIR");
    let config = format!("{root}/monceda-edge.toml");

    let output = Command::new(edge_bin())
        .args(["node", "info", "--config", &config, "--json"])
        .output()
        .expect("failed to run node info json");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    let value: serde_json::Value = serde_json::from_str(&stdout).expect("invalid node info JSON");

    assert_eq!(value["version"], "0.9.0");
    assert_eq!(value["deployment_mode"], "development");
    assert_eq!(value["mode"], "reverse-proxy");
    assert_eq!(value["endpoint"], "https://127.0.0.1:8443");
    assert_eq!(value["http"]["port"], 8080);
    assert_eq!(value["tls"]["port"], 8443);
    assert_eq!(value["routes"], 2);
}

#[test]
fn node_status_json_outputs_valid_json() {
    let root = env!("CARGO_MANIFEST_DIR");
    let config = format!("{root}/monceda-edge.toml");

    let output = Command::new(edge_bin())
        .args(["node", "status", "--config", &config, "--json"])
        .output()
        .expect("failed to run node status json");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    let value: serde_json::Value = serde_json::from_str(&stdout).expect("invalid node status JSON");

    assert!(value["reachable"].is_boolean());
    assert!(value["endpoint"].is_string());
}

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

    assert_eq!(value["version"], env!("CARGO_PKG_VERSION"));
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

#[test]
fn node_help_lists_check() {
    let output = Command::new(edge_bin())
        .args(["node", "--help"])
        .output()
        .expect("failed to run node help");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("check"));
}

#[test]
fn node_check_json_outputs_valid_json() {
    let root = env!("CARGO_MANIFEST_DIR");
    let config = format!("{root}/monceda-edge.toml");

    let output = Command::new(edge_bin())
        .args(["node", "check", "--config", &config, "--json"])
        .output()
        .expect("failed to run node check json");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: serde_json::Value = serde_json::from_str(&stdout).expect("invalid node check JSON");

    assert!(value["result"] == "PASS" || value["result"] == "WARN");
    assert_eq!(value["version"], env!("CARGO_PKG_VERSION"));
    assert!(value["endpoint"].is_string());
    assert!(value["reachable"].is_boolean());
    assert!(value["routes"].is_number());
}

#[test]
fn node_check_help_lists_strict() {
    let output = Command::new(edge_bin())
        .args(["node", "check", "--help"])
        .output()
        .expect("failed to run node check help");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--strict"));
}

#[test]
fn node_check_strict_succeeds_when_node_is_reachable() {
    let root = env!("CARGO_MANIFEST_DIR");
    let config = format!("{root}/monceda-edge.toml");

    let output = Command::new(edge_bin())
        .args(["node", "check", "--config", &config, "--strict"])
        .output()
        .expect("failed to run strict node check");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success()
            || stderr.contains("Node check failed in strict mode")
            || stdout.contains("Result:           PASS"),
        "stdout: {stdout}\nstderr: {stderr}"
    );
}

#[test]
fn node_help_lists_routes() {
    let output = Command::new(edge_bin())
        .args(["node", "--help"])
        .output()
        .expect("failed to run node help");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("routes"));
}

#[test]
fn node_routes_outputs_configured_routes() {
    let root = env!("CARGO_MANIFEST_DIR");
    let config = format!("{root}/monceda-edge.toml");

    let output = Command::new(edge_bin())
        .args(["node", "routes", "--config", &config])
        .output()
        .expect("failed to run node routes");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Monceda Edge Node Routes"));
    assert!(stdout.contains("dashboard.local"));
    assert!(stdout.contains("http://127.0.0.1:3000"));
    assert!(stdout.contains("app.local"));
    assert!(stdout.contains("http://127.0.0.1:4000"));
}

#[test]
fn node_routes_json_outputs_valid_json() {
    let root = env!("CARGO_MANIFEST_DIR");
    let config = format!("{root}/monceda-edge.toml");

    let output = Command::new(edge_bin())
        .args(["node", "routes", "--config", &config, "--json"])
        .output()
        .expect("failed to run node routes json");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: serde_json::Value = serde_json::from_str(&stdout).expect("invalid node routes JSON");

    assert_eq!(value["count"], 2);
    assert!(value["routes"].is_array());
    assert_eq!(value["routes"][0]["host"], "dashboard.local");
    assert_eq!(value["routes"][0]["upstream"], "http://127.0.0.1:3000");
}

#[test]
fn node_routes_help_lists_check() {
    let output = Command::new(edge_bin())
        .args(["node", "routes", "--help"])
        .output()
        .expect("failed to run node routes help");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--check"));
}

#[test]
fn node_routes_check_json_outputs_valid_json() {
    let root = env!("CARGO_MANIFEST_DIR");
    let config = format!("{root}/monceda-edge.toml");

    let output = Command::new(edge_bin())
        .args(["node", "routes", "--config", &config, "--check", "--json"])
        .output()
        .expect("failed to run node routes check json");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    let value: serde_json::Value = serde_json::from_str(&stdout).expect("invalid route check JSON");

    assert_eq!(value["count"], 2);
    assert!(value["healthy"].is_number());
    assert!(value["unreachable"].is_number());
    assert!(value["routes"].is_array());
    assert!(value["routes"][0]["reachable"].is_boolean());
}

#[test]
fn node_routes_help_lists_strict() {
    let output = Command::new(edge_bin())
        .args(["node", "routes", "--help"])
        .output()
        .expect("failed to run node routes help");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--strict"));
}

#[test]
fn node_routes_check_strict_reports_failure_when_route_is_down() {
    let root = env!("CARGO_MANIFEST_DIR");
    let config = format!("{root}/monceda-edge.toml");

    let output = Command::new(edge_bin())
        .args(["node", "routes", "--config", &config, "--check", "--strict"])
        .output()
        .expect("failed to run strict route check");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success()
            || stderr.contains("Route check failed in strict mode")
            || stdout.contains("Unreachable: 0"),
        "stdout: {stdout}\nstderr: {stderr}"
    );
}

#[test]
fn node_routes_help_lists_host() {
    let output = Command::new(edge_bin())
        .args(["node", "routes", "--help"])
        .output()
        .expect("failed to run node routes help");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--host"));
}

#[test]
fn node_routes_filters_by_host() {
    let root = env!("CARGO_MANIFEST_DIR");
    let config = format!("{root}/monceda-edge.toml");

    let output = Command::new(edge_bin())
        .args([
            "node",
            "routes",
            "--config",
            &config,
            "--host",
            "dashboard.local",
        ])
        .output()
        .expect("failed to run filtered node routes");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Routes: 1"));
    assert!(stdout.contains("dashboard.local"));
    assert!(!stdout.contains("app.local"));
}

#[test]
fn node_routes_host_json_outputs_single_route() {
    let root = env!("CARGO_MANIFEST_DIR");
    let config = format!("{root}/monceda-edge.toml");

    let output = Command::new(edge_bin())
        .args([
            "node",
            "routes",
            "--config",
            &config,
            "--host",
            "dashboard.local",
            "--json",
        ])
        .output()
        .expect("failed to run filtered node routes json");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: serde_json::Value =
        serde_json::from_str(&stdout).expect("invalid filtered routes JSON");

    assert_eq!(value["count"], 1);
    assert_eq!(value["routes"][0]["host"], "dashboard.local");
}

#[test]
fn node_routes_help_lists_timeout() {
    let output = Command::new(edge_bin())
        .args(["node", "routes", "--help"])
        .output()
        .expect("failed to run node routes help");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--timeout"));
}

#[test]
fn node_routes_check_json_includes_latency() {
    let root = env!("CARGO_MANIFEST_DIR");
    let config = format!("{root}/monceda-edge.toml");

    let output = Command::new(edge_bin())
        .args([
            "node",
            "routes",
            "--config",
            &config,
            "--host",
            "app.local",
            "--check",
            "--json",
            "--timeout",
            "1",
        ])
        .output()
        .expect("failed to run route latency check");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: serde_json::Value = serde_json::from_str(&stdout).expect("invalid route check JSON");

    assert_eq!(value["count"], 1);
    assert!(value["routes"][0]["latency_ms"].is_number());
}

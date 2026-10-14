use axum::{
    body::{to_bytes, Body},
    extract::State,
    http::{header, HeaderName, HeaderValue, Request, StatusCode},
    response::Response,
    routing::get,
    Json, Router,
};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::HashMap,
    path::{Component, Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

const VERSION: &str = "0.9.2";

#[derive(Parser)]
#[command(name = "monceda-edge", about = "Portable open-source edge platform")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum ConfigCommands {
    Validate {
        #[arg(long, default_value = "monceda-edge.toml")]
        config: String,
    },

    Show {
        #[arg(long, default_value = "monceda-edge.toml")]
        config: String,

        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum NodeCommands {
    Status {
        #[arg(long, default_value = "monceda-edge.toml")]
        config: String,

        #[arg(long)]
        json: bool,
    },

    Info {
        #[arg(long, default_value = "monceda-edge.toml")]
        config: String,

        #[arg(long)]
        json: bool,
    },

    Check {
        #[arg(long, default_value = "monceda-edge.toml")]
        config: String,

        #[arg(long)]
        json: bool,

        #[arg(long)]
        strict: bool,
    },

    Routes {
        #[arg(long, default_value = "monceda-edge.toml")]
        config: String,

        #[arg(long)]
        json: bool,

        #[arg(long)]
        check: bool,
    },
}

#[derive(Subcommand)]
enum Commands {
    Serve {
        #[arg(long)]
        host: Option<String>,

        #[arg(long)]
        port: Option<u16>,

        #[arg(long, default_value = "monceda-edge.toml")]
        config: String,
    },

    Status {
        #[arg(long, default_value = "monceda-edge.toml")]
        config: String,
    },

    Node {
        #[command(subcommand)]
        command: NodeCommands,
    },

    Doctor {
        #[arg(long, default_value = "monceda-edge.toml")]
        config: String,
    },

    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    Version,
}

#[derive(Debug, Clone, Deserialize)]
struct Config {
    #[serde(default)]
    server: ServerConfig,

    #[serde(default)]
    proxy: ProxyConfig,

    #[serde(default)]
    cache: CacheConfig,

    #[serde(default)]
    tls: TlsConfig,

    #[serde(default)]
    http: HttpConfig,

    #[serde(default)]
    acme: AcmeConfig,

    #[serde(default)]
    deployment: DeploymentConfig,

    #[serde(default)]
    public_node: PublicNodeConfig,

    #[serde(default)]
    routes: Vec<RouteConfig>,
}

#[derive(Debug, Clone, Deserialize)]
struct ServerConfig {
    #[serde(default = "default_host")]
    host: String,

    #[serde(default = "default_port")]
    port: u16,

    #[serde(default = "default_static_dir")]
    static_dir: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            static_dir: default_static_dir(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct ProxyConfig {
    #[serde(default)]
    enabled: bool,

    #[serde(default = "default_upstream")]
    upstream: String,

    #[serde(default = "default_health_path")]
    health_path: String,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            upstream: default_upstream(),
            health_path: default_health_path(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct CacheConfig {
    #[serde(default = "default_cache_enabled")]
    enabled: bool,

    #[serde(default = "default_cache_ttl")]
    ttl_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: default_cache_enabled(),
            ttl_seconds: default_cache_ttl(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct TlsConfig {
    #[serde(default)]
    enabled: bool,

    #[serde(default = "default_tls_port")]
    port: u16,

    #[serde(default = "default_tls_cert")]
    cert: String,

    #[serde(default = "default_tls_key")]
    key: String,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            port: default_tls_port(),
            cert: default_tls_cert(),
            key: default_tls_key(),
        }
    }
}

fn default_tls_port() -> u16 {
    8443
}

fn default_tls_cert() -> String {
    "certs/localhost-cert.pem".to_string()
}

fn default_tls_key() -> String {
    "certs/localhost-key.pem".to_string()
}

#[derive(Debug, Clone, Deserialize)]
struct PublicNodeConfig {
    #[serde(default)]
    enabled: bool,

    #[serde(default)]
    primary_domain: String,

    #[serde(default)]
    expected_ipv4: String,
}

impl Default for PublicNodeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            primary_domain: String::new(),
            expected_ipv4: String::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct DeploymentConfig {
    #[serde(default = "default_deployment_mode")]
    mode: String,

    #[serde(default)]
    require_public_ports: bool,

    #[serde(default)]
    require_public_domains: bool,
}

impl Default for DeploymentConfig {
    fn default() -> Self {
        Self {
            mode: default_deployment_mode(),
            require_public_ports: false,
            require_public_domains: false,
        }
    }
}

fn default_deployment_mode() -> String {
    "development".to_string()
}

#[derive(Debug, Clone, Deserialize)]
struct AcmeConfig {
    #[serde(default)]
    enabled: bool,

    #[serde(default = "default_acme_environment")]
    environment: String,

    #[serde(default)]
    email: String,

    #[serde(default = "default_acme_cache_dir")]
    cache_dir: String,

    #[serde(default = "default_acme_challenge")]
    challenge: String,
}

impl Default for AcmeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            environment: default_acme_environment(),
            email: String::new(),
            cache_dir: default_acme_cache_dir(),
            challenge: default_acme_challenge(),
        }
    }
}

fn default_acme_environment() -> String {
    "staging".to_string()
}

fn default_acme_cache_dir() -> String {
    "certs/acme".to_string()
}

fn default_acme_challenge() -> String {
    "http-01".to_string()
}

#[derive(Debug, Clone, Deserialize)]
struct HttpConfig {
    #[serde(default)]
    enabled: bool,

    #[serde(default = "default_http_port")]
    port: u16,

    #[serde(default = "default_true")]
    redirect_to_https: bool,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            port: default_http_port(),
            redirect_to_https: true,
        }
    }
}

fn default_http_port() -> u16 {
    8080
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct RouteConfig {
    host: String,
    upstream: String,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8787
}

fn default_static_dir() -> String {
    "public".to_string()
}

fn default_upstream() -> String {
    "http://127.0.0.1:3000".to_string()
}

fn default_health_path() -> String {
    "/".to_string()
}

fn default_cache_enabled() -> bool {
    true
}

fn default_cache_ttl() -> u64 {
    30
}

#[derive(Clone)]
struct AppState {
    config: Arc<Config>,
    client: reqwest::Client,
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
}

#[derive(Clone)]
struct CacheEntry {
    status: u16,
    body: Vec<u8>,
    headers: Vec<(String, String)>,
    expires_at: Instant,
}

#[derive(Serialize)]
struct HealthResponse {
    service: &'static str,
    status: &'static str,
    version: &'static str,
    mode: &'static str,
}

#[tokio::main]
async fn main() {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    let cli = Cli::parse();

    let result = match cli.command {
        Some(Commands::Serve { host, port, config }) => serve(&config, host, port).await,

        Some(Commands::Status { config }) => status(&config, false).await,

        Some(Commands::Node { command }) => match command {
            NodeCommands::Status { config, json } => status(&config, json).await,
            NodeCommands::Info { config, json } => node_info(&config, json),
            NodeCommands::Check {
                config,
                json,
                strict,
            } => node_check(&config, json, strict).await,

            NodeCommands::Routes {
                config,
                json,
                check,
            } => node_routes(&config, json, check).await,
        },

        Some(Commands::Doctor { config }) => doctor(&config),

        Some(Commands::Config { command }) => match command {
            ConfigCommands::Validate { config } => validate_config(&config),

            ConfigCommands::Show { config, json } => show_config(&config, json),
        },

        Some(Commands::Version) => {
            println!("Monceda Edge v{VERSION}");

            Ok(())
        }

        None => serve("monceda-edge.toml", None, None).await,
    };

    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

async fn serve(
    config_path: &str,
    host_override: Option<String>,
    port_override: Option<u16>,
) -> Result<(), String> {
    let mut config = load_config(config_path)?;

    if let Some(host) = host_override {
        config.server.host = host;
    }

    if let Some(port) = port_override {
        config.server.port = port;
    }

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let state = AppState {
        config: Arc::new(config.clone()),
        client,
        cache: Arc::new(RwLock::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/_edge/health", get(edge_health))
        .route("/_edge/upstream", get(upstream_health))
        .route("/_edge/cache", get(cache_status))
        .route("/_edge/routes", get(routes_status))
        .route("/_edge/acme", get(acme_status))
        .route("/_edge/readiness", get(readiness_status))
        .route("/_edge/onboarding", get(onboarding_status))
        .route("/_edge/network-check", get(network_check_status))
        .route(
            "/_edge/acme-activation",
            get(acme_activation_status_endpoint),
        )
        .fallback(edge_fallback)
        .with_state(state)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http());

    let mode = if config.proxy.enabled {
        "reverse-proxy"
    } else {
        "static"
    };

    println!("Monceda Edge v{VERSION}");

    if config.acme.enabled {
        if let Err(error) = std::fs::create_dir_all(&config.acme.cache_dir) {
            eprintln!(
                "Unable to create ACME cache directory {}: {}",
                config.acme.cache_dir, error
            );
        }

        let domains = acme_domains(&config);

        println!(
            "ACME: enabled ({}, {})",
            config.acme.environment, config.acme.challenge
        );

        println!("ACME cache: {}", config.acme.cache_dir);

        if domains.is_empty() {
            println!("ACME: no eligible public domains configured");
        } else {
            println!("ACME eligible domains: {}", domains.join(", "));
        }

        if config.acme.email.trim().is_empty() {
            println!("ACME contact: not configured");
        } else {
            println!("ACME contact: configured");
        }
    } else {
        println!("ACME: disabled");
    }
    println!("Mode: {mode}");

    if config.proxy.enabled {
        println!("Upstream: {}", config.proxy.upstream);

        if !config.routes.is_empty() {
            println!("Host routes: {}", config.routes.len());

            for route in &config.routes {
                println!("  {} -> {}", route.host, route.upstream);
            }
        }
        println!(
            "Cache: {} (TTL {}s)",
            if config.cache.enabled {
                "enabled"
            } else {
                "disabled"
            },
            config.cache.ttl_seconds
        );
    } else {
        println!("Static directory: {}", config.server.static_dir);
    }

    if config.tls.enabled && acme_runtime_ready(&config) {
        println!("TLS mode: ACME-managed certificate");

        return run_acme_runtime(config, app).await;
    }

    if config.tls.enabled {
        start_http_listener(&config).await;
        use axum_server::tls_rustls::RustlsConfig;

        let address = format!("{}:{}", config.server.host, config.tls.port);

        let socket_address: std::net::SocketAddr = address
            .parse()
            .map_err(|e| format!("Invalid HTTPS address {address}: {e}"))?;

        let tls_config = RustlsConfig::from_pem_file(&config.tls.cert, &config.tls.key)
            .await
            .map_err(|e| format!("Unable to load TLS certificate/key: {e}"))?;

        println!("TLS: enabled");
        println!("HTTPS listening on https://{address}");
        println!("Certificate: {}", config.tls.cert);

        axum_server::bind_rustls(socket_address, tls_config)
            .serve(app.into_make_service())
            .await
            .map_err(|e| format!("HTTPS server error: {e}"))
    } else {
        let address = format!("{}:{}", config.server.host, config.server.port);

        let listener = match tokio::net::TcpListener::bind(&address).await {
            Ok(listener) => listener,

            Err(error) if error.kind() == std::io::ErrorKind::AddrInUse => {
                return Err(format!(
                    "Monceda Edge could not start: port {} is already in use.\n\
                         Run: lsof -i :{}\n\
                         Or use another port: monceda-edge serve --port 8788",
                    config.server.port, config.server.port
                ));
            }

            Err(error) => {
                return Err(format!("Monceda Edge could not bind to {address}: {error}"));
            }
        };

        println!("TLS: disabled");
        println!("HTTP listening on http://{address}");

        axum::serve(listener, app)
            .await
            .map_err(|e| format!("Server error: {e}"))
    }
}

async fn edge_health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "monceda-edge",
        status: "ok",
        version: VERSION,
        mode: if state.config.proxy.enabled {
            "reverse-proxy"
        } else {
            "static"
        },
    })
}

async fn upstream_health(State(state): State<AppState>) -> Json<serde_json::Value> {
    if !state.config.proxy.enabled {
        return Json(json!({
            "enabled": false,
            "status": "disabled"
        }));
    }

    let url = format!(
        "{}{}",
        state.config.proxy.upstream.trim_end_matches('/'),
        normalize_health_path(&state.config.proxy.health_path)
    );

    match state.client.get(&url).send().await {
        Ok(response) => Json(json!({
            "enabled": true,
            "status": if response.status().is_success() {
                "healthy"
            } else {
                "unhealthy"
            },
            "upstream": state.config.proxy.upstream,
            "http_status": response.status().as_u16()
        })),

        Err(error) => Json(json!({
            "enabled": true,
            "status": "unreachable",
            "upstream": state.config.proxy.upstream,
            "error": error.to_string()
        })),
    }
}

async fn public_network_preflight(config: &Config) -> serde_json::Value {
    let domain = config
        .public_node
        .primary_domain
        .trim()
        .trim_end_matches('.')
        .to_ascii_lowercase();

    if domain.is_empty() {
        return json!({
            "checked": false,
            "primary_domain": "",
            "dns_resolves": false,
            "resolved_ips": [],
            "expected_ipv4": config.public_node.expected_ipv4,
            "expected_ipv4_match": false,
            "http01": {
                "configured_port": config.http.port,
                "requires_public_port_80": true,
                "configured_correctly": config.http.port == 80
            },
            "https": {
                "configured_port": config.tls.port,
                "recommended_public_port_443": true,
                "configured_correctly": config.tls.port == 443
            },
            "external_reachability_verified": false,

        "public_probe": {
            "path": "/.well-known/monceda-edge/reachability",
            "expected_status": 200,
            "expected_header": "x-monceda-probe: reachable"
        },
            "message": "No public primary_domain configured"
        });
    }

    let lookup = tokio::net::lookup_host((domain.as_str(), 80)).await;

    let mut resolved_ips: Vec<String> = match lookup {
        Ok(addresses) => addresses.map(|address| address.ip().to_string()).collect(),

        Err(_) => Vec::new(),
    };

    resolved_ips.sort();
    resolved_ips.dedup();

    let expected_ipv4 = config.public_node.expected_ipv4.trim();

    let expected_ipv4_match =
        !expected_ipv4.is_empty() && resolved_ips.iter().any(|ip| ip == expected_ipv4);

    json!({
        "checked": true,
        "primary_domain": domain,
        "dns_resolves": !resolved_ips.is_empty(),
        "resolved_ips": resolved_ips,
        "expected_ipv4": expected_ipv4,
        "expected_ipv4_match": expected_ipv4_match,

        "http01": {
            "configured_port": config.http.port,
            "requires_public_port_80": true,
            "configured_correctly": config.http.port == 80
        },

        "https": {
            "configured_port": config.tls.port,
            "recommended_public_port_443": true,
            "configured_correctly": config.tls.port == 443
        },

        "external_reachability_verified": false,

        "public_probe": {
            "path": "/.well-known/monceda-edge/reachability",
            "expected_status": 200,
            "expected_header": "x-monceda-probe: reachable"
        },

        "message":
            "DNS resolution is local verification only; public inbound reachability must still be tested externally"
    })
}

fn public_onboarding_status(config: &Config) -> serde_json::Value {
    let domain = config
        .public_node
        .primary_domain
        .trim()
        .trim_end_matches('.')
        .to_ascii_lowercase();

    let domain_configured = !domain.is_empty();
    let domain_eligible = domain_configured && acme_domain_eligible(&domain);

    let route_exists = config.routes.iter().any(|route| {
        route
            .host
            .trim()
            .trim_end_matches('.')
            .eq_ignore_ascii_case(&domain)
    });

    let public_ports = config.http.port == 80 && config.tls.port == 443;

    let contact_configured = !config.acme.email.trim().is_empty();

    let cache_configured = !config.acme.cache_dir.trim().is_empty();

    let staging = config.acme.environment.eq_ignore_ascii_case("staging");

    let production = config.acme.environment.eq_ignore_ascii_case("production");

    let configuration_ready = config.public_node.enabled
        && domain_eligible
        && route_exists
        && public_ports
        && config.acme.enabled
        && contact_configured
        && cache_configured;

    json!({
        "enabled": config.public_node.enabled,
        "primary_domain": domain,
        "expected_ipv4": config.public_node.expected_ipv4,

        "checks": {
            "domain_configured": domain_configured,
            "domain_eligible": domain_eligible,
            "route_configured": route_exists,
            "http_port_80": config.http.port == 80,
            "https_port_443": config.tls.port == 443,
            "public_ports": public_ports,
            "acme_enabled": config.acme.enabled,
            "contact_configured": contact_configured,
            "cache_configured": cache_configured
        },

        "acme": {
            "environment": config.acme.environment,
            "staging": staging,
            "production": production,
            "challenge": config.acme.challenge
        },

        "configuration_ready": configuration_ready,

        "safe_to_test_staging":
            configuration_ready && staging,

        "production_activation_ready":
            configuration_ready && production
    })
}

fn production_readiness(config: &Config) -> serde_json::Value {
    let production_mode = config.deployment.mode.eq_ignore_ascii_case("production");

    let eligible_domains = acme_domains(config);

    let ports_ok = config.http.port == 80 && config.tls.port == 443;

    let domains_ok = !eligible_domains.is_empty();

    let contact_ok = !config.acme.email.trim().is_empty();

    let acme_enabled = config.acme.enabled;

    let acme_production = config.acme.environment.eq_ignore_ascii_case("production");

    let cache_ok = !config.acme.cache_dir.trim().is_empty();

    let public_ports_requirement_ok = !config.deployment.require_public_ports || ports_ok;

    let public_domains_requirement_ok = !config.deployment.require_public_domains || domains_ok;

    let ready = production_mode
        && public_ports_requirement_ok
        && public_domains_requirement_ok
        && contact_ok
        && acme_enabled
        && acme_production
        && cache_ok;

    json!({
        "ready": ready,
        "mode": config.deployment.mode,
        "public_ports": {
            "required": config.deployment.require_public_ports,
            "http": config.http.port,
            "https": config.tls.port,
            "valid": ports_ok
        },
        "public_domains": {
            "required": config.deployment.require_public_domains,
            "eligible": eligible_domains,
            "valid": domains_ok
        },
        "acme": {
            "enabled": acme_enabled,
            "environment": config.acme.environment,
            "production_directory": acme_production,
            "contact_configured": contact_ok,
            "cache_configured": cache_ok
        }
    })
}

fn acme_activation_status(config: &Config) -> serde_json::Value {
    let domain = config
        .public_node
        .primary_domain
        .trim()
        .trim_end_matches('.')
        .to_ascii_lowercase();

    let domain_eligible = !domain.is_empty() && acme_domain_eligible(&domain);

    let route_exists = config.routes.iter().any(|route| {
        route
            .host
            .trim()
            .trim_end_matches('.')
            .eq_ignore_ascii_case(&domain)
    });

    let port_80 = config.http.port == 80;

    let port_443 = config.tls.port == 443;

    let contact = !config.acme.email.trim().is_empty();

    let cache = !config.acme.cache_dir.trim().is_empty();

    let staging = config.acme.environment.eq_ignore_ascii_case("staging");

    let production = config.acme.environment.eq_ignore_ascii_case("production");

    let runtime_ready = config.acme.enabled
        && config.public_node.enabled
        && domain_eligible
        && route_exists
        && port_80
        && port_443
        && contact
        && cache;

    json!({
        "runtime_ready": runtime_ready,

        "mode":
            if !runtime_ready {
                "blocked"
            } else if staging {
                "staging"
            } else if production {
                "production"
            } else {
                "unknown"
            },

        "checks": {
            "public_node_enabled":
                config.public_node.enabled,

            "primary_domain":
                domain,

            "domain_eligible":
                domain_eligible,

            "route_configured":
                route_exists,

            "http_port_80":
                port_80,

            "https_port_443":
                port_443,

            "acme_enabled":
                config.acme.enabled,

            "contact_configured":
                contact,

            "persistent_cache":
                cache
        },

        "environment": {
            "configured":
                config.acme.environment,

            "staging":
                staging,

            "production":
                production
        },

        "safe_staging_activation":
            runtime_ready && staging,

        "production_activation":
            runtime_ready && production
    })
}

fn acme_runtime_ready(config: &Config) -> bool {
    let domain = config
        .public_node
        .primary_domain
        .trim()
        .trim_end_matches('.')
        .to_ascii_lowercase();

    let domain_eligible = !domain.is_empty() && acme_domain_eligible(&domain);

    let route_exists = config.routes.iter().any(|route| {
        route
            .host
            .trim()
            .trim_end_matches('.')
            .eq_ignore_ascii_case(&domain)
    });

    let public_ports = config.http.port == 80 && config.tls.port == 443;

    config.acme.enabled
        && config.public_node.enabled
        && domain_eligible
        && route_exists
        && public_ports
        && !config.acme.email.trim().is_empty()
        && !config.acme.cache_dir.trim().is_empty()
}

fn acme_domain_eligible(host: &str) -> bool {
    let host = host.trim().trim_end_matches('.').to_ascii_lowercase();

    if host.is_empty()
        || host == "localhost"
        || host.ends_with(".local")
        || host.parse::<std::net::IpAddr>().is_ok()
    {
        return false;
    }

    host.contains('.')
}

fn acme_domains(config: &Config) -> Vec<String> {
    let mut domains: Vec<String> = config
        .routes
        .iter()
        .map(|route| route.host.trim().trim_end_matches('.').to_ascii_lowercase())
        .filter(|host| acme_domain_eligible(host))
        .collect();

    domains.sort();
    domains.dedup();
    domains
}

async fn acme_activation_status_endpoint(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(acme_activation_status(&state.config))
}

async fn network_check_status(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(public_network_preflight(&state.config).await)
}

async fn onboarding_status(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(public_onboarding_status(&state.config))
}

async fn readiness_status(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(production_readiness(&state.config))
}

async fn acme_status(State(state): State<AppState>) -> Json<serde_json::Value> {
    let domains = acme_domains(&state.config);

    let configured_hosts: Vec<serde_json::Value> = state
        .config
        .routes
        .iter()
        .map(|route| {
            let host = route.host.trim().trim_end_matches('.');

            json!({
                "host": host,
                "eligible": acme_domain_eligible(host),
            })
        })
        .collect();

    Json(json!({
        "enabled": state.config.acme.enabled,
        "environment": state.config.acme.environment,
        "challenge": state.config.acme.challenge,
        "cache_dir": state.config.acme.cache_dir,
        "contact_configured": !state.config.acme.email.trim().is_empty(),
        "eligible_domains": domains,
        "configured_hosts": configured_hosts,
        "ready_for_issuance":
            acme_runtime_ready(&state.config),
        "tls_mode":
            if acme_runtime_ready(&state.config) {
                "acme"
            } else {
                "configured-certificate"
            },
    }))
}

async fn routes_status(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(json!({
        "enabled": !state.config.routes.is_empty(),
        "count": state.config.routes.len(),
        "routes": state.config.routes,
        "default_upstream": state.config.proxy.upstream,
    }))
}

async fn cache_status(State(state): State<AppState>) -> Json<serde_json::Value> {
    let mut cache = state.cache.write().await;
    cache.retain(|_, entry| entry.expires_at > Instant::now());

    Json(json!({
        "enabled": state.config.cache.enabled,
        "entries": cache.len(),
        "ttl_seconds": state.config.cache.ttl_seconds
    }))
}

async fn edge_fallback(State(state): State<AppState>, request: Request<Body>) -> Response {
    if state.config.proxy.enabled {
        proxy_request(state, request).await
    } else {
        serve_static(state, request).await
    }
}

async fn proxy_request(state: AppState, request: Request<Body>) -> Response {
    let method = request.method().clone();

    let host = request
        .uri()
        .authority()
        .map(|authority| authority.host().to_string())
        .or_else(|| {
            request
                .headers()
                .get(axum::http::header::HOST)
                .and_then(|value| value.to_str().ok())
                .map(|value| value.split(':').next().unwrap_or("").to_string())
        })
        .unwrap_or_default()
        .trim_end_matches('.')
        .to_ascii_lowercase();

    let selected_upstream = select_upstream(&state.config, &host);

    let path_and_query = request
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or("/")
        .to_string();

    let cache_key = format!("{} {} {}", method, host, path_and_query);

    if method == axum::http::Method::GET && state.config.cache.enabled {
        if let Some(response) = get_cached_response(&state, &cache_key).await {
            return response;
        }
    }

    let upstream_url = format!(
        "{}{}",
        selected_upstream.trim_end_matches('/'),
        path_and_query
    );

    let (parts, body) = request.into_parts();

    let body = match to_bytes(body, 32 * 1024 * 1024).await {
        Ok(body) => body,
        Err(error) => {
            return text_response(
                StatusCode::BAD_REQUEST,
                format!("Unable to read request body: {error}"),
            );
        }
    };

    let mut upstream_request = state.client.request(method.clone(), &upstream_url);

    for (name, value) in parts.headers.iter() {
        if is_hop_by_hop_request_header(name.as_str()) {
            continue;
        }

        upstream_request = upstream_request.header(name, value);
    }

    let upstream_response = match upstream_request.body(body).send().await {
        Ok(response) => response,
        Err(error) => {
            return text_response(
                StatusCode::BAD_GATEWAY,
                format!(
                    "Monceda Edge could not reach upstream {}: {error}",
                    selected_upstream
                ),
            );
        }
    };

    let status = upstream_response.status();

    let should_cache = method == axum::http::Method::GET
        && state.config.cache.enabled
        && status.is_success()
        && !response_disables_cache(upstream_response.headers());

    let response_headers = collect_response_headers(upstream_response.headers());

    let body = match upstream_response.bytes().await {
        Ok(bytes) => bytes.to_vec(),
        Err(error) => {
            return text_response(
                StatusCode::BAD_GATEWAY,
                format!("Unable to read upstream response: {error}"),
            );
        }
    };

    if should_cache {
        let entry = CacheEntry {
            status: status.as_u16(),
            body: body.clone(),
            headers: response_headers.clone(),
            expires_at: Instant::now() + Duration::from_secs(state.config.cache.ttl_seconds),
        };

        state.cache.write().await.insert(cache_key, entry);
    }

    build_response(
        status,
        body,
        response_headers,
        false,
        &host,
        &selected_upstream,
    )
}

async fn get_cached_response(state: &AppState, key: &str) -> Option<Response> {
    let mut cache = state.cache.write().await;

    let entry = cache.get(key)?.clone();

    if entry.expires_at <= Instant::now() {
        cache.remove(key);
        return None;
    }

    let status = StatusCode::from_u16(entry.status).unwrap_or(StatusCode::OK);

    {
        let parts: Vec<&str> = key.splitn(3, ' ').collect();
        let host = parts.get(1).copied().unwrap_or("");
        let upstream = select_upstream(&state.config, host);

        Some(build_response(
            status,
            entry.body,
            entry.headers,
            true,
            host,
            &upstream,
        ))
    }
}

async fn serve_static(state: AppState, request: Request<Body>) -> Response {
    if request.method() != axum::http::Method::GET && request.method() != axum::http::Method::HEAD {
        return text_response(
            StatusCode::METHOD_NOT_ALLOWED,
            "Method not allowed".to_string(),
        );
    }

    let path = request.uri().path();

    let relative = if path == "/" {
        PathBuf::from("index.html")
    } else {
        let raw = path.trim_start_matches('/');
        let candidate = PathBuf::from(raw);

        if candidate.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        }) {
            return text_response(StatusCode::BAD_REQUEST, "Invalid path".to_string());
        }

        candidate
    };

    let full_path = Path::new(&state.config.server.static_dir).join(relative);

    let metadata = match tokio::fs::metadata(&full_path).await {
        Ok(metadata) => metadata,
        Err(_) => {
            return text_response(StatusCode::NOT_FOUND, "404 - Not Found".to_string());
        }
    };

    let full_path = if metadata.is_dir() {
        full_path.join("index.html")
    } else {
        full_path
    };

    let bytes = match tokio::fs::read(&full_path).await {
        Ok(bytes) => bytes,
        Err(_) => {
            return text_response(StatusCode::NOT_FOUND, "404 - Not Found".to_string());
        }
    };

    let mime = mime_guess::from_path(&full_path)
        .first_or_octet_stream()
        .to_string();

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime)
        .header("x-monceda-edge", VERSION);

    if request.method() == axum::http::Method::HEAD {
        response
            .body(Body::empty())
            .unwrap_or_else(|_| Response::new(Body::empty()))
    } else {
        response
            .body(Body::from(bytes))
            .unwrap_or_else(|_| Response::new(Body::empty()))
    }
}

fn collect_response_headers(headers: &reqwest::header::HeaderMap) -> Vec<(String, String)> {
    headers
        .iter()
        .filter_map(|(name, value)| {
            if is_hop_by_hop_response_header(name.as_str()) {
                return None;
            }

            value
                .to_str()
                .ok()
                .map(|value| (name.to_string(), value.to_string()))
        })
        .collect()
}

fn build_response(
    status: StatusCode,
    body: Vec<u8>,
    headers: Vec<(String, String)>,
    cache_hit: bool,
    host: &str,
    upstream: &str,
) -> Response {
    let mut response = Response::builder().status(status);

    if let Some(header_map) = response.headers_mut() {
        for (name, value) in headers {
            if let (Ok(name), Ok(value)) = (
                HeaderName::from_bytes(name.as_bytes()),
                HeaderValue::from_str(&value),
            ) {
                header_map.append(name, value);
            }
        }

        header_map.insert(
            HeaderName::from_static("x-monceda-edge"),
            HeaderValue::from_static(VERSION),
        );

        header_map.insert(
            HeaderName::from_static("x-monceda-cache"),
            HeaderValue::from_static(if cache_hit { "HIT" } else { "MISS" }),
        );

        if let Ok(value) = HeaderValue::from_str(host) {
            header_map.insert(HeaderName::from_static("x-monceda-host"), value);
        }

        if let Ok(value) = HeaderValue::from_str(upstream) {
            header_map.insert(HeaderName::from_static("x-monceda-upstream"), value);
        }
    }

    response
        .body(Body::from(body))
        .unwrap_or_else(|_| Response::new(Body::empty()))
}

fn text_response(status: StatusCode, message: String) -> Response {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .header("x-monceda-edge", VERSION)
        .body(Body::from(message))
        .unwrap_or_else(|_| Response::new(Body::empty()))
}

fn response_disables_cache(headers: &reqwest::header::HeaderMap) -> bool {
    if headers.contains_key(reqwest::header::SET_COOKIE) {
        return true;
    }

    headers
        .get(reqwest::header::CACHE_CONTROL)
        .and_then(|value| value.to_str().ok())
        .map(|value| {
            let value = value.to_ascii_lowercase();
            value.contains("no-store") || value.contains("private") || value.contains("no-cache")
        })
        .unwrap_or(false)
}

fn is_hop_by_hop_request_header(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "host"
            | "connection"
            | "content-length"
            | "transfer-encoding"
            | "upgrade"
            | "proxy-connection"
    )
}

fn is_hop_by_hop_response_header(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "connection" | "content-length" | "transfer-encoding" | "upgrade" | "proxy-connection"
    )
}

async fn run_acme_runtime(config: Config, app: Router) -> Result<(), String> {
    use futures::StreamExt;
    use rustls_acme::caches::DirCache;
    use rustls_acme::tower::TowerHttp01ChallengeService;
    use rustls_acme::AcmeConfig as RustlsAcmeConfig;
    use rustls_acme::UseChallenge::Http01;
    use std::path::PathBuf;

    let domains = acme_domains(&config);

    if domains.is_empty() {
        return Err("ACME runtime requested without eligible domains".to_string());
    }

    let email = config.acme.email.trim();

    if email.is_empty() {
        return Err("ACME runtime requested without contact email".to_string());
    }

    let production = config.acme.environment.eq_ignore_ascii_case("production");

    let cache_path = PathBuf::from(&config.acme.cache_dir);

    std::fs::create_dir_all(&cache_path).map_err(|error| {
        format!(
            "Unable to create ACME cache {}: {}",
            cache_path.display(),
            error
        )
    })?;

    let mut acme_state = RustlsAcmeConfig::new(domains.clone())
        .contact_push(format!("mailto:{}", email))
        .cache(DirCache::new(cache_path))
        .directory_lets_encrypt(production)
        .challenge_type(Http01)
        .state();

    let acceptor = acme_state.axum_acceptor(acme_state.default_rustls_config());

    let challenge_service: TowerHttp01ChallengeService =
        acme_state.http01_challenge_tower_service();

    tokio::spawn(async move {
        while let Some(event) = acme_state.next().await {
            match event {
                Ok(ok) => {
                    println!("ACME event: {:?}", ok);
                }

                Err(error) => {
                    eprintln!("ACME event error: {:?}", error);
                }
            }
        }
    });

    let http_address = format!("{}:{}", config.server.host, config.http.port);

    let https_address = format!("{}:{}", config.server.host, config.tls.port);

    let http_socket: std::net::SocketAddr = http_address
        .parse()
        .map_err(|error| format!("Invalid ACME HTTP address {}: {}", http_address, error))?;

    let https_socket: std::net::SocketAddr = https_address
        .parse()
        .map_err(|error| format!("Invalid ACME HTTPS address {}: {}", https_address, error))?;

    let https_port = config.tls.port;
    let redirect_enabled = config.http.redirect_to_https;

    let http_app = Router::new()
        .route_service(
            "/.well-known/acme-challenge/{challenge_token}",
            challenge_service,
        )
        .fallback(
            move |uri: axum::http::Uri, headers: axum::http::HeaderMap| async move {
                http_redirect(uri, headers, https_port, redirect_enabled).await
            },
        );

    println!(
        "ACME TLS mode: {}",
        if production { "production" } else { "staging" }
    );

    println!("ACME domains: {}", domains.join(", "));

    println!("ACME HTTP listening on http://{}", http_address);

    println!("ACME HTTPS listening on https://{}", https_address);

    println!("ACME cache: {}", config.acme.cache_dir);

    let http_server = axum_server::bind(http_socket).serve(http_app.into_make_service());

    let https_server = axum_server::bind(https_socket)
        .acceptor(acceptor)
        .serve(app.into_make_service());

    tokio::try_join!(http_server, https_server)
        .map_err(|error| format!("ACME server error: {}", error))?;

    Ok(())
}

async fn public_reachability_probe() -> axum::response::Response {
    use axum::http::{HeaderValue, StatusCode};
    use axum::response::IntoResponse;

    let mut response = (
        StatusCode::OK,
        format!("monceda-edge-reachable:{}", VERSION),
    )
        .into_response();

    response
        .headers_mut()
        .insert("x-monceda-edge", HeaderValue::from_static(VERSION));

    response
        .headers_mut()
        .insert("x-monceda-probe", HeaderValue::from_static("reachable"));

    response
}

async fn start_http_listener(config: &Config) {
    if !config.http.enabled {
        return;
    }

    let address = format!("{}:{}", config.server.host, config.http.port);

    let listener = match tokio::net::TcpListener::bind(&address).await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("HTTP listener could not bind to {}: {}", address, error);
            return;
        }
    };

    let https_port = config.tls.port;
    let redirect_enabled = config.http.redirect_to_https;

    let http_app = Router::new()
        .route(
            "/.well-known/acme-challenge/{token}",
            get(acme_http01_placeholder),
        )
        .route(
            "/.well-known/monceda-edge/reachability",
            get(public_reachability_probe),
        )
        .fallback(
            move |uri: axum::http::Uri, headers: axum::http::HeaderMap| async move {
                http_redirect(uri, headers, https_port, redirect_enabled).await
            },
        );

    println!("HTTP listening on http://{}", address);

    if redirect_enabled {
        println!("HTTP redirect: enabled -> HTTPS port {}", https_port);
    }

    println!("ACME HTTP-01 path reserved: /.well-known/acme-challenge/*");

    tokio::spawn(async move {
        if let Err(error) = axum::serve(listener, http_app).await {
            eprintln!("HTTP server error: {}", error);
        }
    });
}

async fn acme_http01_placeholder(
    axum::extract::Path(token): axum::extract::Path<String>,
) -> Response {
    text_response(
        StatusCode::NOT_FOUND,
        format!("ACME challenge token not provisioned: {}", token),
    )
}

async fn http_redirect(
    uri: axum::http::Uri,
    headers: axum::http::HeaderMap,
    https_port: u16,
    redirect_enabled: bool,
) -> Response {
    if !redirect_enabled {
        return text_response(StatusCode::OK, "Monceda Edge HTTP listener".to_string());
    }

    let raw_host = headers
        .get(axum::http::header::HOST)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("localhost");

    let host = raw_host.split(':').next().unwrap_or("localhost");

    let authority = if https_port == 443 {
        host.to_string()
    } else {
        format!("{}:{}", host, https_port)
    };

    let path = uri
        .path_and_query()
        .map(|value| value.as_str())
        .unwrap_or("/");

    let location = format!("https://{}{}", authority, path);

    let mut response = Response::builder().status(StatusCode::PERMANENT_REDIRECT);

    if let Some(headers) = response.headers_mut() {
        if let Ok(value) = HeaderValue::from_str(&location) {
            headers.insert(axum::http::header::LOCATION, value);
        }

        headers.insert(
            HeaderName::from_static("x-monceda-edge"),
            HeaderValue::from_static(VERSION),
        );

        headers.insert(
            HeaderName::from_static("x-monceda-http"),
            HeaderValue::from_static("redirect"),
        );
    }

    response.body(Body::empty()).unwrap_or_else(|_| {
        text_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Unable to create redirect".to_string(),
        )
    })
}

fn select_upstream(config: &Config, host: &str) -> String {
    let host = host.trim().trim_end_matches('.').to_ascii_lowercase();

    for route in &config.routes {
        if route
            .host
            .trim()
            .trim_end_matches('.')
            .eq_ignore_ascii_case(&host)
        {
            return route.upstream.clone();
        }
    }

    config.proxy.upstream.clone()
}

fn normalize_health_path(path: &str) -> String {
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    }
}

fn show_config(config_path: &str, json: bool) -> Result<(), String> {
    let config = load_config(config_path)?;

    if json {
        let value = serde_json::json!({
            "server": {
                "host": config.server.host,
                "port": config.server.port,
                "static_dir": config.server.static_dir,
            },
            "http": {
                "enabled": config.http.enabled,
                "port": config.http.port,
                "redirect_to_https": config.http.redirect_to_https,
            },
            "tls": {
                "enabled": config.tls.enabled,
                "port": config.tls.port,
                "cert": config.tls.cert,
                "key": config.tls.key,
            },
            "proxy": {
                "enabled": config.proxy.enabled,
                "upstream": config.proxy.upstream,
                "health_path": config.proxy.health_path,
            },
            "cache": {
                "enabled": config.cache.enabled,
                "ttl_seconds": config.cache.ttl_seconds,
            },
            "acme": {
                "enabled": config.acme.enabled,
                "environment": config.acme.environment,
                "email_configured": !config.acme.email.trim().is_empty(),
                "cache_dir": config.acme.cache_dir,
                "challenge": config.acme.challenge,
            },
            "deployment": {
                "mode": config.deployment.mode,
                "require_public_ports": config.deployment.require_public_ports,
                "require_public_domains": config.deployment.require_public_domains,
            },
            "public_node": {
                "enabled": config.public_node.enabled,
                "primary_domain": config.public_node.primary_domain,
                "expected_ipv4": config.public_node.expected_ipv4,
            },
            "routes": config.routes,
        });

        println!(
            "{}",
            serde_json::to_string_pretty(&value)
                .map_err(|error| format!("Unable to serialize config: {error}"))?
        );

        return Ok(());
    }

    println!("Monceda Edge Effective Configuration");
    println!("────────────────────────────────");
    println!();

    println!("Server");
    println!("  host        {}", config.server.host);
    println!("  port        {}", config.server.port);
    println!("  static_dir  {}", config.server.static_dir);
    println!();

    println!("HTTP");
    println!("  enabled     {}", config.http.enabled);
    println!("  port        {}", config.http.port);
    println!("  redirect    {}", config.http.redirect_to_https);
    println!();

    println!("TLS");
    println!("  enabled     {}", config.tls.enabled);
    println!("  port        {}", config.tls.port);
    println!("  cert        {}", config.tls.cert);
    println!("  key         {}", config.tls.key);
    println!();

    println!("Proxy");
    println!("  enabled     {}", config.proxy.enabled);
    println!("  upstream    {}", config.proxy.upstream);
    println!("  health      {}", config.proxy.health_path);
    println!();

    println!("Cache");
    println!("  enabled     {}", config.cache.enabled);
    println!("  ttl         {}s", config.cache.ttl_seconds);
    println!();

    println!("ACME");
    println!("  enabled      {}", config.acme.enabled);
    println!("  environment  {}", config.acme.environment);
    println!("  challenge    {}", config.acme.challenge);
    println!("  cache_dir    {}", config.acme.cache_dir);
    println!(
        "  email        {}",
        if config.acme.email.trim().is_empty() {
            "not configured"
        } else {
            "configured"
        }
    );
    println!();

    println!("Deployment");
    println!("  mode                    {}", config.deployment.mode);
    println!(
        "  require_public_ports    {}",
        config.deployment.require_public_ports
    );
    println!(
        "  require_public_domains  {}",
        config.deployment.require_public_domains
    );
    println!();

    println!("Public Node");
    println!("  enabled         {}", config.public_node.enabled);
    println!(
        "  primary_domain  {}",
        if config.public_node.primary_domain.trim().is_empty() {
            "—"
        } else {
            &config.public_node.primary_domain
        }
    );
    println!(
        "  expected_ipv4   {}",
        if config.public_node.expected_ipv4.trim().is_empty() {
            "—"
        } else {
            &config.public_node.expected_ipv4
        }
    );
    println!();

    println!("Routes");

    if config.routes.is_empty() {
        println!("  none");
    } else {
        for route in &config.routes {
            println!("  {} -> {}", route.host, route.upstream);
        }
    }

    Ok(())
}

fn validate_config(config_path: &str) -> Result<(), String> {
    let mut failures = 0usize;

    fn pass(message: &str) {
        println!("[PASS] {message}");
    }

    fn fail(message: &str, failures: &mut usize) {
        *failures += 1;
        println!("[FAIL] {message}");
    }

    println!("Monceda Edge Config Validator");
    println!("────────────────────────────────");
    println!();

    let contents = match std::fs::read_to_string(config_path) {
        Ok(contents) => contents,
        Err(error) => {
            fail(
                &format!("Unable to read configuration {config_path}: {error}"),
                &mut failures,
            );

            println!();
            println!("Result: FAIL");

            return Err("Configuration validation failed".to_string());
        }
    };

    let config: Config = match toml::from_str(&contents) {
        Ok(config) => {
            pass("TOML parsed successfully");
            config
        }

        Err(error) => {
            fail(&format!("Invalid TOML: {error}"), &mut failures);

            println!();
            println!("Result: FAIL");

            return Err("Configuration validation failed".to_string());
        }
    };

    if config.server.host.trim().is_empty() {
        fail("Server host cannot be empty", &mut failures);
    } else if config.server.port == 0 {
        fail("Server port cannot be 0", &mut failures);
    } else if config.server.static_dir.trim().is_empty() {
        fail("Static directory cannot be empty", &mut failures);
    } else {
        pass("Server configuration valid");
    }

    if config.http.enabled && config.http.port == 0 {
        fail("HTTP port cannot be 0", &mut failures);
    } else {
        pass("HTTP configuration valid");
    }

    if config.tls.enabled {
        if config.tls.port == 0 {
            fail("TLS port cannot be 0", &mut failures);
        }

        if config.tls.cert.trim().is_empty() {
            fail("TLS certificate path cannot be empty", &mut failures);
        }

        if config.tls.key.trim().is_empty() {
            fail("TLS private key path cannot be empty", &mut failures);
        }
    }

    if config.tls.enabled
        && config.tls.port != 0
        && !config.tls.cert.trim().is_empty()
        && !config.tls.key.trim().is_empty()
    {
        pass("TLS configuration valid");
    } else if !config.tls.enabled {
        pass("TLS configuration valid");
    }

    if config.proxy.enabled {
        if !(config.proxy.upstream.starts_with("http://")
            || config.proxy.upstream.starts_with("https://"))
        {
            fail(
                &format!("Invalid proxy upstream: {}", config.proxy.upstream),
                &mut failures,
            );
        } else {
            pass("Proxy configuration valid");
        }
    } else {
        pass("Proxy configuration valid");
    }

    let mut hosts = std::collections::HashSet::<String>::new();
    let route_failures_before = failures;

    for route in &config.routes {
        let host = route.host.trim().trim_end_matches('.').to_lowercase();

        if host.is_empty() {
            fail("Route host cannot be empty", &mut failures);
        } else if !hosts.insert(host.clone()) {
            fail(&format!("Duplicate route host: {host}"), &mut failures);
        }

        if !(route.upstream.starts_with("http://") || route.upstream.starts_with("https://")) {
            fail(
                &format!(
                    "Invalid route upstream for {}: {}",
                    route.host, route.upstream
                ),
                &mut failures,
            );
        }
    }

    if failures == route_failures_before {
        pass(&format!("{} route(s) valid", config.routes.len()));
    }

    if config.acme.enabled {
        if config.acme.environment != "staging" && config.acme.environment != "production" {
            fail(
                &format!("Unknown ACME environment: {}", config.acme.environment),
                &mut failures,
            );
        }

        if config.acme.challenge != "http-01" {
            fail(
                &format!("Unsupported ACME challenge: {}", config.acme.challenge),
                &mut failures,
            );
        }

        if config.acme.cache_dir.trim().is_empty() {
            fail("ACME cache directory cannot be empty", &mut failures);
        }
    }

    if failures == route_failures_before
        && (!config.acme.enabled
            || ((config.acme.environment == "staging" || config.acme.environment == "production")
                && config.acme.challenge == "http-01"
                && !config.acme.cache_dir.trim().is_empty()))
    {
        pass("ACME configuration valid");
    }

    if config.deployment.require_public_ports && (config.http.port != 80 || config.tls.port != 443)
    {
        fail(
            "Public deployment requires HTTP 80 and HTTPS 443",
            &mut failures,
        );
    }

    if config.deployment.require_public_domains
        && config.public_node.primary_domain.trim().is_empty()
    {
        fail("Public deployment requires primary_domain", &mut failures);
    }

    if failures == 0 {
        pass("Deployment configuration valid");
    }

    println!();
    println!("────────────────────────────────");

    if failures > 0 {
        println!("Result: FAIL ({failures} failure(s))");

        Err(format!(
            "Configuration validation failed with {failures} failure(s)"
        ))
    } else {
        println!("Result: PASS");
        Ok(())
    }
}

fn doctor(config_path: &str) -> Result<(), String> {
    let mut warnings = 0usize;
    let mut failures = 0usize;

    fn pass(message: &str) {
        println!("[PASS] {message}");
    }

    fn warn(message: &str, warnings: &mut usize) {
        *warnings += 1;
        println!("[WARN] {message}");
    }

    fn fail(message: &str, failures: &mut usize) {
        *failures += 1;
        println!("[FAIL] {message}");
    }

    println!("Monceda Edge Doctor");
    println!("────────────────────────────────");
    println!();

    let config_file = std::path::Path::new(config_path);

    if !config_file.exists() {
        fail(
            &format!("Configuration file not found: {config_path}"),
            &mut failures,
        );

        println!();
        println!("Result: FAIL");

        return Err(format!("Doctor found {failures} failure(s)"));
    }

    if !config_file.is_file() {
        fail(
            &format!("Configuration path is not a file: {config_path}"),
            &mut failures,
        );

        println!();
        println!("Result: FAIL");

        return Err(format!("Doctor found {failures} failure(s)"));
    }

    pass(&format!("Configuration file found: {config_path}"));

    let config = match load_config(config_path) {
        Ok(config) => {
            pass("Configuration parsed successfully");
            config
        }

        Err(error) => {
            fail(&error, &mut failures);

            println!();
            println!("Result: FAIL");

            return Err(format!("Doctor found {failures} failure(s)"));
        }
    };

    if config.server.host.trim().is_empty() {
        fail("Server host is empty", &mut failures);
    } else {
        pass(&format!("Server host configured: {}", config.server.host));
    }

    if config.server.port == 0 {
        fail("Server port cannot be 0", &mut failures);
    } else {
        pass(&format!(
            "Internal server port configured: {}",
            config.server.port
        ));
    }

    if config.http.enabled {
        if config.http.port == 0 {
            fail("HTTP listener port cannot be 0", &mut failures);
        } else {
            pass(&format!(
                "HTTP listener enabled on port {}",
                config.http.port
            ));
        }
    } else {
        warn("HTTP listener is disabled", &mut warnings);
    }

    if config.tls.enabled {
        if config.tls.port == 0 {
            fail("HTTPS listener port cannot be 0", &mut failures);
        } else {
            pass(&format!(
                "HTTPS/TLS listener enabled on port {}",
                config.tls.port
            ));
        }

        if std::path::Path::new(&config.tls.cert).is_file() {
            pass(&format!("TLS certificate readable: {}", config.tls.cert));
        } else {
            fail(
                &format!("TLS certificate missing: {}", config.tls.cert),
                &mut failures,
            );
        }

        if std::path::Path::new(&config.tls.key).is_file() {
            pass(&format!("TLS private key readable: {}", config.tls.key));
        } else {
            fail(
                &format!("TLS private key missing: {}", config.tls.key),
                &mut failures,
            );
        }
    } else {
        warn("TLS is disabled", &mut warnings);
    }

    if config.proxy.enabled {
        if config.proxy.upstream.starts_with("http://")
            || config.proxy.upstream.starts_with("https://")
        {
            pass(&format!(
                "Default upstream valid: {}",
                config.proxy.upstream
            ));
        } else {
            fail(
                &format!(
                    "Default upstream is not HTTP/HTTPS: {}",
                    config.proxy.upstream
                ),
                &mut failures,
            );
        }
    } else {
        warn("Reverse proxy is disabled", &mut warnings);
    }

    if config.routes.is_empty() {
        warn("No host routes configured", &mut warnings);
    } else {
        pass(&format!("{} route(s) configured", config.routes.len()));

        let mut hosts = std::collections::HashSet::<String>::new();

        let route_failures_before = failures;
        for route in &config.routes {
            let host = route.host.trim().trim_end_matches('.').to_lowercase();

            if host.is_empty() {
                fail("Route contains an empty host", &mut failures);
            } else if !hosts.insert(host.clone()) {
                fail(&format!("Duplicate route host: {host}"), &mut failures);
            }

            if !(route.upstream.starts_with("http://") || route.upstream.starts_with("https://")) {
                fail(
                    &format!(
                        "Invalid upstream for route {}: {}",
                        route.host, route.upstream
                    ),
                    &mut failures,
                );
            }
        }

        if failures == route_failures_before {
            pass("Route definitions are structurally valid");
        }
    }

    if config.cache.enabled {
        if config.cache.ttl_seconds == 0 {
            warn("Cache enabled with TTL 0 seconds", &mut warnings);
        } else {
            pass(&format!(
                "Cache enabled with TTL {}s",
                config.cache.ttl_seconds
            ));
        }
    } else {
        warn("Cache is disabled", &mut warnings);
    }

    if config.acme.enabled {
        match config.acme.environment.as_str() {
            "staging" => {
                pass("ACME enabled in staging environment");
            }

            "production" => {
                pass("ACME enabled in production environment");
            }

            other => {
                fail(&format!("Unknown ACME environment: {other}"), &mut failures);
            }
        }

        if config.acme.email.trim().is_empty() {
            warn("ACME contact email is not configured", &mut warnings);
        } else {
            pass("ACME contact email configured");
        }

        if config.acme.challenge == "http-01" {
            pass("ACME HTTP-01 challenge configured");
        } else {
            warn(
                &format!(
                    "ACME challenge is '{}'; HTTP-01 is currently expected",
                    config.acme.challenge
                ),
                &mut warnings,
            );
        }

        if config.acme.cache_dir.trim().is_empty() {
            fail("ACME cache directory is empty", &mut failures);
        } else {
            pass(&format!(
                "ACME cache directory configured: {}",
                config.acme.cache_dir
            ));
        }
    } else {
        warn("ACME is disabled", &mut warnings);
    }

    if config.public_node.enabled {
        pass("Public-node mode enabled");

        if config.public_node.primary_domain.trim().is_empty() {
            fail("Public-node primary domain is missing", &mut failures);
        } else {
            pass(&format!(
                "Primary public domain configured: {}",
                config.public_node.primary_domain
            ));
        }

        if config.public_node.expected_ipv4.trim().is_empty() {
            warn("Expected public IPv4 is not configured", &mut warnings);
        } else {
            pass(&format!(
                "Expected public IPv4 configured: {}",
                config.public_node.expected_ipv4
            ));
        }
    } else {
        warn("Public-node mode is disabled", &mut warnings);
    }

    if config.deployment.require_public_ports {
        if config.http.port == 80 && config.tls.port == 443 {
            pass("Required public ports are HTTP 80 / HTTPS 443");
        } else {
            fail(
                &format!(
                    "Public deployment requires ports 80/443, configured {}/{}",
                    config.http.port, config.tls.port
                ),
                &mut failures,
            );
        }
    }

    if config.deployment.require_public_domains
        && config.public_node.primary_domain.trim().is_empty()
    {
        fail(
            "Deployment requires a public domain but none is configured",
            &mut failures,
        );
    }

    println!();
    println!("────────────────────────────────");

    if failures > 0 {
        println!("Result: FAIL ({failures} failure(s), {warnings} warning(s))");

        Err(format!("Doctor found {failures} failure(s)"))
    } else if warnings > 0 {
        println!("Result: WARN ({warnings} warning(s))");

        Ok(())
    } else {
        println!("Result: PASS");
        Ok(())
    }
}

fn load_config(path: &str) -> Result<Config, String> {
    match std::fs::read_to_string(path) {
        Ok(contents) => {
            toml::from_str(&contents).map_err(|e| format!("Invalid config {path}: {e}"))
        }

        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Config {
            server: ServerConfig::default(),
            proxy: ProxyConfig::default(),
            cache: CacheConfig::default(),
            tls: TlsConfig::default(),
            http: HttpConfig::default(),
            acme: AcmeConfig::default(),
            deployment: DeploymentConfig::default(),
            public_node: PublicNodeConfig::default(),
            routes: Vec::new(),
        }),

        Err(error) => Err(format!("Unable to read config {path}: {error}")),
    }
}

fn node_info(config_path: &str, json_output: bool) -> Result<(), String> {
    let config = load_config(config_path)?;

    let host = match config.server.host.as_str() {
        "0.0.0.0" | "::" => "127.0.0.1",
        other => other,
    };

    let (scheme, port) = if config.tls.enabled {
        ("https", config.tls.port)
    } else if config.http.enabled {
        ("http", config.http.port)
    } else {
        ("http", config.server.port)
    };

    let mode = if config.proxy.enabled {
        "reverse-proxy"
    } else {
        "static"
    };

    let endpoint = format!("{scheme}://{host}:{port}");

    if json_output {
        let value = serde_json::json!({
            "version": VERSION,
            "deployment_mode": config.deployment.mode,
            "mode": mode,
            "endpoint": endpoint,
            "server": {
                "host": config.server.host,
                "port": config.server.port
            },
            "http": {
                "enabled": config.http.enabled,
                "port": config.http.port
            },
            "tls": {
                "enabled": config.tls.enabled,
                "port": config.tls.port
            },
            "proxy": {
                "enabled": config.proxy.enabled,
                "upstream": config.proxy.upstream
            },
            "cache": {
                "enabled": config.cache.enabled,
                "ttl_seconds": config.cache.ttl_seconds
            },
            "routes": config.routes.len(),
            "public_node": {
                "enabled": config.public_node.enabled,
                "primary_domain": config.public_node.primary_domain,
                "expected_ipv4": config.public_node.expected_ipv4
            }
        });

        println!(
            "{}",
            serde_json::to_string_pretty(&value)
                .map_err(|error| format!("Unable to serialize node info: {error}"))?
        );

        return Ok(());
    }

    println!("Monceda Edge Node Information");
    println!("────────────────────────────");
    println!("Version:          {VERSION}");
    println!("Deployment mode:  {}", config.deployment.mode);
    println!("Mode:             {mode}");
    println!("Endpoint:         {endpoint}");
    println!("Server host:      {}", config.server.host);
    println!("Server port:      {}", config.server.port);

    println!(
        "HTTP:             {}",
        if config.http.enabled {
            format!("enabled ({})", config.http.port)
        } else {
            "disabled".to_string()
        }
    );

    println!(
        "TLS:              {}",
        if config.tls.enabled {
            format!("enabled ({})", config.tls.port)
        } else {
            "disabled".to_string()
        }
    );

    println!(
        "Proxy:            {}",
        if config.proxy.enabled {
            format!("enabled ({})", config.proxy.upstream)
        } else {
            "disabled".to_string()
        }
    );

    println!(
        "Cache:            {}",
        if config.cache.enabled {
            "enabled"
        } else {
            "disabled"
        }
    );

    println!("Routes:           {}", config.routes.len());

    println!(
        "Public node:      {}",
        if config.public_node.enabled {
            "enabled"
        } else {
            "disabled"
        }
    );

    if config.public_node.enabled {
        println!("Primary domain:   {}", config.public_node.primary_domain);
        println!("Expected IPv4:    {}", config.public_node.expected_ipv4);
    }

    Ok(())
}

async fn node_routes(config_path: &str, json_output: bool, check: bool) -> Result<(), String> {
    let config = load_config(config_path)?;

    if !check {
        if json_output {
            let routes: Vec<serde_json::Value> = config
                .routes
                .iter()
                .map(|route| {
                    serde_json::json!({
                        "host": route.host,
                        "upstream": route.upstream
                    })
                })
                .collect();

            let value = serde_json::json!({
                "count": routes.len(),
                "routes": routes
            });

            println!(
                "{}",
                serde_json::to_string_pretty(&value)
                    .map_err(|error| format!("Unable to serialize node routes: {error}"))?
            );

            return Ok(());
        }

        println!("Monceda Edge Node Routes");
        println!("────────────────────────");
        println!("Routes: {}", config.routes.len());

        if config.routes.is_empty() {
            println!("No host routes configured.");
            return Ok(());
        }

        println!();

        for (index, route) in config.routes.iter().enumerate() {
            println!("{}. {} -> {}", index + 1, route.host, route.upstream);
        }

        return Ok(());
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|error| format!("Unable to create route check client: {error}"))?;

    let mut checked_routes = Vec::new();
    let mut healthy = 0usize;

    for route in &config.routes {
        let result = client.get(&route.upstream).send().await;

        let (reachable, status) = match result {
            Ok(response) => {
                let code = response.status();
                (code.is_success(), Some(code.as_u16()))
            }
            Err(_) => (false, None),
        };

        if reachable {
            healthy += 1;
        }

        checked_routes.push(serde_json::json!({
            "host": route.host,
            "upstream": route.upstream,
            "reachable": reachable,
            "http_status": status
        }));
    }

    let unreachable = checked_routes.len().saturating_sub(healthy);

    if json_output {
        let value = serde_json::json!({
            "count": checked_routes.len(),
            "healthy": healthy,
            "unreachable": unreachable,
            "routes": checked_routes
        });

        println!(
            "{}",
            serde_json::to_string_pretty(&value)
                .map_err(|error| format!("Unable to serialize route checks: {error}"))?
        );

        return Ok(());
    }

    println!("Monceda Edge Node Routes");
    println!("────────────────────────");
    println!("Routes: {}", checked_routes.len());

    if checked_routes.is_empty() {
        println!("No host routes configured.");
        return Ok(());
    }

    println!();

    for (index, route) in checked_routes.iter().enumerate() {
        let host = route["host"].as_str().unwrap_or("");
        let upstream = route["upstream"].as_str().unwrap_or("");
        let reachable = route["reachable"].as_bool().unwrap_or(false);

        let state = if reachable { "UP" } else { "DOWN" };

        match route["http_status"].as_u64() {
            Some(status) => {
                println!(
                    "{}. {} -> {}  [{} HTTP {}]",
                    index + 1,
                    host,
                    upstream,
                    state,
                    status
                );
            }
            None => {
                println!("{}. {} -> {}  [{}]", index + 1, host, upstream, state);
            }
        }
    }

    println!();
    println!("Healthy:     {healthy}");
    println!("Unreachable: {unreachable}");

    Ok(())
}

async fn node_check(config_path: &str, json_output: bool, strict: bool) -> Result<(), String> {
    let config = load_config(config_path)?;

    let host = match config.server.host.as_str() {
        "0.0.0.0" | "::" => "127.0.0.1",
        other => other,
    };

    let (scheme, port) = if config.tls.enabled {
        ("https", config.tls.port)
    } else if config.http.enabled {
        ("http", config.http.port)
    } else {
        ("http", config.server.port)
    };

    let endpoint = format!("{scheme}://{host}:{port}/_edge/health");

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(
            matches!(host, "127.0.0.1" | "localhost" | "::1") && scheme == "https",
        )
        .build()
        .map_err(|error| format!("Unable to create node check client: {error}"))?;

    let response = client.get(&endpoint).send().await;

    let (reachable, http_status) = match response {
        Ok(response) => (
            response.status().is_success(),
            Some(response.status().as_u16()),
        ),
        Err(_) => (false, None),
    };

    let mode = if config.proxy.enabled {
        "reverse-proxy"
    } else {
        "static"
    };

    if json_output {
        let value = serde_json::json!({
            "result": if reachable { "PASS" } else { "WARN" },
            "version": VERSION,
            "deployment_mode": config.deployment.mode,
            "mode": mode,
            "endpoint": endpoint,
            "reachable": reachable,
            "http_status": http_status,
            "proxy": {
                "enabled": config.proxy.enabled,
                "upstream": config.proxy.upstream
            },
            "tls": {
                "enabled": config.tls.enabled,
                "port": config.tls.port
            },
            "http": {
                "enabled": config.http.enabled,
                "port": config.http.port
            },
            "routes": config.routes.len(),
            "public_node": {
                "enabled": config.public_node.enabled,
                "primary_domain": config.public_node.primary_domain
            }
        });

        println!(
            "{}",
            serde_json::to_string_pretty(&value)
                .map_err(|error| format!("Unable to serialize node check: {error}"))?
        );

        if strict && !reachable {
            return Err("Node check failed in strict mode".to_string());
        }

        return Ok(());
    }

    println!("Monceda Edge Node Check");
    println!("───────────────────────");
    println!("Version:          {VERSION}");
    println!("Deployment mode:  {}", config.deployment.mode);
    println!("Mode:             {mode}");
    println!("Endpoint:         {endpoint}");
    println!("Reachable:        {}", if reachable { "yes" } else { "no" });

    if let Some(status) = http_status {
        println!("HTTP status:      {status}");
    }

    println!(
        "Proxy:            {}",
        if config.proxy.enabled {
            "enabled"
        } else {
            "disabled"
        }
    );
    println!("Routes:           {}", config.routes.len());
    println!(
        "Public node:      {}",
        if config.public_node.enabled {
            "enabled"
        } else {
            "disabled"
        }
    );
    println!(
        "Result:           {}",
        if reachable { "PASS" } else { "WARN" }
    );

    if strict && !reachable {
        return Err("Node check failed in strict mode".to_string());
    }

    Ok(())
}

async fn status(config_path: &str, json_output: bool) -> Result<(), String> {
    let config = load_config(config_path)?;

    let host = match config.server.host.as_str() {
        "0.0.0.0" | "::" => "127.0.0.1",
        other => other,
    };

    let (scheme, port) = if config.tls.enabled {
        ("https", config.tls.port)
    } else if config.http.enabled {
        ("http", config.http.port)
    } else {
        ("http", config.server.port)
    };

    let url = format!("{scheme}://{host}:{port}/_edge/health");

    let is_local = matches!(host, "127.0.0.1" | "localhost" | "::1");

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(is_local && scheme == "https")
        .build()
        .map_err(|error| format!("Unable to create status client: {error}"))?;

    match client.get(&url).send().await {
        Ok(response) if response.status().is_success() => {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "running".to_string());

            if json_output {
                let health = serde_json::from_str::<serde_json::Value>(&body)
                    .unwrap_or_else(|_| serde_json::json!({ "raw": body }));

                let value = serde_json::json!({
                    "reachable": true,
                    "endpoint": url,
                    "health": health
                });

                println!(
                    "{}",
                    serde_json::to_string_pretty(&value)
                        .map_err(|error| format!("Unable to serialize node status: {error}"))?
                );

                return Ok(());
            }

            println!("Monceda Edge is running.");
            println!("Endpoint: {url}");
            println!("{body}");
            Ok(())
        }

        Ok(response) => {
            if json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "reachable": false,
                        "endpoint": url,
                        "http_status": response.status().as_u16()
                    }))
                    .map_err(|error| format!("Unable to serialize node status: {error}"))?
                );

                return Ok(());
            }

            Err(format!(
                "Monceda Edge responded with HTTP {} at {url}",
                response.status()
            ))
        }

        Err(_) => {
            if json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "reachable": false,
                        "endpoint": url
                    }))
                    .map_err(|error| format!("Unable to serialize node status: {error}"))?
                );

                return Ok(());
            }

            Err(format!("Monceda Edge is not reachable at {url}"))
        }
    }
}

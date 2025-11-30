# Prometheus Integration Example

This example demonstrates how to integrate `axum-gate` with Prometheus metrics to monitor authentication and authorization events.

## Features

- **Built-in Metrics**: Enables axum-gate's built-in Prometheus metrics for audit logging
- **Custom Metrics**: Shows how to add your own application-specific metrics
- **Metrics Endpoint**: Exposes a `/metrics` endpoint for Prometheus scraping
- **Authorization Tracking**: Monitors successful and failed authorization attempts
- **JWT Validation**: Tracks JWT validation failures with categorization
- **Account Operations**: Audits account creation, deletion, and related operations

## Built-in Metrics

When the `prometheus` feature is enabled, axum-gate automatically tracks:

- `axum_gate_authz_authorized_total` - Successful authorization decisions
- `axum_gate_authz_denied_total{reason}` - Failed authorizations with reason codes
- `axum_gate_jwt_invalid_total{kind}` - JWT validation failures (issuer/token)
- `axum_gate_account_delete_outcome_total{outcome,secret_restored}` - Account deletion events
- `axum_gate_account_insert_outcome_total{outcome,reason}` - Account creation events

## Running the Example

```bash
cargo run --example prometheus
```

Then visit:
- http://localhost:3000/ - Home page with login form
- http://localhost:3000/admin - Admin-only area (requires admin role)
- http://localhost:3000/metrics - Prometheus metrics endpoint

## Test Accounts

- **admin/admin** - Admin role (can access `/admin`)
- **user/user** - User role (cannot access `/admin`)

## Key Code Patterns

### Enabling Metrics

```rust
// Enable metrics with default registry
Gate::cookie("my-app", jwt_codec)
    .with_prometheus_metrics()

// Or use a custom registry
let registry = Registry::new();
Gate::cookie("my-app", jwt_codec)
    .with_prometheus_registry(&registry)
```

### Custom Metrics

The example also shows how to add your own metrics alongside axum-gate's built-in ones:

```rust
let login_attempts = Counter::with_opts(
    Opts::new("myapp_login_attempts_total", "Total login attempts")
).unwrap();

// Register with the same registry used by axum-gate
registry.register(Box::new(login_attempts.clone())).unwrap();
```

### Metrics Endpoint

```rust
async fn metrics_handler(State(registry): State<Registry>) -> Result<String, StatusCode> {
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    encoder.encode_to_string(&metric_families)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
```

## What Gets Tracked

1. **Login Attempts**: Every login attempt increments a counter
2. **Authorization Success**: Each successful access to protected routes
3. **Authorization Failures**: Failed access attempts with categorized reasons:
   - `no_token` - No JWT cookie present
   - `invalid_token` - JWT validation failed
   - `insufficient_role` - User doesn't have required role
   - `policy_denied` - Custom policy rejected the request
4. **JWT Issues**: Separate tracking for JWT-specific problems
5. **Account Operations**: User registration, deletion, and related events

## Integrating with Prometheus

Add this scrape config to your `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'axum-gate-example'
    static_configs:
      - targets: ['localhost:3000']
    metrics_path: '/metrics'
    scrape_interval: 15s
```

## Security Considerations

The built-in metrics are designed with security in mind:

- **No Sensitive Data**: Metrics never include passwords, tokens, or personal information
- **Stable Identifiers**: Uses UUIDs and user IDs rather than exposing internal state
- **Low Cardinality**: Reason codes are kept coarse-grained to prevent label explosion
- **Audit Focus**: Concentrates on security-relevant events rather than detailed application state

## Production Usage

In production:

1. Use a proper secret management system for JWT keys
2. Configure appropriate log levels for audit events
3. Set up proper Prometheus retention and alerting
4. Consider using a custom registry to isolate metrics
5. Monitor the metrics for unusual patterns that might indicate attacks

## Dependencies

This example requires the `prometheus` feature:

```toml
axum-gate = { version = "1.1.0", features = ["prometheus"] }
prometheus = "0.13"
```

//! The descriptions: stable, and the ones committed as `crates/server/openapi.json` and
//! `crates/server/admin-openapi.json`, which `npm run api:generate` regenerates.

use life_pixel_server::{admin, openapi};
use serde_json::Value;

/// The routes of the internal admin API, each with its methods.
const ADMIN_ROUTES: [(&str, &[&str]); 12] = [
    ("/audit-log", &["get"]),
    ("/environment", &["get"]),
    ("/metrics/product", &["get"]),
    ("/support-requests", &["get"]),
    ("/support-requests/{id}", &["get", "patch"]),
    ("/support-requests/{id}/messages", &["post"]),
    ("/support-requests/{id}/screenshot", &["get"]),
    ("/users", &["get"]),
    ("/users/{id}", &["delete", "get"]),
    ("/users/{id}/export", &["post"]),
    ("/users/{id}/reactivate", &["post"]),
    ("/users/{id}/suspend", &["post"]),
];

fn committed() -> String {
    read_committed("openapi.json")
}

fn read_committed(name: &str) -> String {
    let path = format!("{}/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path}: {error}"))
}

/// Whether every object of `value` has its keys sorted.
fn is_sorted(value: &Value) -> bool {
    match value {
        Value::Object(object) => {
            let keys: Vec<&String> = object.keys().collect();
            keys.is_sorted() && object.values().all(is_sorted)
        }
        Value::Array(items) => items.iter().all(is_sorted),
        _ => true,
    }
}

#[test]
fn the_description_is_stable_and_committed() {
    let printed = openapi::to_json().unwrap();
    assert_eq!(printed, openapi::to_json().unwrap());
    assert!(printed.ends_with("}\n"));
    assert!(
        printed == committed(),
        "crates/server/openapi.json is stale: run `npm run api:generate` in frontend/"
    );
}

#[test]
fn the_description_has_sorted_keys_the_routes_and_the_security_schemes() {
    let description: Value = serde_json::from_str(&openapi::to_json().unwrap()).unwrap();
    assert!(is_sorted(&description));
    assert!(description["paths"]["/healthz"]["get"].is_object());
    let schemes = &description["components"]["securitySchemes"];
    assert_eq!(schemes["session"]["name"], "__Host-lp_session");
    assert_eq!(schemes["csrf"]["name"], "X-CSRF-Token");
    assert_eq!(schemes["bearer"]["scheme"], "bearer");
    assert!(description["components"]["schemas"]["Problem"].is_object());
}

#[test]
fn the_admin_description_is_stable_and_committed() {
    let printed = admin::openapi::to_json().unwrap();
    assert_eq!(printed, admin::openapi::to_json().unwrap());
    assert!(printed.ends_with("}\n"));
    assert!(
        printed == read_committed("admin-openapi.json"),
        "crates/server/admin-openapi.json is stale: run `npm run api:generate` in frontend/"
    );
}

#[test]
fn the_admin_description_has_every_route_under_its_prefix_and_its_security() {
    let description: Value = serde_json::from_str(&admin::openapi::to_json().unwrap()).unwrap();
    assert!(is_sorted(&description));
    let paths = description["paths"].as_object().unwrap();
    assert_eq!(paths.len(), ADMIN_ROUTES.len());
    for (path, methods) in ADMIN_ROUTES {
        let operations = paths[&format!("/internal/admin/v1{path}")]
            .as_object()
            .unwrap();
        let listed: Vec<&str> = operations.keys().map(String::as_str).collect();
        assert_eq!(listed, methods, "{path}");
    }
    let schemes = &description["components"]["securitySchemes"];
    assert_eq!(schemes["adminSecret"]["scheme"], "bearer");
    assert_eq!(schemes["adminId"]["name"], "X-Admin-Id");
    assert_eq!(schemes["adminEmail"]["name"], "X-Admin-Email");
    let required = &description["security"][0];
    for scheme in ["adminSecret", "adminId", "adminEmail"] {
        assert!(required[scheme].is_array(), "{scheme}");
    }
}

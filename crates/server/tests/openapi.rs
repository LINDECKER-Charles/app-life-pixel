//! The description: stable, and the one committed as `crates/server/openapi.json`, which
//! `npm run api:generate` regenerates.

use life_pixel_server::openapi;
use serde_json::Value;

fn committed() -> String {
    let path = format!("{}/openapi.json", env!("CARGO_MANIFEST_DIR"));
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

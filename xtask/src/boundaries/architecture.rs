use super::{Member, Violation, Workspace};

/// Crates that do I/O, read the clock, draw randomness or run async code: the pure crates stay
/// free of them, those capabilities being injected.
const RUNTIME_CRATES: &[&str] = &[
    "async-trait",
    "chrono",
    "futures",
    "getrandom",
    "rand",
    "time",
    "tokio",
];

/// Crates of a transport or of a storage adapter: HTTP, Tauri, SQL, object storage.
const TRANSPORT_CRATES: &[&str] = &[
    "axum",
    "hyper",
    "object_store",
    "reqwest",
    "sqlx",
    "tauri",
    "tower",
    "tower-http",
    "utoipa",
    "utoipa-axum",
];

/// The MCP protocol, which `service` never knows about.
const MCP_CRATES: &[&str] = &["rmcp"];

const FORMAT: &str = "life-pixel-format";
const CORE: &str = "life-pixel-core";
const COMPILER: &str = "life-pixel-compiler";
const SERVICE: &str = "life-pixel-service";
const MCP: &str = "life-pixel-mcp";
const PLAYER: &str = "life-pixel-player";

/// The place of a workspace member in the dependency direction of AGENTS.md.
struct Layer {
    /// The member.
    package: &'static str,
    /// The only workspace members it may depend on.
    allowed_members: &'static [&'static str],
    /// Groups of external crates it must not depend on.
    forbidden_crates: &'static [&'static [&'static str]],
}

/// Every workspace member, bottom-up. The leaves — the server, the admin server, the CLI, the
/// desktop app and xtask — appear in no `allowed_members`: nothing depends on them. A new member
/// is declared here before it builds, since an undeclared member is a violation.
const LAYERS: [Layer; 12] = [
    Layer {
        package: FORMAT,
        allowed_members: &[],
        forbidden_crates: &[RUNTIME_CRATES],
    },
    Layer {
        package: PLAYER,
        allowed_members: &[FORMAT],
        forbidden_crates: &[RUNTIME_CRATES],
    },
    Layer {
        package: CORE,
        allowed_members: &[],
        forbidden_crates: &[RUNTIME_CRATES, TRANSPORT_CRATES],
    },
    Layer {
        package: COMPILER,
        allowed_members: &[CORE, FORMAT],
        forbidden_crates: &[RUNTIME_CRATES, TRANSPORT_CRATES],
    },
    Layer {
        package: "life-pixel-editor-wasm",
        allowed_members: &[CORE, FORMAT, COMPILER],
        forbidden_crates: &[TRANSPORT_CRATES],
    },
    Layer {
        package: SERVICE,
        allowed_members: &[CORE, FORMAT, COMPILER],
        forbidden_crates: &[TRANSPORT_CRATES, MCP_CRATES],
    },
    Layer {
        package: MCP,
        allowed_members: &[CORE, FORMAT, COMPILER, SERVICE],
        forbidden_crates: &[TRANSPORT_CRATES],
    },
    Layer {
        package: "life-pixel-server",
        allowed_members: &[CORE, FORMAT, COMPILER, SERVICE, MCP],
        forbidden_crates: &[],
    },
    Layer {
        package: "life-pixel-admin-server",
        allowed_members: &[CORE, FORMAT, COMPILER, SERVICE],
        forbidden_crates: &[],
    },
    Layer {
        package: "life-pixel-cli",
        allowed_members: &[CORE, FORMAT, COMPILER, SERVICE, MCP],
        forbidden_crates: &[],
    },
    Layer {
        package: "life-pixel-desktop",
        allowed_members: &[CORE, FORMAT, COMPILER, SERVICE, MCP],
        forbidden_crates: &[],
    },
    Layer {
        package: "xtask",
        allowed_members: &[CORE, FORMAT, COMPILER, PLAYER],
        forbidden_crates: &[],
    },
];

/// Every broken dependency direction of `workspace`: a member missing from [`LAYERS`], a member
/// depending on one it must not, or on an external crate its layer forbids.
pub fn check(workspace: &Workspace) -> Vec<Violation> {
    let member_names: Vec<&str> = workspace
        .members
        .iter()
        .map(|member| member.name.as_str())
        .collect();
    workspace
        .members
        .iter()
        .flat_map(|member| check_member(member, &member_names))
        .collect()
}

fn check_member(member: &Member, member_names: &[&str]) -> Vec<Violation> {
    let Some(layer) = LAYERS.iter().find(|layer| layer.package == member.name) else {
        return vec![Violation::UndeclaredMember {
            package: member.name.clone(),
        }];
    };
    member
        .dependencies
        .iter()
        .filter(|dependency| !is_allowed(layer, dependency, member_names))
        .map(|dependency| Violation::ForbiddenDependency {
            package: member.name.clone(),
            dependency: dependency.clone(),
        })
        .collect()
}

fn is_allowed(layer: &Layer, dependency: &str, member_names: &[&str]) -> bool {
    if member_names.contains(&dependency) {
        return layer.allowed_members.contains(&dependency);
    }
    !layer
        .forbidden_crates
        .iter()
        .any(|group| group.contains(&dependency))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn member(name: &str, dependencies: &[&str]) -> Member {
        Member {
            name: name.to_owned(),
            license: None,
            dependencies: dependencies
                .iter()
                .map(|&dependency| dependency.to_owned())
                .collect(),
        }
    }

    fn workspace(members: Vec<Member>) -> Workspace {
        Workspace {
            members,
            player_js: None,
        }
    }

    fn sound_members() -> Vec<Member> {
        vec![
            member(CORE, &["serde"]),
            member(COMPILER, &[CORE, "zip"]),
            member(SERVICE, &[CORE, COMPILER, "tokio"]),
            member(MCP, &[SERVICE, "rmcp"]),
            member("life-pixel-server", &[SERVICE, MCP, "axum", "sqlx"]),
        ]
    }

    fn forbidden(package: &str, dependency: &str) -> Violation {
        Violation::ForbiddenDependency {
            package: package.to_owned(),
            dependency: dependency.to_owned(),
        }
    }

    #[test]
    fn a_sound_workspace_passes() {
        assert_eq!(check(&workspace(sound_members())), []);
    }

    #[test]
    fn a_member_missing_from_the_layers_fails() {
        let mut members = sound_members();
        members.push(member("life-pixel-shortcut", &[]));

        let violations = check(&workspace(members));

        let package = "life-pixel-shortcut".to_owned();
        assert_eq!(violations, [Violation::UndeclaredMember { package }]);
    }

    #[test]
    fn the_core_depending_on_another_member_fails() {
        let mut members = sound_members();
        members[0].dependencies.push(SERVICE.to_owned());

        assert_eq!(check(&workspace(members)), [forbidden(CORE, SERVICE)]);
    }

    #[test]
    fn a_pure_crate_reaching_for_a_runtime_fails() {
        let mut members = sound_members();
        members[1].dependencies.push("tokio".to_owned());

        assert_eq!(check(&workspace(members)), [forbidden(COMPILER, "tokio")]);
    }

    #[test]
    fn the_service_knowing_http_or_mcp_fails() {
        let mut members = sound_members();
        members[2]
            .dependencies
            .extend(["axum".to_owned(), "rmcp".to_owned()]);

        let violations = check(&workspace(members));

        assert_eq!(
            violations,
            [forbidden(SERVICE, "axum"), forbidden(SERVICE, "rmcp")]
        );
    }

    #[test]
    fn the_mcp_crate_knowing_its_transport_fails() {
        let mut members = sound_members();
        members[3].dependencies.push("axum".to_owned());

        assert_eq!(check(&workspace(members)), [forbidden(MCP, "axum")]);
    }

    #[test]
    fn depending_on_a_leaf_fails() {
        let mut members = sound_members();
        members[3].dependencies.push("life-pixel-server".to_owned());

        let violations = check(&workspace(members));

        assert_eq!(violations, [forbidden(MCP, "life-pixel-server")]);
    }

    #[test]
    fn an_external_crate_only_counts_when_its_layer_forbids_it() {
        let members = vec![member("life-pixel-server", &["tokio", "axum", "rmcp"])];

        assert_eq!(check(&workspace(members)), []);
    }
}

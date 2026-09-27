//! The boundaries of AGENTS.md: what ships in users' apps stays MIT and depends on no AGPL code,
//! and the workspace crates depend on each other in one direction only.

mod architecture;
mod licence;
mod member;
mod violation;
mod workspace;

pub use member::Member;
pub use violation::Violation;
pub use workspace::Workspace;

/// Every broken boundary of `workspace`, each reported once even when both rules catch it.
pub fn check(workspace: &Workspace) -> Vec<Violation> {
    let mut violations: Vec<Violation> = Vec::new();
    let found = licence::check(workspace)
        .into_iter()
        .chain(architecture::check(workspace));
    for violation in found {
        if !violations.contains(&violation) {
            violations.push(violation);
        }
    }
    violations
}

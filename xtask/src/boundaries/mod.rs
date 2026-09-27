//! The licence boundary of AGENTS.md: what ships in users' apps stays MIT and depends on no
//! AGPL code.

mod member;
mod rules;
mod violation;
mod workspace;

pub use member::Member;
pub use rules::check;
pub use violation::Violation;
pub use workspace::Workspace;

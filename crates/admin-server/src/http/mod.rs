//! The HTTP conventions every route shares: problems, request ids, client addresses, security
//! headers, the session and its CSRF check, and the static files. One line per module.

pub mod client_address;
pub mod problem;
pub mod request_id;
pub mod security_headers;
pub mod session;
pub mod static_files;

//! The subcommands that do more than one call: one module per subcommand, `admins` for
//! `create-admin` and `disable-admin`.

pub mod admins;
pub mod healthcheck;
pub mod serve;

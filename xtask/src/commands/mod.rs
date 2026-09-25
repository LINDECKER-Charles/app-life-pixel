//! The dispatcher. A command is a file of this folder holding a `clap::Args` struct, whose doc
//! comment is its help, and a `run(self) -> anyhow::Result<()>` method; it is registered below
//! with one line: `<module>: <struct>,`.

/// Declares each command's module, the `Command` subcommand enum and its dispatch.
macro_rules! commands {
    ($($module:ident: $command:ident,)+) => {
        $(mod $module;)+

        /// A subcommand of `cargo xtask`.
        #[derive(clap::Subcommand)]
        pub enum Command {
            $(
                #[allow(missing_docs, reason = "the help comes from the command's struct")]
                $command($module::$command),
            )+
        }

        impl Command {
            /// Runs the command.
            pub fn run(self) -> anyhow::Result<()> {
                match self {
                    $(Self::$command(command) => command.run(),)+
                }
            }
        }
    };
}

commands! {
    build_editor: BuildEditor,
    build_player: BuildPlayer,
    check_boundaries: CheckBoundaries,
    measure_sizes: MeasureSizes,
}

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    bin_name = "em",
    name = "em",
    author = "ちょうていしょ",
    version = env!("CARGO_PKG_VERSION"),
    about = "A powerful, profile-based environment manager",
    long_about = None,
    color = clap::ColorChoice::Always
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize the shell environment
    Init,

    /// Manage environment profiles
    #[command(subcommand)]
    #[command(visible_alias = "pf")]
    Profile(ProfileCommands),

    /// Activate profiles or specific key-value pairs in the current session
    #[command(visible_alias = "use")]
    Activate {
        /// Profiles to activate or key-value pairs to set (e.g., work API_KEY=123)
        #[arg(required = true)]
        items: Vec<String>,
    },

    /// Deactivate profiles or specific keys in the current session
    #[command(visible_aliases = ["unuse", "drop"])]
    Deactivate {
        /// Profiles or keys to deactivate
        #[arg(required = true)]
        items: Vec<String>,
    },

    /// Manage global environment settings
    #[command(subcommand)]
    Global(GlobalCommands),

    /// Check the status of the current environment
    Status(CommandsStatusArgs),

    /// Launch the terminal UI
    Ui,

    /// Check for issues in the profiles directory (missing files, circular dependencies)
    Check,

    /// Attempt to fix issues in the profiles directory
    Fix,
}

#[derive(Subcommand, Debug)]
pub enum ProfileCommands {
    /// List all available profiles
    List {
        /// Whether to expand profile contents in a tree structure
        #[arg(short, long)]
        expand: bool,
    },
    /// Create a new, empty profile
    Create { name: String },
    /// Rename a profile
    Rename(ProfileRenameArgs),
    /// Delete a profile
    #[command(visible_alias = "rm")]
    Delete { name: String },
    /// Add nested profiles or variables to a specific profile
    Add {
        /// The name of the profile to modify
        #[arg(required = true)]
        name: String,
        /// Nested profiles to add or variables to set (e.g., another_profile KEY=VALUE)
        #[arg(required = true)]
        items: Vec<String>,
    },
    /// Remove nested profiles or variables from a specific profile
    Remove {
        /// The name of the profile to modify
        #[arg(required = true)]
        name: String,
        /// Nested profiles or variable keys to remove
        #[arg(required = true)]
        items: Vec<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum GlobalCommands {
    /// Add profiles or key-value pairs to the global settings
    Add {
        /// Profiles to add or key-value pairs to set (e.g., work EDITOR=vim)
        #[arg(required = true)]
        items: Vec<String>,
    },
    /// Remove profiles or keys from the global settings
    Remove {
        /// Profiles or keys to remove
        #[arg(required = true)]
        items: Vec<String>,
    },
    /// List all global settings
    List {
        /// Whether to expand profile contents in a tree structure
        #[arg(short, long)]
        expand: bool,
    },
    /// Clear all global settings and unset corresponding environment variables in current shell
    Clean,
}

#[derive(Debug, Args)]
pub struct CommandsStatusArgs {
    /// Check the activation status of specific profiles
    pub profiles: Vec<String>,
    /// Whether to expand profile contents in a tree structure
    #[arg(short, long)]
    pub expand: bool,
}

#[derive(Debug, Args)]
pub struct ProfileRenameArgs {
    #[arg(help = "Source profile name", value_name = "SOURCE_PROFILE_NAME")]
    pub src_name: String,
    #[arg(help = "Destination profile name", value_name = "DEST_PROFILE_NAME")]
    pub dest_name: String,
}

impl Cli {
    pub fn parse_args() -> Self {
        Cli::parse()
    }
}

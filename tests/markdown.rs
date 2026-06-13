use clap::{Arg, ArgAction, Command, Parser, Subcommand, ValueEnum};
use clap_markdown::{
    MarkdownOptions, generate_markdown, generate_markdown_for_command,
    generate_markdown_for_command_with_options,
};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Network {
    Mainnet,
    Testnet,
    Devnet,
}

#[derive(Debug, Parser)]
#[command(name = "example", about = "Example command line interface")]
struct Cli {
    /// Path to the configuration file.
    #[arg(short, long, env = "EXAMPLE_CONFIG", default_value = "config.toml")]
    config: String,

    /// Increase output verbosity.
    #[arg(short, long, action = ArgAction::Count)]
    verbose: u8,

    /// Network to connect to.
    #[arg(long, value_enum, default_value_t = Network::Mainnet)]
    network: Network,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Start the daemon.
    Daemon {
        /// RPC bind address.
        #[arg(long, default_value = "127.0.0.1:8080")]
        rpc_bind: String,
    },
}

#[test]
fn parser_type_renders_parameter_metadata_and_anchors() {
    let markdown = generate_markdown::<Cli>();

    assert!(markdown.contains("# `example`"));
    assert!(markdown.contains("Example command line interface"));
    assert!(markdown.contains("- [`-c <CONFIG>, --config <CONFIG>`](#example-config)"));
    assert!(markdown.contains("<a id=\"example-config\"></a>"));
    assert!(markdown.contains("### `-c <CONFIG>, --config <CONFIG>`"));
    assert!(markdown.contains("Path to the configuration file"));
    assert!(markdown.contains("| Anchor | `#example-config` |"));
    assert!(markdown.contains("| Default value | `config.toml` |"));
    assert!(markdown.contains("| Environment | `EXAMPLE_CONFIG` |"));
}

#[test]
fn parser_type_renders_flags_enums_and_recursive_subcommands() {
    let markdown = generate_markdown::<Cli>();

    assert!(markdown.contains("<a id=\"example-verbose\"></a>"));
    assert!(markdown.contains("### `-v, --verbose`"));
    assert!(markdown.contains("| Value | No |"));
    assert!(markdown.contains("| Multiple | Yes |"));
    assert!(markdown.contains("| Possible values | `mainnet`, `testnet`, `devnet` |"));
    assert!(markdown.contains("<a id=\"example-daemon\"></a>"));
    assert!(markdown.contains("- [`daemon`](#example-daemon): Start the daemon"));
    assert!(markdown.contains("## `example daemon`"));
    assert!(markdown.contains("<a id=\"example-daemon-rpc-bind\"></a>"));
    assert!(markdown.contains("#### `--rpc-bind <RPC_BIND>`"));
    assert!(markdown.contains("| Default value | `127.0.0.1:8080` |"));
}

#[test]
fn generated_help_and_version_actions_are_not_documented() {
    let markdown = generate_markdown_for_command(
        Command::new("wallet").version("1.2.3").arg(
            Arg::new("password")
                .long("password")
                .help("Wallet password"),
        ),
    );

    assert!(markdown.contains("--password"));
    assert!(!markdown.contains("--help"));
    assert!(!markdown.contains("--version"));
    assert!(!markdown.contains("#wallet-help"));
    assert!(!markdown.contains("#wallet-version"));
}

#[test]
fn options_can_hide_toc_hidden_items_and_subcommands() {
    let command = Command::new("app")
        .arg(Arg::new("public").long("public"))
        .arg(Arg::new("secret").long("secret").hide(true))
        .subcommand(Command::new("run").arg(Arg::new("threads").long("threads")));

    let markdown = generate_markdown_for_command_with_options(
        command,
        MarkdownOptions {
            include_hidden: false,
            include_subcommands: false,
            include_toc: false,
        },
    );

    assert!(markdown.contains("### `--public`"));
    assert!(!markdown.contains("--secret"));
    assert!(!markdown.contains("## Parameters"));
    assert!(!markdown.contains("## Subcommands"));
    assert!(!markdown.contains("app run"));
}

#[test]
fn options_can_include_hidden_items() {
    let command = Command::new("app")
        .arg(Arg::new("public").long("public"))
        .arg(Arg::new("secret").long("secret").hide(true));

    let markdown = generate_markdown_for_command_with_options(
        command,
        MarkdownOptions {
            include_hidden: true,
            include_subcommands: true,
            include_toc: true,
        },
    );

    assert!(markdown.contains("### `--public`"));
    assert!(markdown.contains("### `--secret`"));
    assert!(markdown.contains("<a id=\"app-secret\"></a>"));
}

#[test]
fn table_cells_escape_markdown_control_characters() {
    let command = Command::new("app").arg(
        Arg::new("pattern")
            .long("pattern")
            .value_name("LEFT|RIGHT")
            .default_value("[a|b]")
            .help("Pattern value"),
    );

    let markdown = generate_markdown_for_command(command);

    assert!(markdown.contains("### `--pattern <LEFT|RIGHT>`"));
    assert!(markdown.contains("| Value name | `LEFT\\|RIGHT` |"));
    assert!(markdown.contains("| Default value | `\\[a\\|b\\]` |"));
}

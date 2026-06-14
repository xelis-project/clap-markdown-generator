use clap::{Arg, ArgAction, Command, Parser, Subcommand, ValueEnum};
use clap_markdown::{
    CommandHeadingStyle, MarkdownOptions, ParameterContentStyle, ParameterHeadingStyle,
    SummaryEntryStyle, SummaryOptions, SummaryValueStyle, generate_markdown,
    generate_markdown_for_command, generate_markdown_for_command_with_options,
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

#[derive(Debug, Parser)]
#[command(name = "example", about = "Example command line interface")]
struct ReadmeCli {
    /// Path to the configuration file.
    #[arg(short, long, env = "EXAMPLE_CONFIG", default_value = "config.toml")]
    config: String,

    /// Network to connect to.
    #[arg(long, value_enum, default_value_t = Network::Mainnet)]
    network: Network,

    #[command(subcommand)]
    command: ReadmeCommands,
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

#[derive(Debug, Subcommand)]
enum ReadmeCommands {
    /// Start the daemon.
    Daemon {
        /// RPC bind address.
        #[arg(long, default_value = "127.0.0.1:8080")]
        rpc_bind: String,
    },
}

#[test]
fn readme_usage_output_stays_in_sync() {
    let markdown = generate_markdown::<ReadmeCli>();

    assert_eq!(
        markdown,
        r#"# `example`

Example command line interface

## Parameters

- [`-c <CONFIG>, --config <CONFIG>`](#example-config): Path to the configuration file
- [`--network <NETWORK>`](#example-network): Network to connect to

<a id="example-config"></a>
### `-c <CONFIG>, --config <CONFIG>`

Path to the configuration file

| Field | Value |
| --- | --- |
| Anchor | `#example-config` |
| Required | No |
| Value | Yes |
| Value name | `CONFIG` |
| Default value | `config.toml` |
| Environment | `EXAMPLE_CONFIG` |

<a id="example-network"></a>
### `--network <NETWORK>`

Network to connect to

| Field | Value |
| --- | --- |
| Anchor | `#example-network` |
| Required | No |
| Value | Yes |
| Value name | `NETWORK` |
| Default value | `mainnet` |
| Possible values | `mainnet`, `testnet`, `devnet` |

## Subcommands

<a id="example-daemon"></a>
- [`daemon`](#example-daemon): Start the daemon

## `example daemon`

Start the daemon

### Parameters

- [`--rpc-bind <RPC_BIND>`](#example-daemon-rpc-bind): RPC bind address

<a id="example-daemon-rpc-bind"></a>
#### `--rpc-bind <RPC_BIND>`

RPC bind address

| Field | Value |
| --- | --- |
| Anchor | `#example-daemon-rpc-bind` |
| Required | No |
| Value | Yes |
| Value name | `RPC_BIND` |
| Default value | `127.0.0.1:8080` |
"#
    );
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
fn parameter_summary_uses_only_the_first_comment_line() {
    let markdown = generate_markdown_for_command(
        Command::new("app").arg(
            Arg::new("config")
                .long("config")
                .help("First line summary.\n\nSecond line with more details."),
        ),
    );

    assert!(markdown.contains("- [`--config`](#app-config): First line summary."));
    assert!(markdown.contains("First line summary.\n\nSecond line with more details."));
    assert!(!markdown.contains("- [`--config`](#app-config): Second line with more details."));
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
            ..MarkdownOptions::default()
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
            ..MarkdownOptions::default()
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

#[test]
fn options_can_skip_detailed_parameter_sections() {
    let markdown = generate_markdown_for_command_with_options(
        Command::new("app").arg(Arg::new("config").long("config").help("Config path")),
        MarkdownOptions {
            skip_parameter_details: true,
            ..MarkdownOptions::default()
        },
    );

    assert!(markdown.contains("- [`--config`](#app-config): Config path"));
    assert!(!markdown.contains("### `--config`"));
    assert!(!markdown.contains("| Field | Value |"));
}

#[test]
fn options_can_hide_html_anchor_ids() {
    let markdown = generate_markdown_for_command_with_options(
        Command::new("app")
            .arg(Arg::new("config").long("config"))
            .subcommand(Command::new("run").arg(Arg::new("threads").long("threads"))),
        MarkdownOptions {
            include_html_anchors: false,
            ..MarkdownOptions::default()
        },
    );

    assert!(!markdown.contains("<a id="));
    assert!(markdown.contains("- [`--config`](#app-config)"));
    assert!(markdown.contains("| Anchor | `#app-config` |"));
    assert!(markdown.contains("- [`run`](#app-run)"));
    assert!(markdown.contains("| Anchor | `#app-run-threads` |"));
}

#[test]
fn command_heading_can_be_customized_or_skipped() {
    let custom = generate_markdown_for_command_with_options(
        Command::new("app").arg(Arg::new("config").long("config")),
        MarkdownOptions {
            command_heading: CommandHeadingStyle::custom(|command| {
                format!("Command: `{}`", command.display)
            }),
            skip_parameter_details: true,
            ..MarkdownOptions::default()
        },
    );

    assert!(custom.starts_with("# Command: `app`"));

    let skipped = generate_markdown_for_command_with_options(
        Command::new("app").arg(Arg::new("config").long("config")),
        MarkdownOptions {
            command_heading: CommandHeadingStyle::None,
            skip_parameter_details: true,
            ..MarkdownOptions::default()
        },
    );

    assert!(!skipped.contains("# `app`"));
    assert!(skipped.starts_with("## Parameters"));
}

#[test]
fn summary_can_render_names_only_without_descriptions() {
    let markdown = generate_markdown_for_command_with_options(
        Command::new("app").arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("CONFIG")
                .help("Config path"),
        ),
        MarkdownOptions {
            skip_parameter_details: true,
            summary: SummaryOptions {
                enabled: true,
                value_style: SummaryValueStyle::NamesOnly,
                include_description: false,
                ..SummaryOptions::default()
            },
            ..MarkdownOptions::default()
        },
    );

    assert!(markdown.contains("- [`-c, --config`](#app-config)"));
    assert!(!markdown.contains("- [`-c <CONFIG>, --config <CONFIG>`](#app-config)"));
    assert!(!markdown.contains(": Config path"));
}

#[test]
fn summary_can_be_disabled() {
    let markdown = generate_markdown_for_command_with_options(
        Command::new("app").arg(Arg::new("config").long("config")),
        MarkdownOptions {
            summary: SummaryOptions {
                enabled: false,
                ..SummaryOptions::default()
            },
            ..MarkdownOptions::default()
        },
    );

    assert!(!markdown.contains("## Parameters"));
    assert!(markdown.contains("### `--config`"));
}

#[test]
fn parameter_heading_can_use_name_or_custom_template() {
    let named = generate_markdown_for_command_with_options(
        Command::new("app").arg(Arg::new("config").long("config")),
        MarkdownOptions {
            parameter_heading: ParameterHeadingStyle::Name,
            ..MarkdownOptions::default()
        },
    );

    assert!(named.contains("### config"));
    assert!(!named.contains("### `--config`"));

    let custom = generate_markdown_for_command_with_options(
        Command::new("app").arg(
            Arg::new("config")
                .long("config")
                .value_name("CONFIG")
                .help("Config path"),
        ),
        MarkdownOptions {
            parameter_heading: ParameterHeadingStyle::custom(|parameter| {
                format!("{} ({})", parameter.name, parameter.display)
            }),
            ..MarkdownOptions::default()
        },
    );

    assert!(custom.contains("### config (--config <CONFIG>)"));
}

#[test]
fn summary_entries_can_use_custom_callback() {
    let markdown = generate_markdown_for_command_with_options(
        Command::new("app").arg(
            Arg::new("config")
                .long("config")
                .value_name("CONFIG")
                .help("Config path"),
        ),
        MarkdownOptions {
            skip_parameter_details: true,
            summary: SummaryOptions {
                entry: SummaryEntryStyle::custom(|parameter| {
                    format!("- `{}` uses anchor `#{}`", parameter.name, parameter.anchor)
                }),
                ..SummaryOptions::default()
            },
            ..MarkdownOptions::default()
        },
    );

    assert!(markdown.contains("- `config` uses anchor `#app-config`"));
    assert!(!markdown.contains("- [`--config <CONFIG>`](#app-config): Config path"));
}

#[test]
fn parameter_content_can_use_custom_callback() {
    let markdown = generate_markdown_for_command_with_options(
        Command::new("app").arg(
            Arg::new("config")
                .long("config")
                .value_name("CONFIG")
                .help("Config path"),
        ),
        MarkdownOptions {
            parameter_content: ParameterContentStyle::custom(|parameter| {
                format!(
                    "Custom content for `{}` with values: `{}`.",
                    parameter.name,
                    parameter.value_names.join(", ")
                )
            }),
            ..MarkdownOptions::default()
        },
    );

    assert!(!markdown.contains("| Field | Value |"));
    assert!(markdown.contains("Custom content for `config` with values: `CONFIG`."));
}

#[test]
fn parameter_content_can_render_as_text() {
    let markdown = generate_markdown_for_command_with_options(
        Command::new("app").arg(
            Arg::new("config")
                .long("config")
                .value_name("CONFIG")
                .default_value("config.toml")
                .env("APP_CONFIG")
                .help("Config path"),
        ),
        MarkdownOptions {
            parameter_content: ParameterContentStyle::Text,
            ..MarkdownOptions::default()
        },
    );

    assert!(!markdown.contains("| Field | Value |"));
    assert!(markdown.contains(
        "Anchor: `#app-config`. Required: No. Value: Yes. Value name: `CONFIG`. Default value: `config.toml`. Environment: `APP_CONFIG`."
    ));
}

# clap-markdown-generator

Generate Markdown documentation from [`clap`](https://docs.rs/clap) command definitions.

`clap-markdown-generator` is useful when a CLI parser is already the source of truth and you want reference documentation for every available parameter without maintaining a second hand-written list. The generated Markdown includes anchors, descriptions, default values, environment variables, possible values, required/value metadata, and recursive subcommand sections.

## Installation

```toml
[dependencies]
clap-markdown-generator = "0.1"
```

Your CLI type should use `clap` as usual:

```toml
[dependencies]
clap = { version = "4.5", features = ["derive", "env"] }
clap-markdown-generator = "0.1"
```

## Usage

```rust
use clap::{Parser, Subcommand, ValueEnum};

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

fn main() {
    let markdown = clap_markdown_generator::generate_markdown::<Cli>();
    println!("{markdown}");
}
```

The output contains stable HTML anchors for each parameter. The parameter list uses only the first line of each parameter description as its summary, while the detailed parameter section keeps the full description.

```markdown
# `example`

Example command line interface

## Parameters

- [`-c <CONFIG>, --config <CONFIG>`](#example-config): Path to the configuration file
- [`--network <NETWORK>`](#example-network): Network to connect to

<a id="example-config"></a>
### `-c <CONFIG>, --config <CONFIG>`

Path to the configuration file

| Field | Value |
| --- | --- |
| Usage | `-c <CONFIG>, --config <CONFIG>` |
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
| Usage | `--network <NETWORK>` |
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
| Usage | `--rpc-bind <RPC_BIND>` |
| Required | No |
| Value | Yes |
| Value name | `RPC_BIND` |
| Default value | `127.0.0.1:8080` |
```

## API

Use `generate_markdown::<T>()` for a type that derives `clap::Parser` or otherwise implements `clap::CommandFactory`.

```rust
let markdown = clap_markdown_generator::generate_markdown::<Cli>();
```

Use `generate_markdown_for_command` when you already have a `clap::Command`.

```rust
let command = <Cli as clap::CommandFactory>::command();
let markdown = clap_markdown_generator::generate_markdown_for_command(command);
```

Use `generate_markdown_for_command_with_options` to customize rendering.

```rust
use clap_markdown_generator::{
    CommandHeadingStyle, MarkdownOptions, ParameterContentStyle, ParameterHeadingStyle,
    SummaryEntryStyle, SummaryOptions, SummaryValueStyle,
    generate_markdown_for_command_with_options,
};

let markdown = generate_markdown_for_command_with_options(
    <Cli as clap::CommandFactory>::command(),
    MarkdownOptions {
        include_hidden: false,
        include_subcommands: true,
        include_toc: true,
        skip_parameter_details: false,
        include_html_anchors: true,
        include_usage: true,
        command_heading: CommandHeadingStyle::Display,
        summary: SummaryOptions {
            enabled: true,
            value_style: SummaryValueStyle::NamesAndValues,
            include_description: true,
            entry: SummaryEntryStyle::Default,
        },
        parameter_heading: ParameterHeadingStyle::Display,
        parameter_content: ParameterContentStyle::Table,
    },
);
```

## Rendering Options

| Option | Default | Description |
| --- | --- | --- |
| `include_hidden` | `false` | Include clap arguments and subcommands marked as hidden. |
| `include_subcommands` | `true` | Recursively document subcommands. |
| `include_toc` | `true` | Include the parameter summary section. |
| `skip_parameter_details` | `false` | Skip the detailed parameter sections like `### --config <CONFIG>`. |
| `include_html_anchors` | `true` | Include explicit HTML anchor id elements like `<a id="example-config"></a>`. |
| `include_usage` | `true` | Include usage content like `Usage: --config <CONFIG>` in detailed parameter sections. |
| `command_heading` | `Display` | Render command headings with the default full command path, skip them, or use a callback. |
| `summary` | names + values + descriptions | Configure whether the parameter summary is enabled, whether it shows only names or names plus values, whether it includes the first description line, or use a callback for each entry. |
| `parameter_heading` | `Display` | Render detailed parameter headings as the full display form, the clap argument name, or a callback. |
| `parameter_content` | `Table` | Render detailed parameter metadata as a table, compact text, or a callback. |

### Summary Style

The summary can be disabled entirely or reduced to just names:

```rust
MarkdownOptions {
    summary: SummaryOptions {
        enabled: true,
        value_style: SummaryValueStyle::NamesOnly,
        include_description: false,
    },
    ..MarkdownOptions::default()
}
```

This changes summary entries from:

```markdown
- [`-c <CONFIG>, --config <CONFIG>`](#example-config): Path to the configuration file.
```

to:

```markdown
- [`-c, --config`](#example-config)
```

### Detail Style

Detailed parameter sections can be skipped:

```rust
MarkdownOptions {
    skip_parameter_details: true,
    ..MarkdownOptions::default()
}
```

Explicit HTML anchor id elements can be hidden while keeping generated anchor names available for links and metadata:

```rust
MarkdownOptions {
    include_html_anchors: false,
    ..MarkdownOptions::default()
}
```

Command headings can be customized or skipped:

```rust
MarkdownOptions {
    command_heading: CommandHeadingStyle::custom(|command| {
        format!("Command: `{}`", command.display)
    }),
    ..MarkdownOptions::default()
}
```

Use `CommandHeadingStyle::None` to omit command headings. A custom callback can also return an empty string to skip a specific heading.

Summary entries can use a callback for complete control over each list item:

```rust
MarkdownOptions {
    summary: SummaryOptions {
        entry: SummaryEntryStyle::custom(|parameter| {
            format!("- `{}` uses anchor `#{}`", parameter.name, parameter.anchor)
        }),
        ..SummaryOptions::default()
    },
    ..MarkdownOptions::default()
}
```

Parameter headings can use the full display form, only the clap argument name, or a callback:

```rust
MarkdownOptions {
    parameter_heading: ParameterHeadingStyle::Name,
    ..MarkdownOptions::default()
}
```

```rust
MarkdownOptions {
    parameter_heading: ParameterHeadingStyle::custom(|parameter| {
        format!("{} ({})", parameter.name, parameter.display)
    }),
    ..MarkdownOptions::default()
}
```

Parameter content can use compact text instead of a table:

```rust
MarkdownOptions {
    parameter_content: ParameterContentStyle::Text,
    ..MarkdownOptions::default()
}
```

Or provide your own callback:

```rust
MarkdownOptions {
    parameter_content: ParameterContentStyle::custom(|parameter| {
        format!(
            "Required: {}. Values: {}.",
            parameter.required,
            parameter.value_names.join(", ")
        )
    }),
    ..MarkdownOptions::default()
}
```

`clap-markdown-generator` filters clap-generated help and version actions from the output so the documentation focuses on user-defined CLI parameters.

## Development

```sh
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

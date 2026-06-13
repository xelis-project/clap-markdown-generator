# clap-markdown

Generate Markdown documentation from [`clap`](https://docs.rs/clap) command definitions.

`clap-markdown` is useful when a CLI parser is already the source of truth and you want reference documentation for every available parameter without maintaining a second hand-written list. The generated Markdown includes anchors, descriptions, default values, environment variables, possible values, required/value metadata, and recursive subcommand sections.

## Installation

```toml
[dependencies]
clap-markdown = "0.1"
```

Your CLI type should use `clap` as usual:

```toml
[dependencies]
clap = { version = "4.5", features = ["derive", "env"] }
clap-markdown = "0.1"
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
    let markdown = clap_markdown::generate_markdown::<Cli>();
    println!("{markdown}");
}
```

The output contains stable HTML anchors for each parameter. The parameter list uses only the first line of each parameter description as its summary, while the detailed parameter section keeps the full description.

```markdown
# `example`

Example command line interface

## Parameters

- [`-c <CONFIG>, --config <CONFIG>`](#example-config): Path to the configuration file

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
```

## API

Use `generate_markdown::<T>()` for a type that derives `clap::Parser` or otherwise implements `clap::CommandFactory`.

```rust
let markdown = clap_markdown::generate_markdown::<Cli>();
```

Use `generate_markdown_for_command` when you already have a `clap::Command`.

```rust
let command = <Cli as clap::CommandFactory>::command();
let markdown = clap_markdown::generate_markdown_for_command(command);
```

Use `generate_markdown_for_command_with_options` to customize rendering.

```rust
use clap_markdown::{MarkdownOptions, generate_markdown_for_command_with_options};

let markdown = generate_markdown_for_command_with_options(
    <Cli as clap::CommandFactory>::command(),
    MarkdownOptions {
        include_hidden: false,
        include_subcommands: true,
        include_toc: true,
    },
);
```

## Rendering Options

| Option | Default | Description |
| --- | --- | --- |
| `include_hidden` | `false` | Include clap arguments and subcommands marked as hidden. |
| `include_subcommands` | `true` | Recursively document subcommands. |
| `include_toc` | `true` | Include a small parameter list with links to each anchor. |

`clap-markdown` filters clap-generated help and version actions from the output so the documentation focuses on user-defined CLI parameters.

## Development

```sh
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

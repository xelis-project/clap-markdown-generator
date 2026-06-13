//! Generate Markdown documentation from a [`clap`] command definition.
//!
//! The most common entry point is [`generate_markdown`], which accepts any type
//! deriving `clap::Parser`.
//!
//! ```
//! use clap::Parser;
//!
//! #[derive(Parser)]
//! #[command(name = "node", about = "Runs a node")]
//! struct Cli {
//!     /// Path to the config file
//!     #[arg(long, default_value = "config.toml")]
//!     config: String,
//! }
//!
//! let markdown = clap_markdown::generate_markdown::<Cli>();
//! assert!(markdown.contains("--config <CONFIG>"));
//! assert!(markdown.contains("<a id=\"node-config\"></a>"));
//! ```

use std::ffi::OsStr;

use clap::{Arg, ArgAction, Command, CommandFactory};

/// Generate Markdown for any type that can build a [`clap::Command`].
///
/// This works with structs deriving `clap::Parser`, `clap::Args`, or any type
/// manually implementing [`CommandFactory`].
pub fn generate_markdown<T>() -> String
where
    T: CommandFactory,
{
    generate_markdown_for_command(T::command())
}

/// Generate Markdown for an existing [`clap::Command`].
pub fn generate_markdown_for_command(command: Command) -> String {
    MarkdownRenderer::default().render(command)
}

/// Rendering options for Markdown generation.
#[derive(Debug, Clone)]
pub struct MarkdownOptions {
    /// Include hidden clap arguments and subcommands.
    pub include_hidden: bool,
    /// Include subcommands recursively.
    pub include_subcommands: bool,
    /// Include a small table of contents with links to each parameter.
    pub include_toc: bool,
}

impl Default for MarkdownOptions {
    fn default() -> Self {
        Self {
            include_hidden: false,
            include_subcommands: true,
            include_toc: true,
        }
    }
}

/// Generate Markdown for an existing [`clap::Command`] with explicit options.
pub fn generate_markdown_for_command_with_options(
    command: Command,
    options: MarkdownOptions,
) -> String {
    MarkdownRenderer { options }.render(command)
}

#[derive(Debug, Default)]
struct MarkdownRenderer {
    options: MarkdownOptions,
}

#[derive(Debug, Clone)]
struct ParameterDoc {
    anchor: String,
    display: String,
    description: Option<String>,
    required: bool,
    multiple: bool,
    takes_value: bool,
    value_names: Vec<String>,
    default_values: Vec<String>,
    env: Option<String>,
    possible_values: Vec<String>,
}

impl MarkdownRenderer {
    fn render(&self, mut command: Command) -> String {
        command.build();

        let mut output = String::new();
        let command_path = vec![command.get_name().to_owned()];

        self.render_command(&command, &command_path, 1, &mut output);
        trim_trailing_blank_lines(&mut output);
        output.push('\n');
        output
    }

    fn render_command(
        &self,
        command: &Command,
        command_path: &[String],
        heading_level: usize,
        output: &mut String,
    ) {
        push_heading(
            output,
            heading_level,
            &format!("`{}`", command_path.join(" ")),
        );

        if let Some(description) = command_description(command) {
            output.push_str(&description);
            output.push_str("\n\n");
        }

        let parameters = self.collect_parameters(command, command_path);
        if self.options.include_toc && !parameters.is_empty() {
            push_heading(output, heading_level + 1, "Parameters");
            for parameter in &parameters {
                output.push_str(&format!(
                    "- [`{}`](#{})",
                    escape_markdown_text(&parameter.display),
                    parameter.anchor
                ));

                if let Some(description) = &parameter.description
                    && let Some(summary) = summary_line(description)
                {
                    output.push_str(&format!(": {}", summary));
                }

                output.push('\n');
            }
            output.push('\n');
        }

        for parameter in &parameters {
            self.render_parameter(parameter, heading_level + 2, output);
        }

        let subcommands = self.visible_subcommands(command);
        if self.options.include_subcommands && !subcommands.is_empty() {
            push_heading(output, heading_level + 1, "Subcommands");
            for subcommand in &subcommands {
                let mut subcommand_path = command_path.to_vec();
                subcommand_path.push(subcommand.get_name().to_owned());

                let anchor = slugify(&subcommand_path.join("-"));
                output.push_str(&format!("<a id=\"{}\"></a>\n", anchor));
                output.push_str(&format!("- [`{}`](#{})", subcommand.get_name(), anchor));
                if let Some(description) = command_description(subcommand)
                    && let Some(summary) = summary_line(&description)
                {
                    output.push_str(&format!(": {}", summary));
                }
                output.push('\n');
            }
            output.push('\n');

            for subcommand in subcommands {
                let mut subcommand_path = command_path.to_vec();
                subcommand_path.push(subcommand.get_name().to_owned());
                self.render_command(subcommand, &subcommand_path, heading_level + 1, output);
            }
        }
    }

    fn render_parameter(
        &self,
        parameter: &ParameterDoc,
        heading_level: usize,
        output: &mut String,
    ) {
        output.push_str(&format!("<a id=\"{}\"></a>\n", parameter.anchor));
        push_heading(
            output,
            heading_level,
            &format!("`{}`", escape_markdown_text(&parameter.display)),
        );

        if let Some(description) = &parameter.description {
            output.push_str(description);
            output.push_str("\n\n");
        }

        output.push_str("| Field | Value |\n");
        output.push_str("| --- | --- |\n");
        output.push_str(&format!("| Anchor | `#{}` |\n", parameter.anchor));
        output.push_str(&format!("| Required | {} |\n", yes_no(parameter.required)));
        output.push_str(&format!(
            "| Value | {} |\n",
            if parameter.takes_value { "Yes" } else { "No" }
        ));

        if !parameter.value_names.is_empty() {
            output.push_str(&format!(
                "| Value name | {} |\n",
                parameter
                    .value_names
                    .iter()
                    .map(|value| format!("`{}`", escape_table_cell(value)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        if parameter.multiple {
            output.push_str("| Multiple | Yes |\n");
        }

        if !parameter.default_values.is_empty() {
            output.push_str(&format!(
                "| Default value | {} |\n",
                parameter
                    .default_values
                    .iter()
                    .map(|value| format!("`{}`", escape_table_cell(value)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        if let Some(env) = &parameter.env {
            output.push_str(&format!("| Environment | `{}` |\n", escape_table_cell(env)));
        }

        if !parameter.possible_values.is_empty() {
            output.push_str(&format!(
                "| Possible values | {} |\n",
                parameter
                    .possible_values
                    .iter()
                    .map(|value| format!("`{}`", escape_table_cell(value)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        output.push('\n');
    }

    fn collect_parameters(&self, command: &Command, command_path: &[String]) -> Vec<ParameterDoc> {
        command
            .get_arguments()
            .filter(|arg| !is_generated_action(arg))
            .filter(|arg| self.options.include_hidden || !arg.is_hide_set())
            .map(|arg| parameter_doc(arg, command_path))
            .collect()
    }

    fn visible_subcommands<'a>(&self, command: &'a Command) -> Vec<&'a Command> {
        if !self.options.include_subcommands {
            return Vec::new();
        }

        command
            .get_subcommands()
            .filter(|subcommand| !is_generated_help_subcommand(subcommand))
            .filter(|subcommand| self.options.include_hidden || !subcommand.is_hide_set())
            .collect()
    }
}

fn parameter_doc(arg: &Arg, command_path: &[String]) -> ParameterDoc {
    let display = display_arg(arg);
    let anchor_parts = command_path
        .iter()
        .map(String::as_str)
        .chain(std::iter::once(arg.get_id().as_str()))
        .collect::<Vec<_>>()
        .join("-");

    ParameterDoc {
        anchor: slugify(&anchor_parts),
        display,
        description: arg_description(arg),
        required: arg.is_required_set(),
        multiple: arg_allows_multiple(arg),
        takes_value: arg_takes_value(arg),
        value_names: arg
            .get_value_names()
            .unwrap_or_default()
            .iter()
            .map(ToString::to_string)
            .collect(),
        default_values: arg
            .get_default_values()
            .iter()
            .map(|value| os_to_string(value.as_os_str()))
            .collect(),
        env: arg.get_env().map(os_to_string),
        possible_values: possible_values(arg),
    }
}

fn display_arg(arg: &Arg) -> String {
    let mut names = Vec::new();

    if let Some(short) = arg.get_short() {
        names.push(format!("-{}", short));
    }

    if let Some(long) = arg.get_long() {
        names.push(format!("--{}", long));
    }

    if names.is_empty() {
        names.push(format!("<{}>", arg.get_id()));
    }

    let value_names = arg
        .get_value_names()
        .unwrap_or_default()
        .iter()
        .map(|name| format!("<{}>", name))
        .collect::<Vec<_>>();

    if !value_names.is_empty() && arg_takes_value(arg) {
        let values = value_names.join(" ");
        names
            .into_iter()
            .map(|name| {
                if name.starts_with('-') {
                    format!("{} {}", name, values)
                } else {
                    name
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        names.join(", ")
    }
}

fn command_description(command: &Command) -> Option<String> {
    command
        .get_long_about()
        .or_else(|| command.get_about())
        .map(ToString::to_string)
        .map(normalize_description)
        .filter(|description| !description.is_empty())
}

fn arg_description(arg: &Arg) -> Option<String> {
    arg.get_long_help()
        .or_else(|| arg.get_help())
        .map(ToString::to_string)
        .map(normalize_description)
        .filter(|description| !description.is_empty())
}

fn possible_values(arg: &Arg) -> Vec<String> {
    arg.get_value_parser()
        .possible_values()
        .into_iter()
        .flatten()
        .map(|value| value.get_name().to_owned())
        .collect()
}

fn arg_takes_value(arg: &Arg) -> bool {
    matches!(arg.get_action(), ArgAction::Set | ArgAction::Append)
}

fn arg_allows_multiple(arg: &Arg) -> bool {
    matches!(arg.get_action(), ArgAction::Append | ArgAction::Count)
}

fn is_generated_action(arg: &Arg) -> bool {
    matches!(
        arg.get_action(),
        ArgAction::Help | ArgAction::HelpShort | ArgAction::HelpLong | ArgAction::Version
    )
}

fn is_generated_help_subcommand(command: &Command) -> bool {
    command.get_name() == "help"
        && command_description(command)
            .is_some_and(|description| description.starts_with("Print this message or the help"))
}

fn os_to_string(value: &OsStr) -> String {
    value.to_string_lossy().into_owned()
}

fn push_heading(output: &mut String, level: usize, title: &str) {
    output.push_str(&"#".repeat(level.max(1)));
    output.push(' ');
    output.push_str(title);
    output.push_str("\n\n");
}

fn trim_trailing_blank_lines(output: &mut String) {
    while output.ends_with('\n') {
        output.pop();
    }
}

fn normalize_description(description: String) -> String {
    description.trim().to_owned()
}

fn summary_line(description: &str) -> Option<&str> {
    description
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
}

fn yes_no(value: bool) -> &'static str {
    if value { "Yes" } else { "No" }
}

fn escape_markdown_text(value: &str) -> String {
    value.replace('[', "\\[").replace(']', "\\]")
}

fn escape_table_cell(value: &str) -> String {
    escape_markdown_text(value).replace('|', "\\|")
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut previous_dash = false;

    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            previous_dash = false;
        } else if !previous_dash && !slug.is_empty() {
            slug.push('-');
            previous_dash = true;
        }
    }

    if slug.ends_with('-') {
        slug.pop();
    }

    slug
}

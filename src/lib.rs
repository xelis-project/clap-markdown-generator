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
//! let markdown = clap_markdown_generator::generate_markdown::<Cli>();
//! assert!(markdown.contains("--config <CONFIG>"));
//! assert!(markdown.contains("<a id=\"node-config\"></a>"));
//! ```

use std::{ffi::OsStr, fmt, sync::Arc};

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
    /// Include a small parameter summary with links to each parameter.
    pub include_toc: bool,
    /// Skip the detailed parameter sections.
    pub skip_parameter_details: bool,
    /// Include explicit HTML anchor id elements before parameters and subcommands.
    pub include_html_anchors: bool,
    /// Include a parameter usage line in detailed parameter content.
    pub include_usage: bool,
    /// Controls how command headings are rendered.
    pub command_heading: CommandHeadingStyle,
    /// Controls how the parameter summary is rendered.
    pub summary: SummaryOptions,
    /// Controls how each detailed parameter heading is rendered.
    pub parameter_heading: ParameterHeadingStyle,
    /// Controls how each detailed parameter body is rendered.
    pub parameter_content: ParameterContentStyle,
}

impl Default for MarkdownOptions {
    fn default() -> Self {
        Self {
            include_hidden: false,
            include_subcommands: true,
            include_toc: true,
            skip_parameter_details: false,
            include_html_anchors: true,
            include_usage: true,
            command_heading: CommandHeadingStyle::Display,
            summary: SummaryOptions::default(),
            parameter_heading: ParameterHeadingStyle::Display,
            parameter_content: ParameterContentStyle::Table,
        }
    }
}

/// Rendering options for the parameter summary.
#[derive(Debug, Clone)]
pub struct SummaryOptions {
    /// Include the parameter summary.
    pub enabled: bool,
    /// Controls whether values like `<CONFIG>` are shown in summary entries.
    pub value_style: SummaryValueStyle,
    /// Include the first line of each parameter description.
    pub include_description: bool,
    /// Controls how each parameter summary entry is rendered.
    pub entry: SummaryEntryStyle,
}

impl Default for SummaryOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            value_style: SummaryValueStyle::NamesAndValues,
            include_description: true,
            entry: SummaryEntryStyle::Default,
        }
    }
}

/// Information available to command heading callbacks.
#[derive(Debug, Clone)]
pub struct CommandInfo {
    /// Current command name.
    pub name: String,
    /// Full command path from the root command to the current command.
    pub path: Vec<String>,
    /// Full command path formatted for display, such as `app run`.
    pub display: String,
    /// Command description, if one is configured.
    pub description: Option<String>,
    /// Markdown heading level used for this command.
    pub heading_level: usize,
}

/// Information available to parameter formatting callbacks.
#[derive(Debug, Clone)]
pub struct ParameterInfo {
    /// Stable anchor id for this parameter.
    pub anchor: String,
    /// clap argument id.
    pub name: String,
    /// Full display form, such as `-c <CONFIG>, --config <CONFIG>`.
    pub display: String,
    /// Display form without values, such as `-c, --config`.
    pub display_names: String,
    /// Parameter description, if one is configured.
    pub description: Option<String>,
    /// Whether this parameter is required.
    pub required: bool,
    /// Whether this parameter accepts multiple occurrences or values.
    pub multiple: bool,
    /// Whether this parameter takes a value.
    pub takes_value: bool,
    /// clap value names.
    pub value_names: Vec<String>,
    /// clap default values.
    pub default_values: Vec<String>,
    /// Environment variable name, if one is configured.
    pub env: Option<String>,
    /// Possible values exposed by the clap value parser.
    pub possible_values: Vec<String>,
}

/// Controls how command headings are rendered.
#[derive(Clone)]
pub enum CommandHeadingStyle {
    /// Render the full command path inside backticks.
    Display,
    /// Skip command headings.
    None,
    /// Render the command heading with a callback.
    ///
    /// If the callback returns an empty string, the heading is skipped.
    Custom(Arc<dyn Fn(&CommandInfo) -> String + Send + Sync + 'static>),
}

impl CommandHeadingStyle {
    /// Create a custom command heading callback.
    pub fn custom(formatter: impl Fn(&CommandInfo) -> String + Send + Sync + 'static) -> Self {
        Self::Custom(Arc::new(formatter))
    }
}

impl fmt::Debug for CommandHeadingStyle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Display => formatter.write_str("Display"),
            Self::None => formatter.write_str("None"),
            Self::Custom(_) => formatter.write_str("Custom(<callback>)"),
        }
    }
}

/// Controls how parameter summary entries are rendered.
#[derive(Clone)]
pub enum SummaryEntryStyle {
    /// Render the default linked summary entry.
    Default,
    /// Render each summary entry with a callback.
    ///
    /// The callback should return a complete Markdown list item. If it returns
    /// an empty string, the parameter is skipped from the summary.
    Custom(Arc<dyn Fn(&ParameterInfo) -> String + Send + Sync + 'static>),
}

impl SummaryEntryStyle {
    /// Create a custom summary entry callback.
    pub fn custom(formatter: impl Fn(&ParameterInfo) -> String + Send + Sync + 'static) -> Self {
        Self::Custom(Arc::new(formatter))
    }
}

impl fmt::Debug for SummaryEntryStyle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Default => formatter.write_str("Default"),
            Self::Custom(_) => formatter.write_str("Custom(<callback>)"),
        }
    }
}

/// Controls how parameter names are rendered in the summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SummaryValueStyle {
    /// Render only parameter names, such as `-c, --config`.
    NamesOnly,
    /// Render parameter names and their values, such as `-c <CONFIG>, --config <CONFIG>`.
    NamesAndValues,
}

/// Controls how detailed parameter headings are rendered.
#[derive(Clone)]
pub enum ParameterHeadingStyle {
    /// Render the full clap display form, such as ``### `-c <CONFIG>, --config <CONFIG>```.
    Display,
    /// Render the clap argument id, such as `### config`.
    Name,
    /// Render the parameter heading with a callback.
    ///
    /// If the callback returns an empty string, the heading is skipped.
    Custom(Arc<dyn Fn(&ParameterInfo) -> String + Send + Sync + 'static>),
}

impl ParameterHeadingStyle {
    /// Create a custom parameter heading callback.
    pub fn custom(formatter: impl Fn(&ParameterInfo) -> String + Send + Sync + 'static) -> Self {
        Self::Custom(Arc::new(formatter))
    }
}

impl fmt::Debug for ParameterHeadingStyle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Display => formatter.write_str("Display"),
            Self::Name => formatter.write_str("Name"),
            Self::Custom(_) => formatter.write_str("Custom(<callback>)"),
        }
    }
}

/// Controls how detailed parameter content is rendered.
#[derive(Clone)]
pub enum ParameterContentStyle {
    /// Render the metadata table.
    Table,
    /// Render metadata as compact prose.
    Text,
    /// Render the parameter content with a callback.
    ///
    /// If the callback returns an empty string, no content is rendered.
    Custom(Arc<dyn Fn(&ParameterInfo) -> String + Send + Sync + 'static>),
}

impl ParameterContentStyle {
    /// Create a custom parameter content callback.
    pub fn custom(formatter: impl Fn(&ParameterInfo) -> String + Send + Sync + 'static) -> Self {
        Self::Custom(Arc::new(formatter))
    }
}

impl fmt::Debug for ParameterContentStyle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Table => formatter.write_str("Table"),
            Self::Text => formatter.write_str("Text"),
            Self::Custom(_) => formatter.write_str("Custom(<callback>)"),
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
        let command_info = command_info(command, command_path, heading_level);
        let command_heading = self.command_heading(&command_info);
        if !command_heading.is_empty() {
            push_heading(output, heading_level, &command_heading);
        }

        if let Some(description) = &command_info.description {
            output.push_str(description);
            output.push_str("\n\n");
        }

        let parameters = self.collect_parameters(command, command_path);
        if self.options.include_toc && self.options.summary.enabled && !parameters.is_empty() {
            push_heading(output, heading_level + 1, "Parameters");
            for parameter in &parameters {
                self.render_summary_entry(parameter, output);
            }
            output.push('\n');
        }

        if !self.options.skip_parameter_details {
            for parameter in &parameters {
                self.render_parameter(parameter, heading_level + 2, output);
            }
        }

        let subcommands = self.visible_subcommands(command);
        if self.options.include_subcommands && !subcommands.is_empty() {
            push_heading(output, heading_level + 1, "Subcommands");
            for subcommand in &subcommands {
                let mut subcommand_path = command_path.to_vec();
                subcommand_path.push(subcommand.get_name().to_owned());

                let anchor = slugify(&subcommand_path.join("-"));
                if self.options.include_html_anchors {
                    output.push_str(&format!("<a id=\"{}\"></a>\n", anchor));
                }
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
        parameter: &ParameterInfo,
        heading_level: usize,
        output: &mut String,
    ) {
        if self.options.include_html_anchors {
            output.push_str(&format!("<a id=\"{}\"></a>\n", parameter.anchor));
        }
        let heading = self.parameter_heading(parameter);
        if !heading.is_empty() {
            push_heading(output, heading_level, &heading);
        }

        if let Some(description) = &parameter.description {
            output.push_str(description);
            output.push_str("\n\n");
        }

        match &self.options.parameter_content {
            ParameterContentStyle::Table => self.render_parameter_table(parameter, output),
            ParameterContentStyle::Text => self.render_parameter_text(parameter, output),
            ParameterContentStyle::Custom(formatter) => {
                push_custom_block(output, &formatter(parameter));
            }
        }
    }

    fn render_summary_entry(&self, parameter: &ParameterInfo, output: &mut String) {
        match &self.options.summary.entry {
            SummaryEntryStyle::Default => self.render_default_summary_entry(parameter, output),
            SummaryEntryStyle::Custom(formatter) => {
                let entry = formatter(parameter);
                if !entry.is_empty() {
                    output.push_str(&entry);
                    if !entry.ends_with('\n') {
                        output.push('\n');
                    }
                }
            }
        }
    }

    fn render_default_summary_entry(&self, parameter: &ParameterInfo, output: &mut String) {
        let summary_display = match self.options.summary.value_style {
            SummaryValueStyle::NamesOnly => &parameter.display_names,
            SummaryValueStyle::NamesAndValues => &parameter.display,
        };
        output.push_str(&format!(
            "- [`{}`](#{})",
            escape_markdown_text(summary_display),
            parameter.anchor
        ));

        if self.options.summary.include_description
            && let Some(description) = &parameter.description
            && let Some(summary) = summary_line(description)
        {
            output.push_str(&format!(": {}", summary));
        }

        output.push('\n');
    }

    fn render_parameter_table(&self, parameter: &ParameterInfo, output: &mut String) {
        output.push_str("| Field | Value |\n");
        output.push_str("| --- | --- |\n");
        if self.options.include_usage {
            output.push_str(&format!(
                "| Usage | `{}` |\n",
                escape_table_cell(&parameter.display)
            ));
        }
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

    fn render_parameter_text(&self, parameter: &ParameterInfo, output: &mut String) {
        let mut parts = vec![
            format!("Required: {}.", yes_no(parameter.required)),
            format!(
                "Value: {}.",
                if parameter.takes_value { "Yes" } else { "No" }
            ),
        ];

        if self.options.include_usage {
            parts.push(format!(
                "Usage: `{}`.",
                escape_markdown_text(&parameter.display)
            ));
        }

        if !parameter.value_names.is_empty() {
            parts.push(format!(
                "Value name: {}.",
                parameter
                    .value_names
                    .iter()
                    .map(|value| format!("`{}`", escape_markdown_text(value)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        if parameter.multiple {
            parts.push("Multiple: Yes.".to_owned());
        }

        if !parameter.default_values.is_empty() {
            parts.push(format!(
                "Default value: {}.",
                parameter
                    .default_values
                    .iter()
                    .map(|value| format!("`{}`", escape_markdown_text(value)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        if let Some(env) = &parameter.env {
            parts.push(format!("Environment: `{}`.", escape_markdown_text(env)));
        }

        if !parameter.possible_values.is_empty() {
            parts.push(format!(
                "Possible values: {}.",
                parameter
                    .possible_values
                    .iter()
                    .map(|value| format!("`{}`", escape_markdown_text(value)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        output.push_str(&parts.join(" "));
        output.push_str("\n\n");
    }

    fn collect_parameters(&self, command: &Command, command_path: &[String]) -> Vec<ParameterInfo> {
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

    fn command_heading(&self, command: &CommandInfo) -> String {
        match &self.options.command_heading {
            CommandHeadingStyle::Display => format!("`{}`", escape_markdown_text(&command.display)),
            CommandHeadingStyle::None => String::new(),
            CommandHeadingStyle::Custom(formatter) => formatter(command),
        }
    }

    fn parameter_heading(&self, parameter: &ParameterInfo) -> String {
        match &self.options.parameter_heading {
            ParameterHeadingStyle::Display => {
                format!("`{}`", escape_markdown_text(&parameter.display))
            }
            ParameterHeadingStyle::Name => parameter.name.clone(),
            ParameterHeadingStyle::Custom(formatter) => formatter(parameter),
        }
    }
}

fn command_info(command: &Command, command_path: &[String], heading_level: usize) -> CommandInfo {
    CommandInfo {
        name: command.get_name().to_owned(),
        path: command_path.to_vec(),
        display: command_path.join(" "),
        description: command_description(command),
        heading_level,
    }
}

fn parameter_doc(arg: &Arg, command_path: &[String]) -> ParameterInfo {
    let display = display_arg(arg);
    let display_names = display_arg_names(arg);
    let anchor_parts = command_path
        .iter()
        .map(String::as_str)
        .chain(std::iter::once(arg.get_id().as_str()))
        .collect::<Vec<_>>()
        .join("-");

    ParameterInfo {
        anchor: slugify(&anchor_parts),
        name: arg.get_id().to_string(),
        display,
        display_names,
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
    display_arg_with_values(arg, true)
}

fn display_arg_names(arg: &Arg) -> String {
    display_arg_with_values(arg, false)
}

fn display_arg_with_values(arg: &Arg, include_values: bool) -> String {
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

    if !include_values {
        return names.join(", ");
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

fn push_custom_block(output: &mut String, block: &str) {
    if block.is_empty() {
        return;
    }

    output.push_str(block);

    if !block.ends_with('\n') {
        output.push('\n');
    }

    if !output.ends_with("\n\n") {
        output.push('\n');
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

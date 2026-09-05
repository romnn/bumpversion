//! CLI logging setup for bumpversion and related tools.
//!
//! Configures `tracing` subscriber with compact formatting and color choice.
use termcolor::ColorChoice;
use tracing_subscriber::layer::SubscriberExt;

/// Errors while setting up logging.
///
/// Both point at a defect in the CLI rather than at its input: the directive is built here, and
/// the subscriber is installed once.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// The default log filter directive did not parse.
    #[error("invalid log filter {directive:?}")]
    Filter {
        /// The directive that was built.
        directive: String,
        /// Underlying parse error.
        #[source]
        source: tracing_subscriber::filter::ParseError,
    },
    /// A global tracing subscriber is already installed.
    #[error("could not install the tracing subscriber")]
    Subscriber(#[from] tracing::subscriber::SetGlobalDefaultError),
}

/// Setup logging
///
/// # Errors
/// - If the logging directive cannot be parsed.
/// - If the global tracing subscriber cannot be installed.
pub fn setup(
    log_level: Option<tracing::metadata::Level>,
    color_choice: ColorChoice,
) -> Result<bool, Error> {
    let default_log_level = log_level.unwrap_or(tracing::metadata::Level::WARN);
    let default_log_directive = format!(
        "none,bumpversion={}",
        default_log_level.to_string().to_ascii_lowercase()
    );
    let default_env_filter = tracing_subscriber::filter::EnvFilter::builder()
        .with_regex(true)
        .with_default_directive(default_log_level.into())
        .parse(&default_log_directive)
        .map_err(|source| Error::Filter {
            directive: default_log_directive,
            source,
        })?;

    // `RUST_LOG` holds a filter directive, so it is parsed directly. (`with_env_var`
    // takes the *name* of a variable to read, so passing the value made every
    // `RUST_LOG` setting fail to resolve and fall back to the default.)
    let env_filter_directive = std::env::var("RUST_LOG")
        .ok()
        .filter(|directive| !directive.trim().is_empty());
    let env_filter = match env_filter_directive {
        Some(directive) => {
            match tracing_subscriber::filter::EnvFilter::builder()
                .with_regex(true)
                .with_default_directive(default_log_level.into())
                .parse(&directive)
            {
                Ok(env_filter) => env_filter,
                Err(err) => {
                    eprintln!("invalid log filter {directive:?}: {err}");
                    eprintln!("falling back to default logging");
                    default_env_filter
                }
            }
        }
        None => default_env_filter,
    };

    // autodetect logging format
    let use_color = match color_choice {
        ColorChoice::Always | ColorChoice::AlwaysAnsi => true,
        ColorChoice::Never => false,
        ColorChoice::Auto => {
            use std::io::IsTerminal;
            std::io::stdout().is_terminal()
        }
    };

    let fmt_layer_pretty_compact = tracing_subscriber::fmt::Layer::new()
        .compact()
        .without_time()
        .with_ansi(use_color)
        .with_writer(std::io::stdout);

    let subscriber = tracing_subscriber::registry()
        .with(fmt_layer_pretty_compact)
        .with(env_filter);
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(use_color)
}

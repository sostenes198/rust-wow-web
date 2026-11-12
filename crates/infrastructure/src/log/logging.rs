use std::str::FromStr;

use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use crate::LoggingError;

const VALID_LEVELS: &str = "off, error, warn, info, debug or trace";

pub struct Logging {}

impl Logging {
    pub fn initialize(level: &str) -> Result<(), LoggingError> {
        let filter = Self::build_filter(level)?;

        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .try_init()
            .map_err(|error| LoggingError::InitFailed(error.to_string()))
    }

    fn build_filter(level: &str) -> Result<EnvFilter, LoggingError> {
        Self::validate(level)?;

        EnvFilter::try_new(level)
            .map_err(|error| LoggingError::invalid_level(level, error.to_string()))
    }

    fn validate(spec: &str) -> Result<(), LoggingError> {
        if spec.trim().is_empty() {
            return Err(LoggingError::invalid_level(spec, "must not be empty"));
        }

        for directive in spec.split(',').map(str::trim) {
            if directive.is_empty() {
                return Err(LoggingError::invalid_level(
                    spec,
                    "contains an empty directive",
                ));
            }

            if !directive.contains('=') && LevelFilter::from_str(directive).is_err() {
                return Err(LoggingError::invalid_level(
                    spec,
                    format!("'{directive}' is not a valid level (expected {VALID_LEVELS})"),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_tests::LoggingCapture;

    fn filter(level: &str) -> EnvFilter {
        Logging::build_filter(level).unwrap()
    }

    fn levels_present(level: &str) -> Vec<&'static str> {
        LoggingCapture::levels_present(&LoggingCapture::emit_all_levels(filter(level)))
    }

    fn assert_invalid(level: &str) {
        match Logging::build_filter(level) {
            Err(LoggingError::InvalidLevel { value, .. }) => assert_eq!(value, level),
            other => panic!("expected InvalidLevel for {level:?}, got {other:?}"),
        }
    }

    #[test]
    fn build_filter_keeps_a_valid_level() {
        assert_eq!(filter("debug").to_string(), "debug");
    }

    #[test]
    fn build_filter_accepts_levels_case_insensitively() {
        assert_eq!(filter("INFO").to_string(), "info");
        assert_eq!(filter("Debug").to_string(), "debug");
    }

    #[test]
    fn build_filter_keeps_per_target_directives() {
        assert_eq!(
            filter("warn,infrastructure=trace").to_string(),
            "infrastructure=trace,warn"
        );
    }

    #[test]
    fn build_filter_rejects_a_level_with_a_typo() {
        assert_invalid("inf");
        assert_invalid("verbose");
    }

    #[test]
    fn build_filter_rejects_empty_or_blank_level() {
        assert_invalid("");
        assert_invalid("   ");
    }

    #[test]
    fn build_filter_rejects_a_malformed_target_directive() {
        assert_invalid("info,infrastructure=bar");
    }

    #[test]
    fn build_filter_rejects_a_list_with_one_bad_directive() {
        assert_invalid("warn,inf");
    }

    #[test]
    fn build_filter_rejects_empty_directives_in_the_list() {
        // LevelFilter::from_str("") is Ok(ERROR), so this needs its own check.
        assert_invalid("info,");
        assert_invalid(",info");
        assert_invalid("warn,,info");
    }

    #[test]
    fn rejection_message_names_the_offending_directive() {
        let message = Logging::build_filter("warn,inf").unwrap_err().to_string();

        assert!(message.contains("invalid log level 'warn,inf'"));
        assert!(message.contains("'inf' is not a valid level"));
    }

    #[test]
    fn trace_level_emits_every_event() {
        let result = levels_present("trace");
        assert_eq!(result, ["trace", "debug", "info", "warn", "error"]);
    }

    #[test]
    fn info_level_hides_debug_and_trace() {
        let result = levels_present("info");
        assert_eq!(result, ["info", "warn", "error"]);
    }

    #[test]
    fn error_level_emits_only_errors() {
        let result = levels_present("error");
        assert_eq!(result, ["error"]);
    }

    #[test]
    fn output_carries_level_and_message() {
        let output = LoggingCapture::emit_all_levels(filter("warn"));

        assert!(output.contains("WARN"));
        assert!(output.contains("msg-warn"));
    }

    #[test]
    fn initialize_rejects_an_invalid_level_without_installing_anything() {
        assert!(matches!(
            Logging::initialize("inf"),
            Err(LoggingError::InvalidLevel { .. })
        ))
    }

    #[test]
    fn initialize_installs_the_global_subscriber_only_once() {
        assert!(Logging::initialize("info").is_ok());

        assert!(matches! {
            Logging::initialize("info"),
            Err(LoggingError::InitFailed(_))
        })
    }
}

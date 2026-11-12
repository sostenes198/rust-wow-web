use thiserror::Error;

#[derive(Debug, Error)]
pub enum LoggingError {
    #[error("invalid log level '{value}': {motive}")]
    InvalidLevel { value: String, motive: String },

    #[error("failed to initialize logging: {0}")]
    InitFailed(String),
}

impl LoggingError {
    pub(crate) fn invalid_level(value: &str, motive: impl Into<String>) -> Self {
        Self::InvalidLevel {
            value: value.to_string(),
            motive: motive.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_level_message_has_value_and_motive() {
        let error = LoggingError::invalid_level("inf", "is not a level");

        assert_eq!(error.to_string(), "invalid log level 'inf': is not a level");
    }

    #[test]
    fn init_failed_message_has_the_cause() {
        let error = LoggingError::InitFailed("already set".to_string());

        assert_eq!(
            error.to_string(),
            "failed to initialize logging: already set"
        );
    }

    #[test]
    fn should_implement_std_error() {
        fn required_derive_error<E: std::error::Error>(_: &E) {}

        required_derive_error(&LoggingError::InitFailed(String::new()));
    }
}

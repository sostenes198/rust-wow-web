use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("enviroment variable required is missing: {0}")]
    Missing(&'static str),

    #[error("invalid value to {name}: '{value}' ({motive})")]
    Invalid {
        name: &'static str,
        value: String,
        motive: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    const AUTH_PORT: &str = "AUTH_PORT";

    #[test]
    fn missing_envirioment_required() {
        let error = ConfigError::Missing(AUTH_PORT);

        assert_eq!(
            error.to_string(),
            format!("enviroment variable required is missing: {AUTH_PORT}")
        )
    }

    #[test]
    fn invalid_enviroment() {
        let error = ConfigError::Invalid {
            name: AUTH_PORT,
            value: "abc".to_string(),
            motive: "is not a number".to_string(),
        };

        assert_eq!(
            error.to_string(),
            format!("invalid value to {AUTH_PORT}: 'abc' (is not a number)")
        )
    }

    #[test]
    fn should_implement_std_error() {
        fn required_derive_error<E: std::error::Error>(_: &E) {}

        required_derive_error(&ConfigError::Missing("Value"));
    }
}

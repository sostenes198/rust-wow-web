use crate::config::config_error::ConfigError;
use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub auth_host: String,
    pub auth_port: u16,
    pub world_host: String,
    pub world_port: u16,
    pub log_level: String,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let _ = dotenvy::dotenv();

        Ok(Self {
            auth_host: Self::required("AUTH_HOST")?,
            auth_port: Self::parse_to_u16(
                "AUTH_PORT",
                &Self::with_default_value("AUTH_PORT", "3724"),
            )?,
            world_host: Self::required("WORLD_HOST")?,
            world_port: Self::parse_to_u16(
                "WORLD_PORT",
                &Self::with_default_value("WORLD_PORT", "8085"),
            )?,
            log_level: Self::with_default_value("LOG_LEVEL", "info"),
        })
    }

    pub fn auth_addr(&self) -> String {
        format!("{}:{}", self.auth_host, self.auth_port)
    }

    pub fn world_addr(&self) -> String {
        format!("{}:{}", self.world_host, self.world_port)
    }

    fn required(name: &'static str) -> Result<String, ConfigError> {
        match env::var(name) {
            Ok(value) => Ok(value),
            Err(env::VarError::NotPresent) => Err(ConfigError::Missing(name)),
            Err(env::VarError::NotUnicode(_)) => Err(ConfigError::Invalid {
                name,
                value: String::from("<is not UTF-8>"),
                motive: String::from("value must be UTF-8 text"),
            }),
        }
    }

    fn with_default_value(name: &'static str, defaulf: &str) -> String {
        env::var(name).unwrap_or_else(|_| defaulf.to_string())
    }

    fn parse_to_u16(name: &'static str, value: &str) -> Result<u16, ConfigError> {
        value.parse::<u16>().map_err(|e| ConfigError::Invalid {
            name,
            value: value.to_string(),
            motive: e.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use std::ffi::OsString;

    use common_tests::EnvGuard;

    const VARS: [&str; 5] = [
        "AUTH_HOST",
        "AUTH_PORT",
        "WORLD_HOST",
        "WORLD_PORT",
        "LOG_LEVEL",
    ];

    fn sample() -> AppConfig {
        AppConfig {
            auth_host: "10.0.0.1".to_string(),
            auth_port: 3724,
            world_host: "10.0.0.2".to_string(),
            world_port: 8085,
            log_level: "debug".to_string(),
        }
    }

    // ---------- auth_addr / world_addr / derives ----------
    #[test]
    fn auth_addr_joins_host_and_port() {
        assert_eq!(sample().auth_addr(), "10.0.0.1:3724")
    }

    #[test]
    fn world_addr_joins_host_and_port() {
        assert_eq!(sample().world_addr(), "10.0.0.2:8085");
    }

    #[test]
    fn config_can_be_cloned_and_debug_printed() {
        let config = sample();
        let copy = config.clone();

        assert_eq!(copy.auth_host, config.auth_host);
        assert!(format!("{copy:?}").contains("auth_port: 3724"));
    }

    // ---------- required ----------
    // These use variable names nobody else touches, so no lock is needed.

    #[test]
    fn required_return_the_value_when_present() {
        let env = EnvGuard::new(&["CFG_TEST_REQUIRED_OK"]);
        env.set("CFG_TEST_REQUIRED_OK", "abc");

        assert_eq!(AppConfig::required("CFG_TEST_REQUIRED_OK").unwrap(), "abc");
    }

    #[test]
    fn required_fails_with_missing_when_absent() {
        let _env = EnvGuard::new(&["CFG_TEST_REQUIRED_ABSENT"]);

        let error = AppConfig::required("CFG_TEST_REQUIRED_ABSENT").unwrap_err();

        assert_eq!(
            error.to_string(),
            "enviroment variable required is missing: CFG_TEST_REQUIRED_ABSENT"
        );
    }

    #[cfg(unix)]
    #[test]
    fn required_fails_with_invalid_when_not_utf8() {
        use std::os::unix::ffi::OsStringExt;

        let not_utf8 = OsString::from_vec(vec![0xff, 0xfe]);
        let env = EnvGuard::new(&["CFG_TEST_REQUIRED_BINARY"]);
        env.set("CFG_TEST_REQUIRED_BINARY", not_utf8);

        let error = AppConfig::required("CFG_TEST_REQUIRED_BINARY").unwrap_err();

        assert_eq!(
            error.to_string(),
            "invalid value to CFG_TEST_REQUIRED_BINARY: '<is not UTF-8>' \
             (value must be UTF-8 text)"
        );
    }

    // ---------- with_default_value ----------

    #[test]
    fn with_default_value_prefers_the_enviroment() {
        let env = EnvGuard::new(&["CFG_TEST_DEFAULT_SET"]);
        env.set("CFG_TEST_DEFAULT_SET", "from-env");

        assert_eq!(
            AppConfig::with_default_value("CFG_TEST_DEFAULT_SET", "fallback"),
            "from-env"
        )
    }

    #[test]
    fn with_default_value_falls_back_when_absent() {
        let _env = EnvGuard::new(&["CFG_TEST_DEFAULT_ABSENT"]);
        assert_eq!(
            AppConfig::with_default_value("CFG_TEST_DEFAULT_ABSENT", "fallback"),
            "fallback"
        );
    }

    // ---------- parse_to_u16 ----------

    #[test]
    fn parse_to_u16_accepts_valid_ports() {
        assert_eq!(AppConfig::parse_to_u16("AUTH_PORT", "3724").unwrap(), 3724);
        assert_eq!(AppConfig::parse_to_u16("AUTH_PORT", "0").unwrap(), 0);
        assert_eq!(
            AppConfig::parse_to_u16("AUTH_PORT", "65535").unwrap(),
            65535
        );
    }

    #[test]
    fn parse_to_u16_rejects_text() {
        let error = AppConfig::parse_to_u16("AUTH_PORT", "abc").unwrap_err();

        assert_eq!(
            error.to_string(),
            "invalid value to AUTH_PORT: 'abc' (invalid digit found in string)"
        )
    }

    #[test]
    fn parse_to_u16_rejects_empty_and_negative() {
        assert!(AppConfig::parse_to_u16("AUTH_PORT", "").is_err());
        assert!(AppConfig::parse_to_u16("AUTH_PORT", "-1").is_err());
    }

    // ---------- load ----------

    #[test]
    fn load_reads_every_variable() {
        let env = EnvGuard::new(&VARS);
        env.set("AUTH_HOST", "1.1.1.1");
        env.set("AUTH_PORT", "1111");
        env.set("WORLD_HOST", "2.2.2.2");
        env.set("WORLD_PORT", "2222");
        env.set("LOG_LEVEL", "trace");

        let config = AppConfig::load().unwrap();

        assert_eq!(config.auth_host, "1.1.1.1");
        assert_eq!(config.auth_port, 1111);
        assert_eq!(config.world_host, "2.2.2.2");
        assert_eq!(config.world_port, 2222);
        assert_eq!(config.log_level, "trace");
    }

    #[test]
    fn load_applies_defaults_for_optional_variables() {
        let env = EnvGuard::new(&VARS);
        env.set("AUTH_HOST", "1.1.1.1");
        env.set("WORLD_HOST", "1.1.1.1");

        let config = AppConfig::load().unwrap();

        assert_eq!(config.auth_port, 3724);
        assert_eq!(config.world_host, "1.1.1.1");
        assert_eq!(config.world_port, 8085);
        assert_eq!(config.log_level, "info");
    }

    #[test]
    fn load_fails_when_auth_host_is_missing() {
        let _env = EnvGuard::new(&VARS);

        let error = AppConfig::load().unwrap_err();

        assert_eq!(
            error.to_string(),
            "enviroment variable required is missing: AUTH_HOST"
        );
    }

    #[test]
    fn load_fails_when_auth_port_is_invalid() {
        let env = EnvGuard::new(&VARS);
        env.set("AUTH_HOST", "1.1.1.1");
        env.set("AUTH_PORT", "abc");

        let error = AppConfig::load().unwrap_err();

        assert!(error.to_string().starts_with("invalid value to AUTH_PORT"));
    }

    #[test]
    fn load_fails_when_word_host_is_missing() {
        let env = EnvGuard::new(&VARS);
        env.set("AUTH_HOST", "1.1.1.1");

        let error = AppConfig::load().unwrap_err();

        assert_eq!(
            error.to_string(),
            "enviroment variable required is missing: WORLD_HOST"
        );
    }

    #[test]
    fn load_fails_when_world_port_is_invalid() {
        let env = EnvGuard::new(&VARS);
        env.set("AUTH_HOST", "1.1.1.1");
        env.set("WORLD_HOST", "127.0.0.1");
        env.set("WORLD_PORT", "99999");

        let error = AppConfig::load().unwrap_err();

        assert!(error.to_string().starts_with("invalid value to WORLD_PORT"));
    }
}

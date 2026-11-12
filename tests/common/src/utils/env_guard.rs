use std::{
    env,
    ffi::OsStr,
    path::PathBuf,
    sync::{Mutex, MutexGuard, PoisonError},
};

static ENV_LOCK: Mutex<()> = Mutex::new(());

pub struct EnvGuard {
    _lock: MutexGuard<'static, ()>,
    vars: Vec<&'static str>,
    cwd: PathBuf,
}

impl EnvGuard {
    pub fn new(vars: &[&'static str]) -> Self {
        let lock = ENV_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let cwd = env::current_dir().unwrap();

        let guard = Self {
            _lock: lock,
            vars: vars.to_vec(),
            cwd,
        };

        guard.clear();
        env::set_current_dir(env::temp_dir()).unwrap();

        guard
    }

    pub fn set(&self, name: &str, value: impl AsRef<OsStr>) {
        unsafe {
            env::set_var(name, value);
        }
    }

    fn clear(&self) {
        for name in &self.vars {
            unsafe {
                env::remove_var(name);
            }
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.cwd);
        self.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clears_the_given_variables_and_cleans_up_on_drop() {
        // SAFETY: this test is the only one in this crate using this name.
        unsafe { env::set_var("TEST_SUPPORT_VAR", "from-shell") };

        {
            let guard = EnvGuard::new(&["TEST_SUPPORT_VAR"]);
            assert!(env::var("TEST_SUPPORT_VAR").is_err());

            guard.set("TEST_SUPPORT_VAR", "inside");
            assert_eq!(env::var("TEST_SUPPORT_VAR").unwrap(), "inside");
        }

        assert!(env::var("TEST_SUPPORT_VAR").is_err());
    }
}

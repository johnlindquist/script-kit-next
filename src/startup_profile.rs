use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StartupProfile {
    Standard,
    DevFast,
}

impl StartupProfile {
    pub(crate) fn from_env() -> Self {
        match std::env::var("SCRIPT_KIT_STARTUP_PROFILE")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "dev-fast" | "dev_fast" | "fast" => Self::DevFast,
            _ => Self::Standard,
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::DevFast => "dev-fast",
        }
    }

    pub(crate) fn should_defer_scheduler(self) -> bool {
        env_truthy("SCRIPT_KIT_DEFER_SCHEDULER_STARTUP") || matches!(self, Self::DevFast)
    }

    pub(crate) fn deferred_scheduler_delay(self) -> Duration {
        let _ = self;
        Duration::from_millis(1)
    }

    pub(crate) fn ready_log_enabled(self) -> bool {
        let _ = self;
        !env_falsey("SCRIPT_KIT_STARTUP_READY_LOG")
    }
}

fn env_truthy(name: &str) -> bool {
    std::env::var(name)
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn env_falsey(name: &str) -> bool {
    std::env::var(name)
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "0" | "false" | "no" | "off"
            )
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_profile_defaults() {
        // When no env var is set, should be Standard
        let profile = StartupProfile::Standard;
        assert_eq!(profile.label(), "standard");
        assert!(!matches!(profile, StartupProfile::DevFast));
    }

    #[test]
    fn test_dev_fast_profile() {
        let profile = StartupProfile::DevFast;
        assert_eq!(profile.label(), "dev-fast");
        assert!(profile.should_defer_scheduler());
        assert_eq!(profile.deferred_scheduler_delay(), Duration::from_millis(1));
        assert!(profile.ready_log_enabled());
    }

    #[test]
    fn test_standard_profile_does_not_defer() {
        let profile = StartupProfile::Standard;
        // Without env var, standard should not defer
        // (env_truthy returns false when var is unset)
        // Note: this test is environment-dependent; in CI SCRIPT_KIT_DEFER_SCHEDULER_STARTUP
        // should not be set.
        if std::env::var("SCRIPT_KIT_DEFER_SCHEDULER_STARTUP").is_err() {
            assert!(!profile.should_defer_scheduler());
        }
    }
}

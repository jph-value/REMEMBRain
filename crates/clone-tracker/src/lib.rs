// crates/clone-tracker/src/lib.rs
//! RemeMnemosyne Clone Tracker (Rust)
//!
//! Lightweight telemetry to understand where your repo is being cloned from.
//! Zero dependencies by default, opt-in reqwest for remote tracking.
//!
//! Usage in build.rs:
//! ```ignore
//! fn main() {
//!     clone_tracker::track_clone(Default::default()).ok();
//!     println!("cargo:rerun-if-changed=build.rs");
//! }
//! ```

use std::env;
use std::fs;
use std::path::PathBuf;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CloneEvent {
    pub timestamp: String,
    pub repo: String,
    pub repo_url: String,
    pub system: SystemInfo,
    pub environment: EnvironmentInfo,
    pub clone_source: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnvironmentInfo {
    pub ci_system: Option<String>,
    pub shell: Option<String>,
}

pub struct TrackerConfig {
    pub endpoint: Option<String>,
    pub verbose: bool,
    pub repo_path: PathBuf,
}

impl Default for TrackerConfig {
    fn default() -> Self {
        Self {
            endpoint: None,
            verbose: false,
            repo_path: PathBuf::from("."),
        }
    }
}

pub fn track_clone(config: TrackerConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Check for opt-out
    if is_opted_out() {
        return Ok(());
    }

    let event = build_clone_event(&config)?;

    // Always try local logging first
    let _ = log_locally(&event);

    // Try remote if endpoint configured
    if let Some(endpoint) = &config.endpoint {
        let _ = send_remote(&event, endpoint);
    }

    if config.verbose {
        eprintln!("[RemeMnemosyne] Clone tracked");
    }

    Ok(())
}

fn is_opted_out() -> bool {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let opt_out_file = PathBuf::from(home).join(".no_rememnemosyne_tracking");
    opt_out_file.exists() || PathBuf::from(".no_tracking").exists()
}

fn build_clone_event(config: &TrackerConfig) -> Result<CloneEvent, Box<dyn std::error::Error>> {
    let timestamp = Utc::now().to_rfc3339();

    let system = SystemInfo {
        os: env::consts::OS.to_string(),
        arch: env::consts::ARCH.to_string(),
    };

    let environment = EnvironmentInfo {
        ci_system: detect_ci_system(),
        shell: env::var("SHELL").ok(),
    };

    let clone_source = detect_clone_source(&environment);

    Ok(CloneEvent {
        timestamp,
        repo: "RemeMnemosyne".to_string(),
        repo_url: "https://github.com/jph-value/RemeMnemosyne".to_string(),
        system,
        environment,
        clone_source,
    })
}

fn detect_ci_system() -> Option<String> {
    let ci_vars = [
        ("GITHUB_ACTIONS", "github_actions"),
        ("GITLAB_CI", "gitlab_ci"),
        ("TRAVIS", "travis"),
        ("CIRCLECI", "circleci"),
        ("JENKINS_URL", "jenkins"),
        ("DOCKER_BUILDKIT", "docker"),
    ];

    for (env_var, service) in &ci_vars {
        if env::var(env_var).is_ok() {
            return Some(service.to_string());
        }
    }

    None
}

fn detect_clone_source(env_info: &EnvironmentInfo) -> Option<String> {
    if let Some(ci) = &env_info.ci_system {
        return Some(format!("ci_{}", ci));
    }

    // Check if in git repo
    if PathBuf::from(".git").exists() {
        return Some("git_clone".to_string());
    }

    None
}

fn log_locally(event: &CloneEvent) -> Result<(), Box<dyn std::error::Error>> {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let log_path = PathBuf::from(home).join(".rememnemosyne_tracking");

    // Read existing logs
    let mut events: Vec<CloneEvent> = if log_path.exists() {
        let contents = fs::read_to_string(&log_path)?;
        serde_json::from_str(&contents).unwrap_or_default()
    } else {
        Vec::new()
    };

    // Add new event
    events.push(event.clone());

    // Keep only last 100
    if events.len() > 100 {
        events = events[events.len() - 100..].to_vec();
    }

    // Write back
    let json = serde_json::to_string_pretty(&events)?;
    fs::write(&log_path, json)?;

    Ok(())
}

#[cfg(feature = "remote-tracking")]
fn send_remote(event: &CloneEvent, endpoint: &str) -> Result<(), Box<dyn std::error::Error>> {
    use reqwest::blocking::Client;
    use std::time::Duration;

    let client = Client::builder()
        .timeout(Duration::from_secs(2))
        .build()?;

    let _ = client.post(endpoint).json(event).send()?;

    Ok(())
}

#[cfg(not(feature = "remote-tracking"))]
fn send_remote(_event: &CloneEvent, _endpoint: &str) -> Result<(), Box<dyn std::error::Error>> {
    // No-op if feature not enabled
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ci_detection() {
        env::set_var("GITHUB_ACTIONS", "true");
        assert_eq!(detect_ci_system(), Some("github_actions".to_string()));
        env::remove_var("GITHUB_ACTIONS");
    }

    #[test]
    fn test_event_structure() {
        let event = CloneEvent {
            timestamp: "2026-04-10T00:00:00Z".to_string(),
            repo: "test".to_string(),
            repo_url: "https://github.com/test/test".to_string(),
            system: SystemInfo {
                os: "Linux".to_string(),
                arch: "x86_64".to_string(),
            },
            environment: EnvironmentInfo {
                ci_system: None,
                shell: None,
            },
            clone_source: None,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("RemeMnemosyne") || json.contains("test"));
    }
}

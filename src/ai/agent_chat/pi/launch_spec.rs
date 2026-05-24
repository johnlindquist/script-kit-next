use std::path::PathBuf;

use crate::ai::agent_chat::profiles::ResolvedAgentChatProfile;
use crate::config::AgentChatBackend;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiLaunchSpec {
    pub pi_binary: PathBuf,
    pub cwd: Option<PathBuf>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub thinking: Option<String>,
    pub system_prompt: Option<String>,
    pub append_system_prompt: Option<String>,
    pub tools: Option<Vec<String>>,
    pub disable_extensions: bool,
    pub extension_paths: Vec<String>,
    pub extension_policy: Option<String>,
    pub disable_skills: bool,
    pub skill_paths: Vec<String>,
    pub disable_prompt_templates: bool,
    pub prompt_template_paths: Vec<String>,
    pub hide_cwd_in_prompt: bool,
    pub session_dir: Option<String>,
    pub no_session: bool,
    pub session_durability: Option<String>,
}

impl PiLaunchSpec {
    pub fn from_profile(profile: &ResolvedAgentChatProfile) -> Option<Self> {
        if profile.backend != AgentChatBackend::Pi {
            return None;
        }

        Some(Self {
            pi_binary: PathBuf::from("pi"),
            cwd: profile.cwd.clone(),
            provider: profile.provider.clone(),
            model: profile.model.clone(),
            thinking: profile.thinking.clone(),
            system_prompt: profile.system_prompt.clone(),
            append_system_prompt: profile.append_system_prompt.clone(),
            tools: profile.tools.clone(),
            disable_extensions: profile.disable_extensions.unwrap_or(false),
            extension_paths: Vec::new(),
            extension_policy: profile.extension_policy.clone(),
            disable_skills: profile.disable_skills.unwrap_or(false),
            skill_paths: Vec::new(),
            disable_prompt_templates: profile.disable_prompt_templates.unwrap_or(false),
            prompt_template_paths: Vec::new(),
            hide_cwd_in_prompt: profile.hide_cwd_in_prompt.unwrap_or(false),
            session_dir: profile.session_dir.clone(),
            no_session: profile.no_session.unwrap_or(false),
            session_durability: profile.session_durability.clone(),
        })
    }

    pub fn argv(&self) -> Vec<String> {
        let mut argv = vec!["--mode".to_string(), "rpc".to_string()];

        push_arg(&mut argv, "--provider", self.provider.as_deref());
        push_arg(&mut argv, "--model", self.model.as_deref());
        push_arg(&mut argv, "--thinking", self.thinking.as_deref());
        push_arg(&mut argv, "--system-prompt", self.system_prompt.as_deref());
        push_arg(
            &mut argv,
            "--append-system-prompt",
            self.append_system_prompt.as_deref(),
        );

        match self.tools.as_ref() {
            Some(tools) if tools.is_empty() => argv.push("--no-tools".to_string()),
            Some(tools) => {
                argv.push("--tools".to_string());
                argv.push(tools.join(","));
            }
            None => {}
        }

        if self.disable_extensions {
            argv.push("--no-extensions".to_string());
        } else {
            push_repeated_arg(&mut argv, "--extension", &self.extension_paths);
        }
        push_arg(
            &mut argv,
            "--extension-policy",
            self.extension_policy.as_deref(),
        );

        if self.disable_skills {
            argv.push("--no-skills".to_string());
        } else {
            push_repeated_arg(&mut argv, "--skill", &self.skill_paths);
        }

        if self.disable_prompt_templates {
            argv.push("--no-prompt-templates".to_string());
        } else {
            push_repeated_arg(&mut argv, "--prompt-template", &self.prompt_template_paths);
        }

        if self.hide_cwd_in_prompt {
            argv.push("--hide-cwd-in-prompt".to_string());
        }

        push_arg(&mut argv, "--session-dir", self.session_dir.as_deref());
        if self.no_session {
            argv.push("--no-session".to_string());
        }
        push_arg(
            &mut argv,
            "--session-durability",
            self.session_durability.as_deref(),
        );

        argv
    }
}

fn push_arg(argv: &mut Vec<String>, name: &str, value: Option<&str>) {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return;
    };
    argv.push(name.to_string());
    argv.push(value.to_string());
}

fn push_repeated_arg(argv: &mut Vec<String>, name: &str, values: &[String]) {
    for value in values {
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        argv.push(name.to_string());
        argv.push(value.to_string());
    }
}

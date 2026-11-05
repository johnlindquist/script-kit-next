//! Bind the profile's effective tool grants, external MCP definitions, listener and
//! approval mode before reusing a session. A per-process policy id is not a cache key.
use super::{launch_spec, recipe, GooseLaunchSpec, GooseProfileLaunch};
use crate::{
    ai::agent_chat::{approval::AgentApprovalMode, warm_key},
    mcp_agent_tools::{self, AgentToolPolicy},
};
use serde_json::Value;
use uuid::Uuid;

pub(crate) struct BoundLaunch {
    spec: GooseLaunchSpec,
    policy: AgentToolPolicy,
    endpoint_generation: Option<Uuid>,
    profile_id: String,
    subagents: bool,
}
impl BoundLaunch {
    pub(crate) fn from_snapshot(
        launch: &GooseProfileLaunch,
        mode: AgentApprovalMode,
        endpoint: Option<Uuid>,
        user_mcp: &[Value],
    ) -> Self {
        let mut spec = launch.spec.clone();
        spec.launcher_author = launch.tools.launcher_author();
        if spec.launcher_author.is_some() {
            // Goose reads project and global hint files before a model call even
            // when no file tool is granted. Override only this child process.
            spec.env.insert("CONTEXT_FILE_NAMES".into(), "[]".into());
            spec.hidden_session = true;
        }
        // A dedicated launcher writer reviews each save, independent of ordinary chat settings.
        let mode = if spec.launcher_author == Some(sk_personal::api::Endpoint::Apply) {
            AgentApprovalMode::AskBeforeChanges
        } else {
            mode
        };
        spec.approval_mode = mode;
        spec.env
            .insert("GOOSE_MODE".into(), mode.goose_mode().into());
        spec.recipe = recipe::build_recipe(recipe::RecipeInput {
            profile: &launch.profile,
            provider: launch.provider.as_deref(),
            model: launch.model.as_deref(),
            tools: &launch.tools,
            endpoint: None,
            user_mcp,
        });
        let policy =
            AgentToolPolicy::from_profile(&launch.profile, &launch.tools.script_kit, &spec.cwd);
        let endpoint_generation = if policy.allowed_tools.is_empty() {
            None
        } else {
            endpoint
        };
        Self {
            spec,
            policy,
            endpoint_generation,
            profile_id: launch.profile.id.clone(),
            subagents: launch.tools.platform.contains(&"summon"),
        }
    }
    pub(crate) fn key(&self) -> String {
        warm_key::launch_warm_key(
            &self.spec,
            &self.profile_id,
            &self.policy,
            self.endpoint_generation,
        )
    }
    pub(crate) fn spec_for_spawn(&self) -> anyhow::Result<GooseLaunchSpec> {
        use crate::runtime_policy::{check, ExternalEffect};
        check(ExternalEffect::Process)?;
        check(ExternalEffect::Provider)?;
        self.for_endpoint(mcp_agent_tools::endpoint())
    }

    fn for_endpoint(
        &self,
        current: Option<mcp_agent_tools::AgentToolsEndpoint>,
    ) -> anyhow::Result<GooseLaunchSpec> {
        use anyhow::Context as _;
        let mut spec = self.spec.clone();
        let endpoint = if self.policy.allowed_tools.is_empty() {
            None
        } else {
            anyhow::ensure!(
                current.as_ref().map(|e| e.generation) == self.endpoint_generation,
                "Script Kit’s tool connection changed before the agent started"
            );
            current
        };
        anyhow::ensure!(
            spec.launcher_author.is_none() || endpoint.is_some(),
            "The launcher authoring connection is unavailable"
        );
        let root = launch_spec::goose_root_from_env(&spec.env);
        launch_spec::ensure_goose_root(&root)
            .context("Could not prepare the session’s extension policy")?;
        if self.subagents {
            launch_spec::ensure_goose_permission_defaults(&root)
                .context("Could not prepare the session’s delegation policy")?;
        }
        if !self.policy.allowed_tools.is_empty() {
            if let Some(endpoint) = endpoint {
                let extensions = spec.recipe["extensions"]
                    .as_array_mut()
                    .ok_or_else(|| anyhow::anyhow!("Invalid bound agent recipe"))?;
                let policy_id = mcp_agent_tools::register_bound_policy(
                    self.policy.clone(),
                    spec.launcher_author == Some(sk_personal::api::Endpoint::Apply),
                );
                spec.agent_policy_id = Some(policy_id.clone());
                let mut extension = recipe::script_kit_extension(&recipe::ScriptKitToolsEndpoint {
                    uri: endpoint.uri,
                    bearer_token: endpoint.token,
                    policy_id,
                });
                if spec.launcher_author.is_some() {
                    extension["available_tools"] = serde_json::json!(self.policy.allowed_tools);
                }
                extensions.insert(0, extension);
            } else {
                tracing::warn!(target:"script_kit::agent_chat",event="goose_script_kit_tools_unavailable",profile_id=%self.profile_id,"Session starts without Script Kit tools");
            }
        }
        Ok(spec)
    }
}

#[cfg(test)]
#[path = "bound_launch_tests.rs"]
pub(crate) mod tests;

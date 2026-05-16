# Bundle Map

- Feature id: `038-agent-skills-ai-context-catalog`
- Oracle slug: `agent-skills-ai-context-atlas`
- Bundle path: `/Users/johnlindquist/.oracle/bundles/agent-skills-ai-context-atlas.txt`
- Bundle size: `191879` bytes
- Bundle SHA-256: `afdab915b4d16c05edefd836aa3cb3cfe551f5d4587d70b29f2a2b7a168e76ea`
- Pack summary: 33 files, 866 matches, 230 context windows, about 50.7K exact tokens.
- Pack command:

```bash
packx AGENTS.md CLAUDE.md lat.md/ai-context.md lat.md/agent-skills.md lat.md/acp-chat.md lat.md/workspace.md lat.md/verification.md .agents/skills/mcp-context-resources/SKILL.md .agents/skills/acp-context-composer/SKILL.md .agents/skills/acp-chat-core/SKILL.md .agents/skills/sdk-script-execution/SKILL.md .agents/skills/protocol-automation/SKILL.md src/mcp_resources src/mcp_protocol src/mcp_server src/mcp_script_tools src/context_snapshot src/ai/message_parts.rs src/ai/context_mentions src/ai/window/context_picker src/ai/window/context_preview.rs src/ai/window/context_preflight.rs src/ai/window/context_recommendations.rs src/ai/acp/context.rs src/app_impl/attachment_portal.rs src/app_impl/tab_ai_mode src/agents src/plugins/skills.rs src/scripts/search/skills.rs tests/context_part_start_chat_flow.rs tests/context_snapshot.rs tests/context_part_submission_flow.rs tests/transaction_trace_resources.rs tests/mcp_resources_sdk_reference.rs tests/context_preflight.rs tests/context_part_resolution.rs tests/context_contract_end_to_end.rs tests/context_picker.rs tests/context_part_composer_state.rs tests/tab_ai_context.rs tests/context_preflight_source_audits.rs tests/sdk/test-ai-context-parts.ts --no-interactive --limit 49k -l 4 -s "kit://context" -s "AiContextPart" -s "ContextResolutionReceipt" -s "CaptureContextOptions" -s "ContextPreviewInfo" -s "open_attachment_portal" -s "FocusedTarget" -s "ResourceUri" -s "SkillFile" -s "ASK_ANYTHING" -s "profile=minimal" -s "skill" -o /Users/johnlindquist/.oracle/bundles/agent-skills-ai-context-atlas.txt
```

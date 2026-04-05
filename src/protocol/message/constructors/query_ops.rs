use super::*;

impl Message {
    // ============================================================
    // Constructor methods for file search
    // ============================================================

    /// Create a file search request
    pub fn file_search(request_id: String, query: String, only_in: Option<String>) -> Self {
        Message::FileSearch {
            request_id,
            query,
            only_in,
        }
    }

    /// Create a file search result response
    pub fn file_search_result(request_id: String, files: Vec<FileSearchResultEntry>) -> Self {
        Message::FileSearchResult { request_id, files }
    }

    // ============================================================
    // Constructor methods for screenshot capture
    // ============================================================

    /// Create a capture screenshot request
    pub fn capture_screenshot(request_id: String) -> Self {
        Message::CaptureScreenshot {
            request_id,
            hi_dpi: None,
            target: None,
        }
    }

    /// Create a capture screenshot request with hi_dpi option
    pub fn capture_screenshot_with_options(request_id: String, hi_dpi: Option<bool>) -> Self {
        Message::CaptureScreenshot {
            request_id,
            hi_dpi,
            target: None,
        }
    }

    /// Create a screenshot result response
    pub fn screenshot_result(request_id: String, data: String, width: u32, height: u32) -> Self {
        Message::ScreenshotResult {
            request_id,
            data,
            width,
            height,
            error: None,
        }
    }

    /// Create a screenshot error result response
    pub fn screenshot_error(request_id: String, error: String) -> Self {
        Message::ScreenshotResult {
            request_id,
            data: String::new(),
            width: 0,
            height: 0,
            error: Some(error),
        }
    }

    // ============================================================
    // Constructor methods for state query
    // ============================================================

    /// Create a get state request
    pub fn get_state(request_id: String) -> Self {
        Message::GetState {
            request_id,
            target: None,
        }
    }

    /// Create a state result response
    #[allow(clippy::too_many_arguments)]
    pub fn state_result(
        request_id: String,
        prompt_type: String,
        prompt_id: Option<String>,
        placeholder: Option<String>,
        input_value: String,
        choice_count: usize,
        visible_choice_count: usize,
        selected_index: i32,
        selected_value: Option<String>,
        is_focused: bool,
        window_visible: bool,
    ) -> Self {
        Message::StateResult {
            request_id,
            prompt_type,
            prompt_id,
            placeholder,
            input_value,
            choice_count,
            visible_choice_count,
            selected_index,
            selected_value,
            is_focused,
            window_visible,
        }
    }

    // ============================================================
    // Constructor methods for element query
    // ============================================================

    /// Create a get elements request
    pub fn get_elements(request_id: String) -> Self {
        Message::GetElements {
            request_id,
            limit: None,
            target: None,
        }
    }

    /// Create a get elements request with limit
    pub fn get_elements_with_limit(request_id: String, limit: usize) -> Self {
        Message::GetElements {
            request_id,
            limit: Some(limit),
            target: None,
        }
    }

    /// Create an elements result response with observation receipt metadata
    pub fn elements_result(
        request_id: String,
        elements: Vec<ElementInfo>,
        total_count: usize,
        focused_semantic_id: Option<String>,
        selected_semantic_id: Option<String>,
        warnings: Vec<String>,
    ) -> Self {
        let truncated = elements.len() < total_count;
        Message::ElementsResult {
            request_id,
            elements,
            total_count,
            truncated,
            focused_semantic_id,
            selected_semantic_id,
            warnings,
        }
    }

    // ============================================================
    // Constructor methods for layout info
    // ============================================================

    /// Create a get layout info request
    pub fn get_layout_info(request_id: String) -> Self {
        Message::GetLayoutInfo { request_id }
    }

    /// Create a layout info result response
    pub fn layout_info_result(request_id: String, info: LayoutInfo) -> Self {
        Message::LayoutInfoResult { request_id, info }
    }

    // ============================================================
    // Constructor methods for error reporting
    // ============================================================

    /// Create a script error message from ScriptErrorData
    pub fn set_error(error_data: ScriptErrorData) -> Self {
        Message::SetError {
            error_message: error_data.error_message,
            stderr_output: error_data.stderr_output,
            exit_code: error_data.exit_code,
            stack_trace: error_data.stack_trace,
            script_path: error_data.script_path,
            suggestions: error_data.suggestions,
            timestamp: error_data.timestamp,
        }
    }

    /// Create a simple script error message with just the message and path
    pub fn script_error(error_message: String, script_path: String) -> Self {
        Message::SetError {
            error_message,
            stderr_output: None,
            exit_code: None,
            stack_trace: None,
            script_path,
            suggestions: Vec::new(),
            timestamp: None,
        }
    }

    /// Create a full script error message with all optional fields
    pub fn script_error_full(
        error_message: String,
        script_path: String,
        stderr_output: Option<String>,
        exit_code: Option<i32>,
        stack_trace: Option<String>,
        suggestions: Vec<String>,
        timestamp: Option<String>,
    ) -> Self {
        Message::SetError {
            error_message,
            stderr_output,
            exit_code,
            stack_trace,
            script_path,
            suggestions,
            timestamp,
        }
    }

    // ============================================================
    // Constructor methods for scriptlet operations
    // ============================================================

    /// Create a run scriptlet request
    pub fn run_scriptlet(
        request_id: String,
        scriptlet: ScriptletData,
        inputs: Option<serde_json::Value>,
        args: Vec<String>,
    ) -> Self {
        Message::RunScriptlet {
            request_id,
            scriptlet,
            inputs,
            args,
        }
    }

    /// Create a get scriptlets request
    pub fn get_scriptlets(request_id: String) -> Self {
        Message::GetScriptlets {
            request_id,
            kit: None,
            group: None,
        }
    }

    /// Create a get scriptlets request with filters
    pub fn get_scriptlets_filtered(
        request_id: String,
        kit: Option<String>,
        group: Option<String>,
    ) -> Self {
        Message::GetScriptlets {
            request_id,
            kit,
            group,
        }
    }

    /// Create a scriptlet list response
    pub fn scriptlet_list(request_id: String, scriptlets: Vec<ScriptletData>) -> Self {
        Message::ScriptletList {
            request_id,
            scriptlets,
        }
    }

    /// Create a successful scriptlet result
    pub fn scriptlet_result_success(
        request_id: String,
        output: Option<String>,
        exit_code: Option<i32>,
    ) -> Self {
        Message::ScriptletResult {
            request_id,
            success: true,
            output,
            error: None,
            exit_code,
        }
    }

    /// Create a failed scriptlet result
    pub fn scriptlet_result_error(
        request_id: String,
        error: String,
        exit_code: Option<i32>,
    ) -> Self {
        Message::ScriptletResult {
            request_id,
            success: false,
            output: None,
            error: Some(error),
            exit_code,
        }
    }

    // ============================================================
    // Constructor methods for ACP state query
    // ============================================================

    /// Create a getAcpState request
    pub fn get_acp_state(request_id: String) -> Self {
        Message::GetAcpState {
            request_id,
            target: None,
        }
    }

    /// Create an ACP state result response
    pub fn acp_state_result(request_id: String, state: AcpStateSnapshot) -> Self {
        Message::AcpStateResult { request_id, state }
    }

    // ============================================================
    // Constructor methods for ACP test probe
    // ============================================================

    /// Create a resetAcpTestProbe request
    pub fn reset_acp_test_probe(request_id: String) -> Self {
        Message::ResetAcpTestProbe { request_id }
    }

    /// Create a getAcpTestProbe request
    pub fn get_acp_test_probe(request_id: String, tail: Option<usize>) -> Self {
        Message::GetAcpTestProbe {
            request_id,
            tail,
            target: None,
        }
    }

    /// Create an ACP test probe result response
    pub fn acp_test_probe_result(request_id: String, probe: AcpTestProbeSnapshot) -> Self {
        Message::AcpTestProbeResult { request_id, probe }
    }

    // ============================================================
    // Constructor methods for ACP setup actions
    // ============================================================

    /// Create a performAcpSetupAction request
    pub fn perform_acp_setup_action(
        request_id: String,
        action: AcpSetupActionKind,
        agent_id: Option<String>,
    ) -> Self {
        Message::PerformAcpSetupAction {
            request_id,
            action,
            agent_id,
            target: None,
        }
    }

    /// Create a successful ACP setup action result
    pub fn acp_setup_action_result_success(request_id: String, state: AcpStateSnapshot) -> Self {
        Message::AcpSetupActionResult {
            request_id,
            success: true,
            error: None,
            state: Some(state),
        }
    }

    /// Create a failed ACP setup action result
    pub fn acp_setup_action_result_error(request_id: String, error: String) -> Self {
        Message::AcpSetupActionResult {
            request_id,
            success: false,
            error: Some(error),
            state: None,
        }
    }

    // ============================================================
    // Constructor methods for wait/batch transaction layer
    // ============================================================

    /// Create a waitFor request
    pub fn wait_for(
        request_id: String,
        condition: WaitCondition,
        timeout: Option<u64>,
        poll_interval: Option<u64>,
    ) -> Self {
        Message::WaitFor {
            request_id,
            condition,
            timeout,
            poll_interval,
            trace: TransactionTraceMode::Off,
            target: None,
        }
    }

    /// Create a waitFor request with trace mode
    pub fn wait_for_with_trace(
        request_id: String,
        condition: WaitCondition,
        timeout: Option<u64>,
        poll_interval: Option<u64>,
        trace: TransactionTraceMode,
    ) -> Self {
        Message::WaitFor {
            request_id,
            condition,
            timeout,
            poll_interval,
            trace,
            target: None,
        }
    }

    /// Create a waitFor result
    pub fn wait_for_result(
        request_id: String,
        success: bool,
        elapsed: u64,
        error: Option<TransactionError>,
    ) -> Self {
        Message::WaitForResult {
            request_id,
            success,
            elapsed,
            error,
            trace: None,
        }
    }

    /// Create a waitFor result with embedded trace receipt
    pub fn wait_for_result_with_trace(
        request_id: String,
        success: bool,
        elapsed: u64,
        error: Option<TransactionError>,
        trace: Option<TransactionTrace>,
    ) -> Self {
        Message::WaitForResult {
            request_id,
            success,
            elapsed,
            error,
            trace,
        }
    }

    /// Create a batch request
    pub fn batch(
        request_id: String,
        commands: Vec<BatchCommand>,
        options: Option<BatchOptions>,
    ) -> Self {
        Message::Batch {
            request_id,
            commands,
            options,
            trace: TransactionTraceMode::Off,
            target: None,
        }
    }

    /// Create a batch request with trace mode
    pub fn batch_with_trace(
        request_id: String,
        commands: Vec<BatchCommand>,
        options: Option<BatchOptions>,
        trace: TransactionTraceMode,
    ) -> Self {
        Message::Batch {
            request_id,
            commands,
            options,
            trace,
            target: None,
        }
    }

    /// Create a batch result
    pub fn batch_result(
        request_id: String,
        success: bool,
        results: Vec<BatchResultEntry>,
        failed_at: Option<usize>,
        total_elapsed: u64,
    ) -> Self {
        Message::BatchResult {
            request_id,
            success,
            results,
            failed_at,
            total_elapsed,
            trace: None,
        }
    }

    // ============================================================
    // Constructor methods for automation window targeting
    // ============================================================

    /// Create a listAutomationWindows request
    pub fn list_automation_windows(request_id: String) -> Self {
        Message::ListAutomationWindows { request_id }
    }

    /// Create an automation window list result response
    pub fn automation_window_list_result(
        request_id: String,
        windows: Vec<AutomationWindowInfo>,
        focused_window_id: Option<String>,
    ) -> Self {
        Message::AutomationWindowListResult {
            request_id,
            windows,
            focused_window_id,
        }
    }

    // ============================================================
    // Constructor methods for GPUI event simulation
    // ============================================================

    /// Create a simulateGpuiEvent request
    pub fn simulate_gpui_event(
        request_id: String,
        event: SimulatedGpuiEvent,
        target: Option<AutomationWindowTarget>,
    ) -> Self {
        Message::SimulateGpuiEvent {
            request_id,
            target,
            event,
        }
    }

    /// Create a simulateGpuiEvent success result
    pub fn simulate_gpui_event_result_success(request_id: String) -> Self {
        Message::SimulateGpuiEventResult {
            request_id,
            success: true,
            error: None,
        }
    }

    /// Create a simulateGpuiEvent error result
    pub fn simulate_gpui_event_result_error(request_id: String, error: String) -> Self {
        Message::SimulateGpuiEventResult {
            request_id,
            success: false,
            error: Some(error),
        }
    }

    /// Create a batch result with embedded trace receipt
    pub fn batch_result_with_trace(
        request_id: String,
        success: bool,
        results: Vec<BatchResultEntry>,
        failed_at: Option<usize>,
        total_elapsed: u64,
        trace: Option<TransactionTrace>,
    ) -> Self {
        Message::BatchResult {
            request_id,
            success,
            results,
            failed_at,
            total_elapsed,
            trace,
        }
    }
}

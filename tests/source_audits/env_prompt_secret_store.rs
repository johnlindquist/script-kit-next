#[test]
fn secret_store_load_failures_are_typed_not_empty_maps() {
    let secrets = super::read_source("src/secrets.rs");

    assert!(
        secrets.contains("pub enum SecretStoreErrorKind")
            && secrets.contains("InvalidFormat")
            && secrets.contains("DecryptFailed")
            && secrets.contains("ParseFailed")
            && secrets.contains("ReadFailed")
            && secrets.contains("PathUnavailable"),
        "secret store should expose typed failure classes"
    );
    assert!(
        secrets.contains(
            "fn load_secrets_from_disk() -> Result<HashMap<String, SecretEntry>, SecretStoreError>"
        ),
        "secret store loading should return Result so failures cannot collapse to missing secrets"
    );
    assert!(
        secrets.contains("pub fn get_secret_info_result")
            && secrets.contains("pub fn get_secret_result"),
        "EnvPrompt should have result-returning secret lookup APIs"
    );
    assert!(
        secrets.contains("*guard = Some(Ok(secrets.clone()))")
            && !secrets.contains("*guard = Some(secrets.clone())"),
        "secret store should cache successful loads but not cache load errors"
    );
}

#[test]
fn env_prompt_receives_secret_store_health_and_cached_secret() {
    let prompt = super::read_source("src/prompts/env/prompt.rs");
    let handler = super::read_source("src/prompt_handler/mod.rs");
    let execution = super::read_source("src/app_execute/execution_helpers.rs");

    assert!(
        prompt.contains("pub secret_store_error: Option<SecretStoreError>")
            && prompt.contains("stored_secret_value: Option<String>")
            && prompt.contains("EnvPrompt skipped auto-submit"),
        "EnvPrompt should distinguish stored secret, missing secret, and storage failure"
    );
    assert!(
        handler.contains("secrets::get_secret_info_result(&key)")
            && handler.contains("EnvPrompt secret store unavailable"),
        "SDK EnvPrompt routing should preserve secret store lookup errors"
    );
    assert!(
        execution.contains("secrets::get_secret_info_result(&key)")
            && execution.contains("API key prompt secret store unavailable"),
        "API key prompts should preserve secret store lookup errors"
    );
}

#[test]
fn env_prompt_elements_report_storage_error_without_secret_values() {
    let collect = super::read_source("src/app_layout/collect_elements.rs");

    assert!(
        collect.contains("env-secret-store-error")
            && collect.contains("secret_store_error")
            && collect.contains("status_kind: Some(error.kind_str().to_string())"),
        "getElements should expose a machine-readable storage error status"
    );
    assert!(
        collect.contains("value: Some(error.kind_str().to_string())")
            && !collect.contains("value: Some(error.message"),
        "storage error receipts should expose stable kind, not low-level error text or values"
    );
}

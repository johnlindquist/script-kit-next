//! JSON for the installed app's `--personalize` operation. These receipts describe the stored
//! workspace; only a reply from the running window can attest to adoption there.
use super::*;
use sk_one_window::personal::{TextRole, Weight};
mod endpoint;
mod review;
pub use endpoint::{Adoption, Endpoint};
pub use review::{SaveReview, review_apply};
#[derive(Deserialize)]
#[serde(
    tag = "op",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum Query {
    Describe,
    Inspect,
    Preview { change: Change },
    Apply { request: Request },
    Why { target: String },
    Status { request_id: Uuid },
}
fn marker(s: &str) -> Result<Edit> {
    [
        Edit::PinText(TextRole::Muted),
        Edit::PinWeight(Weight::Regular),
        Edit::SelectionFirst(None),
        Edit::ClipboardFirst(None),
    ]
    .into_iter()
    .find(|e| e.target() == s)
    .ok_or(Error::InvalidRequest)
}
pub fn answer(root: &Path, body: &[u8], safe_mode: bool) -> Result<Value> {
    if body.len() > 65536 {
        return Err(Error::InvalidRequest);
    }
    let query: Query = serde_json::from_slice(body).map_err(|_| Error::InvalidRequest)?;
    serve(root, query, safe_mode, Author::LocalAgent)
}
fn serve(root: &Path, query: Query, safe_mode: bool, author: Author) -> Result<Value> {
    if matches!(query, Query::Describe) {
        return Ok(catalog());
    }
    let store = Store::open(root)?;
    match query {
        Query::Describe => Ok(catalog()),
        Query::Inspect => {
            Ok(json!({"scope":"stored","safeMode":safe_mode,"state":store.inspect()?}))
        }
        Query::Preview { change } => {
            Ok(json!({"scope":"preview","wouldStore":store.preview(change)?}))
        }
        Query::Apply { request } => {
            if safe_mode {
                return Err(Error::SafeMode);
            }
            Ok(json!({"scope":"stored","result":store.apply(request,author)?}))
        }
        Query::Why { target } => match target.strip_prefix("personal.view.") {
            Some(id) => store.why_view(id),
            None => store.why(marker(&target)?),
        },
        Query::Status { request_id } => store.status(request_id),
    }
}
pub fn safe_mode() -> bool {
    std::env::var_os("SCRIPT_KIT_PERSONAL_SAFE_MODE").is_some_and(|s| s != "0")
}

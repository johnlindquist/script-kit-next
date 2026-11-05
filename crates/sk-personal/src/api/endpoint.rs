//! Per-profile and per-bearer access uses two different routes. Parsing a read never
//! grants a write. The workspace and author come from the host, never from model arguments.
use super::*;
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Adoption {
    pub workspace: Option<String>,
    pub applied_revision: Option<String>,
    pub effective: LauncherRules,
    /// Configuration identities only, not view results or input.
    pub active_view_ids: Vec<String>,
    pub error: Option<Error>,
    pub pending_write: bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Endpoint {
    Read,
    Apply,
}
impl Endpoint {
    pub fn scope(self) -> &'static str {
        match self {
            Self::Read => "launcher:read",
            Self::Apply => "launcher:apply",
        }
    }
    pub fn profile_tool(self) -> &'static str {
        match self {
            Self::Read => "launcher_read",
            Self::Apply => "launcher_apply",
        }
    }
    pub fn mcp_tool(self) -> &'static str {
        match self {
            Self::Read => "kit/launcher_read",
            Self::Apply => "kit/launcher_apply",
        }
    }
    pub fn from_mcp(name: &str) -> Option<Self> {
        [Self::Read, Self::Apply]
            .into_iter()
            .find(|e| e.mcp_tool() == name)
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::Read => {
                "Describe, inspect, preview or explain scoped launcher preferences; check request status or the running window's adopted revision. No desktop or document contents."
            }
            Self::Apply => {
                "Apply one explicitly requested launcher edit, disable/enable, or undo. Bind workspace and revision from inspect, and reuse the same UUID only for an identical request. Returns stored state, not an assertion about the window."
            }
        }
    }
    pub fn schema(self) -> Value {
        let uuid = json!({"type":"string","format":"uuid"});
        let cat = catalog();
        let targets = cat["targets"].as_array().expect("static launcher catalog");
        let edits:Vec<_>=targets.iter().map(|t|json!({"type":"object","additionalProperties":false,"required":["target","value"],"properties":{"target":{"const":t["id"]},"value":{"enum":t["values"]}}})).collect();
        let change = json!({"oneOf":[{"type":"object","additionalProperties":false,"required":["op","edit"],"properties":{"op":{"const":"edit"},"edit":{"oneOf":edits}}},{"type":"object","additionalProperties":false,"required":["op","view"],"properties":{"op":{"const":"putView"},"view":cat["views"]["schema"]}},{"type":"object","additionalProperties":false,"required":["op","id"],"properties":{"op":{"const":"removeView"},"id":cat["views"]["schema"]["properties"]["id"]}} ,{"type":"object","additionalProperties":false,"required":["op","of"],"properties":{"op":{"const":"undo"},"of":uuid}},{"type":"object","additionalProperties":false,"required":["op"],"properties":{"op":{"enum":["disable","enable"]}}}]});
        match self {
            Self::Read => {
                json!({"type":"object","additionalProperties":false,"required":["op"],"properties":{"op":{"enum":["describe","inspect","preview","why","status","live"]},"change":change,"target":{"anyOf":[{"enum":targets.iter().map(|v|&v["id"]).collect::<Vec<_>>()},{"type":"string","pattern":"^personal[.]view[.][a-z]([a-z0-9]|-[a-z0-9])*$","maxLength":54}]},"requestId":uuid}})
            }
            Self::Apply => {
                json!({"type":"object","additionalProperties":false,"required":["op","request"],"properties":{"op":{"const":"apply"},"request":{"type":"object","additionalProperties":false,"required":["workspace","expectedRevision","requestId","change"],"properties":{"workspace":{"type":"string","pattern":"^[a-f0-9]{64}$"},"expectedRevision":{"type":"string","pattern":"^(stock|[a-f0-9]{64})$"},"requestId":uuid,"change":change}}}})
            }
        }
    }
    pub(super) fn query(self, args: Value) -> Result<Query> {
        // The transport has already parsed JSON. Count bytes without allocating a second unbounded copy.
        struct Limit(usize);
        impl std::io::Write for Limit {
            fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
                self.0 = self
                    .0
                    .checked_sub(b.len())
                    .ok_or_else(|| std::io::Error::other("input too large"))?;
                Ok(b.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        serde_json::to_writer(Limit(65536), &args).map_err(|_| Error::InvalidRequest)?;
        let count = args.as_object().ok_or(Error::InvalidRequest)?.len();
        let query: Query = serde_json::from_value(args).map_err(|_| Error::InvalidRequest)?;
        let (owner, fields) = match &query {
            Query::Apply { .. } => (Self::Apply, 2),
            Query::Describe | Query::Inspect => (Self::Read, 1),
            Query::Preview { .. } | Query::Why { .. } | Query::Status { .. } => (Self::Read, 2),
        };
        if self != owner || count != fields {
            return Err(Error::InvalidRequest);
        }
        Ok(query)
    }
    pub fn stored(self, root: &Path, args: Value, safe: bool, author: Author) -> Result<Value> {
        serve(root, self.query(args)?, safe, author)
    }
}

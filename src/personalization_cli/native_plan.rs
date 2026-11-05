//! A local developer handoff, never a build, grant or installer. Bind comparisons to the
//! actual executable file rather than trusting a release number or a short development SHA.
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sk_personal::Error;
use std::{fs::File, io::Read};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeRequest {
    request: String,
    proof: String,
}

pub(super) fn describe(input: impl Read) -> Result<Value, Error> {
    let mut bytes = Vec::new();
    input
        .take(8193)
        .read_to_end(&mut bytes)
        .map_err(|_| Error::InvalidRequest)?;
    if bytes.len() > 8192 {
        return Err(Error::InvalidRequest);
    }
    let args: NativeRequest = serde_json::from_slice(&bytes).map_err(|_| Error::InvalidRequest)?;
    for text in [&args.request, &args.proof] {
        if text.trim().is_empty() || text.len() > 2048 || text.chars().any(char::is_control) {
            return Err(Error::InvalidRequest);
        }
    }
    let path = std::env::current_exe().map_err(|_| Error::StorageUnavailable)?;
    let mut file = File::open(path).map_err(|_| Error::StorageUnavailable)?;
    let mut hash = Sha256::new();
    std::io::copy(&mut file, &mut hash).map_err(|_| Error::StorageUnavailable)?;
    Ok(json!({
        "scope":"nativeCandidatePlan", "state":"notBuilt", "request":args.request, "proof":args.proof,
        "base":{"version":env!("CARGO_PKG_VERSION"),"executableSha256":format!("{:x}",hash.finalize()),"gitHint":env!("GIT_HASH")},
        "source":"Match the executable hash to the signed release manifest or a verified local ArtifactReference. Use its exact source commit, not gitHint, a branch name or the version alone. Stop if the base cannot be identified.",
        "isolation":"Use a separate worktree and disposable Kit home. Do not start the candidate with the running app's home or write into a signed bundle.",
        "candidate":{"operation":"build-ops act app-build","feature":"owned-ui-evaluation","output":"new ArtifactReference","guide":"docs/guides/personal-launcher.md#when-kit-needs-a-native-change"},
        "comparison":"Exercise the requested proof on the baseline and candidate, with the same inputs. Keep both binary hashes, exact source commits and actual observed results. A build alone does not verify the change.",
        "switch":{"authorized":false,"installed":false,"instruction":"Keep the current installation and its data. Offer the verified candidate and a rollback to the prior signed version; switch only with a separate explicit decision."}
    }))
}

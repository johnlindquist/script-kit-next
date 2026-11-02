#!/usr/bin/env bash
# Read back one exact build. Never compile, sign, rerun, or write a registry.
set -euo pipefail
sha=${1:?expected commit}; receipt=${2:?build receipt}; ref=${3:?artifact reference}; out=${4:?empty evidence directory}
[[ $sha =~ ^[0-9a-f]{40}$ && $(git rev-parse HEAD) = "$sha" && -z $(git status --porcelain) ]]
[[ -d $out && ! -L $out && ! -e $out/qualification.json && ! -L $out/qualification.json && ! -e $out/script-kit-gpui && ! -L $out/script-kit-gpui ]]
jq -e --slurpfile ref "$ref" '
  .pass and .validation.passed and .cleanup.closed and (.cleanup.survivors == []) and (.cleanup.failureCodes == []) and
  (.buildOps.result | .status == "succeeded" and .artifact == $ref[0] and .cleanup.closed and
    (.resources | .scope == "target-agent" and .measurement == "allocated-blocks" and .hardQuota == false and .automaticEviction == false and
      .refusal == null and (.checks | length > 0 and all(.[]; .complete and .withinLimits and .targetAgentBudgetBytes == 42949672960 and .minimumFreeBytes >= 26843545600)) and
      (.monitoring | .complete and .workerClosed and .trigger == null and .callbackFailed == false and .transientFailureCount == 0 and
        .maximumSampledAllocatedBytes <= 42949672960 and .minimumSampledAvailableBytes >= 26843545600 and
        (.compilerPauses | .held > 0 and .held == .resumed and .afterExit > 0 and .maxElapsedMs > 0 and .maxElapsedMs <= 8000))))' "$receipt" >/dev/null
id=$(jq -er '.buildOps.result.task.id' "$receipt"); gen=$(jq -er '.buildOps.result.task.generation' "$receipt")
query=$(mktemp "${TMPDIR:-/tmp}/compiler-task.XXXXXXXX"); trap 'rm -f "$query"' EXIT
bun scripts/devtools/devtools.ts build-ops query job --id "$id" --generation "$gen" > "$query"
jq -e --arg sha "$sha" --slurpfile build "$receipt" '
  .pass and .validation.passed and (.buildOps.result.task | .identity == $build[0].buildOps.result.task and .state == "closed" and
    .source.gitHead == $sha and .source.repositoryDirty == false and .cleanup.closed and .cleanup.referencesFinalized and
    .result.status == "succeeded" and .result.resources.monitoring == $build[0].buildOps.result.resources.monitoring)' "$query" >/dev/null
bin=$(bun scripts/agentic/build-artifact.ts verify-reference "$PWD" "$ref")
manifest=$(jq -er '.manifestPath' "$ref"); digest=$(shasum -a 256 "$bin" | cut -d' ' -f1)
[[ $(shasum -a 256 "$manifest" | cut -d' ' -f1) = "$(jq -er .manifestSha256 "$ref")" && $(lipo -archs "$bin") = arm64 ]]
jq -e --arg sha "$sha" --arg id "$id" --arg gen "$gen" --arg digest "$digest" '
  .artifactKind == "application" and .binarySha256 == $digest and .source.gitHead == $sha and .source.repositoryDirty == false and
  .publication.immutable and .publication.exportedWhileLeaseHeld and .publication.buildTask.id == $id and .publication.buildTask.generation == $gen and
  .target.packageName == "script-kit-gpui" and .target.targetName == "script-kit-gpui" and .target.targetTriple == "aarch64-apple-darwin" and
  .target.requestedProfile == "release" and (.target.features | index("owned-ui-evaluation") != null) and
  .effectiveConfiguration.requestedPolicy.compilerSampling == "owned-pause-v1" and .effectiveConfiguration.requestedPolicy.compilerWorkers == 1' "$manifest" >/dev/null
install -m 0755 "$bin" "$out/script-kit-gpui"; [[ $(shasum -a 256 "$out/script-kit-gpui" | cut -d' ' -f1) = "$digest" ]]
jq -n --arg sha "$sha" --arg digest "$digest" --slurpfile ref "$ref" --slurpfile receipt "$receipt" --slurpfile manifest "$manifest" '{
  commit:$sha,executableSha256:$digest,manifestSha256:$ref[0].manifestSha256,compilerInputSha256:$manifest[0].source.compilerInputSha256,
  toolchainChannel:$manifest[0].toolchain.channel,target:$manifest[0].target.targetTriple,profile:$manifest[0].target.requestedProfile,
  task:$receipt[0].buildOps.result.task,closed:true,monitoring:$receipt[0].buildOps.result.resources.monitoring
}' > "$out/qualification.json"
jq -c . "$out/qualification.json"

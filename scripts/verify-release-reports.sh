#!/usr/bin/env bash
# Read-only companion to release-evidence.ts scorecard. The expected report digest
# comes from the signing job; the separate scorecard is derived from the downloaded ZIP.
set -euo pipefail
export LC_ALL=C
refuse() { printf 'Release reports: %s\n' "$1" >&2; exit 1; }
[[ $# == 3 ]] || { echo 'Usage: bash scripts/verify-release-reports.sh <downloads> <recomputed-scorecard.json> <signer-workspace-sha256>' >&2; exit 64; }
[[ -d "$1" && ! -L "$1" ]] || refuse 'missing or indirect downloads directory'
manifest=$1/release-manifest.json
workspace=$1/installed-workspace-check.json
scorecard=$1/release-scorecard.json
[[ $3 =~ ^[0-9a-f]{64}$ ]] || refuse 'missing or invalid signing-job report digest'
for file in "$manifest" "$workspace" "$scorecard" "$2"; do
  [[ -f "$file" && ! -L "$file" && -s "$file" ]] || refuse 'a required report is absent, indirect, or empty'
done
[[ ! "$scorecard" -ef "$2" ]] || refuse 'the scorecard must be recomputed separately'
cmp -s "$scorecard" "$2" || refuse 'scorecard differs from the verified archive and manifest'
actual=$(shasum -a 256 < "$workspace")
[[ ${actual%% *} == "$3" ]] || refuse 'packaged workspace report differs from the signing job'
# Parse single JSON objects. The manifest and scorecard have already passed the archive verifier.
jq -es --slurpfile m "$manifest" --slurpfile c "$2" '
  def digest: type == "string" and length == 64 and test("^[0-9a-f]{64}$");
  length == 1 and ($m | length) == 1 and ($c | length) == 1 and
  (.[0] as $w | $m[0] as $manifest | $c[0] as $card |
   ($w.sourceSha | type == "string" and length == 40 and test("^[0-9a-f]{40}$")) and
   $w.sourceSha == $manifest.source_sha and $w.sourceSha == $card.sourceSha and
   ($manifest.artifacts | length) == 1 and ($w.zipSha256 | digest) and
   $w.zipSha256 == $manifest.artifacts[0].sha256 and $w.zipSha256 == $card.archiveSha256 and
   ($w.sdkSha256 | digest) and $w.sdkSha256 == $manifest.sdk.sha256 and $w.version == $manifest.version)
' "$workspace" >/dev/null 2>&1 || refuse 'packaged workspace report belongs to a different source, SDK, or archive'
version=$(jq -er '.version' "$manifest" 2>/dev/null) || refuse 'missing manifest version'
jq -e --arg version "$version" --argjson packaged true -f "$(dirname "$0")/verify-installed-workspace.jq" "$workspace" >/dev/null 2>&1 || refuse 'invalid packaged settings witness'
printf 'Verified release scorecard and packaged workspace report.\n'

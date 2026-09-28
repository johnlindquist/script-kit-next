# Pinned goose ACP sidecar build inputs, sourced by prepare-goose-sidecar.sh
# and scripts/agentic/ensure-goose-sidecar.sh.
# goose v1.52.0
GOOSE_SIDECAR_REF="302b60806639ea9f0ae8f053f49f8bf0e88b26f4"
GOOSE_SIDECAR_FEATURES="rustls-tls,disable-update"

# A built sidecar records these inputs in `<binary>.stamp`; a binary whose
# stamp differs was built from other inputs and must not be reused.
goose_sidecar_stamp() {
  printf 'ref=%s features=%s\n' "${GOOSE_SIDECAR_REF}" "${GOOSE_SIDECAR_FEATURES}"
}

goose_sidecar_stamp_matches() {
  [[ -x "$1" && -f "$1.stamp" && "$(<"$1.stamp")" == "$(goose_sidecar_stamp)" ]]
}

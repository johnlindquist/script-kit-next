#!/bin/bash
# Verify the actual bundle with Apple's tool. Only a *known transport interruption*
# permits another lookup; a missing ticket, invalid app, rejection or unknown failure does not.
set -euo pipefail
[[ $# == 2 && -d $2 && ! -L $2 && $2 != -* ]] || { echo 'Expected staple|validate and a bundle directory' >&2; exit 64; }
mode=$1 app=$2 details=''
interrupted() {
  [[ $1 == 68 && $2 == *'Error Domain=NSURLErrorDomain Code=-1005 '* && $2 == *'https://api.apple-cloudkit.com/database/1/com.apple.gk.ticket-delivery/production/public/records/lookup'* ]]
}
backoff() {
  printf 'Apple ticket lookup lost its connection (%s, attempt %s of 3)\n' "$1" "$2" >&2
  sleep "$((2 * $2))"
}
# Keep details for the caller: only Apple's positive no-local-ticket observation permits
# another *mutation* after a staple whose outcome was uncertain.
validate_ticket() {
  local n rc
  for n in 1 2 3; do
    if details=$(xcrun stapler validate "$app" 2>&1); then return 0; else rc=$?; fi
    if ! interrupted "$rc" "$details" || [[ $n == 3 ]]; then return "$rc"; fi
    backoff validate "$n" || return $?
  done
}
finish_validation() {
  if validate_ticket; then printf '%s\n' "$details"; exit 0; else rc=$?; printf '%s\n' "$details" >&2; exit "$rc"; fi
}
case "$mode" in
  validate) finish_validation ;;
  staple)
    # Mutation is only for the signed candidate inside the hosted release checkout.
    [[ ${GITHUB_ACTIONS:-} == true && ${GITHUB_REF_TYPE:-} == tag && ${RUNNER_OS:-} == macOS && ${GITHUB_WORKSPACE:-} -ef . && $app == 'target/release/bundle/osx/Script Kit.app' && $(cd "$app" && pwd -P) == "$(pwd -P)/$app" ]] || { echo 'Refusing to staple outside the release build' >&2; exit 64; }
    jq -se 'length == 1 and (.[0] | .status == "Accepted" and (.id | type == "string" and test("^[0-9a-f]{8}(-[0-9a-f]{4}){3}-[0-9a-f]{12}$"; "i")))' "$RUNNER_TEMP/notarization-result.json" >/dev/null || { echo 'Apple did not confirm an accepted submission ID' >&2; exit 1; }
    for n in 1 2 3; do
      if result=$(xcrun stapler staple "$app" 2>&1); then finish_validation; else rc=$?; fi
      if ! interrupted "$rc" "$result"; then printf '%s\n' "$result" >&2; exit "$rc"; fi
      if validate_ticket; then printf '%s\n' "$details"; exit 0; else check=$?; fi
      if [[ $check != 65 || $details != *'does not have a ticket stapled to it.'* ]]; then printf '%s\n' "$details" >&2; exit "$check"; fi
      if [[ $n == 3 ]]; then printf '%s\n' "$result" >&2; exit "$rc"; fi
      backoff staple "$n"
    done ;;
  *) echo 'Use staple or validate' >&2; exit 64 ;;
esac

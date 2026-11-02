# macOS release pipeline

A version tag must match the package, lockfile and bundle versions. An unsigned Actions artifact is not an installable GitHub Release. Publication requires the tagged source's test gates, Developer ID signing, Apple notarization, stapling, Gatekeeper and verification of the re-extracted ZIP.

## Compile once; verify at each boundary

`build-macos-app` has no Apple signing credentials. It uses `build-ops act app-build --profile release --features owned-ui-evaluation` to produce an immutable executable. The pinned `cargo-bundle` packages a copy of that executable with `CARGO_BUNDLE_SKIP_BUILD=1`. It does not compile a replacement. `scripts/agentic/release-compile.ts seal` verifies the finalized managed build, the binary in the unsigned bundle and the ZIP. The builder uploads the ZIP and a portable compile receipt, and reports its artifact ID and receipt SHA-256 separately through job outputs.

The signing job downloads that exact artifact from the same run. `verify-zip` checks the independently supplied receipt digest, repository, tag, commit, run, target, compiler and configuration. A later attempt can reuse a successful earlier build of that *same run*. The job validates archive membership before extracting it. `verify-executable` checks the extracted executable before the Apple keychain step. After signing, `verify-signed-payload` checks that only signing-owned Mach-O data changed before the app is sent to Apple.

The portable receipt is historical evidence, not a locally executable reference. No compiler pool, owner token, lease or mutable task registry is copied into the signer. `build-ops act publish-signed-bundle --release-input <receipt> --expected-input-sha256 <producer-output> --bundle <app> --attestation <attestation> --artifact-out <new-reference>` publishes a locally owned, immutable signed derivation with the original compile evidence. It still verifies the app, sidecar, security attestation, normalized program bytes and bundle tree. `--input <local-reference>` remains available for all-local derivations. Supplying both input modes is refused. Never compute the expected digest from the download you are about to verify.

## Recovering a stopped compiler proof

A normal proof produces complete unit, compile-fail, contract and generated-export evidence. Only the narrowly recognized cold compiler directory churn may instead produce a **failed measurement**, with its closed-owner receipt, and a one-use digest-bound cache handoff. A fresh Mac can start the *entire* proof immediately without waiting for packaging. A cache hit or refusal alone cannot satisfy the aggregate gate: the signer uses the artifact ID selected by the successful proof, not just its display name. Debug and release builds watch their active compiler directory under the same time, allocation and identity limits. An incomplete sample is never a passing measurement. A test failure, storage-limit breach, unknown identity or diagnostic, missing evidence or cache, unclosed worker, or second refusal stops the release.

For the actual run, inspect the proof and retry jobs separately from signing and publication. A retry doesn't authorize moving the tag, running a different binary or treating a failed measurement as a pass.

# A signed archive's resource probe, not evidence of a LaunchServices installation.
def digest: type == "string" and length == 64 and test("^[a-f0-9]{64}$");
.schemaVersion == 1 and .version == $version and ($packaged | type == "boolean") and .packaged == $packaged and
.freshWorkspace == true and (.starterScriptsIndexed | type == "number" and . > 0 and . == floor) and
.userEditsPreserved == true and .settingsTransaction == "verified" and .scratchRemoved == true and
(.settingsProof | .key == "editorFontSize" and (.beforeValue | type == "number") and
  .value == (if .beforeValue == 27 then 28 else 27 end) and
  (.beforeSha256 | digest) and (.writtenSha256 | digest) and .beforeSha256 != .writtenSha256 and
  .afterRejectionSha256 == .writtenSha256 and .rejection == "stale_revision")

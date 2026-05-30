export type JsonObject = Record<string, unknown>;

export type OutputPolicy = {
  outputPath?: string | null;
  previewBytes: number;
  inlineFullOutput: boolean;
};

export function fingerprint(value: string) {
  let hash = 2166136261;
  for (const char of value) {
    hash ^= char.charCodeAt(0);
    hash = Math.imul(hash, 16777619);
  }
  return (hash >>> 0).toString(16).padStart(8, "0");
}

export function byteLength(value: string) {
  return new TextEncoder().encode(value).length;
}

export function summarizeText(value: string, previewBytes: number) {
  const bytes = byteLength(value);
  const preview = value.slice(0, previewBytes);
  const previewLength = byteLength(preview);
  return {
    bytes,
    preview,
    previewBytes: previewLength,
    omittedBytes: Math.max(0, bytes - previewLength),
    truncated: bytes > previewLength,
    fingerprint: fingerprint(value),
  };
}

export function tryParseJson(value: string) {
  try {
    const parsed = JSON.parse(value) as JsonObject;
    return {
      parsed: true,
      tool: parsed.tool ?? null,
      command: parsed.command ?? null,
      classification: parsed.classification ?? null,
      status: parsed.status ?? null,
      fingerprint: fingerprint(JSON.stringify(parsed)),
    };
  } catch {
    return {
      parsed: false,
      tool: null,
      command: null,
      classification: null,
      status: null,
      fingerprint: null,
    };
  }
}

export async function outputSummary(
  label: string,
  stdout: string,
  stderr: string,
  policy: OutputPolicy,
) {
  let artifactPath: string | null = null;
  if (policy.outputPath) {
    artifactPath = policy.outputPath;
    await Bun.write(
      artifactPath,
      JSON.stringify(
        {
          schemaVersion: 1,
          label,
          stdout,
          stderr,
          stdoutJson: tryParseJson(stdout),
          stderrJson: tryParseJson(stderr),
        },
        null,
        2,
      ),
    );
  }

  return {
    label,
    artifactPath,
    inlineFullOutput: policy.inlineFullOutput,
    stdout: policy.inlineFullOutput ? stdout : null,
    stderr: policy.inlineFullOutput ? stderr : null,
    stdoutSummary: summarizeText(stdout, policy.previewBytes),
    stderrSummary: summarizeText(stderr, policy.previewBytes),
    stdoutJson: tryParseJson(stdout),
    stderrJson: tryParseJson(stderr),
  };
}

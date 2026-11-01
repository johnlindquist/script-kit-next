import { createHash } from "node:crypto";
import {
  ARCHIVE,
  check,
  ID,
  type Json,
  REPO,
  selectAsset,
  sha,
  verifyManifest,
} from "./installed-release-contract";

// Public, anonymous HTTPS only. No gh token, shell, or arbitrary release-provided URL.
export async function publicJson(path: string): Promise<Json> {
  const res = await fetch(`https://api.github.com/repos/${REPO}/${path}`, {
    headers: { Accept: "application/vnd.github+json", "X-GitHub-Api-Version": "2022-11-28" },
    redirect: "error",
    credentials: "omit",
    signal: AbortSignal.timeout(20_000),
  });
  check(res.ok && res.body, "public_api_unavailable", "blocked");
  const chunks: Uint8Array[] = [];
  let bytes = 0;
  for await (const chunk of res.body) {
    bytes += chunk.byteLength;
    check(bytes <= 8 * 1024 ** 2, "public_api_response_too_large");
    chunks.push(chunk);
  }
  return JSON.parse(Buffer.concat(chunks).toString("utf8"));
}
export async function* assetBytes(a: ReturnType<typeof selectAsset>): AsyncGenerator<Uint8Array> {
  const signal = AbortSignal.timeout(180_000);
  let url = a.url;
  for (let redirect = 0; redirect < 4; redirect++) {
    const res = await fetch(url, { redirect: "manual", credentials: "omit", signal });
    if (res.status >= 300 && res.status < 400) {
      const next = new URL(res.headers.get("location") ?? "", url);
      await res.body?.cancel();
      check(
        next.protocol === "https:" && !next.username && !next.password && !next.port &&
          ["release-assets.githubusercontent.com", "objects.githubusercontent.com"].includes(next.hostname),
        "untrusted_asset_redirect",
      );
      url = next.href;
      continue;
    }
    check(res.status === 200 && res.body, "public_download_unavailable", "blocked");
    const hash = createHash("sha256");
    let bytes = 0;
    for await (const chunk of res.body) {
      bytes += chunk.byteLength;
      check(bytes <= a.size, "asset_size_exceeded");
      hash.update(chunk);
      yield chunk;
    }
    check(bytes === a.size && hash.digest("hex") === a.sha256, "github_asset_digest_mismatch");
    return;
  }
  check(false, "too_many_asset_redirects");
}
// Source is addressed by the peeled upstream commit, never by the QA checkout.
export function verifySourceVersion(file: Json, tag: string): string {
  check(
    file.type === "file" && file.path === "Cargo.toml" && file.encoding === "base64" &&
      Number.isSafeInteger(file.size) && file.size > 0 && file.size <= 1024 ** 2,
    "missing_source_version",
  );
  const bytes = Buffer.from(file.content, "base64");
  check(
    bytes.length === file.size &&
      createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex") === file.sha,
    "invalid_source_blob",
  );
  const manifest = Bun.TOML.parse(bytes.toString("utf8")) as Json;
  const bin = manifest.package?.metadata?.bundle?.bin?.["script-kit-gpui"];
  check(
    manifest.package?.name === "script-kit-gpui" && manifest.package.version === tag.slice(1) &&
      bin?.identifier === ID && bin.version === tag.slice(1),
    "source_version_mismatch",
  );
  return file.sha;
}
// All filesystem mutations belong to the guarded orchestrator; this module is read-only.
export async function fetchPublicRelease(tag: string, runCreatedAt: string) {
  const [release, ref] = await Promise.all([
    publicJson(`releases/tags/${tag}`),
    publicJson(`git/ref/tags/${tag}`),
  ]);
  check(ref.ref === `refs/tags/${tag}` && sha.test(ref.object?.sha), "upstream_tag_mismatch");
  let object = ref.object;
  if (object.type === "tag") {
    const annotated = await publicJson(`git/tags/${object.sha}`);
    check(annotated.sha === object.sha && annotated.tag === tag, "annotated_tag_mismatch");
    object = annotated.object;
  }
  check(object.type === "commit" && sha.test(object.sha), "tag_not_commit");
  const zip = selectAsset(release, tag, ARCHIVE, runCreatedAt),
    manifest = selectAsset(release, tag, "release-manifest.json", runCreatedAt);
  const sourceManifestBlob = verifySourceVersion(
    await publicJson(`contents/Cargo.toml?ref=${object.sha}`),
    tag,
  );
  const chunks: Uint8Array[] = [];
  for await (const bytes of assetBytes(manifest)) chunks.push(bytes);
  const identity = verifyManifest(JSON.parse(Buffer.concat(chunks).toString("utf8")), tag, object.sha, zip);
  return {
    tag,
    sourceSha: object.sha as string,
    sourceManifestBlob,
    releaseId: release.id as number,
    publishedAt: release.published_at as string,
    archive: zip,
    manifest,
    ...identity,
  };
}

import {
  createReadStream,
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import type { Octokit } from "@octokit/rest";
import semver from "semver";
import { stringify } from "smol-toml";
import { sha256Stream } from "./artifacts.mts";
import {
  type ImageReceipt,
  manifestChecksums,
  type ReleaseManifest,
} from "./release-bundle.mts";
import {
  type Candidate,
  parseCandidate,
  repoParameters,
  streamDownload,
} from "./release-candidate.mts";
import {
  contentHash,
  jsonRecord,
  readJson,
  safePath,
  writeJson,
} from "./release-inputs.mts";
import { type Metadata, parseMetadata } from "./release-metadata.mts";

export type ReleaseObject = {
  id: number;
  draft: boolean;
  tag_name: string;
  prerelease: boolean;
  body: string | null;
};
export type AssetObject = {
  id: number;
  name: string;
  size: number;
  state: string;
};
export interface ReleaseBackend {
  tagCommit(tag: string): Promise<string | null>;
  createTag(tag: string, commit: string): Promise<void>;
  release(tag: string): Promise<ReleaseObject | null>;
  draft(candidate: Candidate, notes: string): Promise<ReleaseObject>;
  assets(release: number): Promise<AssetObject[]>;
  download(asset: AssetObject, destination: string): Promise<void>;
  upload(release: number, path: string, name: string): Promise<void>;
  removeIncomplete(asset: AssetObject): Promise<void>;
  finalize(release: number): Promise<void>;
}
function releaseObject(
  data: Omit<ReleaseObject, "body"> & { body?: string | null },
): ReleaseObject {
  return {
    id: data.id,
    draft: data.draft,
    tag_name: data.tag_name,
    prerelease: data.prerelease,
    body: data.body ?? null,
  };
}
function missing(error: unknown) {
  return (
    typeof error === "object" &&
    error !== null &&
    "status" in error &&
    error.status === 404
  );
}
function uploadBody(path: string) {
  const source = createReadStream(path),
    iterator = source[Symbol.asyncIterator]();
  return new ReadableStream<Uint8Array>({
    async pull(controller) {
      const item = await iterator.next();
      if (item.done) controller.close();
      else controller.enqueue(new Uint8Array(item.value));
    },
    cancel() {
      source.destroy();
    },
  });
}
export class GithubReleaseBackend implements ReleaseBackend {
  private readonly client: Octokit;
  private readonly repository: string;
  private readonly token: string;
  constructor(client: Octokit, repository: string, token: string) {
    this.client = client;
    this.repository = repository;
    this.token = token;
  }
  async tagCommit(tag: string) {
    const repo = repoParameters(this.repository);
    try {
      let object = (
        await this.client.rest.git.getRef({ ...repo, ref: `tags/${tag}` })
      ).data.object;
      for (let i = 0; object.type === "tag" && i < 8; i++)
        object = (
          await this.client.rest.git.getTag({ ...repo, tag_sha: object.sha })
        ).data.object;
      if (object.type !== "commit")
        throw new Error("Release tag does not resolve to a commit");
      return object.sha;
    } catch (error) {
      if (missing(error)) return null;
      throw error;
    }
  }
  async createTag(tag: string, commit: string) {
    await this.client.rest.git.createRef({
      ...repoParameters(this.repository),
      ref: `refs/tags/${tag}`,
      sha: commit,
    });
  }
  async release(tag: string) {
    try {
      return releaseObject(
        (
          await this.client.rest.repos.getReleaseByTag({
            ...repoParameters(this.repository),
            tag,
          })
        ).data,
      );
    } catch (error) {
      if (missing(error)) return null;
      throw error;
    }
  }
  async draft(candidate: Candidate, notes: string) {
    return releaseObject(
      (
        await this.client.rest.repos.createRelease({
          ...repoParameters(this.repository),
          tag_name: candidate.tag,
          target_commitish: candidate.source_sha,
          name: `Konobangu ${candidate.release_version}`,
          body: `${notes}\n\n<!-- konobangu-candidate:${candidate.candidate_id} -->`,
          draft: true,
          prerelease: candidate.prerelease,
        })
      ).data,
    );
  }
  async assets(release: number) {
    return this.client.paginate(this.client.rest.repos.listReleaseAssets, {
      ...repoParameters(this.repository),
      release_id: release,
      per_page: 100,
    });
  }
  async download(asset: AssetObject, destination: string) {
    await streamDownload(
      `https://api.github.com/repos/${this.repository}/releases/assets/${asset.id}`,
      destination,
      this.token,
    );
  }
  async upload(release: number, path: string, name: string) {
    const init: RequestInit & { duplex: "half" } = {
      method: "POST",
      headers: {
        Authorization: `Bearer ${this.token}`,
        "Content-Type": "application/octet-stream",
        "Content-Length": String(statSync(path).size),
        "X-GitHub-Api-Version": "2022-11-28",
      },
      body: uploadBody(path),
      duplex: "half",
      signal: AbortSignal.timeout(600000),
    };
    const response = await fetch(
      `https://uploads.github.com/repos/${this.repository}/releases/${release}/assets?name=${encodeURIComponent(name)}`,
      init,
    );
    if (!response.ok)
      throw new Error(
        `Release asset upload failed (${response.status}): ${name}`,
      );
    await response.body?.cancel();
  }
  async removeIncomplete(asset: AssetObject) {
    if (asset.state !== "starter" || asset.size !== 0)
      throw new Error("Refusing to remove a completed/conflicting asset");
    await this.client.rest.repos.deleteReleaseAsset({
      ...repoParameters(this.repository),
      asset_id: asset.id,
    });
  }
  async finalize(release: number) {
    await this.client.rest.repos.updateRelease({
      ...repoParameters(this.repository),
      release_id: release,
      draft: false,
    });
  }
}
export function assertManifestMetadata(actual: Metadata, expected: Metadata) {
  if (contentHash(actual) !== contentHash(expected))
    throw new Error(
      "Release manifest metadata snapshot differs from committed source",
    );
}
export function parseManifest(value: unknown): ReleaseManifest {
  const data = jsonRecord(value),
    candidate = parseCandidate(data.candidate);
  if (
    data.schema_version !== 1 ||
    !Array.isArray(data.archives) ||
    !Array.isArray(data.images)
  )
    throw new Error("Invalid release manifest schema");
  const metadata = parseMetadata(stringify(jsonRecord(data.metadata)));
  if (
    metadata.project.version !== candidate.release_version ||
    metadata.project.repository_url !==
      `https://github.com/${candidate.repository}` ||
    JSON.stringify(metadata.release.artifacts) !==
      JSON.stringify(candidate.artifacts)
  )
    throw new Error("Manifest metadata differs from candidate");
  const archives = data.archives.map((value) => {
    const entry = jsonRecord(value);
    if (
      typeof entry.target !== "string" ||
      !metadata.release.targets.some((target) => target === entry.target) ||
      typeof entry.name !== "string" ||
      entry.name !==
        `konobangu-${candidate.release_version}-${entry.target}.tar.gz` ||
      typeof entry.sha256 !== "string" ||
      !/^[0-9a-f]{64}$/.test(entry.sha256) ||
      !Number.isSafeInteger(entry.size) ||
      Number(entry.size) < 1 ||
      !Array.isArray(entry.files)
    )
      throw new Error("Invalid archive receipt");
    const target = metadata.release.targets.find(
      (target) => target === entry.target,
    );
    if (!target) throw new Error("Unknown release target");
    const files = entry.files.map((value) => {
      const file = jsonRecord(value);
      if (
        typeof file.path !== "string" ||
        typeof file.sha256 !== "string" ||
        !/^[0-9a-f]{64}$/.test(file.sha256) ||
        !Number.isSafeInteger(file.size) ||
        Number(file.size) < 0 ||
        ![0o644, 0o755].includes(Number(file.mode))
      )
        throw new Error("Invalid archive file receipt");
      return {
        path: safePath(file.path),
        sha256: file.sha256,
        size: Number(file.size),
        mode: Number(file.mode),
      };
    });
    const paths = files.map((file) => file.path),
      binary = target.includes("windows") ? "recorder-cli.exe" : "recorder-cli";
    const required = [
      binary,
      "webui/index.html",
      "webui/THIRD-PARTY-NOTICES.md",
      "LICENSE",
      "CHANGELOG.md",
      "recorder.config.toml",
      "licenses/THIRD-PARTY-NOTICES.md",
      "release.json",
    ];
    if (
      new Set(paths.map((path) => path.normalize("NFC").toLowerCase())).size !==
        paths.length ||
      required.some((path) => !paths.includes(path)) ||
      paths.some(
        (path) =>
          !required.includes(path) &&
          !path.startsWith("webui/") &&
          !path.startsWith("licenses/"),
      ) ||
      files.some((file) => file.mode !== (file.path === binary ? 0o755 : 0o644))
    )
      throw new Error(
        "Archive file collection is incomplete, conflicting or contains unexpected paths",
      );
    return {
      name: entry.name,
      target,
      sha256: entry.sha256,
      size: Number(entry.size),
      files,
    };
  });
  if (
    new Set(archives.map((entry) => entry.name)).size !== archives.length ||
    (candidate.artifacts.bundles
      ? archives.length !== metadata.release.targets.length
      : archives.length !== 0)
  )
    throw new Error("Selected archive set is incomplete");
  const images = data.images.map((value) =>
    parseImageReceipt(value, candidate),
  );
  for (const kind of ["runtime_image", "testing_torrents_image"] as const)
    if (
      images.filter((image) => image.kind === kind).length !==
      Number(candidate.artifacts[kind])
    )
      throw new Error(`Selected image set is incomplete: ${kind}`);
  return { schema_version: 1, candidate, metadata, archives, images };
}
export function imageName(candidate: Candidate, kind: ImageReceipt["kind"]) {
  return `ghcr.io/${candidate.repository.toLowerCase()}${kind === "testing_torrents_image" ? "-testing-torrents" : ""}`;
}
export function parseImageReceipt(
  value: unknown,
  candidate: Candidate,
): ImageReceipt {
  const data = jsonRecord(value);
  if (
    (data.kind !== "runtime_image" && data.kind !== "testing_torrents_image") ||
    !candidate.artifacts[data.kind] ||
    data.image !== imageName(candidate, data.kind) ||
    typeof data.digest !== "string" ||
    !/^sha256:[0-9a-f]{64}$/.test(data.digest) ||
    data.candidate_id !== candidate.candidate_id ||
    data.source_sha !== candidate.source_sha ||
    data.metadata_sha256 !== candidate.metadata_sha256
  )
    throw new Error("Image receipt does not match candidate");
  return {
    kind: data.kind,
    image: data.image,
    digest: data.digest,
    candidate_id: candidate.candidate_id,
    source_sha: candidate.source_sha,
    metadata_sha256: candidate.metadata_sha256,
  };
}
export async function reserveVersion(
  backend: ReleaseBackend,
  candidate: Candidate,
  output: string,
  notes: string,
) {
  if (!Object.values(candidate.artifacts).some(Boolean))
    throw new Error("No release artifacts are selected");
  mkdirSync(output, { recursive: true });
  const tag = await backend.tagCommit(candidate.tag);
  if (tag && tag !== candidate.source_sha)
    throw new Error("Version tag belongs to a different source");
  if (!tag) {
    try {
      await backend.createTag(candidate.tag, candidate.source_sha);
    } catch (error) {
      if ((await backend.tagCommit(candidate.tag)) !== candidate.source_sha)
        throw error;
    }
  }
  const release =
    (await backend.release(candidate.tag)) ??
    (await backend.draft(candidate, notes));
  if (release.tag_name !== candidate.tag)
    throw new Error("Release tag differs from candidate");
  if (
    !release.body?.includes(
      `<!-- konobangu-candidate:${candidate.candidate_id} -->`,
    )
  )
    throw new Error("Draft release belongs to a different candidate");
  if (release.prerelease !== candidate.prerelease)
    throw new Error("Release prerelease policy differs");
  const assets = await backend.assets(release.id);
  let existing = assets.find((asset) => asset.name === "candidate.json");
  if (
    existing &&
    release.draft &&
    existing.state === "starter" &&
    existing.size === 0
  ) {
    await backend.removeIncomplete(existing);
    existing = undefined;
  }
  if (existing) {
    const downloaded = join(output, "reserved-candidate.json");
    rmSync(downloaded, { force: true });
    await backend.download(existing, downloaded);
    if (
      parseCandidate(readJson(downloaded)).candidate_id !==
      candidate.candidate_id
    )
      throw new Error("Version is reserved by a different candidate");
  } else {
    if (
      !release.draft ||
      assets.some((asset) => asset.name !== "candidate.json")
    )
      throw new Error(
        "Existing release has no verifiable candidate reservation",
      );
    const path = join(output, "candidate.json");
    writeJson(path, candidate);
    await backend.upload(release.id, path, "candidate.json");
    const reservation = (await backend.assets(release.id)).find(
      (asset) => asset.name === "candidate.json",
    );
    if (!reservation)
      throw new Error("Candidate reservation upload is missing");
    const downloaded = join(output, "reserved-candidate.json");
    rmSync(downloaded, { force: true });
    await backend.download(reservation, downloaded);
    if (
      parseCandidate(readJson(downloaded)).candidate_id !==
      candidate.candidate_id
    )
      throw new Error("Candidate reservation upload failed verification");
  }
  return release;
}
async function verifiedAsset(
  backend: ReleaseBackend,
  asset: AssetObject,
  output: string,
  expected: string,
) {
  const path = join(output, `download-${asset.name}`);
  rmSync(path, { force: true });
  await backend.download(asset, path);
  if ((await sha256Stream(path)) !== expected)
    throw new Error(`Published asset hash mismatch: ${asset.name}`);
  rmSync(path);
}
export async function publishManifest(
  backend: ReleaseBackend,
  manifest: ReleaseManifest,
  directory: string,
  verifyImage: (image: ImageReceipt) => Promise<void>,
) {
  const candidate = manifest.candidate,
    release = await reserveVersion(backend, candidate, directory, "");
  if (!release.draft) {
    const published = await readPublishedManifest(
      backend,
      candidate,
      directory,
      verifyImage,
      manifest.metadata,
    );
    if (contentHash(published) !== contentHash(manifest))
      throw new Error("Published release is immutable");
    return { release_id: release.id, already_public: true };
  }
  for (const image of manifest.images) await verifyImage(image);
  writeJson(join(directory, "release-manifest.json"), manifest);
  writeFileSync(join(directory, "SHA256SUMS"), manifestChecksums(manifest));
  const names = [
    ...manifest.archives.map((archive) => archive.name),
    "release-manifest.json",
    "SHA256SUMS",
  ];
  for (const name of names) {
    const path = join(directory, name);
    if (!existsSync(path)) throw new Error(`Prepared asset missing: ${name}`);
    const expected = await sha256Stream(path),
      archive = manifest.archives.find((entry) => entry.name === name);
    if (
      archive &&
      (expected !== archive.sha256 || statSync(path).size !== archive.size)
    )
      throw new Error(`Prepared archive differs from its manifest: ${name}`);
    const existing = (await backend.assets(release.id)).find(
      (asset) => asset.name === name,
    );
    if (existing && existing.state === "starter" && existing.size === 0)
      await backend.removeIncomplete(existing);
    else if (existing) {
      await verifiedAsset(backend, existing, directory, expected);
      continue;
    }
    await backend.upload(release.id, path, name);
    const uploaded = (await backend.assets(release.id)).find(
      (asset) => asset.name === name,
    );
    if (!uploaded) throw new Error(`Uploaded asset missing: ${name}`);
    await verifiedAsset(backend, uploaded, directory, expected);
  }
  const actual = (await backend.assets(release.id))
    .map((asset) => asset.name)
    .sort();
  if (
    JSON.stringify(actual) !==
    JSON.stringify([...names, "candidate.json"].sort())
  )
    throw new Error("Release contains unexpected assets");
  await backend.finalize(release.id);
  return { release_id: release.id, already_public: false };
}
export async function readPublishedManifest(
  backend: ReleaseBackend,
  candidate: Candidate,
  output: string,
  verifyImage: (image: ImageReceipt) => Promise<void>,
  expectedMetadata: Metadata,
) {
  const release = await backend.release(candidate.tag);
  if (
    !release ||
    release.draft ||
    (await backend.tagCommit(candidate.tag)) !== candidate.source_sha
  )
    throw new Error("Public release is unavailable or has conflicting tag");
  mkdirSync(output, { recursive: true });
  const assets = await backend.assets(release.id),
    asset = assets.find((asset) => asset.name === "release-manifest.json");
  if (!asset) throw new Error("Public release manifest is missing");
  const path = join(output, "published-manifest.json");
  rmSync(path, { force: true });
  await backend.download(asset, path);
  const manifest = parseManifest(readJson(path));
  assertManifestMetadata(manifest.metadata, expectedMetadata);
  if (manifest.candidate.candidate_id !== candidate.candidate_id)
    throw new Error("Public version belongs to a different candidate");
  const expectedNames = [
    ...manifest.archives.map((archive) => archive.name),
    "candidate.json",
    "release-manifest.json",
    "SHA256SUMS",
  ].sort();
  if (
    JSON.stringify(assets.map((asset) => asset.name).sort()) !==
    JSON.stringify(expectedNames)
  )
    throw new Error("Public release asset set mismatch");
  for (const archive of manifest.archives) {
    const found = assets.find((asset) => asset.name === archive.name);
    if (!found || found.size !== archive.size)
      throw new Error("Public archive is missing or has wrong size");
    await verifiedAsset(backend, found, output, archive.sha256);
  }
  for (const name of ["candidate.json", "SHA256SUMS"]) {
    const found = assets.find((asset) => asset.name === name);
    if (!found) throw new Error(`Public receipt missing: ${name}`);
    const path = join(output, `public-${name}`);
    rmSync(path, { force: true });
    await backend.download(found, path);
    if (
      name === "candidate.json"
        ? parseCandidate(readJson(path)).candidate_id !== candidate.candidate_id
        : readFileSync(path, "utf8") !== manifestChecksums(manifest)
    )
      throw new Error(`Public receipt mismatch: ${name}`);
  }
  for (const image of manifest.images) await verifyImage(image);
  return manifest;
}
export function mayPromoteAlias(
  version: string,
  previousVersion: string | null,
): boolean {
  if (semver.prerelease(version) !== null) return false;
  if (previousVersion === null) return true;
  if (!semver.valid(previousVersion))
    throw new Error("Existing stable alias has no valid version label");
  return semver.gte(version, previousVersion);
}

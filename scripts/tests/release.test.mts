import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  mkdirSync,
  mkdtempDisposableSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { Octokit } from "@octokit/rest";
import { zipSync } from "fflate";
import * as tar from "tar";
import {
  releaseMetadataSnapshot,
  releasePlanData,
} from "../commands/release.mts";
import { sha256Stream } from "../lib/artifacts.mts";
import { root } from "../lib/process.mts";
import {
  assembleBundles,
  type ImageReceipt,
  type ReleaseManifest,
} from "../lib/release-bundle.mts";
import {
  assertReleaseApproval,
  type Candidate,
  candidateFrom,
  candidatePlanFromZip,
  validateVerifiedRun,
  verifyArtifactZip,
  waitForVerifiedRun,
} from "../lib/release-candidate.mts";
import { stageInput, verifyInput } from "../lib/release-inputs.mts";
import {
  loadMetadata,
  projectPath,
  releaseNotes,
  releaseVersion,
  replaceTomlVersion,
  versionEdits,
} from "../lib/release-metadata.mts";
import {
  type AssetObject,
  assertManifestMetadata,
  mayPromoteAlias,
  parseManifest,
  publishManifest,
  type ReleaseBackend,
  type ReleaseObject,
  readPublishedManifest,
  reserveVersion,
} from "../lib/release-publisher.mts";
import {
  GhcrRegistry,
  imagePreflight,
  promoteAliases,
  type Registry,
  type RegistryImage,
  verifyPublicRegistryReceipt,
  verifyRegistryReceipt,
} from "../lib/release-registry.mts";

test("release versions reject ranges and preserve TOML dependency versions", () => {
  for (const value of ["v0.2.0", "^0.2.0", "0.2", "01.2.3", "not-a-version"])
    assert.throws(() => releaseVersion(value));
  assert.equal(releaseVersion("0.2.0-rc.1"), "0.2.0-rc.1");
  const source =
    '[package]\nname = "app"\nversion = "0.1.0"\n\n[dependencies.library]\nversion = "1.0"\n';
  assert.equal(
    replaceTomlVersion(source, "package", "0.2.0"),
    '[package]\nname = "app"\nversion = "0.2.0"\n\n[dependencies.library]\nversion = "1.0"\n',
  );
});
test("metadata plans validate all packages before modifying files", () => {
  using temporary = mkdtempDisposableSync(join(root, "temp/release-fixture-"));
  const directory = temporary.path;

  const metadata = loadMetadata();
  writeFileSync(
    join(directory, "konobangu-metadata.toml"),
    readFileSync(join(root, "konobangu-metadata.toml")),
  );
  for (const entry of [
    ...metadata.node_package,
    ...metadata.rust_package,
    ...metadata.python_package,
  ]) {
    const path = join(directory, entry.manifest);
    mkdirSync(join(path, ".."), { recursive: true });
    writeFileSync(path, readFileSync(join(root, entry.manifest)));
  }
  const edits = versionEdits(directory, "0.2.0");
  assert.equal(
    edits.length,
    metadata.node_package.length +
      metadata.rust_package.length +
      metadata.python_package.length +
      1,
  );
  assert.equal(
    loadMetadata(directory).project.version,
    metadata.project.version,
  );
  writeFileSync(
    join(directory, metadata.rust_package[0].manifest),
    '[package]\nname = "wrong"\nversion = "0.1.0"\n',
  );
  assert.throws(() => versionEdits(directory, "0.2.0"), /name mismatch/);
  assert.equal(
    loadMetadata(directory).project.version,
    metadata.project.version,
  );
  assert.throws(() => projectPath(directory, "../outside"));
});

test("release plans preserve checkout newlines and still detect metadata drift", () => {
  const metadata = loadMetadata();
  const manifests = [
    "konobangu-metadata.toml",
    ...[
      ...metadata.node_package,
      ...metadata.rust_package,
      ...metadata.python_package,
    ].map((entry) => entry.manifest),
  ];
  for (const newline of ["\n", "\r\n"]) {
    using temporary = mkdtempDisposableSync(
      join(root, "temp/newline-fixture-"),
    );
    for (const manifest of manifests) {
      const path = join(temporary.path, manifest);
      mkdirSync(join(path, ".."), { recursive: true });
      writeFileSync(
        path,
        readFileSync(join(root, manifest), "utf8")
          .replaceAll("\r\n", "\n")
          .replaceAll("\n", newline),
      );
    }
    for (const edit of versionEdits(temporary.path, metadata.project.version))
      assert.equal(edit.next, edit.source, edit.path);
    for (const edit of versionEdits(temporary.path, "0.2.0")) {
      assert.equal(
        edit.next,
        edit.next.replaceAll("\r\n", "\n").replaceAll("\n", newline),
        edit.path,
      );
      writeFileSync(edit.path, edit.next);
    }
    assert.equal(loadMetadata(temporary.path).project.version, "0.2.0");
    for (const edit of versionEdits(temporary.path, "0.2.0"))
      assert.equal(edit.next, edit.source, edit.path);

    const manifest = metadata.node_package.find((entry) => entry.versioned);
    assert(manifest);
    const path = join(temporary.path, manifest.manifest);
    const source = readFileSync(path, "utf8");
    writeFileSync(path, source.replace('"0.2.0"', '"0.1.0"'));
    assert.deepEqual(
      versionEdits(temporary.path, "0.2.0")
        .filter((edit) => edit.source !== edit.next)
        .map((edit) => edit.path),
      [path],
    );
  }
});

test("publication selections are independent and empty selections produce no images", () => {
  const metadata = loadMetadata();
  const source = "a".repeat(40);
  for (let mask = 0; mask < 8; mask++) {
    const artifacts = {
      bundles: Boolean(mask & 1),
      runtime_image: Boolean(mask & 2),
      testing_torrents_image: Boolean(mask & 4),
    };
    const plan = releasePlanData(
      { ...metadata, release: { ...metadata.release, artifacts } },
      source,
    );
    assert.deepEqual(plan.artifacts, artifacts);
    assert.equal(plan.has_artifacts, mask !== 0);
    for (const key of ["runtime_image", "testing_torrents_image"] as const)
      assert.equal(Boolean(plan.images[key]), artifacts[key]);
  }
  for (const version of ["0.1.0", "0.1.0-rc.1", "0.1.0+build-alpha"]) {
    const plan = releasePlanData(
      {
        ...metadata,
        project: { ...metadata.project, version },
        release: {
          ...metadata.release,
          artifacts: {
            bundles: false,
            runtime_image: false,
            testing_torrents_image: true,
          },
        },
      },
      source,
    );
    const image = plan.images.testing_torrents_image;
    assert(image);
    assert(
      image.tags.includes(`${image.image}:${version.replaceAll("+", "_")}`),
    );
    assert(image.tags.includes(`${image.image}:sha-${source}`));
    assert(!image.tags.includes(`${image.image}:latest`));
    assert.equal(
      image.latest_tag,
      version !== "0.1.0-rc.1" ? `${image.image}:latest` : null,
    );
  }
});

test("metadata rejects missing, mistyped and unknown publication selections", () => {
  using temporary = mkdtempDisposableSync(
    join(root, "temp/selection-fixture-"),
  );
  const path = join(temporary.path, "konobangu-metadata.toml");
  const source = readFileSync(join(root, "konobangu-metadata.toml"), "utf8");
  for (const invalid of [
    source.replace(/^bundles\s*=\s*(true|false)/m, 'bundles = "true"'),
    source.replace(/^bundles\s*=\s*(true|false)\r?\n/m, ""),
    source.replace(
      "[release.artifacts]",
      "[release.artifacts]\nbundle = false",
    ),
  ]) {
    writeFileSync(path, invalid);
    assert.throws(() => loadMetadata(temporary.path), /release\.artifacts/);
  }
});

test("metadata snapshots survive the receipt JSON round trip", () => {
  const snapshot = releaseMetadataSnapshot();
  assert.deepEqual(JSON.parse(JSON.stringify(snapshot)), snapshot);
});

test("checkout line endings do not change release metadata identity", () => {
  using temporary = mkdtempDisposableSync(join(root, "temp/snapshot-newline-"));
  const path = join(temporary.path, "konobangu-metadata.toml");
  const source = readFileSync(
    join(root, "konobangu-metadata.toml"),
    "utf8",
  ).replaceAll("\r\n", "\n");
  writeFileSync(path, source);
  const snapshot = releaseMetadataSnapshot(temporary.path);
  writeFileSync(path, source.replaceAll("\n", "\r\n"));
  assert.deepEqual(releaseMetadataSnapshot(temporary.path), snapshot);
});

test("release notes require the exact version rather than a prerelease prefix", () => {
  for (const newline of ["\n", "\r\n"]) {
    assert.throws(
      () => releaseNotes(["## 0.2.0-rc.1", "preview"].join(newline), "0.2.0"),
      /exact versioned/,
    );
    assert.equal(
      releaseNotes(["## 0.2.0", "released", "## 0.1.0"].join(newline), "0.2.0"),
      "released",
    );
    assert.equal(
      releaseNotes(
        [
          "## Unreleased",
          "next",
          "## 0.2.0 - 2026-10-05",
          "released",
          "## 0.1.0",
          "old",
        ].join(newline),
        "0.2.0",
      ),
      "released",
    );
  }
});

const source = {
  repository: "apeiraco/konobangu",
  source_sha: "a".repeat(40),
  tests_run_id: 123,
  tests_run_attempt: 2,
  release_version: "0.1.0",
  metadata_sha256: "b".repeat(64),
};
function makeCandidate(mask = 1) {
  const metadata = loadMetadata();
  metadata.release.artifacts = {
    bundles: Boolean(mask & 1),
    runtime_image: Boolean(mask & 2),
    testing_torrents_image: Boolean(mask & 4),
  };
  const inputs =
    mask & 3
      ? [
          {
            id: 1,
            name: "webui-release-input",
            digest: `sha256:${"c".repeat(64)}`,
          },
          ...(mask & 1
            ? metadata.release.targets
            : ["x86_64-unknown-linux-musl"]
          ).map((target, index) => ({
            id: index + 2,
            name: `recorder-release-input-${target}`,
            digest: `sha256:${"d".repeat(64)}`,
          })),
        ]
      : [];
  return { candidate: candidateFrom(source, metadata, inputs), metadata };
}
function imageReceipt(
  candidate: Candidate,
  kind: ImageReceipt["kind"],
): ImageReceipt {
  return {
    kind,
    image: `ghcr.io/apeiraco/konobangu${kind === "testing_torrents_image" ? "-testing-torrents" : ""}`,
    digest: `sha256:${"e".repeat(64)}`,
    candidate_id: candidate.candidate_id,
    source_sha: candidate.source_sha,
    metadata_sha256: candidate.metadata_sha256,
  };
}
class MemoryReleases implements ReleaseBackend {
  commit: string | null = null;
  object: ReleaseObject | null = null;
  stored = new Map<string, { asset: AssetObject; bytes: Buffer }>();
  uploads = 0;
  failName: string | null = null;
  async tagCommit() {
    return this.commit;
  }
  async createTag(_tag: string, commit: string) {
    this.commit = commit;
  }
  async release() {
    return this.object;
  }
  async draft(candidate: Candidate) {
    this.object = {
      id: 42,
      draft: true,
      tag_name: candidate.tag,
      prerelease: candidate.prerelease,
      body: `<!-- konobangu-candidate:${candidate.candidate_id} -->`,
    };
    return this.object;
  }
  async assets() {
    return [...this.stored.values()].map((entry) => entry.asset);
  }
  async download(asset: AssetObject, destination: string) {
    const entry = this.stored.get(asset.name);
    assert(entry);
    writeFileSync(destination, entry.bytes);
  }
  async upload(_id: number, path: string, name: string) {
    this.uploads++;
    if (name === this.failName) {
      this.failName = null;
      throw new Error("interrupted upload");
    }
    const bytes = readFileSync(path);
    this.stored.set(name, {
      asset: { id: this.uploads, name, size: bytes.length, state: "uploaded" },
      bytes,
    });
  }
  async removeIncomplete(asset: AssetObject) {
    this.stored.delete(asset.name);
  }
  async finalize() {
    assert(this.object);
    this.object.draft = false;
  }
}
class MemoryRegistry implements Registry {
  records = new Map<string, RegistryImage>();
  promoted: string[] = [];
  fail = false;
  failImage: string | null = null;
  async inspect(image: string, ref: string) {
    return this.records.get(`${image}:${ref}`) ?? null;
  }
  async alias(image: string, digest: string, tag: string) {
    if (this.fail || this.failImage === image) {
      this.failImage = null;
      this.fail = false;
      throw new Error("alias failure");
    }
    const found = this.records.get(`${image}:${digest}`);
    assert(found);
    this.records.set(`${image}:${tag}`, found);
    this.promoted.push(image);
  }
  add(candidate: Candidate, receipt: ImageReceipt) {
    const record: RegistryImage = {
      digest: receipt.digest,
      variants: [],
      labels: {
        "org.opencontainers.image.source": `https://github.com/${candidate.repository}`,
        "org.opencontainers.image.licenses": "MIT",
        "org.opencontainers.image.revision": candidate.source_sha,
        "org.opencontainers.image.version": candidate.release_version,
        "io.apeiraco.konobangu.candidate-id": candidate.candidate_id,
        "io.apeiraco.konobangu.release-metadata-sha256":
          candidate.metadata_sha256,
      },
    };
    record.variants = (
      receipt.kind === "runtime_image" ? ["amd64"] : ["amd64", "arm64"]
    ).map((architecture) => ({
      os: "linux",
      architecture,
      digest: receipt.digest,
      labels: record.labels,
    }));
    for (const ref of [
      candidate.release_version.replaceAll("+", "_"),
      `sha-${candidate.source_sha}`,
      receipt.digest,
    ])
      this.records.set(`${receipt.image}:${ref}`, record);
  }
}

test("release inputs validate file sets, hashes, source identity and symlink boundaries", async () => {
  using temporary = mkdtempDisposableSync(join(root, "temp/input-空间-"));
  const webui = join(temporary.path, "input");
  mkdirSync(webui);
  writeFileSync(join(webui, "index.html"), "ok");
  writeFileSync(
    join(webui, "THIRD-PARTY-NOTICES.md"),
    "bundled dependency license",
  );
  const output = join(temporary.path, "receipt");
  await stageInput({
    output,
    source,
    kind: "webui",
    target: null,
    input: webui,
    checks: { passed: true },
  });
  await verifyInput(output, source);
  await assert.rejects(
    verifyInput(output, { ...source, tests_run_attempt: 3 }),
    /identity/,
  );
  writeFileSync(join(output, "unlisted"), "unexpected");
  await assert.rejects(verifyInput(output, source), /collection/);
  rmSync(join(output, "unlisted"));
  writeFileSync(join(output, "webui/index.html"), "changed");
  await assert.rejects(verifyInput(output, source), /collection/);
  if (process.platform !== "win32") {
    rmSync(join(webui, "index.html"));
    symlinkSync("../outside", join(webui, "index.html"));
    await assert.rejects(
      stageInput({
        output,
        source,
        kind: "webui",
        target: null,
        input: webui,
        checks: { passed: true },
      }),
      /symlink/,
    );
  }
});

test("all four bundles use identical deterministic tar encoding including Windows executable mode", async () => {
  using temporary = mkdtempDisposableSync(join(root, "temp/archive 空格-"));
  const { candidate, metadata } = makeCandidate(),
    directory = temporary.path,
    inputs = join(directory, "inputs");
  const webui = join(directory, "dist");
  mkdirSync(webui);
  writeFileSync(join(webui, "index.html"), "webui");
  writeFileSync(
    join(webui, "THIRD-PARTY-NOTICES.md"),
    "bundled dependency license",
  );
  await stageInput({
    output: join(inputs, "webui-release-input"),
    source,
    kind: "webui",
    target: null,
    input: webui,
    checks: { passed: true },
  });
  for (const target of metadata.release.targets) {
    const binary = join(
      directory,
      target.includes("windows") ? "recorder-cli.exe" : "recorder-cli",
    );
    writeFileSync(binary, target);
    const licenses = join(directory, "licenses");
    mkdirSync(licenses, { recursive: true });
    writeFileSync(
      join(licenses, "THIRD-PARTY-NOTICES.md"),
      "actual target notices",
    );
    await stageInput({
      output: join(inputs, `recorder-release-input-${target}`),
      source,
      kind: "recorder",
      target,
      input: binary,
      notices: licenses,
      checks: { passed: true },
    });
  }
  const first = await assembleBundles({
    candidate,
    metadata,
    inputs,
    output: join(directory, "first"),
    base: root,
  });
  const second = await assembleBundles({
    candidate,
    metadata,
    inputs,
    output: join(directory, "second"),
    base: root,
  });
  assert.deepEqual(first, second);
  assert.equal(first.length, 4);
  for (const archive of first) {
    const modes: Record<string, number> = {};
    await tar.list({
      file: join(directory, "first", archive.name),
      onReadEntry(entry) {
        modes[entry.path] = entry.mode ?? 0;
        assert.equal(entry.mtime?.getTime(), 0);
        assert.equal(entry.uid, 0);
        assert.equal(entry.gid, 0);
      },
    });
    assert.equal(
      modes[
        archive.target.includes("windows") ? "recorder-cli.exe" : "recorder-cli"
      ],
      0o755,
    );
    assert.equal(modes["webui/"], 0o755);
    assert.equal(readFileSync(join(directory, "first", archive.name))[9], 255);
    assert.equal(modes["recorder.config.toml"], 0o644);
    assert.equal(modes["webui/index.html"], 0o644);
    assert.equal(
      await sha256Stream(join(directory, "first", archive.name)),
      archive.sha256,
    );
  }
});

test("publisher rejects collisions, resumes interrupted assets and verifies immutable public retries", async () => {
  using temporary = mkdtempDisposableSync(join(root, "temp/publisher-"));
  const { candidate, metadata } = makeCandidate(4),
    manifest: ReleaseManifest = {
      schema_version: 1,
      candidate,
      metadata,
      archives: [],
      images: [imageReceipt(candidate, "testing_torrents_image")],
    };
  const backend = new MemoryReleases(),
    verify = async () => {};
  backend.commit = "f".repeat(40);
  await assert.rejects(
    reserveVersion(backend, candidate, temporary.path, "notes"),
    /different source/,
  );
  backend.commit = null;
  await reserveVersion(backend, candidate, temporary.path, "notes");
  const changed = candidateFrom(
    { ...source, tests_run_attempt: 3 },
    metadata,
    [],
  );
  await assert.rejects(
    reserveVersion(backend, changed, temporary.path, "notes"),
    /different candidate/,
  );
  backend.failName = "SHA256SUMS";
  await assert.rejects(
    publishManifest(backend, manifest, temporary.path, verify),
    /interrupted/,
  );
  assert.equal(backend.object?.draft, true);
  await publishManifest(backend, manifest, temporary.path, verify);
  assert.equal(backend.object?.draft, false);
  const uploads = backend.uploads;
  assert.deepEqual(
    await readPublishedManifest(
      backend,
      candidate,
      temporary.path,
      verify,
      metadata,
    ),
    manifest,
  );
  await publishManifest(backend, manifest, temporary.path, verify);
  assert.equal(backend.uploads, uploads);
  const corrupted = backend.stored.get("SHA256SUMS");
  assert(corrupted);
  corrupted.bytes = Buffer.from("tampered");
  await assert.rejects(
    readPublishedManifest(backend, candidate, temporary.path, verify, metadata),
    /mismatch/,
  );
});

test("every metadata selection produces exactly the selected manifest outputs", () => {
  for (let mask = 1; mask < 8; mask++) {
    const { candidate, metadata } = makeCandidate(mask);
    const archives = candidate.artifacts.bundles
      ? metadata.release.targets.map((target) => ({
          target,
          name: `konobangu-0.1.0-${target}.tar.gz`,
          sha256: "f".repeat(64),
          size: 1,
          files: [
            target.includes("windows") ? "recorder-cli.exe" : "recorder-cli",
            "webui/index.html",
            "webui/THIRD-PARTY-NOTICES.md",
            "LICENSE",
            "CHANGELOG.md",
            "recorder.config.toml",
            "licenses/THIRD-PARTY-NOTICES.md",
            "release.json",
          ].map((path) => ({
            path,
            sha256: "f".repeat(64),
            size: 1,
            mode: path.startsWith("recorder-cli") ? 0o755 : 0o644,
          })),
        }))
      : [];
    const images = (["runtime_image", "testing_torrents_image"] as const)
      .filter((kind) => candidate.artifacts[kind])
      .map((kind) => imageReceipt(candidate, kind));
    const manifest = parseManifest({
      schema_version: 1,
      candidate,
      metadata,
      archives,
      images,
    });
    assert.equal(manifest.archives.length, candidate.artifacts.bundles ? 4 : 0);
    assert.equal(
      manifest.images.length,
      Number(Boolean(mask & 2)) + Number(Boolean(mask & 4)),
    );
    if (images.length)
      assert.throws(
        () => parseManifest({ ...manifest, images: [] }),
        /incomplete/,
      );
    if (archives.length)
      assert.throws(
        () => parseManifest({ ...manifest, archives: archives.slice(1) }),
        /incomplete/,
      );
  }
});

test("registry preflight rejects foreign labels, and alias retries do not roll newer stable versions back", async () => {
  const { candidate } = makeCandidate(6),
    runtimeImage = imageReceipt(candidate, "runtime_image"),
    fixture = imageReceipt(candidate, "testing_torrents_image");
  const registry = new MemoryRegistry();
  registry.add(candidate, runtimeImage);
  registry.add(candidate, fixture);
  assert.equal(
    (await imagePreflight(registry, candidate, "runtime_image"))?.digest,
    runtimeImage.digest,
  );
  registry.fail = true;
  await assert.rejects(
    promoteAliases(registry, registry, candidate, [runtimeImage, fixture]),
    /alias failure/,
  );
  assert.equal(
    (
      await promoteAliases(registry, registry, candidate, [
        runtimeImage,
        fixture,
      ])
    ).length,
    2,
  );
  assert.equal(
    (
      await promoteAliases(registry, registry, candidate, [
        runtimeImage,
        fixture,
      ])
    )[0]?.state,
    "already-current",
  );
  registry.records.set(`${runtimeImage.image}:latest`, {
    digest: `sha256:${"f".repeat(64)}`,
    labels: { "org.opencontainers.image.version": "0.2.0" },
    variants: [],
  });
  assert.equal(
    (await promoteAliases(registry, registry, candidate, [runtimeImage]))[0]
      ?.state,
    "newer-version",
  );
  const foreign = registry.records.get(`${runtimeImage.image}:0.1.0`);
  assert(foreign);
  foreign.labels["org.opencontainers.image.source"] =
    "https://github.com/foreign/repository";
  await assert.rejects(
    imagePreflight(registry, candidate, "runtime_image"),
    /different candidate/,
  );
  foreign.labels["org.opencontainers.image.source"] =
    `https://github.com/${candidate.repository}`;
  foreign.labels["io.apeiraco.konobangu.candidate-id"] = "wrong";
  await assert.rejects(
    imagePreflight(registry, candidate, "runtime_image"),
    /different candidate/,
  );
  assert.equal(mayPromoteAlias("0.1.0-rc.1", null), false);
  assert.equal(mayPromoteAlias("0.1.0+build.1", "0.1.0"), true);
});

test("source verification binds workflow, repository, event, SHA, attempt and actual successful jobs", async () => {
  let altered: Record<string, unknown> = {};
  const run = {
    repository: { full_name: source.repository },
    head_repository: { full_name: source.repository },
    workflow_id: 7,
    status: "completed",
    conclusion: "success",
    event: "push",
    head_branch: "release",
    head_sha: source.source_sha,
    run_attempt: 2,
  };
  const client = new Octokit({
    auth: "fake-token",
    request: {
      fetch: async (input: string | URL | Request) => {
        const url = String(input),
          data = url.includes("/jobs")
            ? {
                total_count: 8,
                jobs: [
                  "Static contracts and tooling",
                  "TypeScript and WebUI",
                  "Rust workspace and browser integration",
                  ...loadMetadata().release.targets.map(
                    (target) => `Artifacts (${target})`,
                  ),
                  "Verification report",
                ].map((name) => ({
                  name,
                  status: "completed",
                  conclusion: "success",
                })),
              }
            : url.includes("/workflows/")
              ? { id: 7, path: ".github/workflows/verification.yaml" }
              : { ...run, ...altered };
        const response = new Response(JSON.stringify(data), {
          status: 200,
          headers: { "Content-Type": "application/json" },
        });
        Object.defineProperty(response, "url", { value: url });
        return response;
      },
    },
  });
  await validateVerifiedRun(client, source);
  for (const change of [
    { conclusion: "failure" },
    { event: "pull_request" },
    { head_branch: "master" },
    { head_sha: "e".repeat(40) },
    { run_attempt: 3 },
    { workflow_id: 8 },
    { head_repository: { full_name: "foreign/repository" } },
  ]) {
    altered = change;
    await assert.rejects(
      validateVerifiedRun(client, source),
      /identity\/status/,
    );
  }
});

test("verification completion waits for dispatch to finish and retains a bounded failure", async () => {
  let calls = 0,
    now = 0;
  const client = new Octokit({
    auth: "fake",
    request: {
      fetch: async (input: string | URL | Request) => {
        calls++;
        const response = new Response(
          JSON.stringify({
            head_sha: source.source_sha,
            head_branch: "release",
            event: "push",
            status: calls > 1 ? "completed" : "in_progress",
            conclusion: calls > 1 ? "success" : null,
          }),
          { headers: { "Content-Type": "application/json" } },
        );
        Object.defineProperty(response, "url", { value: String(input) });
        return response;
      },
    },
  });
  await waitForVerifiedRun(client, source, {
    now: () => now,
    sleep: async (ms) => {
      now += ms;
    },
  });
  assert.equal(calls, 2);
  assert.equal(now, 5000);
  calls = -1000;
  now = 0;
  await assert.rejects(
    waitForVerifiedRun(client, source, {
      now: () => now,
      sleep: async (ms) => {
        now += ms;
      },
    }),
    /five minutes/,
  );
});

test("candidate ZIP rejects multiple entries, links, path escapes and expansion bombs before extraction", () => {
  const { candidate } = makeCandidate(4),
    bytes = new TextEncoder().encode(JSON.stringify(candidate));
  assert.deepEqual(
    candidatePlanFromZip(zipSync({ "candidate.json": bytes })),
    candidate,
  );
  for (const archive of [
    zipSync({ "candidate.json": bytes, "evil.json": bytes }),
    zipSync({ "../candidate.json": bytes }),
    zipSync({ "candidate.json": new Uint8Array(256 * 1024 + 1) }),
    zipSync({ "candidate.json": [bytes, { os: 3, attrs: 0o120777 << 16 }] }),
  ])
    assert.throws(() => candidatePlanFromZip(archive));
});

test("empty starter reservation resumes, while a completed foreign reservation remains immutable", async () => {
  using temporary = mkdtempDisposableSync(join(root, "temp/reserve-starter-"));
  const { candidate } = makeCandidate(4),
    backend = new MemoryReleases();
  backend.commit = candidate.source_sha;
  backend.object = {
    id: 42,
    draft: true,
    tag_name: candidate.tag,
    prerelease: false,
    body: `<!-- konobangu-candidate:${candidate.candidate_id} -->`,
  };
  backend.stored.set("candidate.json", {
    asset: { id: 1, name: "candidate.json", size: 0, state: "starter" },
    bytes: Buffer.alloc(0),
  });
  await reserveVersion(backend, candidate, temporary.path, "notes");
  assert.equal(backend.stored.get("candidate.json")?.asset.state, "uploaded");
  const entry = backend.stored.get("candidate.json");
  assert(entry);
  entry.bytes = Buffer.from("{}");
  await assert.rejects(
    reserveVersion(backend, candidate, temporary.path, "notes"),
  );
  assert.equal(backend.stored.get("candidate.json"), entry);
});

test("caller plan authentication checks actual ZIP digest and candidate bytes", async () => {
  const { candidate } = makeCandidate(4),
    bytes = zipSync({
      "candidate.json": new TextEncoder().encode(JSON.stringify(candidate)),
    });
  let archive = bytes;
  const digest = `sha256:${createHash("sha256").update(bytes).digest("hex")}`;
  const client = new Octokit({
    auth: "fake",
    request: {
      fetch: async (input: string | URL | Request) => {
        const url = String(input);
        const data = url.endsWith("/approvals")
          ? [{ state: "approved", environments: [{ name: "stable-release" }] }]
          : url.includes("/jobs")
            ? {
                total_count: 1,
                jobs: [
                  { name: "Stable release approval", conclusion: "success" },
                ],
              }
            : url.includes("/artifacts")
              ? {
                  total_count: 1,
                  artifacts: [
                    {
                      id: 9,
                      name: "release-candidate-plan",
                      expired: false,
                      digest,
                    },
                  ],
                }
              : url.includes("/workflows/")
                ? { id: 8, path: ".github/workflows/release.yaml" }
                : {
                    repository: { full_name: source.repository },
                    workflow_id: 8,
                    event: "workflow_dispatch",
                    head_branch: "release",
                  };
        const response = new Response(JSON.stringify(data), {
          headers: { "Content-Type": "application/json" },
        });
        Object.defineProperty(response, "url", { value: url });
        return response;
      },
    },
  });
  const binary = {
    token: "fake",
    download: async (_url: string, path: string) => {
      writeFileSync(path, archive);
    },
  };
  await assertReleaseApproval(client, candidate, 456, binary);
  archive = zipSync({ "candidate.json": new TextEncoder().encode("{}") });
  await assert.rejects(
    assertReleaseApproval(client, candidate, 456, binary),
    /ZIP digest mismatch/,
  );
});

test("immutable input artifact ZIP digest mismatches fail before official extraction", async () => {
  using temporary = mkdtempDisposableSync(join(root, "temp/input-digest-"));
  const { candidate } = makeCandidate(),
    input = candidate.input_artifacts[0];
  assert(input);
  const client = new Octokit({
    auth: "fake",
    request: {
      fetch: async (url: string | URL | Request) => {
        const response = new Response(
          JSON.stringify({
            id: input.id,
            name: input.name,
            expired: false,
            digest: input.digest,
            workflow_run: {
              id: source.tests_run_id,
              head_sha: source.source_sha,
            },
          }),
          { headers: { "Content-Type": "application/json" } },
        );
        Object.defineProperty(response, "url", { value: String(url) });
        return response;
      },
    },
  });
  await assert.rejects(
    verifyArtifactZip(
      client,
      candidate,
      input,
      "fake",
      join(temporary.path, "input.zip"),
      async (_url, path) => {
        writeFileSync(path, "tampered");
      },
    ),
    /ZIP digest mismatch/,
  );
});

test("partial immutable OCI tags restore only matching candidate digest", async () => {
  const { candidate } = makeCandidate(2),
    image = imageReceipt(candidate, "runtime_image"),
    registry = new MemoryRegistry();
  registry.add(candidate, image);
  registry.records.delete(`${image.image}:sha-${candidate.source_sha}`);
  await assert.rejects(
    imagePreflight(registry, candidate, "runtime_image"),
    /partial/,
  );
  await imagePreflight(registry, candidate, "runtime_image", true);
  assert.equal(
    registry.records.get(`${image.image}:sha-${candidate.source_sha}`)?.digest,
    image.digest,
  );
  const found = registry.records.get(
    `${image.image}:sha-${candidate.source_sha}`,
  );
  assert(found);
  found.labels["org.opencontainers.image.revision"] = "wrong";
  await assert.rejects(
    imagePreflight(registry, candidate, "runtime_image", true),
    /different candidate/,
  );
});

test("OCI reads hash actual manifests and blobs, and aliases PUT exactly the same bytes", async () => {
  const image = "ghcr.io/apeiraco/konobangu",
    labels = { "org.opencontainers.image.version": "0.1.0" };
  const config = JSON.stringify({
      os: "linux",
      architecture: "amd64",
      config: { Labels: labels },
    }),
    configDigest = `sha256:${createHash("sha256").update(config).digest("hex")}`;
  const manifest = JSON.stringify({
      schemaVersion: 2,
      config: { digest: configDigest },
    }),
    digest = `sha256:${createHash("sha256").update(manifest).digest("hex")}`;
  let corrupt = false,
    put: Uint8Array | null = null;
  const network = {
    fetch: (async (input, init) => {
      const url = String(input);
      if (url.includes("/token?")) return new Response('{"token":"fake"}');
      if (init?.method === "PUT") {
        assert(init.body instanceof Uint8Array);
        put = init.body;
        return new Response(null, {
          status: 201,
          headers: { "Docker-Content-Digest": digest },
        });
      }
      if (url.includes("/blobs/"))
        return new Response(corrupt ? "tampered" : config);
      return new Response(manifest, {
        headers: {
          "Docker-Content-Digest": digest,
          "Content-Type": "application/vnd.oci.image.manifest.v1+json",
        },
      });
    }) satisfies typeof fetch,
  };
  const registry = new GhcrRegistry(
    { kind: "authenticated", username: "fake", githubToken: "fake-token" },
    network,
  );
  assert.deepEqual(await registry.inspect(image, digest), {
    digest,
    labels,
    variants: [{ digest, labels, os: "linux", architecture: "amd64" }],
  });
  await registry.alias(image, digest, "latest");
  assert(put);
  assert.equal(new TextDecoder().decode(put), manifest);
  corrupt = true;
  await assert.rejects(registry.inspect(image, digest), /content digest/);
  await assert.rejects(
    registry.inspect(image, `sha256:${"f".repeat(64)}`),
    /does not match/,
  );
});

test("empty publication selections cannot reserve any remote object", async () => {
  using temporary = mkdtempDisposableSync(join(root, "temp/no-publication-"));
  const { candidate } = makeCandidate(0),
    backend = new MemoryReleases();
  await assert.rejects(
    reserveVersion(backend, candidate, temporary.path, ""),
    /No release artifacts/,
  );
  assert.equal(backend.commit, null);
  assert.equal(backend.object, null);
  assert.equal(backend.uploads, 0);
});

test("changed prepared archives cannot be finalized with stale manifest hashes", async () => {
  using temporary = mkdtempDisposableSync(join(root, "temp/prepared-drift-"));
  const { candidate, metadata } = makeCandidate(1),
    backend = new MemoryReleases();
  const archives = [];
  for (const target of metadata.release.targets) {
    const name = `konobangu-${candidate.release_version}-${target}.tar.gz`,
      path = join(temporary.path, name);
    writeFileSync(path, target);
    archives.push({
      name,
      target,
      size: Buffer.byteLength(target),
      sha256: await sha256Stream(path),
      files: [],
    });
  }
  const manifest = {
    schema_version: 1 as const,
    candidate,
    metadata,
    archives,
    images: [],
  };
  writeFileSync(join(temporary.path, archives[0].name), "changed");
  await assert.rejects(
    publishManifest(backend, manifest, temporary.path, async () => {}),
    /Prepared archive differs/,
  );
  assert.equal(backend.object?.draft, true);
  assert.equal(backend.stored.has(archives[0].name), false);
});

test("multi-image alias failure preserves completed progress and retry only fills the remaining alias", async () => {
  const { candidate } = makeCandidate(6),
    runtimeImage = imageReceipt(candidate, "runtime_image"),
    fixture = imageReceipt(candidate, "testing_torrents_image"),
    registry = new MemoryRegistry();
  registry.add(candidate, runtimeImage);
  registry.add(candidate, fixture);
  registry.failImage = fixture.image;
  let progress: Awaited<ReturnType<typeof promoteAliases>> = [];
  await assert.rejects(
    promoteAliases(
      registry,
      registry,
      candidate,
      [runtimeImage, fixture],
      (value) => {
        progress = value;
      },
    ),
    /alias failure/,
  );
  assert.equal(progress.length, 1);
  assert.equal(progress[0]?.state, "promoted");
  assert.equal(
    registry.records.get(`${runtimeImage.image}:latest`)?.digest,
    runtimeImage.digest,
  );
  const retry = await promoteAliases(registry, registry, candidate, [
    runtimeImage,
    fixture,
  ]);
  assert.equal(retry[0]?.state, "already-current");
  assert.equal(retry[1]?.state, "promoted");
  assert.equal(
    registry.promoted.filter((image) => image === runtimeImage.image).length,
    1,
  );
});

test("release manifest snapshots cannot mutate unrelated source metadata", () => {
  const expected = loadMetadata();
  for (const changed of [
    { ...expected, project: { ...expected.project, description: "changed" } },
    {
      ...expected,
      release: { ...expected.release, output_directory: "temp/other" },
    },
    { ...expected, node_package: [] },
  ])
    assert.throws(
      () => assertManifestMetadata(changed, expected),
      /differs from committed source/,
    );
  assert.doesNotThrow(() =>
    assertManifestMetadata(JSON.parse(JSON.stringify(expected)), expected),
  );
});

test("fixture OCI indexes verify every runnable child and reject a self-consistent mixed candidate", async () => {
  const { candidate } = makeCandidate(4),
    image = imageReceipt(candidate, "testing_torrents_image");
  const digestOf = (value: string) =>
    `sha256:${createHash("sha256").update(value).digest("hex")}`;
  function fixture(foreignArm: boolean) {
    const blobs = new Map<string, string>(),
      children = [];
    for (const architecture of ["amd64", "arm64"]) {
      const labels = {
        "org.opencontainers.image.source": `https://github.com/${candidate.repository}`,
        "org.opencontainers.image.licenses": "MIT",
        "org.opencontainers.image.revision": candidate.source_sha,
        "org.opencontainers.image.version": candidate.release_version,
        "io.apeiraco.konobangu.candidate-id":
          foreignArm && architecture === "arm64"
            ? "foreign"
            : candidate.candidate_id,
        "io.apeiraco.konobangu.release-metadata-sha256":
          candidate.metadata_sha256,
      };
      const config = JSON.stringify({
          os: "linux",
          architecture,
          config: { Labels: labels },
        }),
        configDigest = digestOf(config);
      const manifest = JSON.stringify({
          schemaVersion: 2,
          config: { digest: configDigest },
        }),
        digest = digestOf(manifest);
      blobs.set(configDigest, config);
      blobs.set(digest, manifest);
      children.push({ digest, platform: { os: "linux", architecture } });
    }
    children.push({
      digest: `sha256:${"f".repeat(64)}`,
      platform: { os: "unknown", architecture: "unknown" },
    });
    const index = JSON.stringify({ schemaVersion: 2, manifests: children }),
      digest = digestOf(index);
    const network = {
      fetch: (async (input) => {
        const url = String(input);
        if (url.includes("/token?")) return new Response('{"token":"fake"}');
        const reference = url.split("/").at(-1);
        if (reference === `sha256:${"f".repeat(64)}`)
          throw new Error("Attestation must not be treated as runnable");
        const body = reference ? blobs.get(reference) : undefined;
        if (url.includes("/blobs/")) return new Response(body);
        return new Response(body ?? index, {
          headers: {
            "Docker-Content-Digest": body && reference ? reference : digest,
            "Content-Type": body
              ? "application/vnd.oci.image.manifest.v1+json"
              : "application/vnd.oci.image.index.v1+json",
          },
        });
      }) satisfies typeof fetch,
    };
    return {
      digest,
      registry: new GhcrRegistry(
        { kind: "authenticated", username: "fake", githubToken: "fake-token" },
        network,
      ),
    };
  }
  const valid = fixture(false);
  assert.equal(
    (await imagePreflight(valid.registry, candidate, "testing_torrents_image"))
      ?.variants.length,
    2,
  );
  await verifyRegistryReceipt(valid.registry, candidate, {
    ...image,
    digest: valid.digest,
  });
  const mixed = fixture(true);
  await assert.rejects(
    imagePreflight(mixed.registry, candidate, "testing_torrents_image"),
    /different candidate/,
  );
  await assert.rejects(
    verifyRegistryReceipt(mixed.registry, candidate, {
      ...image,
      digest: mixed.digest,
    }),
    /different candidate/,
  );
});

test("a forged small ZIP size cannot bypass the streaming expanded-data limit", () => {
  const archive = Buffer.from(
    zipSync({ "candidate.json": new Uint8Array(1024 * 1024) }),
  );
  const end = archive.length - 22,
    central = archive.readUInt32LE(end + 16),
    local = archive.readUInt32LE(central + 42);
  archive.writeUInt32LE(8, central + 24);
  archive.writeUInt32LE(8, local + 22);
  assert.throws(() => candidatePlanFromZip(archive), /expanded data exceeds/);
});

test("private GHCR images leave the release draft until the same candidate is publicly pullable", async () => {
  using temporary = mkdtempDisposableSync(join(root, "temp/public-registry-"));
  const { candidate, metadata } = makeCandidate(2);
  const image = imageReceipt(candidate, "runtime_image");
  const labels = {
    "org.opencontainers.image.source": `https://github.com/${candidate.repository}`,
    "org.opencontainers.image.licenses": "MIT",
    "org.opencontainers.image.revision": candidate.source_sha,
    "org.opencontainers.image.version": candidate.release_version,
    "io.apeiraco.konobangu.candidate-id": candidate.candidate_id,
    "io.apeiraco.konobangu.release-metadata-sha256": candidate.metadata_sha256,
  };
  const config = JSON.stringify({
    os: "linux",
    architecture: "amd64",
    config: { Labels: labels },
  });
  const digestOf = (value: string) =>
    `sha256:${createHash("sha256").update(value).digest("hex")}`;
  const manifestBytes = JSON.stringify({
    schemaVersion: 2,
    config: { digest: digestOf(config) },
  });
  image.digest = digestOf(manifestBytes);
  let publicVisibility = false,
    anonymousTokenRequests = 0,
    requests = 0;
  const anonymous = new GhcrRegistry(
    { kind: "anonymous" },
    {
      fetch: (async (input, init) => {
        requests++;
        const url = String(input);
        assert.notEqual(init?.method, "PUT");
        assert.equal(init?.credentials, "omit");
        if (url.includes("/token?")) {
          anonymousTokenRequests++;
          assert.equal(new Headers(init?.headers).has("authorization"), false);
          assert.equal(
            new URL(url).searchParams.get("scope")?.endsWith(":pull"),
            true,
          );
          return publicVisibility
            ? new Response('{"token":"anonymous-pull-token"}')
            : new Response(null, { status: 403 });
        }
        assert.equal(
          new Headers(init?.headers).get("authorization"),
          "Bearer anonymous-pull-token",
        );
        if (!publicVisibility) return new Response(null, { status: 403 });
        if (url.includes("/blobs/")) return new Response(config);
        return new Response(manifestBytes, {
          headers: {
            "Docker-Content-Digest": image.digest,
            "Content-Type": "application/vnd.oci.image.manifest.v1+json",
          },
        });
      }) satisfies typeof fetch,
    },
  );
  const authenticated = new MemoryRegistry();
  authenticated.add(candidate, image);
  const backend = new MemoryReleases();
  const manifest: ReleaseManifest = {
    schema_version: 1,
    candidate,
    metadata,
    archives: [],
    images: [image],
  };
  const verify = (image: ImageReceipt) =>
    verifyPublicRegistryReceipt(authenticated, anonymous, candidate, image);
  await assert.rejects(
    publishManifest(backend, manifest, temporary.path, verify),
    /enable public package visibility, then retry the same candidate/,
  );
  assert.equal(backend.object?.draft, true);
  assert.equal(anonymousTokenRequests, 1);
  assert.equal(backend.stored.has("release-manifest.json"), false);
  const beforeWriteAttempt = requests;
  await assert.rejects(
    anonymous.alias(image.image, image.digest, "latest"),
    /read-only/,
  );
  assert.equal(requests, beforeWriteAttempt);
  publicVisibility = true;
  await publishManifest(backend, manifest, temporary.path, verify);
  assert.equal(backend.object?.draft, false);
  const uploads = backend.uploads;
  assert.deepEqual(
    await readPublishedManifest(
      backend,
      candidate,
      temporary.path,
      verify,
      metadata,
    ),
    manifest,
  );
  assert.equal(
    (await promoteAliases(authenticated, anonymous, candidate, [image]))[0]
      ?.state,
    "promoted",
  );
  publicVisibility = false;
  await assert.rejects(
    readPublishedManifest(backend, candidate, temporary.path, verify, metadata),
    /not anonymously pullable/,
  );
  assert.equal(backend.uploads, uploads);
  assert.equal(backend.object?.draft, false);
});

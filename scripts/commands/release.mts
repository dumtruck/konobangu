import { createHash } from "node:crypto";
import {
  appendFileSync,
  existsSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { dirname, isAbsolute, join, relative, resolve } from "node:path";
import semver from "semver";
import { root, run } from "../lib/process.mts";
import {
  assembleBundles,
  assembleTargetBundle,
  createRuntimeImageContext,
  imageContext,
} from "../lib/release-bundle.mts";
import {
  assertReleaseApproval,
  type Candidate,
  candidateFrom,
  collectInputArtifacts,
  githubClient,
  parseCandidate,
  repoParameters,
  selectedInputNames,
  streamDownload,
  validateVerifiedRun,
  verifyArtifactZip,
  waitForVerifiedRun,
} from "../lib/release-candidate.mts";
import {
  jsonRecord,
  readJson,
  sourceIdentity,
  stageInput,
  writeJson,
} from "../lib/release-inputs.mts";
import {
  isReleaseTarget,
  loadMetadata,
  type Metadata,
  parseMetadata,
  type ReleaseMetadataSnapshot,
  releaseNotes,
  releaseTargetHosts,
  versionEdits,
} from "../lib/release-metadata.mts";
import {
  assertManifestMetadata,
  GithubReleaseBackend,
  imageName,
  parseImageReceipt,
  parseManifest,
  publishManifest,
  readPublishedManifest,
  reserveVersion,
} from "../lib/release-publisher.mts";
import {
  GhcrRegistry,
  imagePreflight,
  imageTags,
  promoteAliases,
  verifyPublicRegistryReceipt,
  verifyRegistryReceipt,
} from "../lib/release-registry.mts";
import { artifactPath, type BuildTarget } from "./build.mts";

export function checkReleaseVersion() {
  const metadata = loadMetadata();
  const edits = versionEdits(root, metadata.project.version);
  const changed = edits.filter((entry) => entry.source !== entry.next);
  if (changed.length)
    throw new Error(
      `Run release metadata sync; mismatched manifests: ${changed.map((e) => e.path).join(", ")}`,
    );
  console.log(`Release-managed packages match ${metadata.project.version}`);
}
export function setReleaseVersion(version: string) {
  const edits = versionEdits(root, version);
  const lockfiles = ["Cargo.lock", "pnpm-lock.yaml", "uv.lock"].map((path) => ({
    path: join(root, path),
    source: readFileSync(join(root, path), "utf8"),
  }));
  // Validate every manifest before changing any of them; restore on command failure.
  try {
    for (const edit of edits)
      if (edit.source !== edit.next) writeFileSync(edit.path, edit.next);
    run("cargo", ["update", "--workspace", "--offline"]);
    run("pnpm", ["install", "--lockfile-only", "--ignore-scripts"]);
    run("uv", ["lock", "--offline"]);
  } catch (error) {
    for (const edit of [...edits, ...lockfiles])
      writeFileSync(edit.path, edit.source);
    throw error;
  }
  checkReleaseVersion();
}
export function releaseMetadataSource(base = root) {
  const source = readFileSync(join(base, "konobangu-metadata.toml"), "utf8");
  return {
    metadata: parseMetadata(source, base),
    sha256: createHash("sha256")
      .update(source.replaceAll("\r\n", "\n"))
      .digest("hex"),
  };
}
export function releaseMetadataSnapshot(base = root): ReleaseMetadataSnapshot {
  const { metadata, sha256 } = releaseMetadataSource(base);
  return { sha256, artifacts: metadata.release.artifacts };
}
export function releasePlanData(metadata: Metadata, sourceSha: string) {
  const repository = metadata.project.repository_url.match(
    /^https:\/\/github\.com\/([\w.-]+\/[\w.-]+)$/,
  )?.[1];
  if (!repository)
    throw new Error("Expected a canonical GitHub repository URL");
  const artifacts = metadata.release.artifacts,
    version = metadata.project.version;
  const images = Object.fromEntries(
    (["runtime_image", "testing_torrents_image"] as const)
      .filter((key) => artifacts[key])
      .map((key) => {
        const image = `ghcr.io/${repository.toLowerCase()}${key === "testing_torrents_image" ? "-testing-torrents" : ""}`;
        return [
          key,
          {
            image,
            tags: [
              `${image}:${version.replaceAll("+", "_")}`,
              `${image}:sha-${sourceSha}`,
            ],
            latest_tag:
              semver.prerelease(version) !== null ? null : `${image}:latest`,
          },
        ];
      }),
  );
  return {
    version,
    tag: `${metadata.release.tag_prefix}${version}`,
    source_sha: sourceSha,
    repository: metadata.project.repository_url,
    output: metadata.release.output_directory,
    package_registry_publish: false,
    artifacts,
    has_artifacts: Object.values(artifacts).some(Boolean),
    platform_matrix: {
      include: metadata.release.targets.map((target) => ({
        target,
        ...releaseTargetHosts[target],
      })),
    },
    images,
  };
}
function githubOutput(values: Record<string, unknown>) {
  const output = process.env.GITHUB_OUTPUT;
  if (!output) throw new Error("github-output requires GITHUB_OUTPUT");
  appendFileSync(
    output,
    `${Object.entries(values)
      .map(
        ([key, value]) =>
          `${key}=${typeof value === "object" ? JSON.stringify(value) : value}`,
      )
      .join("\n")}\n`,
  );
}
export function releasePlan(format: "json" | "github-output" = "json") {
  const { metadata, sha256 } = releaseMetadataSource();
  const plan = {
    ...releasePlanData(
      metadata,
      run("git", ["rev-parse", "HEAD"], { capture: true }).stdout.trim(),
    ),
    metadata_sha256: sha256,
    dirty: Boolean(
      run("git", ["status", "--porcelain"], { capture: true }).stdout.trim(),
    ),
  };
  if (format === "json") console.log(JSON.stringify(plan, null, 2));
  else
    githubOutput({
      ...plan,
      ...plan.artifacts,
      release_plan: JSON.stringify(plan),
      platform_matrix: JSON.stringify(plan.platform_matrix),
      needs_webui: plan.artifacts.bundles || plan.artifacts.runtime_image,
      prerelease: semver.prerelease(plan.version) !== null,
      ...Object.fromEntries(
        Object.entries(plan.images).flatMap(([key, image]) => [
          [`${key}_tags`, image.tags.join(",")],
          [`${key}_latest_tag`, image.latest_tag ?? ""],
        ]),
      ),
    });
}
function ownedOutput(path: string) {
  const output = resolve(root, path),
    rel = relative(root, output);
  if (
    isAbsolute(rel) ||
    !rel.startsWith(`temp${process.platform === "win32" ? "\\" : "/"}`) ||
    rel.split(/[\\/]/).some((part) => part === "..")
  )
    throw new Error("Release output must stay inside the owned temp directory");
  return output;
}
function cleanSource(sourceSha?: string) {
  const revision = run("git", ["rev-parse", "HEAD"], {
    capture: true,
  }).stdout.trim();
  if (sourceSha && sourceSha !== revision)
    throw new Error("Checkout does not match tested source");
  if (
    run("git", ["status", "--porcelain", "--untracked-files=no"], {
      capture: true,
    }).stdout.trim()
  )
    throw new Error("Tracked release source drifted");
  return revision;
}
function runtimeClients(candidate: Candidate) {
  const token = process.env.GITHUB_TOKEN;
  if (!token) throw new Error("GITHUB_TOKEN is required");
  const client = githubClient(token);
  return {
    client,
    backend: new GithubReleaseBackend(client, candidate.repository, token),
    registry: new GhcrRegistry({
      kind: "authenticated",
      username: process.env.GITHUB_ACTOR ?? "github-actions",
      githubToken: token,
    }),
    anonymousRegistry: new GhcrRegistry({ kind: "anonymous" }),
  };
}
function checkedCandidate(path: string) {
  const candidate = parseCandidate(readJson(resolve(root, path))),
    { metadata, sha256 } = releaseMetadataSource();
  cleanSource(candidate.source_sha);
  checkReleaseVersion();
  if (
    sha256 !== candidate.metadata_sha256 ||
    metadata.project.version !== candidate.release_version ||
    metadata.project.repository_url !==
      `https://github.com/${candidate.repository}` ||
    JSON.stringify(metadata.release.artifacts) !==
      JSON.stringify(candidate.artifacts)
  )
    throw new Error("Candidate differs from committed metadata");
  releaseNotes(
    readFileSync(join(root, metadata.release.changelog), "utf8"),
    candidate.release_version,
  );
  if (
    candidate.tag !==
      `${metadata.release.tag_prefix}${metadata.project.version}` ||
    candidate.prerelease !==
      (semver.prerelease(metadata.project.version) !== null) ||
    JSON.stringify(
      candidate.input_artifacts.map((artifact) => artifact.name).sort(),
    ) !== JSON.stringify(selectedInputNames(metadata))
  )
    throw new Error("Candidate selection/tag differs from committed metadata");
  return { candidate, metadata };
}
export async function stageReleaseInput(
  kind: string,
  output: string,
  target?: string,
) {
  const { metadata, sha256 } = releaseMetadataSource(),
    sourceSha = cleanSource();
  checkReleaseVersion();
  if (
    !process.env.GITHUB_REPOSITORY ||
    process.env.GITHUB_EVENT_NAME !== "push" ||
    process.env.GITHUB_REF !== "refs/heads/release"
  )
    throw new Error(
      "Publishable inputs require a release push verification run",
    );
  if (
    metadata.project.repository_url !==
    `https://github.com/${process.env.GITHUB_REPOSITORY}`
  )
    throw new Error("CI repository differs from metadata");
  const source = sourceIdentity({
    repository: process.env.GITHUB_REPOSITORY,
    source_sha: sourceSha,
    metadata_sha256: sha256,
    release_version: metadata.project.version,
    tests_run_id: Number(process.env.GITHUB_RUN_ID),
    tests_run_attempt: Number(process.env.GITHUB_RUN_ATTEMPT),
  });
  if (kind === "webui") {
    if (
      !metadata.release.artifacts.bundles &&
      !metadata.release.artifacts.runtime_image
    )
      throw new Error("WebUI input is not selected");
    await stageInput({
      output: ownedOutput(output),
      input: join(root, "apps/webui/dist"),
      source,
      kind: "webui",
      target: null,
      checks: {
        passed: true,
        production_build: true,
        provider_selection: "runtime",
      },
    });
  } else if (kind === "recorder" && isReleaseTarget(target)) {
    if (
      !metadata.release.artifacts.bundles &&
      !(
        metadata.release.artifacts.runtime_image &&
        target === "x86_64-unknown-linux-musl"
      )
    )
      throw new Error("Recorder target is not selected");
    const evidence = join(root, "temp/verification", target);
    const { readdirSync } = await import("node:fs");
    const receipts = readdirSync(evidence)
      .filter((name) => name.startsWith("run-"))
      .map((name) => readJson(join(evidence, name, "receipt.json")));
    const { jsonRecord } = await import("../lib/release-inputs.mts");
    const { sha256: fileHash } = await import("../lib/artifacts.mts");
    const artifact = artifactPath(target),
      checks = receipts
        .map(jsonRecord)
        .reverse()
        .find(
          (receipt) =>
            receipt.target === target &&
            receipt.passed === true &&
            receipt.sha256 === fileHash(artifact),
        );
    if (!checks)
      throw new Error("No passing platform receipt for this executable");
    await stageInput({
      output: ownedOutput(output),
      input: artifact,
      notices: join(dirname(artifact), "licenses"),
      source,
      kind: "recorder",
      target,
      checks,
    });
  } else
    throw new Error("Stage expects webui or recorder with a supported target");
  cleanSource(sourceSha);
}
export async function validateReleaseSource(options: {
  sourceRef: string;
  sourceSha: string;
  runId: string;
  attempt: string;
}) {
  if (
    options.sourceRef !== "refs/heads/release" ||
    !process.env.GITHUB_REPOSITORY
  )
    throw new Error("Release source reference/repository is invalid");
  const { metadata, sha256 } = releaseMetadataSource();
  if (
    metadata.project.repository_url !==
    `https://github.com/${process.env.GITHUB_REPOSITORY}`
  )
    throw new Error("Workflow repository differs from metadata");
  const source = sourceIdentity({
    repository: process.env.GITHUB_REPOSITORY,
    source_sha: options.sourceSha,
    tests_run_id: Number(options.runId),
    tests_run_attempt: Number(options.attempt),
    release_version: metadata.project.version,
    metadata_sha256: sha256,
  });
  const client = githubClient(process.env.GITHUB_TOKEN ?? "");
  await waitForVerifiedRun(client, source);
  await validateVerifiedRun(client, source);
}
export async function validateReleaseCandidate(options: {
  sourceRef: string;
  sourceSha: string;
  runId: string;
  attempt: string;
  output: string;
}) {
  if (options.sourceRef !== "refs/heads/release")
    throw new Error("Release source_ref must be refs/heads/release");
  const { metadata, sha256 } = releaseMetadataSource();
  cleanSource(options.sourceSha);
  checkReleaseVersion();
  const repository = metadata.project.repository_url.replace(
      "https://github.com/",
      "",
    ),
    source = sourceIdentity({
      repository,
      source_sha: options.sourceSha,
      tests_run_id: Number(options.runId),
      tests_run_attempt: Number(options.attempt),
      release_version: metadata.project.version,
      metadata_sha256: sha256,
    });
  const token = process.env.GITHUB_TOKEN,
    client = githubClient(token ?? "");
  await waitForVerifiedRun(client, source);
  await validateVerifiedRun(client, source);
  releaseNotes(
    readFileSync(join(root, metadata.release.changelog), "utf8"),
    metadata.project.version,
  );
  const plan = releasePlanData(metadata, source.source_sha),
    output = ownedOutput(options.output);
  const { mkdirSync } = await import("node:fs");
  mkdirSync(output, { recursive: true });
  if (!plan.has_artifacts) {
    githubOutput({ ...plan.artifacts, has_artifacts: false });
    return;
  }
  const backend = new GithubReleaseBackend(client, repository, token ?? ""),
    existing = await backend.release(plan.tag);
  let candidate: Candidate,
    publicRetry = false;
  if (existing && !existing.draft) {
    const assets = await backend.assets(existing.id),
      found = assets.find((asset) => asset.name === "candidate.json");
    if (!found)
      throw new Error("Public version is missing its candidate receipt");
    const path = join(output, "public-candidate.json");
    const { rmSync } = await import("node:fs");
    rmSync(path, { force: true });
    await backend.download(found, path);
    candidate = parseCandidate(readJson(path));
    const { assertSource } = await import("../lib/release-inputs.mts");
    assertSource(candidate, source);
    if (JSON.stringify(candidate.artifacts) !== JSON.stringify(plan.artifacts))
      throw new Error("Public version selection differs");
    const registry = new GhcrRegistry({
      kind: "authenticated",
      username: process.env.GITHUB_ACTOR ?? "github-actions",
      githubToken: token ?? "",
    });
    const anonymousRegistry = new GhcrRegistry({ kind: "anonymous" });
    const manifest = await readPublishedManifest(
      backend,
      candidate,
      output,
      (image) =>
        verifyPublicRegistryReceipt(
          registry,
          anonymousRegistry,
          candidate,
          image,
        ),
      metadata,
    );
    writeJson(join(output, "release-manifest.json"), manifest);
    publicRetry = true;
    if (
      !candidate.artifacts.runtime_image &&
      !candidate.artifacts.testing_torrents_image
    )
      writeJson(join(output, "publication-receipt.json"), {
        schema_version: 1,
        candidate_id: candidate.candidate_id,
        release_run_id: Number(process.env.GITHUB_RUN_ID),
        release_run_attempt: Number(process.env.GITHUB_RUN_ATTEMPT),
        state: "complete",
        already_public: true,
        aliases: [],
      });
  } else
    candidate = candidateFrom(
      source,
      metadata,
      await collectInputArtifacts(client, source, selectedInputNames(metadata)),
    );
  writeJson(join(output, "candidate.json"), candidate);
  githubOutput({
    ...plan.artifacts,
    has_artifacts: true,
    source_sha: candidate.source_sha,
    metadata_sha256: sha256,
    candidate_id: candidate.candidate_id,
    prerelease: candidate.prerelease,
    public_retry: publicRetry,
    artifact_ids: candidate.input_artifacts
      .map((artifact) => artifact.id)
      .join(","),
    tests_run_id: candidate.tests_run_id,
    runtime_image_tags: plan.images.runtime_image?.tags.join(",") ?? "",
    testing_torrents_image_tags:
      plan.images.testing_torrents_image?.tags.join(",") ?? "",
  });
}
export async function assembleRelease(
  candidateFile: string,
  inputs: string,
  output: string,
) {
  const { candidate, metadata } = checkedCandidate(candidateFile);
  const archives = await assembleBundles({
    candidate,
    metadata,
    inputs: resolve(root, inputs),
    output: ownedOutput(output),
    base: root,
  });
  writeJson(join(resolve(root, output), "assembled.json"), {
    schema_version: 1,
    candidate,
    metadata,
    archives,
    images: [],
  });
  cleanSource(candidate.source_sha);
}
export async function createImageContext(
  candidateFile: string,
  inputs: string,
  output: string,
) {
  const { candidate } = checkedCandidate(candidateFile);
  await imageContext(
    candidate,
    resolve(root, inputs),
    ownedOutput(output),
    root,
  );
  cleanSource(candidate.source_sha);
}
export async function checkPublisher(
  candidateFile: string,
  expectedId?: string,
) {
  const { candidate } = checkedCandidate(candidateFile),
    { client } = runtimeClients(candidate);
  if (expectedId && candidate.candidate_id !== expectedId)
    throw new Error("Caller candidate ID differs from downloaded plan");
  if (process.env.GITHUB_REF !== "refs/heads/release")
    throw new Error("Publisher must execute from release ref");
  await validateVerifiedRun(client, candidate);
  await assertReleaseApproval(
    client,
    candidate,
    Number(process.env.GITHUB_RUN_ID),
    { token: process.env.GITHUB_TOKEN ?? "", download: streamDownload },
  );
  return candidate;
}
export async function releaseImage(
  mode: string,
  candidateFile: string,
  kind: string,
  output: string,
  digest?: string,
) {
  const candidate = await checkPublisher(candidateFile),
    { registry } = runtimeClients(candidate);
  if (kind !== "runtime_image" && kind !== "testing_torrents_image")
    throw new Error("Unknown image kind");
  const image = imageName(candidate, kind),
    existing = await imagePreflight(registry, candidate, kind, true);
  if (mode === "preflight") {
    githubOutput({
      image_exists: Boolean(existing),
      image_digest: existing?.digest ?? "",
      image,
      image_tags: imageTags(candidate)
        .map((tag) => `${image}:${tag}`)
        .join(","),
      candidate_id: candidate.candidate_id,
      source_sha: candidate.source_sha,
      metadata_sha256: candidate.metadata_sha256,
      version: candidate.release_version,
    });
    return;
  }
  if (mode !== "receipt") throw new Error("Use image preflight or receipt");
  const receipt = parseImageReceipt(
    {
      kind,
      image,
      digest: digest ?? existing?.digest,
      candidate_id: candidate.candidate_id,
      source_sha: candidate.source_sha,
      metadata_sha256: candidate.metadata_sha256,
    },
    candidate,
  );
  await verifyRegistryReceipt(registry, candidate, receipt);
  writeJson(resolve(root, output), receipt);
  if (process.env.GITHUB_OUTPUT) githubOutput({ image_digest: receipt.digest });
}
export async function publishRelease(
  mode: string,
  candidateFile: string,
  directory: string,
  imageFiles: string[],
  imagesDirectory?: string,
) {
  const candidate = await checkPublisher(candidateFile),
    { metadata } = checkedCandidate(candidateFile),
    { backend, registry, anonymousRegistry } = runtimeClients(candidate),
    output = ownedOutput(directory);
  if (mode === "reserve") {
    await reserveVersion(
      backend,
      candidate,
      output,
      releaseNotes(
        readFileSync(join(root, metadata.release.changelog), "utf8"),
        candidate.release_version,
      ),
    );
    return;
  }
  if (mode === "finalize") {
    const assembled = jsonRecordForAssemble(
      readJson(join(output, "assembled.json")),
    );
    if (
      parseCandidate(assembled.candidate).candidate_id !==
      candidate.candidate_id
    )
      throw new Error("Assembled candidate changed");
    if (imagesDirectory) {
      const { readdirSync } = await import("node:fs");
      const path = resolve(root, imagesDirectory);
      if (existsSync(path))
        imageFiles = [
          ...imageFiles,
          ...readdirSync(path).map((name) => join(path, name)),
        ];
    }
    const manifest = parseManifest({
      ...assembled,
      images: imageFiles.map((path) => readJson(resolve(root, path))),
    });
    assertManifestMetadata(manifest.metadata, metadata);
    await publishManifest(backend, manifest, output, (image) =>
      verifyPublicRegistryReceipt(
        registry,
        anonymousRegistry,
        candidate,
        image,
      ),
    );
    writeJson(join(output, "publication-receipt.json"), {
      schema_version: 1,
      candidate_id: candidate.candidate_id,
      release_run_id: Number(process.env.GITHUB_RUN_ID),
      release_run_attempt: Number(process.env.GITHUB_RUN_ATTEMPT),
      state: manifest.images.length
        ? "release-public-aliases-pending"
        : "complete",
      aliases: [],
    });
    return;
  }
  if (mode === "aliases") {
    const manifest = await readPublishedManifest(
      backend,
      candidate,
      output,
      (image) =>
        verifyPublicRegistryReceipt(
          registry,
          anonymousRegistry,
          candidate,
          image,
        ),
      metadata,
    );
    const path = join(output, "publication-receipt.json");
    const aliases: Awaited<ReturnType<typeof promoteAliases>> = [];
    const receipt = {
      schema_version: 1,
      candidate_id: candidate.candidate_id,
      release_run_id: Number(process.env.GITHUB_RUN_ID),
      release_run_attempt: Number(process.env.GITHUB_RUN_ATTEMPT),
      state: "release-public-aliases-pending",
      aliases,
    };
    writeJson(path, receipt);
    try {
      receipt.aliases = await promoteAliases(
        registry,
        anonymousRegistry,
        candidate,
        manifest.images,
        (aliases) => {
          receipt.aliases = aliases;
          writeJson(path, receipt);
        },
      );
      receipt.state = "complete";
      writeJson(path, receipt);
    } catch (error) {
      writeJson(path, {
        ...receipt,
        failure: error instanceof Error ? error.message : String(error),
      });
      throw error;
    }
    return;
  }
  throw new Error("Use publish reserve, finalize or aliases");
}
function jsonRecordForAssemble(value: unknown) {
  return jsonRecord(value);
}
export async function dispatchRelease() {
  const { metadata } = releaseMetadataSource(),
    plan = releasePlanData(metadata, cleanSource());
  if (!plan.has_artifacts) return;
  if (
    process.env.GITHUB_EVENT_NAME !== "push" ||
    process.env.GITHUB_REF !== "refs/heads/release"
  )
    throw new Error("Only release pushes may dispatch publication");
  const repository = metadata.project.repository_url.replace(
      "https://github.com/",
      "",
    ),
    client = githubClient(process.env.GITHUB_TOKEN ?? "");
  await client.rest.actions.createWorkflowDispatch({
    ...repoParameters(repository),
    workflow_id: "release.yaml",
    ref: "release",
    inputs: {
      source_ref: "refs/heads/release",
      source_sha: plan.source_sha,
      tests_run_id: process.env.GITHUB_RUN_ID ?? "",
      tests_run_attempt: process.env.GITHUB_RUN_ATTEMPT ?? "",
    },
  });
}
export async function prepareRelease(target: BuildTarget) {
  checkReleaseVersion();
  run("just", ["verify"]);
  run("just", ["platform-check", "--target", target]);
  // Local preparation proves local behavior; public releases require GitHub-attested inputs.
  const { metadata, sha256 } = releaseMetadataSource(),
    revision = run("git", ["rev-parse", "HEAD"], {
      capture: true,
    }).stdout.trim();
  const effectiveTarget =
    target === "native"
      ? run("rustc", ["-vV"], { capture: true }).stdout.match(
          /^host: (.+)$/m,
        )?.[1]
      : target;
  if (!isReleaseTarget(effectiveTarget))
    throw new Error("Unsupported local release target");
  const output = join(
      root,
      metadata.release.output_directory,
      metadata.project.version,
      "local-preview",
    ),
    { mkdirSync } = await import("node:fs");
  mkdirSync(output, { recursive: true });
  const receipt = await assembleTargetBundle({
    target: effectiveTarget,
    metadata,
    output,
    base: root,
    nativeDirectory: dirname(artifactPath(target)),
    webuiDirectory: join(root, "apps/webui/dist"),
    provenance: {
      kind: "local",
      source_sha: revision,
      metadata_sha256: sha256,
      dirty: Boolean(
        run("git", ["status", "--porcelain"], { capture: true }).stdout.trim(),
      ),
      release_version: metadata.project.version,
      checks: { verify: true, platform_check: true },
    },
  });
  if (effectiveTarget === "x86_64-unknown-linux-musl")
    await createRuntimeImageContext({
      output: join(output, "runtime-image-context"),
      base: root,
      nativeDirectory: dirname(artifactPath(target)),
      webuiDirectory: join(root, "apps/webui/dist"),
      provenance: {
        kind: "local",
        source_sha: revision,
        metadata_sha256: sha256,
        dirty: Boolean(
          run("git", ["status", "--porcelain"], {
            capture: true,
          }).stdout.trim(),
        ),
        release_version: metadata.project.version,
        checks: { verify: true, platform_check: true },
      },
    });
  writeJson(join(output, "local-preview.json"), {
    schema_version: 1,
    kind: "local-preview",
    source_sha: revision,
    metadata_sha256: sha256,
    archive: receipt,
  });
  console.log(
    `Local verified preview bundle: ${join(output, receipt.name)}; publication requires GitHub-attested source inputs.`,
  );
}

export async function verifyReleaseInputs(candidateFile: string) {
  const { candidate } = checkedCandidate(candidateFile),
    { client } = runtimeClients(candidate);
  const { mkdtempDisposableSync } = await import("node:fs"),
    { tmpdir } = await import("node:os");
  using temporary = mkdtempDisposableSync(join(tmpdir(), "konobangu-inputs-"));
  for (const artifact of candidate.input_artifacts)
    await verifyArtifactZip(
      client,
      candidate,
      artifact,
      process.env.GITHUB_TOKEN ?? "",
      join(temporary.path, `${artifact.id}.zip`),
    );
}

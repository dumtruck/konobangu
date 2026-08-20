import {
  createWriteStream,
  mkdtempDisposableSync,
  readFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Readable } from "node:stream";
import { pipeline } from "node:stream/promises";
import { setTimeout } from "node:timers/promises";
import { Octokit } from "@octokit/rest";
import { Unzip, UnzipInflate } from "fflate";
import semver from "semver";
import { sha256Stream } from "./artifacts.mts";
import {
  contentHash,
  jsonRecord,
  type SourceIdentity,
  sourceIdentity,
} from "./release-inputs.mts";
import {
  type Metadata,
  type ReleaseArtifacts,
  releaseTargetHosts,
} from "./release-metadata.mts";

export type ArtifactIdentity = { id: number; name: string; digest: string };
export type Candidate = SourceIdentity & {
  schema_version: 1;
  candidate_id: string;
  tag: string;
  prerelease: boolean;
  artifacts: ReleaseArtifacts;
  input_artifacts: ArtifactIdentity[];
};
export const verificationPath = ".github/workflows/verification.yaml";
export function selectedInputNames(metadata: Metadata): string[] {
  const selection = metadata.release.artifacts;
  const targets = selection.bundles
    ? metadata.release.targets
    : selection.runtime_image
      ? ["x86_64-unknown-linux-musl"]
      : [];
  return [
    ...(selection.bundles || selection.runtime_image
      ? ["webui-release-input"]
      : []),
    ...targets.map((target) => `recorder-release-input-${target}`),
  ].sort();
}
export function candidateFrom(
  source: SourceIdentity,
  metadata: Metadata,
  inputs: ArtifactIdentity[],
): Candidate {
  const candidate = {
    ...source,
    schema_version: 1 as const,
    tag: `${metadata.release.tag_prefix}${metadata.project.version}`,
    prerelease: semver.prerelease(metadata.project.version) !== null,
    artifacts: metadata.release.artifacts,
    input_artifacts: [...inputs].sort((a, b) =>
      a.name < b.name ? -1 : a.name > b.name ? 1 : 0,
    ),
  };
  return { ...candidate, candidate_id: contentHash(candidate) };
}
export function parseCandidate(value: unknown): Candidate {
  const data = jsonRecord(value),
    source = sourceIdentity(data),
    selection = jsonRecord(data.artifacts);
  if (
    data.schema_version !== 1 ||
    typeof data.tag !== "string" ||
    !/^v?\d/.test(data.tag) ||
    typeof data.prerelease !== "boolean" ||
    typeof data.candidate_id !== "string" ||
    !/^[0-9a-f]{64}$/.test(data.candidate_id) ||
    Object.keys(selection).length !== 3 ||
    ["bundles", "runtime_image", "testing_torrents_image"].some(
      (key) => typeof selection[key] !== "boolean",
    ) ||
    !Array.isArray(data.input_artifacts)
  )
    throw new Error("Invalid release candidate");
  const inputs = data.input_artifacts.map((value) => {
    const input = jsonRecord(value);
    if (
      !Number.isSafeInteger(input.id) ||
      Number(input.id) < 1 ||
      typeof input.name !== "string" ||
      typeof input.digest !== "string" ||
      !/^sha256:[0-9a-f]{64}$/.test(input.digest)
    )
      throw new Error("Invalid artifact identity");
    return { id: Number(input.id), name: input.name, digest: input.digest };
  });
  if (
    new Set(inputs.map((entry) => entry.id)).size !== inputs.length ||
    new Set(inputs.map((entry) => entry.name)).size !== inputs.length
  )
    throw new Error("Duplicate input artifact");
  const candidate: Candidate = {
    ...source,
    schema_version: 1,
    candidate_id: data.candidate_id,
    tag: data.tag,
    prerelease: data.prerelease,
    artifacts: {
      bundles: selection.bundles === true,
      runtime_image: selection.runtime_image === true,
      testing_torrents_image: selection.testing_torrents_image === true,
    },
    input_artifacts: inputs,
  };
  const { candidate_id: _id, ...identity } = candidate;
  if (contentHash(identity) !== candidate.candidate_id)
    throw new Error("Candidate identity hash mismatch");
  return candidate;
}
export function githubClient(token: string): Octokit {
  if (!token) throw new Error("GITHUB_TOKEN is required");
  return new Octokit({ auth: token });
}
export function repoParameters(repository: string) {
  const [owner, repo, extra] = repository.split("/");
  if (!owner || !repo || extra) throw new Error("Invalid GitHub repository");
  return { owner, repo };
}
export type RunWaitRuntime = {
  now: () => number;
  sleep: (milliseconds: number) => Promise<void>;
};
const runWaitRuntime: RunWaitRuntime = {
  now: () => performance.now(),
  sleep: async (milliseconds) => {
    await setTimeout(milliseconds);
  },
};
export async function waitForVerifiedRun(
  client: Octokit,
  source: SourceIdentity,
  runtime: RunWaitRuntime = runWaitRuntime,
) {
  const deadline = runtime.now() + 300000;
  for (;;) {
    const { data: attempt } = await client.rest.actions.getWorkflowRunAttempt({
      ...repoParameters(source.repository),
      run_id: source.tests_run_id,
      attempt_number: source.tests_run_attempt,
    });
    if (
      attempt.head_sha !== source.source_sha ||
      attempt.event !== "push" ||
      attempt.head_branch !== "release"
    )
      throw new Error("Pending verification identity differs from candidate");
    if (attempt.status === "completed") {
      if (attempt.conclusion !== "success")
        throw new Error("Verification attempt failed");
      return;
    }
    if (runtime.now() >= deadline)
      throw new Error(
        "Verification attempt did not complete within five minutes",
      );
    await runtime.sleep(Math.min(5000, deadline - runtime.now()));
  }
}
export async function validateVerifiedRun(
  client: Octokit,
  source: SourceIdentity,
) {
  const repo = repoParameters(source.repository);
  const [{ data: run }, { data: workflow }, { data: attempt }] =
    await Promise.all([
      client.rest.actions.getWorkflowRun({
        ...repo,
        run_id: source.tests_run_id,
      }),
      client.rest.actions.getWorkflow({
        ...repo,
        workflow_id: verificationPath,
      }),
      client.rest.actions.getWorkflowRunAttempt({
        ...repo,
        run_id: source.tests_run_id,
        attempt_number: source.tests_run_attempt,
      }),
    ]);
  if (
    run.repository.full_name !== source.repository ||
    run.head_repository?.full_name !== source.repository ||
    run.workflow_id !== workflow.id ||
    workflow.path !== verificationPath ||
    attempt.workflow_id !== workflow.id ||
    attempt.status !== "completed" ||
    attempt.conclusion !== "success" ||
    attempt.event !== "push" ||
    attempt.head_branch !== "release" ||
    attempt.head_sha !== source.source_sha ||
    attempt.run_attempt !== source.tests_run_attempt ||
    run.head_sha !== source.source_sha
  )
    throw new Error(
      "Verification run identity/status does not match the release source",
    );
  const jobs = await client.paginate(
    client.rest.actions.listJobsForWorkflowRunAttempt,
    {
      ...repo,
      run_id: source.tests_run_id,
      attempt_number: source.tests_run_attempt,
      per_page: 100,
    },
  );
  if (
    !jobs.length ||
    jobs.some(
      (job) => job.status !== "completed" || job.conclusion !== "success",
    )
  )
    throw new Error("Verification attempt has missing or unsuccessful jobs");
  // A successful attempt must include each resource runner, not merely its final check.
  for (const prefix of [
    "Static contracts",
    "TypeScript and WebUI",
    "Rust workspace",
    "Artifacts (",
    "Verification report",
  ])
    if (!jobs.some((job) => job.name.startsWith(prefix)))
      throw new Error(`Verification stage missing: ${prefix}`);
  for (const target of Object.keys(releaseTargetHosts))
    if (
      !jobs.some(
        (job) =>
          job.name.startsWith("Artifacts (") && job.name.includes(target),
      )
    )
      throw new Error(`Verification artifact target missing: ${target}`);
  return attempt;
}
export async function collectInputArtifacts(
  client: Octokit,
  source: SourceIdentity,
  names: string[],
): Promise<ArtifactIdentity[]> {
  const repo = repoParameters(source.repository);
  const inputs = await client.paginate(
    client.rest.actions.listWorkflowRunArtifacts,
    { ...repo, run_id: source.tests_run_id, per_page: 100 },
  );
  return names.map((name) => {
    const matches = inputs.filter((input) => input.name === name);
    if (matches.length !== 1)
      throw new Error(`Missing or ambiguous input artifact: ${name}`);
    const input = matches[0];
    if (
      !input ||
      input.expired ||
      !input.digest ||
      !/^sha256:[0-9a-f]{64}$/.test(input.digest) ||
      input.workflow_run?.head_sha !== source.source_sha ||
      input.workflow_run?.id !== source.tests_run_id
    )
      throw new Error(`Invalid or expired input artifact: ${name}`);
    return { id: input.id, name: input.name, digest: input.digest };
  });
}
export async function assertReleaseApproval(
  client: Octokit,
  candidate: Candidate,
  releaseRunId: number,
  binary: { token: string; download: typeof streamDownload },
) {
  const repo = repoParameters(candidate.repository);
  const { data: run } = await client.rest.actions.getWorkflowRun({
    ...repo,
    run_id: releaseRunId,
  });
  const { data: workflow } = await client.rest.actions.getWorkflow({
    ...repo,
    workflow_id: ".github/workflows/release.yaml",
  });
  if (
    run.repository.full_name !== candidate.repository ||
    run.workflow_id !== workflow.id ||
    workflow.path !== ".github/workflows/release.yaml" ||
    run.event !== "workflow_dispatch" ||
    run.head_branch !== "release"
  )
    throw new Error("Publisher caller is not the release orchestrator");
  if (!candidate.prerelease) {
    const approvals = await client.rest.actions.getReviewsForRun({
      ...repo,
      run_id: releaseRunId,
    });
    if (
      !approvals.data.some(
        (approval) =>
          approval.state === "approved" &&
          approval.environments.some((env) => env.name === "stable-release"),
      )
    )
      throw new Error("Stable release approval is missing");
    const jobs = await client.paginate(
      client.rest.actions.listJobsForWorkflowRun,
      { ...repo, run_id: releaseRunId, filter: "latest", per_page: 100 },
    );
    if (
      !jobs.some(
        (job) =>
          job.name === "Stable release approval" &&
          job.conclusion === "success",
      )
    )
      throw new Error("Stable release approval job has not succeeded");
  }
  // The immutable plan artifact belongs to this caller; its contents bind the callee.
  const artifacts = await client.paginate(
    client.rest.actions.listWorkflowRunArtifacts,
    { ...repo, run_id: releaseRunId, per_page: 100 },
  );
  const plan = artifacts.filter(
    (entry) => entry.name === "release-candidate-plan" && !entry.expired,
  );
  if (plan.length !== 1 || !plan[0]?.digest)
    throw new Error("Caller candidate plan artifact is missing");
  if (
    typeof plan[0].digest !== "string" ||
    !/^sha256:[0-9a-f]{64}$/.test(plan[0].digest)
  )
    throw new Error("Caller plan digest is missing");
  using temporary = mkdtempDisposableSync(join(tmpdir(), "konobangu-plan-"));
  const zip = join(temporary.path, "plan.zip");
  await binary.download(
    `https://api.github.com/repos/${candidate.repository}/actions/artifacts/${plan[0].id}/zip`,
    zip,
    binary.token,
    1024 * 1024,
  );
  if (`sha256:${await sha256Stream(zip)}` !== plan[0].digest)
    throw new Error("Caller plan ZIP digest mismatch");
  const actual = candidatePlanFromZip(readFileSync(zip));
  if (actual.candidate_id !== candidate.candidate_id)
    throw new Error(
      "Publisher candidate differs from authenticated caller plan",
    );
  return plan[0];
}
export async function streamDownload(
  url: string,
  destination: string,
  token: string,
  maximumBytes = Number.MAX_SAFE_INTEGER,
) {
  if (!url.startsWith("https://api.github.com/"))
    throw new Error("Only GitHub API download URLs are allowed");
  const response = await fetch(url, {
    headers: {
      Authorization: `Bearer ${token}`,
      Accept: "application/octet-stream",
      "X-GitHub-Api-Version": "2022-11-28",
    },
    redirect: "follow",
    signal: AbortSignal.timeout(120000),
  });
  if (!response.ok || !response.body)
    throw new Error(`Artifact download failed: ${response.status}`);
  const reader = response.body.getReader();
  async function* chunks() {
    try {
      let size = 0;
      for (;;) {
        const item = await reader.read();
        if (item.done) return;
        size += item.value.byteLength;
        if (size > maximumBytes) {
          await reader.cancel();
          throw new Error("Artifact download exceeds its size limit");
        }
        yield item.value;
      }
    } finally {
      await reader.cancel();
      reader.releaseLock();
    }
  }
  await pipeline(
    Readable.from(chunks()),
    createWriteStream(destination, { flags: "wx" }),
  );
}

export function candidatePlanFromZip(bytes: Uint8Array): Candidate {
  if (bytes.byteLength > 1024 * 1024)
    throw new Error("Candidate ZIP exceeds compressed size limit");
  const zip = Buffer.from(bytes.buffer, bytes.byteOffset, bytes.byteLength),
    end = zip.length - 22;
  if (
    end < 0 ||
    zip.readUInt32LE(end) !== 0x06054b50 ||
    zip.readUInt16LE(end + 20) !== 0 ||
    zip.readUInt16LE(end + 8) !== 1 ||
    zip.readUInt16LE(end + 10) !== 1 ||
    zip.readUInt16LE(end + 4) !== 0 ||
    zip.readUInt16LE(end + 6) !== 0
  )
    throw new Error("Candidate ZIP must have one non-spanned entry");
  const central = zip.readUInt32LE(end + 16),
    centralSize = zip.readUInt32LE(end + 12);
  if (
    central + centralSize !== end ||
    centralSize < 46 ||
    zip.readUInt32LE(central) !== 0x02014b50
  )
    throw new Error("Invalid candidate ZIP directory");
  const attributes = zip.readUInt32LE(central + 38),
    kind = (attributes >>> 16) & 0xf000;
  if (
    (kind !== 0 && kind !== 0x8000) ||
    (attributes & 0x10) !== 0 ||
    (zip.readUInt16LE(central + 8) & 1) !== 0
  )
    throw new Error(
      "Candidate ZIP links, directories or encryption are forbidden",
    );
  const originalSize = zip.readUInt32LE(central + 24);
  if (originalSize < 1 || originalSize > 256 * 1024)
    throw new Error("Candidate ZIP expanded size is invalid");
  let count = 0,
    expanded = 0,
    completed = false;
  const chunks: Uint8Array[] = [];
  const decoder = new Unzip((file) => {
    count++;
    if (count !== 1 || file.name !== "candidate.json")
      throw new Error("Candidate ZIP must contain one candidate.json entry");
    file.ondata = (error, chunk, final) => {
      if (error) throw error;
      expanded += chunk.byteLength;
      if (expanded > originalSize || expanded > 256 * 1024)
        throw new Error("Candidate ZIP expanded data exceeds its size limit");
      chunks.push(chunk);
      completed = final;
    };
    file.start();
  });
  decoder.register(UnzipInflate);
  // Small input chunks prevent a dishonest size header from causing unbounded decode allocation.
  for (let offset = 0; offset < bytes.byteLength; offset += 128)
    decoder.push(
      bytes.subarray(offset, offset + 128),
      offset + 128 >= bytes.byteLength,
    );
  if (count !== 1 || !completed || expanded !== originalSize)
    throw new Error("Invalid candidate ZIP entries or expanded length");
  return parseCandidate(
    JSON.parse(
      new TextDecoder("utf-8", { fatal: true }).decode(Buffer.concat(chunks)),
    ),
  );
}

export async function verifyArtifactZip(
  client: Octokit,
  candidate: Candidate,
  input: ArtifactIdentity,
  token: string,
  destination: string,
  download: typeof streamDownload = streamDownload,
) {
  const { data: artifact } = await client.rest.actions.getArtifact({
    ...repoParameters(candidate.repository),
    artifact_id: input.id,
  });
  if (
    artifact.expired ||
    artifact.name !== input.name ||
    artifact.digest !== input.digest ||
    artifact.workflow_run?.id !== candidate.tests_run_id ||
    artifact.workflow_run?.head_sha !== candidate.source_sha
  )
    throw new Error("Immutable input artifact identity changed");
  await download(
    `https://api.github.com/repos/${candidate.repository}/actions/artifacts/${input.id}/zip`,
    destination,
    token,
  );
  if (`sha256:${await sha256Stream(destination)}` !== input.digest)
    throw new Error(`Input artifact ZIP digest mismatch: ${input.name}`);
}

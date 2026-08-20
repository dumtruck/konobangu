import { createHash } from "node:crypto";
import {
  cpSync,
  lstatSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { isAbsolute, join, relative, resolve } from "node:path";
import { sha256Stream } from "./artifacts.mts";
import { isReleaseTarget, type ReleaseTarget } from "./release-metadata.mts";

export type SourceIdentity = {
  repository: string;
  source_sha: string;
  tests_run_id: number;
  tests_run_attempt: number;
  release_version: string;
  metadata_sha256: string;
};
export type FileEntry = {
  path: string;
  sha256: string;
  size: number;
  mode: number;
};
export type InputReceipt = SourceIdentity & {
  version: 1;
  kind: "recorder" | "webui";
  target: ReleaseTarget | null;
  features: string[];
  backend: "rayon" | null;
  provider_selection: "runtime";
  checks: Record<string, unknown>;
  files: FileEntry[];
};
export function jsonRecord(value: unknown): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value))
    throw new Error("Expected a JSON object");
  return value as Record<string, unknown>;
}
export function safePath(value: unknown): string {
  if (
    typeof value !== "string" ||
    !value ||
    value.includes("\\") ||
    value.includes("\0") ||
    isAbsolute(value) ||
    value.split("/").some((part) => !part || part === "." || part === "..")
  )
    throw new Error(`Unsafe release path: ${String(value)}`);
  return value;
}
export function readJson(path: string): unknown {
  return JSON.parse(readFileSync(path, "utf8"));
}
export function writeJson(path: string, data: unknown) {
  writeFileSync(path, `${JSON.stringify(data, null, 2)}\n`);
}
export function contentHash(value: unknown) {
  function canonical(input: unknown): unknown {
    if (Array.isArray(input)) return input.map(canonical);
    if (input && typeof input === "object")
      return Object.fromEntries(
        Object.entries(input)
          .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0))
          .map(([key, item]) => [key, canonical(item)]),
      );
    return input;
  }
  return createHash("sha256")
    .update(JSON.stringify(canonical(value)))
    .digest("hex");
}
export function sourceIdentity(value: unknown): SourceIdentity {
  const data = jsonRecord(value);
  if (
    typeof data.repository !== "string" ||
    !/^[\w.-]+\/[\w.-]+$/.test(data.repository) ||
    typeof data.source_sha !== "string" ||
    !/^[0-9a-f]{40}$/.test(data.source_sha) ||
    typeof data.metadata_sha256 !== "string" ||
    !/^[0-9a-f]{64}$/.test(data.metadata_sha256) ||
    typeof data.release_version !== "string" ||
    !Number.isSafeInteger(data.tests_run_id) ||
    Number(data.tests_run_id) < 1 ||
    !Number.isSafeInteger(data.tests_run_attempt) ||
    Number(data.tests_run_attempt) < 1
  )
    throw new Error("Invalid release source identity");
  return {
    repository: data.repository,
    source_sha: data.source_sha,
    metadata_sha256: data.metadata_sha256,
    release_version: data.release_version,
    tests_run_id: Number(data.tests_run_id),
    tests_run_attempt: Number(data.tests_run_attempt),
  };
}
export function assertSource(actual: SourceIdentity, expected: SourceIdentity) {
  if (
    JSON.stringify(sourceIdentity(actual)) !==
    JSON.stringify(sourceIdentity(expected))
  )
    throw new Error("Release source identity mismatch");
}
export async function fileEntries(
  directory: string,
  executable?: string,
): Promise<FileEntry[]> {
  const entries: FileEntry[] = [];
  async function walk(path: string, prefix: string) {
    for (const item of readdirSync(path, { withFileTypes: true }).sort((a, b) =>
      a.name.localeCompare(b.name, "en"),
    )) {
      const name = safePath(prefix ? `${prefix}/${item.name}` : item.name);
      if (name === "receipt.json") continue;
      const full = join(path, item.name),
        stat = lstatSync(full);
      if (stat.isSymbolicLink())
        throw new Error(`Release input cannot contain symlinks: ${name}`);
      if (stat.isDirectory()) await walk(full, name);
      else if (stat.isFile())
        entries.push({
          path: name,
          sha256: await sha256Stream(full),
          size: stat.size,
          mode: name === executable ? 0o755 : 0o644,
        });
      else throw new Error(`Unsupported release file: ${name}`);
    }
  }
  await walk(directory, "");
  return entries.sort((a, b) =>
    a.path < b.path ? -1 : a.path > b.path ? 1 : 0,
  );
}
export function parseReceipt(value: unknown): InputReceipt {
  const data = jsonRecord(value),
    source = sourceIdentity(data);
  if (
    data.version !== 1 ||
    !["recorder", "webui"].includes(String(data.kind)) ||
    data.provider_selection !== "runtime" ||
    !Array.isArray(data.files) ||
    !data.files.length ||
    !Array.isArray(data.features) ||
    data.features.some((feature) => typeof feature !== "string") ||
    (data.kind === "recorder"
      ? !isReleaseTarget(data.target) ||
        data.backend !== "rayon" ||
        JSON.stringify(data.features) !== '["jxl","webp"]'
      : data.target !== null ||
        data.backend !== null ||
        data.features.length !== 0)
  )
    throw new Error("Invalid release input receipt");
  const files = data.files.map((value) => {
    const entry = jsonRecord(value),
      path = safePath(entry.path);
    if (
      path === "receipt.json" ||
      typeof entry.sha256 !== "string" ||
      !/^[0-9a-f]{64}$/.test(entry.sha256) ||
      !Number.isSafeInteger(entry.size) ||
      Number(entry.size) < 0 ||
      ![0o644, 0o755].includes(Number(entry.mode))
    )
      throw new Error("Invalid release file entry");
    return {
      path,
      sha256: entry.sha256,
      size: Number(entry.size),
      mode: Number(entry.mode),
    };
  });
  const names = files.map((file) => file.path);
  if (
    new Set(names.map((name) => name.normalize("NFC").toLowerCase())).size !==
      names.length ||
    JSON.stringify(names) !== JSON.stringify([...names].sort())
  )
    throw new Error("Duplicate or unordered input files");
  const checks = jsonRecord(data.checks);
  if (checks.passed !== true)
    throw new Error("Release input checks did not pass");
  const kind = data.kind === "recorder" ? "recorder" : "webui";
  const target = isReleaseTarget(data.target) ? data.target : null;
  if (kind === "recorder") {
    const binary = target?.includes("windows")
      ? "recorder-cli.exe"
      : "recorder-cli";
    if (
      !names.includes(binary) ||
      !names.includes("licenses/THIRD-PARTY-NOTICES.md") ||
      names.some((name) => name !== binary && !name.startsWith("licenses/"))
    )
      throw new Error("Recorder input contains missing or unexpected files");
    if (files.find((entry) => entry.path === binary)?.mode !== 0o755)
      throw new Error("Recorder executable mode is missing");
  } else if (
    !names.includes("webui/index.html") ||
    !names.includes("webui/THIRD-PARTY-NOTICES.md") ||
    names.some((name) => !name.startsWith("webui/"))
  )
    throw new Error("WebUI input contains unexpected files");
  return {
    ...source,
    version: 1,
    kind,
    target,
    features: data.features,
    backend: kind === "recorder" ? "rayon" : null,
    provider_selection: "runtime",
    checks,
    files,
  };
}
export async function verifyInput(
  directory: string,
  source: SourceIdentity,
): Promise<InputReceipt> {
  const receipt = parseReceipt(readJson(join(directory, "receipt.json")));
  assertSource(receipt, source);
  const binary =
    receipt.kind === "recorder"
      ? receipt.target?.includes("windows")
        ? "recorder-cli.exe"
        : "recorder-cli"
      : undefined;
  if (
    JSON.stringify(await fileEntries(directory, binary)) !==
    JSON.stringify(receipt.files)
  )
    throw new Error("Release input file collection/hash mismatch");
  return receipt;
}
export async function stageInput(options: {
  output: string;
  source: SourceIdentity;
  kind: "recorder" | "webui";
  target: ReleaseTarget | null;
  input: string;
  notices?: string;
  checks: Record<string, unknown>;
}) {
  const output = resolve(options.output),
    input = resolve(options.input);
  if (
    lstatSync(input).isSymbolicLink() ||
    (options.notices && lstatSync(options.notices).isSymbolicLink())
  )
    throw new Error("Release staging input root cannot be a symlink");
  const rel = relative(input, output);
  if (!rel || (!rel.startsWith("..") && !isAbsolute(rel)))
    throw new Error("Release staging output overlaps input");
  rmSync(output, { recursive: true, force: true });
  mkdirSync(output, { recursive: true });
  const binary = options.target?.includes("windows")
    ? "recorder-cli.exe"
    : "recorder-cli";
  if (options.kind === "webui")
    cpSync(input, join(output, "webui"), {
      recursive: true,
      dereference: false,
    });
  else {
    if (!options.notices) throw new Error("Missing recorder notices");
    cpSync(input, join(output, binary));
    cpSync(options.notices, join(output, "licenses"), {
      recursive: true,
      dereference: false,
    });
  }
  const receipt = parseReceipt({
    ...options.source,
    version: 1,
    kind: options.kind,
    target: options.target,
    features: options.kind === "recorder" ? ["jxl", "webp"] : [],
    backend: options.kind === "recorder" ? "rayon" : null,
    provider_selection: "runtime",
    checks: options.checks,
    files: await fileEntries(
      output,
      options.kind === "recorder" ? binary : undefined,
    ),
  });
  writeJson(join(output, "receipt.json"), receipt);
  return receipt;
}

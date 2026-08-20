import {
  chmodSync,
  cpSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { open } from "node:fs/promises";
import { join } from "node:path";
import * as tar from "tar";
import { sha256Stream } from "./artifacts.mts";
import type { Candidate } from "./release-candidate.mts";
import {
  contentHash,
  type FileEntry,
  fileEntries,
  type InputReceipt,
  verifyInput,
  writeJson,
} from "./release-inputs.mts";
import type { Metadata, ReleaseTarget } from "./release-metadata.mts";

export type ArchiveReceipt = {
  name: string;
  target: ReleaseTarget;
  sha256: string;
  size: number;
  files: FileEntry[];
};
export type ImageReceipt = {
  kind: "runtime_image" | "testing_torrents_image";
  image: string;
  digest: string;
  candidate_id: string;
  source_sha: string;
  metadata_sha256: string;
};
export type ReleaseManifest = {
  schema_version: 1;
  candidate: Candidate;
  metadata: Metadata;
  archives: ArchiveReceipt[];
  images: ImageReceipt[];
};
export async function createArchive(
  directory: string,
  output: string,
  executable: string,
) {
  const files = await fileEntries(directory, executable);
  const directories = new Set<string>();
  for (const file of files) {
    const parts = file.path.split("/");
    for (let index = 1; index < parts.length; index++)
      directories.add(parts.slice(0, index).join("/"));
  }
  const paths = [...directories, ...files.map((entry) => entry.path)].sort();
  for (const entry of files) chmodSync(join(directory, entry.path), entry.mode);
  await tar.create(
    {
      cwd: directory,
      file: output,
      gzip: true,
      portable: true,
      mtime: new Date(0),
      noDirRecurse: true,
      onWriteEntry(entry) {
        if (!entry.stat) throw new Error("Tar entry stat is missing");
        entry.stat.mode =
          entry.type === "Directory" || entry.path === executable
            ? 0o755
            : 0o644;
        entry.portable = false;
        entry.noMtime = false;
        entry.myuid = 0;
        entry.myuser = "";
        entry.stat.atime = new Date(0);
        entry.stat.ctime = new Date(0);
        entry.stat.dev = 0;
        entry.stat.ino = 0;
        entry.stat.nlink = 1;
        entry.stat.uid = 0;
        entry.stat.gid = 0;
        entry.mtime = new Date(0);
      },
    },
    paths,
  );
  {
    await using header = await open(output, "r+");
    await header.write(new Uint8Array([255]), 0, 1, 9);
  }
  return { sha256: await sha256Stream(output), files };
}
export async function loadCandidateInputs(
  candidate: Candidate,
  inputs: string,
): Promise<Map<string, { directory: string; receipt: InputReceipt }>> {
  const loaded = new Map<
    string,
    { directory: string; receipt: InputReceipt }
  >();
  for (const input of candidate.input_artifacts) {
    const directory = join(inputs, input.name),
      receipt = await verifyInput(directory, candidate);
    if (
      input.name !==
      (receipt.kind === "webui"
        ? "webui-release-input"
        : `recorder-release-input-${receipt.target}`)
    )
      throw new Error("Input artifact name/target mismatch");
    loaded.set(input.name, { directory, receipt });
  }
  return loaded;
}
export async function assembleTargetBundle(options: {
  target: ReleaseTarget;
  metadata: Metadata;
  output: string;
  base: string;
  nativeDirectory: string;
  webuiDirectory: string;
  provenance:
    | { kind: "github"; candidate: Candidate; input_receipts: InputReceipt[] }
    | {
        kind: "local";
        source_sha: string;
        metadata_sha256: string;
        dirty: boolean;
        release_version: string;
        checks: { verify: boolean; platform_check: boolean };
      };
}) {
  const { target, metadata, output, base } = options;
  mkdirSync(output, { recursive: true });
  const stage = join(output, `stage-${target}`),
    binary = target.includes("windows") ? "recorder-cli.exe" : "recorder-cli";
  rmSync(stage, { recursive: true, force: true });
  mkdirSync(stage);
  cpSync(join(options.nativeDirectory, binary), join(stage, binary));
  cpSync(join(options.nativeDirectory, "licenses"), join(stage, "licenses"), {
    recursive: true,
  });
  cpSync(options.webuiDirectory, join(stage, "webui"), { recursive: true });
  cpSync(join(base, "LICENSE"), join(stage, "LICENSE"));
  cpSync(join(base, metadata.release.changelog), join(stage, "CHANGELOG.md"));
  cpSync(
    join(base, "deploy/runtime-image/recorder.config.toml"),
    join(stage, "recorder.config.toml"),
  );
  writeJson(join(stage, "release.json"), {
    schema_version: 1,
    provenance: options.provenance,
    target,
    features: ["jxl", "webp"],
    backend: "rayon",
  });
  const name = `konobangu-${metadata.project.version}-${target}.tar.gz`,
    archive = join(output, name);
  const result = await createArchive(stage, archive, binary),
    { statSync } = await import("node:fs");
  const receipt = { name, target, ...result, size: statSync(archive).size };
  rmSync(stage, { recursive: true, force: true });
  return receipt;
}
export async function assembleBundles(options: {
  candidate: Candidate;
  metadata: Metadata;
  inputs: string;
  output: string;
  base: string;
}) {
  const { candidate, metadata, inputs, output, base } = options;
  const loaded = await loadCandidateInputs(candidate, inputs),
    archives: ArchiveReceipt[] = [];
  mkdirSync(output, { recursive: true });
  if (!candidate.artifacts.bundles) return archives;
  const webui = loaded.get("webui-release-input");
  if (!webui) throw new Error("Missing WebUI input");
  for (const target of metadata.release.targets) {
    const native = loaded.get(`recorder-release-input-${target}`);
    if (!native || native.receipt.target !== target)
      throw new Error(`Missing recorder input for ${target}`);
    archives.push(
      await assembleTargetBundle({
        target,
        metadata,
        output,
        base,
        nativeDirectory: native.directory,
        webuiDirectory: join(webui.directory, "webui"),
        provenance: {
          kind: "github",
          candidate,
          input_receipts: [webui.receipt, native.receipt],
        },
      }),
    );
  }
  return archives;
}
export async function imageContext(
  candidate: Candidate,
  inputs: string,
  output: string,
  base: string,
) {
  if (!candidate.artifacts.runtime_image)
    throw new Error("Runtime image is not selected");
  const loaded = await loadCandidateInputs(candidate, inputs),
    native = loaded.get("recorder-release-input-x86_64-unknown-linux-musl"),
    webui = loaded.get("webui-release-input");
  if (!native || !webui)
    throw new Error("Runtime image needs musl and WebUI inputs");
  await createRuntimeImageContext({
    output,
    base,
    nativeDirectory: native.directory,
    webuiDirectory: join(webui.directory, "webui"),
    provenance: {
      kind: "github",
      candidate,
      input_receipts: [webui.receipt, native.receipt],
    },
  });
}
export async function createRuntimeImageContext(options: {
  output: string;
  base: string;
  nativeDirectory: string;
  webuiDirectory: string;
  provenance: Parameters<typeof assembleTargetBundle>[0]["provenance"];
}) {
  const { output, base } = options;
  rmSync(output, { recursive: true, force: true });
  mkdirSync(output, { recursive: true });
  mkdirSync(join(output, "data"), { mode: 0o755 });
  writeFileSync(
    join(output, "data", "volume-info.txt"),
    "Application data volume owned by UID/GID 65532.\n",
  );
  cpSync(
    join(options.nativeDirectory, "recorder-cli"),
    join(output, "recorder-cli"),
  );
  chmodSync(join(output, "recorder-cli"), 0o755);
  cpSync(join(options.nativeDirectory, "licenses"), join(output, "licenses"), {
    recursive: true,
  });
  cpSync(options.webuiDirectory, join(output, "webui"), {
    recursive: true,
  });
  cpSync(
    join(base, "deploy/runtime-image/Dockerfile"),
    join(output, "Dockerfile"),
  );
  const baseImage = readFileSync(join(output, "Dockerfile"), "utf8").match(
    /^FROM (\S+)$/m,
  )?.[1];
  if (!baseImage) throw new Error("Runtime base image is missing");
  writeJson(join(output, "licenses", "BASE-IMAGE.json"), {
    image: baseImage,
    upstream: "https://github.com/GoogleContainerTools/distroless",
    build_definitions_license:
      "https://github.com/GoogleContainerTools/distroless/blob/main/LICENSE",
    system_package_notices:
      "Base system package licensing remains part of the immutable upstream image.",
  });
  cpSync(
    join(base, "deploy/runtime-image/recorder.config.toml"),
    join(output, "recorder.config.toml"),
  );
  const files = await fileEntries(output, "recorder-cli");
  const provenance =
    options.provenance.kind === "local"
      ? {
          ...options.provenance,
          local_id: `local-${contentHash({ provenance: options.provenance, files })}`,
        }
      : options.provenance;
  writeJson(join(output, "release.json"), {
    schema_version: 1,
    provenance,
    files,
  });
}
export function manifestChecksums(manifest: ReleaseManifest) {
  return `${manifest.archives.map((entry) => `${entry.sha256}  ${entry.name}`).join("\n")}${manifest.archives.length ? "\n" : ""}`;
}
export function writeManifest(output: string, manifest: ReleaseManifest) {
  writeJson(join(output, "release-manifest.json"), manifest);
  writeFileSync(join(output, "SHA256SUMS"), manifestChecksums(manifest));
}

import { readFileSync } from "node:fs";
import { isAbsolute, relative, resolve } from "node:path";
import semver from "semver";
import { parse } from "smol-toml";
import { root } from "./process.mts";

export type ManagedPackage = {
  name: string;
  manifest: string;
  publish: false;
  versioned?: boolean;
};
export type ReleaseArtifacts = {
  bundles: boolean;
  runtime_image: boolean;
  testing_torrents_image: boolean;
};
export type ReleaseMetadataSnapshot = {
  sha256: string;
  artifacts: ReleaseArtifacts;
};
export type Metadata = {
  tooling: { typescript: { config: string } };
  project: {
    version: string;
    display_name: string;
    description: string;
    license: string;
    repository_url: string;
  };
  release: {
    tag_prefix: string;
    output_directory: string;
    binary: string;
    cargo_package: string;
    changelog: string;
    targets: ReleaseTarget[];
    artifacts: ReleaseArtifacts;
  };
  node_package: ManagedPackage[];
  rust_package: ManagedPackage[];
  python_package: ManagedPackage[];
};
// This maps supported host capabilities, while metadata owns the release selection.
export const releaseTargetHosts = {
  "x86_64-unknown-linux-gnu": { os: "ubuntu-24.04", cache_mode: "read-only" },
  "x86_64-unknown-linux-musl": { os: "ubuntu-24.04", cache_mode: "read-only" },
  "x86_64-pc-windows-msvc": { os: "windows-2025", cache_mode: "read-write" },
  "aarch64-apple-darwin": { os: "macos-15", cache_mode: "read-write" },
} as const;
export type ReleaseTarget = keyof typeof releaseTargetHosts;
export function isReleaseTarget(value: unknown): value is ReleaseTarget {
  return typeof value === "string" && Object.hasOwn(releaseTargetHosts, value);
}
export function releaseVersion(value: string): string {
  if (semver.valid(value) !== value)
    throw new Error(`Expected an exact SemVer version: ${value}`);
  return value;
}
export function projectPath(base: string, path: string): string {
  const target = resolve(base, path);
  const rel = relative(base, target);
  if (isAbsolute(path) || rel.startsWith("..") || !rel)
    throw new Error(`Expected a repository-relative path: ${path}`);
  return target;
}
export function loadMetadata(base = root): Metadata {
  return parseMetadata(
    readFileSync(resolve(base, "konobangu-metadata.toml"), "utf8"),
    base,
  );
}
export function parseMetadata(source: string, base = root): Metadata {
  function object(value: unknown, name: string): Record<string, unknown> {
    if (!value || typeof value !== "object" || Array.isArray(value))
      throw new Error(`Missing ${name}`);
    return value as Record<string, unknown>;
  }
  function text(data: Record<string, unknown>, key: string): string {
    if (typeof data[key] !== "string" || !data[key])
      throw new Error(`Missing ${key}`);
    return data[key];
  }
  const data = object(parse(source), "metadata"),
    project = object(data.project, "project"),
    tooling = object(data.tooling, "tooling");
  const typescript = object(tooling.typescript, "tooling.typescript"),
    release = object(data.release, "release"),
    artifacts = object(release.artifacts, "release.artifacts");
  const artifactKeys = ["bundles", "runtime_image", "testing_torrents_image"];
  if (
    Object.keys(artifacts).length !== artifactKeys.length ||
    artifactKeys.some((key) => typeof artifacts[key] !== "boolean")
  )
    throw new Error(
      "release.artifacts requires boolean bundles, runtime_image and testing_torrents_image fields",
    );
  if (
    !Array.isArray(release.targets) ||
    release.targets.length !== Object.keys(releaseTargetHosts).length ||
    new Set(release.targets).size !== release.targets.length
  )
    throw new Error(
      "release.targets must contain each supported release target exactly once",
    );
  const targets = release.targets.map((target) => {
    if (!isReleaseTarget(target))
      throw new Error("Unknown release.targets entry");
    return target;
  });
  const seen = new Set<string>();
  function packages(name: string): ManagedPackage[] {
    const entries = data[name];
    if (!Array.isArray(entries)) throw new Error(`Missing ${name}`);
    return entries.map((value) => {
      const entry = object(value, name),
        manifest = text(entry, "manifest");
      projectPath(base, manifest);
      if (
        entry.publish !== false ||
        seen.has(manifest) ||
        (entry.versioned !== undefined && typeof entry.versioned !== "boolean")
      )
        throw new Error(`Invalid package entry: ${manifest}`);
      seen.add(manifest);
      return {
        name: text(entry, "name"),
        manifest,
        publish: false,
        ...(typeof entry.versioned === "boolean"
          ? { versioned: entry.versioned }
          : {}),
      };
    });
  }
  const metadata: Metadata = {
    tooling: { typescript: { config: text(typescript, "config") } },
    project: {
      version: releaseVersion(text(project, "version")),
      display_name: text(project, "display_name"),
      description: text(project, "description"),
      license: text(project, "license"),
      repository_url: text(project, "repository_url"),
    },
    release: {
      tag_prefix:
        typeof release.tag_prefix === "string" ? release.tag_prefix : "invalid",
      output_directory: text(release, "output_directory"),
      binary: text(release, "binary"),
      cargo_package: text(release, "cargo_package"),
      changelog: text(release, "changelog"),
      targets,
      artifacts: {
        bundles: artifacts.bundles === true,
        runtime_image: artifacts.runtime_image === true,
        testing_torrents_image: artifacts.testing_torrents_image === true,
      },
    },
    node_package: packages("node_package"),
    rust_package: packages("rust_package"),
    python_package: packages("python_package"),
  };
  projectPath(base, metadata.tooling.typescript.config);
  if (!/^v?$/.test(metadata.release.tag_prefix))
    throw new Error("Unsupported tag prefix");
  projectPath(base, metadata.release.output_directory);
  if (!metadata.release.output_directory.startsWith("temp/"))
    throw new Error("Release output must be under temp/");
  projectPath(base, metadata.release.changelog);
  if (!/^[\w-]+$/.test(metadata.release.binary))
    throw new Error("Invalid release binary");
  return metadata;
}
export function packageVersion(source: string, section: string): string {
  const data = parse(source) as Record<string, Record<string, unknown>>;
  const version = data[section]?.version;
  if (typeof version !== "string")
    throw new Error(`Missing ${section}.version`);
  return version;
}
// Git may check out release manifests with CRLF on Windows.
function newline(source: string): string {
  return source.includes("\r\n") ? "\r\n" : "\n";
}
export function replaceTomlVersion(
  source: string,
  section: string,
  version: string,
): string {
  let replaced = false;
  let active = false;
  const result = source
    .split(/\r?\n/)
    .map((line) => {
      if (/^\s*\[/.test(line)) active = line.trim() === `[${section}]`;
      if (active && /^version\s*=/.test(line)) {
        replaced = true;
        return `version = "${version}"`;
      }
      return line;
    })
    .join(newline(source));
  if (!replaced) throw new Error(`Missing ${section}.version`);
  return result;
}
export function versionEdits(base: string, version: string) {
  releaseVersion(version);
  const metadata = loadMetadata(base);
  const edits: { path: string; source: string; next: string }[] = [];
  for (const entry of metadata.node_package) {
    const path = projectPath(base, entry.manifest);
    const source = readFileSync(path, "utf8");
    const data = JSON.parse(source);
    if (data.name !== entry.name)
      throw new Error(`Package name mismatch: ${entry.manifest}`);
    if (entry.versioned) data.version = version;
    if (entry.manifest === "package.json") {
      data.description = metadata.project.description;
      data.license = metadata.project.license;
      data.repository = {
        type: "git",
        url: `${metadata.project.repository_url}.git`,
      };
    }
    data.private = true;
    edits.push({
      path,
      source,
      next: `${JSON.stringify(data, null, 2)}\n`.replaceAll(
        "\n",
        newline(source),
      ),
    });
  }
  for (const [entries, section] of [
    [metadata.rust_package, "package"],
    [metadata.python_package, "project"],
  ] as const) {
    for (const entry of entries) {
      const path = projectPath(base, entry.manifest);
      const source = readFileSync(path, "utf8");
      const data = parse(source) as Record<string, Record<string, unknown>>;
      if (data[section]?.name !== entry.name)
        throw new Error(`Package name mismatch: ${entry.manifest}`);
      edits.push({
        path,
        source,
        next:
          section === "package"
            ? syncRustMetadata(
                replaceTomlVersion(source, section, version),
                metadata,
              )
            : replaceTomlVersion(source, section, version),
      });
    }
  }
  const path = resolve(base, "konobangu-metadata.toml");
  const source = readFileSync(path, "utf8");
  edits.push({
    path,
    source,
    next: replaceTomlVersion(source, "project", version),
  });
  return edits;
}

function syncRustMetadata(source: string, metadata: Metadata): string {
  let active = false;
  const fields: Record<string, string> = {
    license: JSON.stringify(metadata.project.license),
    repository: JSON.stringify(metadata.project.repository_url),
    publish: "false",
  };
  const seen = new Set<string>();
  const lines = source.split(/\r?\n/);
  const result: string[] = [];
  for (const line of lines) {
    if (/^\s*\[/.test(line)) {
      if (active)
        for (const [key, value] of Object.entries(fields))
          if (!seen.has(key)) result.push(`${key} = ${value}`);
      active = line.trim() === "[package]";
    }
    const key = active
      ? line.match(/^(license|repository|publish)\s*=/)?.[1]
      : undefined;
    if (key) {
      seen.add(key);
      result.push(`${key} = ${fields[key]}`);
    } else result.push(line);
  }
  if (active)
    for (const [key, value] of Object.entries(fields))
      if (!seen.has(key)) result.push(`${key} = ${value}`);
  return result.join(newline(source));
}

export function releaseNotes(changelog: string, version: string): string {
  releaseVersion(version);
  const lines = changelog.split(/\r?\n/);
  const start = lines.findIndex(
    (line) => line === `## ${version}` || line.startsWith(`## ${version} - `),
  );
  if (start < 0)
    throw new Error("Finalize the exact versioned changelog before releasing");
  const end = lines.findIndex(
    (line, index) => index > start && line.startsWith("## "),
  );
  return lines
    .slice(start + 1, end < 0 ? undefined : end)
    .join("\n")
    .trim();
}

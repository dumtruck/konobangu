import { createHash } from "node:crypto";
import type { ImageReceipt } from "./release-bundle.mts";
import type { Candidate } from "./release-candidate.mts";
import { jsonRecord } from "./release-inputs.mts";
import { imageName, mayPromoteAlias } from "./release-publisher.mts";

export type RegistryVariant = {
  os: string;
  architecture: string;
  digest: string;
  labels: Record<string, string>;
};
export type RegistryImage = {
  digest: string;
  labels: Record<string, string>;
  variants: RegistryVariant[];
};
export interface RegistryRead {
  inspect(image: string, reference: string): Promise<RegistryImage | null>;
}
export interface Registry extends RegistryRead {
  alias(image: string, digest: string, tag: string): Promise<void>;
}
export type RegistryAuthentication =
  | { kind: "authenticated"; username: string; githubToken: string }
  | { kind: "anonymous" };
const nativeRegistryNetwork = { fetch: globalThis.fetch };
const mediaTypes =
  "application/vnd.oci.image.manifest.v1+json, application/vnd.oci.image.index.v1+json, application/vnd.docker.distribution.manifest.v2+json, application/vnd.docker.distribution.manifest.list.v2+json";
async function jsonBytes(response: Response, expectedDigest: string) {
  if (!response.body) throw new Error("Registry body is missing");
  const reader = response.body.getReader(),
    chunks: Uint8Array[] = [];
  let size = 0;
  try {
    for (;;) {
      const item = await reader.read();
      if (item.done) break;
      size += item.value.byteLength;
      if (size > 4 * 1024 * 1024) {
        await reader.cancel();
        throw new Error("Registry JSON exceeds its size limit");
      }
      chunks.push(item.value);
    }
  } finally {
    reader.releaseLock();
  }
  const bytes = Buffer.concat(chunks);
  if (
    `sha256:${createHash("sha256").update(bytes).digest("hex")}` !==
    expectedDigest
  )
    throw new Error("Registry response content digest mismatch");
  return {
    bytes,
    data: jsonRecord(
      JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(bytes)),
    ),
  };
}
export class GhcrRegistry implements Registry {
  private readonly tokens = new Map<string, string>();
  private readonly manifests = new Map<
    string,
    { bytes: Buffer; mediaType: string }
  >();
  private readonly authentication: RegistryAuthentication;
  private readonly network: { fetch: typeof fetch };
  constructor(
    authentication: RegistryAuthentication,
    network: { fetch: typeof fetch } = nativeRegistryNetwork,
  ) {
    this.authentication = authentication;
    this.network = network;
  }
  private async request(
    image: string,
    path: string,
    write?: { bytes: Buffer; mediaType: string },
  ) {
    if (write && this.authentication.kind === "anonymous")
      throw new Error("Anonymous GHCR clients are read-only");
    if (!/^ghcr\.io\/[a-z0-9_.-]+\/[a-z0-9_.-]+$/.test(image))
      throw new Error("Only canonical GHCR image paths are allowed");
    const repository = image.slice("ghcr.io/".length),
      scope = `repository:${repository}:${write ? "pull,push" : "pull"}`;
    let token = this.tokens.get(scope);
    if (!token) {
      const headers = new Headers();
      if (this.authentication.kind === "authenticated")
        headers.set(
          "Authorization",
          `Basic ${Buffer.from(`${this.authentication.username}:${this.authentication.githubToken}`).toString("base64")}`,
        );
      const response = await this.network.fetch(
        `https://ghcr.io/token?service=ghcr.io&scope=${encodeURIComponent(scope)}`,
        {
          headers,
          credentials: "omit",
          signal: AbortSignal.timeout(30000),
        },
      );
      if (!response.ok)
        throw new Error(`GHCR authentication failed (${response.status})`);
      const data = jsonRecord(await response.json());
      if (typeof data.token !== "string")
        throw new Error("Invalid GHCR authentication response");
      token = data.token;
      this.tokens.set(scope, token);
    }
    return this.network.fetch(`https://ghcr.io/v2/${repository}/${path}`, {
      method: write ? "PUT" : "GET",
      credentials: "omit",
      headers: {
        Authorization: `Bearer ${token}`,
        Accept: mediaTypes,
        ...(write ? { "Content-Type": write.mediaType } : {}),
      },
      ...(write ? { body: new Uint8Array(write.bytes) } : {}),
      signal: AbortSignal.timeout(30000),
    });
  }
  private async manifest(image: string, reference: string) {
    const response = await this.request(image, `manifests/${reference}`);
    if (response.status === 404) return null;
    if (!response.ok)
      throw new Error(`GHCR manifest request failed (${response.status})`);
    const digest = response.headers.get("docker-content-digest"),
      mediaType = response.headers.get("content-type")?.split(";")[0];
    if (
      !digest ||
      !/^sha256:[0-9a-f]{64}$/.test(digest) ||
      (reference.startsWith("sha256:") && reference !== digest) ||
      !mediaType ||
      !mediaTypes.split(", ").includes(mediaType)
    )
      throw new Error("Registry manifest digest/type does not match reference");
    const result = await jsonBytes(response, digest);
    this.manifests.set(`${image}@${digest}`, {
      bytes: result.bytes,
      mediaType,
    });
    return { digest, data: result.data };
  }
  private async variant(
    image: string,
    digest: string,
    manifest: Record<string, unknown>,
    expected?: { os: string; architecture: string },
  ): Promise<RegistryVariant> {
    const config = jsonRecord(manifest.config);
    if (
      typeof config.digest !== "string" ||
      !/^sha256:[0-9a-f]{64}$/.test(config.digest)
    )
      throw new Error("Image config digest is invalid");
    const response = await this.request(image, `blobs/${config.digest}`);
    if (!response.ok) throw new Error("Image config unavailable");
    const blob = (await jsonBytes(response, config.digest)).data,
      settings = jsonRecord(blob.config),
      labels = jsonRecord(settings.Labels);
    if (
      typeof blob.os !== "string" ||
      typeof blob.architecture !== "string" ||
      !blob.os ||
      !blob.architecture ||
      (expected &&
        (blob.os !== expected.os ||
          blob.architecture !== expected.architecture))
    )
      throw new Error(
        "Image config platform differs from its index descriptor",
      );
    const normalized: Record<string, string> = {};
    for (const [key, value] of Object.entries(labels)) {
      if (typeof value !== "string")
        throw new Error("Invalid OCI image labels");
      normalized[key] = value;
    }
    return {
      digest,
      os: blob.os,
      architecture: blob.architecture,
      labels: normalized,
    };
  }
  async inspect(
    image: string,
    reference: string,
  ): Promise<RegistryImage | null> {
    if (!/^(?:sha256:[0-9a-f]{64}|[\w][\w.-]{0,127})$/.test(reference))
      throw new Error("Invalid OCI reference");
    const root = await this.manifest(image, reference);
    if (!root) return null;
    const variants: RegistryVariant[] = [];
    if (Array.isArray(root.data.manifests)) {
      for (const value of root.data.manifests) {
        const entry = jsonRecord(value),
          platform = jsonRecord(entry.platform);
        if (platform.os === "unknown" && platform.architecture === "unknown")
          continue;
        if (
          typeof platform.os !== "string" ||
          typeof platform.architecture !== "string" ||
          typeof entry.digest !== "string" ||
          !/^sha256:[0-9a-f]{64}$/.test(entry.digest)
        )
          throw new Error("Runnable image index descriptor is invalid");
        const child = await this.manifest(image, entry.digest);
        if (!child) throw new Error("Image platform manifest unavailable");
        variants.push(
          await this.variant(image, child.digest, child.data, {
            os: platform.os,
            architecture: platform.architecture,
          }),
        );
      }
    } else variants.push(await this.variant(image, root.digest, root.data));
    const primary = variants.find(
      (variant) => variant.os === "linux" && variant.architecture === "amd64",
    );
    if (
      !primary ||
      new Set(
        variants.map((variant) => `${variant.os}/${variant.architecture}`),
      ).size !== variants.length
    )
      throw new Error("Image has missing or duplicate runnable platforms");
    return { digest: root.digest, labels: primary.labels, variants };
  }
  async alias(image: string, digest: string, tag: string) {
    if (this.authentication.kind === "anonymous")
      throw new Error("Anonymous GHCR clients are read-only");
    if (
      !/^sha256:[0-9a-f]{64}$/.test(digest) ||
      !/^[\w][\w.-]{0,127}$/.test(tag)
    )
      throw new Error("Invalid OCI alias promotion");
    if (!this.manifests.has(`${image}@${digest}`))
      await this.inspect(image, digest);
    const manifest = this.manifests.get(`${image}@${digest}`);
    if (!manifest) throw new Error("Source manifest is unavailable");
    // Reusing the exact bytes retains the digest for both manifests and indexes.
    const response = await this.request(image, `manifests/${tag}`, manifest);
    if (
      !response.ok ||
      response.headers.get("docker-content-digest") !== digest
    )
      throw new Error(`Registry alias PUT failed (${response.status})`);
    await response.body?.cancel();
  }
}
function assertImage(
  image: RegistryImage,
  candidate: Candidate,
  kind: ImageReceipt["kind"],
) {
  const expected =
    kind === "runtime_image" ? ["linux/amd64"] : ["linux/amd64", "linux/arm64"];
  if (
    JSON.stringify(
      image.variants
        .map((variant) => `${variant.os}/${variant.architecture}`)
        .sort(),
    ) !== JSON.stringify(expected)
  )
    throw new Error(
      "Published image platform set differs from selected product",
    );
  for (const labels of [
    image.labels,
    ...image.variants.map((variant) => variant.labels),
  ]) {
    if (
      labels["org.opencontainers.image.source"] !==
        `https://github.com/${candidate.repository}` ||
      labels["org.opencontainers.image.licenses"] !== "MIT" ||
      labels["org.opencontainers.image.revision"] !== candidate.source_sha ||
      labels["org.opencontainers.image.version"] !==
        candidate.release_version ||
      labels["io.apeiraco.konobangu.candidate-id"] !== candidate.candidate_id ||
      labels["io.apeiraco.konobangu.release-metadata-sha256"] !==
        candidate.metadata_sha256
    )
      throw new Error("OCI image belongs to a different candidate");
  }
}
export function imageTags(candidate: Candidate) {
  const tags = [
    candidate.release_version.replaceAll("+", "_"),
    `sha-${candidate.source_sha}`,
  ];
  if (tags.some((tag) => !/^[\w][\w.-]{0,127}$/.test(tag)))
    throw new Error("Release version cannot form a legal OCI tag");
  return tags;
}
export async function imagePreflight(
  registry: Registry,
  candidate: Candidate,
  kind: ImageReceipt["kind"],
  repair = false,
) {
  if (!candidate.artifacts[kind]) throw new Error("Image is not selected");
  const image = imageName(candidate, kind),
    existing: RegistryImage[] = [],
    missing: string[] = [];
  for (const tag of imageTags(candidate)) {
    const found = await registry.inspect(image, tag);
    if (found) {
      assertImage(found, candidate, kind);
      existing.push(found);
    } else missing.push(tag);
  }
  if (existing.length === 2 && existing[0]?.digest !== existing[1]?.digest)
    throw new Error("Candidate image tags resolve to different digests");
  if (existing.length === 1) {
    const found = existing[0];
    if (!found) throw new Error("Candidate image is missing");
    if (!repair)
      throw new Error("Candidate has a partial immutable tag publication");
    for (const tag of missing) {
      await registry.alias(image, found.digest, tag);
      if ((await registry.inspect(image, tag))?.digest !== found.digest)
        throw new Error("Candidate tag repair failed");
    }
  }
  return existing[0] ?? null;
}
export async function verifyRegistryReceipt(
  registry: RegistryRead,
  candidate: Candidate,
  receipt: ImageReceipt,
) {
  if (receipt.image !== imageName(candidate, receipt.kind))
    throw new Error("Published image differs from selected product");
  const found = await registry.inspect(receipt.image, receipt.digest);
  if (!found || found.digest !== receipt.digest)
    throw new Error("Published image digest is unavailable");
  assertImage(found, candidate, receipt.kind);
  for (const tag of imageTags(candidate)) {
    const tagged = await registry.inspect(receipt.image, tag);
    if (!tagged || tagged.digest !== receipt.digest)
      throw new Error("Published image tag differs from receipt");
  }
}
function publicPullError(image: string, cause: unknown) {
  return new Error(
    `Selected image is not anonymously pullable: ${image}. A maintainer must enable public package visibility, then retry the same candidate.`,
    { cause },
  );
}
export async function verifyPublicRegistryReceipt(
  authenticated: RegistryRead,
  anonymous: RegistryRead,
  candidate: Candidate,
  image: ImageReceipt,
) {
  await verifyRegistryReceipt(authenticated, candidate, image);
  try {
    await verifyRegistryReceipt(anonymous, candidate, image);
  } catch (cause) {
    throw publicPullError(image.image, cause);
  }
}
async function verifyPublicAlias(
  anonymous: RegistryRead,
  candidate: Candidate,
  image: ImageReceipt,
) {
  try {
    const found = await anonymous.inspect(image.image, "latest");
    if (!found || found.digest !== image.digest)
      throw new Error("Anonymous stable alias digest differs from receipt");
    assertImage(found, candidate, image.kind);
  } catch (cause) {
    throw publicPullError(image.image, cause);
  }
}
type AliasProgress = {
  image: string;
  digest: string;
  state: "promoted" | "already-current" | "newer-version" | "prerelease";
}[];
export async function promoteAliases(
  registry: Registry,
  anonymous: RegistryRead,
  candidate: Candidate,
  images: ImageReceipt[],
  progress: (receipt: AliasProgress) => void = () => {},
) {
  const receipt: {
    image: string;
    digest: string;
    state: "promoted" | "already-current" | "newer-version" | "prerelease";
  }[] = [];
  for (const image of images) {
    await verifyPublicRegistryReceipt(registry, anonymous, candidate, image);
    if (candidate.prerelease) {
      receipt.push({ ...image, state: "prerelease" });
      progress([...receipt]);
      continue;
    }
    const previous = await registry.inspect(image.image, "latest");
    if (
      !mayPromoteAlias(
        candidate.release_version,
        previous?.labels["org.opencontainers.image.version"] ?? null,
      )
    ) {
      receipt.push({ ...image, state: "newer-version" });
      progress([...receipt]);
      continue;
    }
    if (
      previous?.labels["org.opencontainers.image.version"] ===
        candidate.release_version &&
      previous.digest !== image.digest
    )
      throw new Error(
        "Existing stable alias belongs to a conflicting candidate of this version",
      );
    if (previous?.digest === image.digest) {
      await verifyPublicAlias(anonymous, candidate, image);
      receipt.push({ ...image, state: "already-current" });
      progress([...receipt]);
      continue;
    }
    await registry.alias(image.image, image.digest, "latest");
    const promoted = await registry.inspect(image.image, "latest");
    if (!promoted || promoted.digest !== image.digest)
      throw new Error(`Stable alias promotion failed: ${image.image}`);
    await verifyPublicAlias(anonymous, candidate, image);
    receipt.push({ ...image, state: "promoted" });
    progress([...receipt]);
  }
  return receipt;
}

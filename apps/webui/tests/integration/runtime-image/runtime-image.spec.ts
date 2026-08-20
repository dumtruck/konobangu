import { createHash, generateKeyPairSync, randomUUID } from "node:crypto";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { request as proxyRequest } from "node:http";
import { createServer as secureServer } from "node:https";
import { join, resolve } from "node:path";
import { expect, test } from "@playwright/test";
import { type } from "arktype";
import spawn from "cross-spawn";
import {
  GenericContainer,
  Network,
  TestContainers,
  Wait,
} from "testcontainers";
import {
  createTrustedTlsFixture,
  type TrustedTlsFixture,
} from "../../e2e/support/trusted-tls.mts";
import {
  acquireBundledAccessToken,
  refreshBundledAccessToken,
} from "../../support/bundled-provider.mts";
import { startOidcProvider } from "../../support/oidc-provider.mts";
import { startMikanSource } from "./support/mikan-source.mts";

const root = resolve(import.meta.dirname, "../../../../..");
const WEBUI = "http://127.0.0.1:5000";

class RuntimeContainer extends GenericContainer {
  constructor(image: string) {
    super(image);
    // GenericContainer deliberately exposes Docker HostConfig to subclasses.
    this.hostConfig.ReadonlyRootfs = true;
  }

  withDataVolume(name: string): this {
    this.hostConfig.Mounts ??= [];
    this.hostConfig.Mounts.push({
      Type: "volume",
      Source: name,
      Target: "/data",
      ReadOnly: false,
    });
    return this;
  }
}

async function bundledIssuerGateway(
  resources: AsyncDisposableStack,
  tls: TrustedTlsFixture,
) {
  let target: (() => string) | undefined;
  const server = resources.use(
    secureServer(tls, (request, response) => {
      if (!target || !request.url?.startsWith("/auth-provider/")) {
        response.writeHead(503).end();
        return;
      }
      const proxy = proxyRequest(
        new URL(request.url, target()),
        { method: request.method, headers: request.headers },
        (upstream) => {
          response.writeHead(upstream.statusCode ?? 502, upstream.headers);
          upstream.pipe(response);
        },
      );
      proxy.on("error", () => response.writeHead(502).end());
      response.on("close", () => proxy.destroy());
      request.pipe(proxy);
    }),
  );
  await new Promise<void>((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  const address = server.address();
  if (address === null || typeof address === "string")
    throw new Error("Bundled runtime issuer did not bind TCP");
  return {
    issuer: `https://localhost:${address.port}/auth-provider`,
    setTarget(value: () => string) {
      target = value;
    },
  };
}

async function docker(args: string[]): Promise<string> {
  const child = spawn("docker", args, { stdio: ["ignore", "pipe", "pipe"] });
  let output = "";
  let errors = "";
  child.stdout?.on("data", (part) => {
    output += part;
  });
  child.stderr?.on("data", (part) => {
    errors += part;
  });
  await new Promise<void>((resolve, reject) => {
    child.once("error", reject);
    child.once("close", (code) =>
      code === 0
        ? resolve()
        : reject(new Error(`Docker failed (${code}): ${errors}`)),
    );
  });
  return output;
}

const ImageReceipt = type({
  schema_version: "1",
  provenance: type({
    kind: "'github'",
    candidate: {
      source_sha: "string",
      release_version: "string",
      candidate_id: "string",
      metadata_sha256: "string",
    },
  }).or({
    kind: "'local'",
    source_sha: "string",
    metadata_sha256: "string",
    release_version: "string",
    local_id: "string",
    dirty: "boolean",
    checks: { verify: "boolean", platform_check: "boolean" },
  }),
});
const CreatedSubscription = type({
  data: { subscriptionsCreateOne: { id: "number", subscriberId: "number" } },
});
const CreatedTask = type({
  data: { subscriberTasksCreateOne: { id: "string" } },
});
const TaskList = type({
  data: {
    subscriberTasks: {
      nodes: type({
        id: "string",
        status:
          "'Pending' | 'Scheduled' | 'Running' | 'Done' | 'Failed' | 'Killed'",
        attempts: "number",
        doneAt: "string | null",
        lastError: "string | null",
      }).array(),
    },
  },
});
const DerivativeManifest = type({
  version: "2",
  source_sha256: "string",
  entries: type({
    format: "'image/jxl' | 'image/webp'",
    path: "string",
    length: "number",
    sha256: "string",
  }).array(),
});
const MediaReceipt = type({
  source_sha256: "string",
  webp_sha256: "string",
  jxl_sha256: "string",
  jxl: "true",
});
const SubscriptionList = type({
  data: {
    subscriptions: {
      nodes: type({ id: "number", displayName: "string" }).array(),
    },
  },
});
const ContainerState = type({
  Config: { User: "string" },
  HostConfig: { ReadonlyRootfs: "boolean" },
  Mounts: type({
    Destination: "string",
    RW: "boolean",
    Type: "string",
    "Name?": "string",
  }).array(),
});
const Principal = type({ authType: "'oidc'" });

test("candidate runtime image serves bundled and external OIDC, persists data/grants, and exits cleanly without root or writable rootfs", async () => {
  test.setTimeout(360_000);
  await using resources = new AsyncDisposableStack();
  const contextInput = process.env.KONOBANGU_RUNTIME_CONTEXT;
  const imageInput = process.env.KONOBANGU_RUNTIME_IMAGE;
  if ((contextInput === undefined) === (imageInput === undefined))
    throw new Error(
      "Supply exactly one tested candidate KONOBANGU_RUNTIME_CONTEXT or KONOBANGU_RUNTIME_IMAGE",
    );
  let image: string;
  let provenance: unknown;
  if (contextInput) {
    const directory = resolve(root, contextInput);
    const receipt = ImageReceipt.assert(
      JSON.parse(await readFile(join(directory, "release.json"), "utf8")),
    );
    provenance = receipt.provenance;
    image = `konobangu-runtime-image-smoke:${randomUUID()}`;
    const candidate =
      receipt.provenance.kind === "github"
        ? receipt.provenance.candidate
        : {
            source_sha: receipt.provenance.source_sha,
            release_version: receipt.provenance.release_version,
            metadata_sha256: receipt.provenance.metadata_sha256,
            candidate_id: receipt.provenance.local_id,
          };
    if (
      receipt.provenance.kind === "local" &&
      !candidate.candidate_id.startsWith("local-")
    )
      throw new Error(
        "A local runtime image must not impersonate a GitHub candidate",
      );
    await GenericContainer.fromDockerfile(directory)
      .withBuildkit()
      .withBuildArgs({
        SOURCE_SHA: candidate.source_sha,
        RELEASE_VERSION: candidate.release_version,
        CANDIDATE_ID: candidate.candidate_id,
        METADATA_SHA256: candidate.metadata_sha256,
      })
      .build(image);
    resources.defer(async () => {
      await docker(["image", "rm", image]);
    });
  } else {
    if (!imageInput) throw new Error("Candidate image input disappeared");
    image = imageInput;
  }
  const evidence = join(root, "temp/verification/runtime-image");
  await mkdir(evidence, { recursive: true });
  await rm(join(evidence, "latest.json"), { force: true });
  const directory = await mkdtemp(join(evidence, "run-"));
  resources.defer(() => rm(directory, { recursive: true, force: true }));
  const network = resources.use(await new Network().start());
  resources.use(
    await new GenericContainer("postgres:18-alpine")
      .withEnvironment({
        POSTGRES_DB: "konobangu",
        POSTGRES_USER: "migration_owner",
        POSTGRES_PASSWORD: "fixture-database-password",
      })
      .withNetwork(network)
      .withNetworkAliases("database")
      .withWaitStrategy(
        Wait.forLogMessage("database system is ready to accept connections", 2),
      )
      .start(),
  );
  const idp = resources.use(
    await startOidcProvider({
      redirectUris: [`${WEBUI}/auth/callback`],
      issuerHostname: "host.testcontainers.internal",
    }),
  );
  const poster = await readFile(join(root, "assets/favicon-32x32.png"));
  const mikan = resources.use(await startMikanSource(poster));
  await TestContainers.exposeHostPorts(idp.port, mikan.port);
  const secrets = join(directory, "secrets");
  await mkdir(secrets);
  await writeFile(
    join(secrets, "database"),
    "postgres://migration_owner:fixture-database-password@database:5432/konobangu",
  );
  await writeFile(join(secrets, "provider-password"), "fixture-admin-password");
  await writeFile(
    join(secrets, "provider-key.pem"),
    generateKeyPairSync("rsa", {
      modulusLength: 2048,
      privateKeyEncoding: { type: "pkcs8", format: "pem" },
      publicKeyEncoding: { type: "spki", format: "pem" },
    }).privateKey,
    { mode: 0o444, flag: "wx" },
  );
  const tls = resources.use(await createTrustedTlsFixture());
  const receipts: unknown[] = [];
  for (const provider of ["bundled", "oidc"] as const) {
    const gateway =
      provider === "bundled"
        ? await bundledIssuerGateway(resources, tls)
        : null;
    const volume = `konobangu-runtime-image-${provider}-${randomUUID()}`;
    await docker(["volume", "create", volume]);
    // Container disposal precedes volume disposal; Docker owns empty-volume copy-up.
    resources.defer(async () => {
      await docker(["volume", "rm", volume]);
    });
    const container = resources.use(
      await new RuntimeContainer(image)
        .withDataVolume(volume)
        .withNetwork(network)
        .withEnvironment({
          DATABASE__URL_FILE: "/run/secrets/database",
          DATABASE__MIGRATION__AUTO_RUN: "true",
          AUTH__WEBUI__ALLOWED_ORIGINS: `[{type="strict",value="${WEBUI}"}]`,
          AUTH__PROVIDER__TYPE: provider,
          ...(gateway
            ? {
                AUTH__PROVIDER__ISSUER: gateway.issuer,
                AUTH__PROVIDER__CLIENT__CLIENT_ID: "konobangu-test",
                AUTH__PROVIDER__CLIENT__REDIRECT_URIS: `["${WEBUI}/auth/callback"]`,
                AUTH__PROVIDER__CLIENT__SCOPES:
                  '["openid","profile","konobangu:api","offline_access"]',
                AUTH__PROVIDER__CREDENTIALS__USERNAME: "fixture-admin",
                AUTH__PROVIDER__CREDENTIALS__PASSWORD_FILE:
                  "/run/secrets/provider-password",
                AUTH__PROVIDER__SIGNING__PRIVATE_KEY_PEM_FILE:
                  "/run/secrets/provider-key.pem",
              }
            : {
                AUTH__PROVIDER__ISSUER: idp.issuer,
                AUTH__PROVIDER__BROWSER__CLIENT_ID: "konobangu-test",
              }),
          STORAGE__DATA_DIR: "/data",
          MIKAN__BASE_URL: mikan.baseUrl,
        })
        .withBindMounts([
          { source: secrets, target: "/run/secrets", mode: "ro" },
        ])
        .withTmpFs({ "/tmp": "rw,nosuid,nodev,size=64m,mode=1777" })
        .withExposedPorts(5001)
        .withWaitStrategy(
          Wait.forHttp("/api/auth/config", 5001)
            .withHeaders({ Origin: WEBUI })
            .forStatusCode(200),
        )
        .withStartupTimeout(120_000)
        .start(),
    );
    // Docker can assign a new ephemeral binding when the container restarts.
    const base = () =>
      `http://${container.getHost()}:${container.getMappedPort(5001)}`;
    gateway?.setTarget(base);
    let tokens:
      | Awaited<ReturnType<typeof acquireBundledAccessToken>>
      | undefined;
    let expiresAt = 0;
    const credential = async () => {
      if (!gateway) return `Bearer ${await idp.issueAccessToken("A")}`;
      if (!tokens || Date.now() >= expiresAt - 5_000) {
        tokens = await acquireBundledAccessToken({
          issuer: gateway.issuer,
          redirectUri: `${WEBUI}/auth/callback`,
          signal: AbortSignal.timeout(15_000),
        });
        expiresAt = Date.now() + tokens.expires_in * 1000;
      }
      return `Bearer ${tokens.access_token}`;
    };
    const headers = async () => ({
      Authorization: await credential(),
      Origin: WEBUI,
      "Content-Type": "application/json",
    });
    const graphql = async (query: string, variables?: object) => {
      const response = await fetch(`${base()}/api/graphql`, {
        method: "POST",
        headers: await headers(),
        body: JSON.stringify({ query, variables }),
      });
      expect(response.status).toBe(200);
      const result: unknown = await response.json();
      return result;
    };
    expect(
      (await fetch(`${base()}/api/auth/config`, { headers: { Origin: WEBUI } }))
        .status,
    ).toBe(200);
    const principal = await fetch(`${base()}/api/auth/user-info`, {
      headers: await headers(),
    });
    expect(principal.status).toBe(200);
    expect(Principal.assert(await principal.json()).authType).toBe("oidc");
    const rejectedBasic = await fetch(`${base()}/api/auth/user-info`, {
      headers: {
        Authorization: `Basic ${Buffer.from("fixture-admin:fixture-admin-password").toString("base64")}`,
      },
    });
    expect(rejectedBasic.status).toBe(401);
    expect(
      rejectedBasic.headers.get("www-authenticate")?.startsWith("Basic ") ??
        false,
    ).toBe(false);
    expect(principal.headers.get("Set-Cookie")).toBeNull();
    expect((await fetch(`${base()}/subscriptions/manage`)).status).toBe(200);
    const subscription = CreatedSubscription.assert(
      await graphql(
        "mutation ($data: SubscriptionsInsertInput!) { subscriptionsCreateOne(data: $data) { id subscriberId } }",
        {
          data: {
            displayName: "runtime-image-persistence",
            sourceUrl: `${mikan.baseUrl}RSS/MyBangumi?token=${provider}`,
            enabled: true,
            category: "mikan_subscriber",
          },
        },
      ),
    ).data.subscriptionsCreateOne;
    const identifier = subscription.id;
    const taskId = CreatedTask.assert(
      await graphql(
        "mutation ($data: SubscriberTasksInsertInput!) { subscriberTasksCreateOne(data: $data) { id } }",
        {
          data: {
            job: {
              taskType: "sync_one_subscription_feeds_incremental",
              subscriptionId: identifier,
            },
          },
        },
      ),
    ).data.subscriberTasksCreateOne.id;
    const taskQuery = `{ subscriberTasks(filter:{id:{eq:${JSON.stringify(taskId)}}}) { nodes { id status attempts doneAt lastError } } }`;
    await expect
      .poll(
        async () =>
          TaskList.assert(await graphql(taskQuery)).data.subscriberTasks
            .nodes[0]?.status,
        { timeout: 60_000 },
      )
      .toBe("Done");
    const task = TaskList.assert(await graphql(taskQuery)).data.subscriberTasks
      .nodes[0];
    expect(task).toMatchObject({ id: taskId, attempts: 1, lastError: null });
    expect(task?.doneAt).toBeTruthy();
    expect(mikan.requests).toContain(`/RSS/MyBangumi?token=${provider}`);
    expect(mikan.requests).toContain(`/Home/Episode/runtime-image-${provider}`);
    expect(mikan.requests).toContain(
      `/images/Bangumi/runtime-image-${provider}.png`,
    );
    const posterPath = `public/image/mikan_poster/runtime-image-${provider}.png`;
    const posterUrl = `${base()}/api/static/${posterPath}`;
    const original = await fetch(posterUrl);
    expect(original.status).toBe(200);
    expect(Buffer.from(await original.arrayBuffer())).toEqual(poster);
    await expect
      .poll(
        async () => {
          const response = await fetch(`${posterUrl}.derivatives.json`);
          return response.status === 200
            ? DerivativeManifest.assert(await response.json()).entries.length
            : 0;
        },
        { timeout: 60_000 },
      )
      .toBe(2);
    const manifest = DerivativeManifest.assert(
      await (await fetch(`${posterUrl}.derivatives.json`)).json(),
    );
    expect(manifest.source_sha256).toBe(
      createHash("sha256").update(poster).digest("hex"),
    );
    expect(manifest.entries.map((entry) => entry.format).sort()).toEqual([
      "image/jxl",
      "image/webp",
    ]);
    for (const entry of manifest.entries) {
      const encoded = await fetch(`${posterUrl}?optimize=accept`, {
        headers: { Accept: entry.format },
      });
      expect(encoded.status).toBe(200);
      expect(encoded.headers.get("Content-Type")).toBe(entry.format);
      const bytes = Buffer.from(await encoded.arrayBuffer());
      expect(bytes.length).toBe(entry.length);
      expect(createHash("sha256").update(bytes).digest("hex")).toBe(
        entry.sha256,
      );
    }
    const privatePath = `subscribers/${subscription.subscriberId}/runtime-image-media`;
    const smoke = await container.exec([
      "/app/recorder-cli",
      "media-smoke",
      "--output",
      `/data/${privatePath}`,
    ]);
    expect(smoke.exitCode, smoke.stderr).toBe(0);
    const media = MediaReceipt.assert(JSON.parse(smoke.stdout.trim()));
    const inspect = ContainerState.assert(
      JSON.parse(
        await docker(["inspect", container.getId(), "--format", "{{json .}}"]),
      ),
    );
    expect(inspect.Config.User).toBe("65532:65532");
    expect(inspect.HostConfig.ReadonlyRootfs).toBe(true);
    expect(
      inspect.Mounts.some(
        (mount) =>
          mount.Destination === "/data" &&
          mount.RW &&
          mount.Type === "volume" &&
          mount.Name === volume,
      ),
    ).toBe(true);
    await container.restart({ timeout: 30_000 });
    if (gateway) {
      if (!tokens?.refresh_token)
        throw new Error("Bundled provider did not issue a refresh grant");
      tokens = await refreshBundledAccessToken({
        issuer: gateway.issuer,
        refreshToken: tokens.refresh_token,
        signal: AbortSignal.timeout(15_000),
      });
      expiresAt = Date.now() + tokens.expires_in * 1000;
    }
    const persisted = await fetch(`${base()}/api/graphql`, {
      method: "POST",
      headers: await headers(),
      body: JSON.stringify({
        query: `{ subscriptions(filter:{id:{eq:${identifier}}}) { nodes { id displayName } } }`,
      }),
    });
    expect(persisted.status).toBe(200);
    expect(
      SubscriptionList.assert(await persisted.json()).data.subscriptions.nodes,
    ).toContainEqual({
      id: identifier,
      displayName: "runtime-image-persistence",
    });
    expect(
      TaskList.assert(await graphql(taskQuery)).data.subscriberTasks.nodes[0],
    ).toMatchObject({ id: taskId, status: "Done", attempts: 1 });
    expect(
      DerivativeManifest.assert(
        await (
          await fetch(`${base()}/api/static/${posterPath}.derivatives.json`)
        ).json(),
      ),
    ).toEqual(manifest);
    for (const [file, hash] of [
      ["source.png", media.source_sha256],
      ["encoded.webp", media.webp_sha256],
      ["encoded.jxl", media.jxl_sha256],
    ]) {
      const privateUrl = `${base()}/api/static/${privatePath}/${file}`;
      const privateFile = await fetch(privateUrl, { headers: await headers() });
      expect(privateFile.status).toBe(200);
      expect(privateFile.headers.get("Cache-Control")).toContain("private");
      expect(
        createHash("sha256")
          .update(Buffer.from(await privateFile.arrayBuffer()))
          .digest("hex"),
      ).toBe(hash);
      expect((await fetch(privateUrl)).status).toBe(401);
    }
    await docker(["kill", "--signal=TERM", container.getId()]);
    const exitCode = await docker(["wait", container.getId()]);
    expect(exitCode.trim()).toBe("0");
    receipts.push({
      provider,
      container: container.getId(),
      readOnly: true,
      user: inspect.Config.User,
      migrated: true,
      subscription: identifier,
      task: { id: taskId, status: "Done", attempts: 1 },
      sourceRequests: mikan.requests.filter((path) => path.includes(provider)),
      media: manifest,
      privateMedia: media,
      emptyNamedVolume: volume,
      dataPersisted: true,
      refreshPersisted: provider === "bundled",
      sigtermExit: 0,
    });
  }
  await writeFile(
    join(evidence, "latest.json"),
    JSON.stringify(
      {
        image,
        imageId: (
          await docker(["image", "inspect", image, "--format", "{{.Id}}"])
        ).trim(),
        provenance,
        testSources: {
          spec: createHash("sha256")
            .update(await readFile(import.meta.filename))
            .digest("hex"),
          mikan: createHash("sha256")
            .update(
              await readFile(
                join(import.meta.dirname, "support/mikan-source.mts"),
              ),
            )
            .digest("hex"),
        },
        results: receipts,
      },
      null,
      2,
    ),
  );
});

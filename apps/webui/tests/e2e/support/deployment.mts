import { generateKeyPairSync } from "node:crypto";
import { createReadStream, createWriteStream } from "node:fs";
import {
  mkdir,
  mkdtemp,
  readdir,
  readFile,
  rm,
  stat,
  writeFile,
} from "node:fs/promises";
import {
  createServer as httpServer,
  request as proxyRequest,
  type RequestListener,
} from "node:http";
import { createServer as secureServer } from "node:https";
import { createServer } from "node:net";
import { networkInterfaces } from "node:os";
import { extname, join, relative, resolve, sep } from "node:path";
import { finished } from "node:stream/promises";
import spawn from "cross-spawn";
import { createServer as createViteServer } from "vite";
import { acquireBundledAccessToken } from "../../support/bundled-provider.mts";
import { startOidcProvider } from "../../support/oidc-provider.mts";
import { createDevSourceCopy, type DevSourceCopy } from "./dev-source-copy.mts";
import { createTrustedTlsFixture } from "./trusted-tls.mts";

const root = resolve(import.meta.dirname, "../../../../..");
const UI_HOST = "ui.konobangu.test";
const API_HOST = "api.konobangu-api.test";

export type DeploymentDescriptor = {
  baseUrl: string;
  crossSiteUrl: string;
  crossSiteApiUrl: string;
  lanUrl: string;
  lanApiUrl: string;
  lanBundledUrl: string;
  alternateUrl: string;
  bundledUrl: string;
  bundledApiUrl: string;
  readonly bundledRealm: string;
  instanceUrl: string;
  instanceApiUrl: string;
  issuer: string;
  bundledIssuer: string;
  devIssuer: string;
  directory: string;
  devUrl: string;
  assetPaths: string[];
  readonly nameProxy: string | null;
  readonly devSourceRoot: string | null;
};

export interface ProviderLoginChallenge {
  readonly url: string;
  readonly destination: string | null;
  readonly status: 401;
  readonly challenge: string | null;
}

async function reservePort(): Promise<number> {
  await using reservation = createServer();
  await new Promise<void>((resolve, reject) => {
    reservation.once("error", reject);
    reservation.listen(0, "127.0.0.1", resolve);
  });
  const address = reservation.address();
  if (address === null || typeof address === "string")
    throw new Error("Port reservation did not bind TCP");
  return address.port;
}

async function run(
  command: string,
  args: string[],
  signal: AbortSignal,
  options: {
    env?: NodeJS.ProcessEnv;
    capture?: boolean;
    inspectOutput?: (output: string) => void;
  } = {},
): Promise<string> {
  const child = spawn(command, args, {
    cwd: root,
    signal,
    env: options.env,
    stdio: options.capture ? ["ignore", "pipe", "inherit"] : "inherit",
  });
  let output = "";
  child.stdout?.on("data", (chunk) => {
    output += chunk;
  });
  await new Promise<void>((resolve, reject) => {
    child.once("error", reject);
    child.once("close", (code, interrupted) => {
      options.inspectOutput?.(output);
      code === 0
        ? resolve()
        : reject(new Error(`${command} failed (${code ?? interrupted})`));
    });
  });
  return output;
}

function compilerMessages(output: string): void {
  for (const line of output.split("\n").filter(Boolean)) {
    const event: unknown = JSON.parse(line);
    if (
      typeof event === "object" &&
      event !== null &&
      "reason" in event &&
      event.reason === "compiler-message" &&
      "message" in event &&
      typeof event.message === "object" &&
      event.message !== null &&
      "rendered" in event.message &&
      typeof event.message.rendered === "string"
    )
      process.stderr.write(event.message.rendered);
  }
}

async function listenTls(
  server: ReturnType<typeof secureServer> | undefined,
  port: number,
): Promise<void> {
  if (server === undefined) return;
  await new Promise<void>((resolve) =>
    server.listen(port, "127.0.0.1", resolve),
  );
}

function bundledIssuerHandler(
  issuer: string,
  backend: string,
  providerChallenges: ProviderLoginChallenge[],
): RequestListener {
  return (request, response) => {
    const url = new URL(request.url ?? "/", issuer);
    if (!url.pathname.startsWith("/auth-provider/")) {
      response.writeHead(404).end();
      return;
    }
    const proxy = proxyRequest(
      new URL(`${url.pathname}${url.search}`, backend),
      {
        method: request.method,
        headers: request.headers,
      },
      (upstream) => {
        if (
          url.pathname === "/auth-provider/login" &&
          upstream.statusCode === 401
        ) {
          const destination = request.headers["sec-fetch-dest"];
          const challenge = upstream.headers["www-authenticate"];
          providerChallenges.push({
            url: `${url.origin}${url.pathname}`,
            destination: typeof destination === "string" ? destination : null,
            status: 401,
            challenge: typeof challenge === "string" ? challenge : null,
          });
        }
        response.writeHead(upstream.statusCode ?? 502, upstream.headers);
        upstream.pipe(response);
      },
    );
    proxy.on("error", () => response.writeHead(502).end());
    response.on("close", () => proxy.destroy());
    request.pipe(proxy);
  };
}

/** The test framework owns every server/container; no development database is used. */
export async function startDeployment({ codegenOnly = false } = {}) {
  await using ownership = new AsyncDisposableStack();
  const cancellation = new AbortController();
  ownership.defer(() => cancellation.abort(new Error("Auth fixture ended")));
  const evidence = join(root, "temp/verification/browser");
  await mkdir(evidence, { recursive: true });
  const directory = await mkdtemp(join(evidence, "run-"));
  const fixtureWebui = join(directory, "webui");
  await mkdir(fixtureWebui);
  const ports = await Promise.all(Array.from({ length: 5 }, reservePort));
  const uiPorts = await Promise.all(Array.from({ length: 4 }, reservePort));
  const apiPorts = await Promise.all(Array.from({ length: 3 }, reservePort));
  const lanPorts = await Promise.all(Array.from({ length: 3 }, reservePort));
  const issuerPorts = await Promise.all(Array.from({ length: 2 }, reservePort));
  const bundledIssuers = issuerPorts.map(
    (port) => `https://localhost:${port}/auth-provider`,
  );
  const bundledPrivateKeys = issuerPorts.map(
    () =>
      generateKeyPairSync("rsa", {
        modulusLength: 2048,
        publicKeyEncoding: { type: "spki", format: "pem" },
        privateKeyEncoding: { type: "pkcs8", format: "pem" },
      }).privateKey,
  );
  const lanAddress = Object.values(networkInterfaces())
    .flat()
    .find(
      (address) => address?.family === "IPv4" && !address.internal,
    )?.address;
  if (!lanAddress)
    throw new Error(
      "Browser acceptance requires a non-loopback IPv4 interface",
    );
  const lanUrl = `http://${lanAddress}:${lanPorts[0]}`;
  const lanApiUrl = `http://${lanAddress}:${lanPorts[1]}`;
  const lanBundledUrl = `http://${lanAddress}:${lanPorts[2]}`;
  const devPort = await reservePort();
  const devUrl = `http://127.0.0.1:${devPort}`;
  let devSource: DevSourceCopy | undefined;
  let devLifetime = new AsyncDisposableStack();
  const providerChallenges: ProviderLoginChallenge[] = [];
  const startDev = async () => {
    if (!devSource) throw new Error("A development source fixture is required");
    cancellation.signal.throwIfAborted();
    const dev = devLifetime.adopt(
      await createViteServer({
        root: devSource.appRoot,
        configFile: join(devSource.appRoot, "vite.config.ts"),
        cacheDir: join(devSource.appRoot, ".cache/vite"),
        server: {
          host: "127.0.0.1",
          port: devPort,
          strictPort: true,
          // Projected sources borrow real workspace dependencies through symlinks.
          // Keep both repository roots available without exposing their parent directory.
          fs: { allow: [root, devSource.repoRoot] },
          proxy: {
            "/api": {
              target: `http://127.0.0.1:${ports[4]}`,
              changeOrigin: false,
            },
          },
        },
      }),
      (value) => value.close(),
    );
    await dev.listen();
  };
  const tls = ownership.use(await createTrustedTlsFixture());
  const baseUrl = `http://127.0.0.1:${ports[0]}`;
  const uiOrigins = uiPorts.map((port) => `https://${UI_HOST}:${port}`);
  const apiOrigins = apiPorts.map((port) => `https://${API_HOST}:${port}`);
  const webuiOrigins = [
    baseUrl,
    uiOrigins[0],
    uiOrigins[1],
    uiOrigins[2],
    devUrl,
  ];
  const idp = ownership.use(
    await startOidcProvider({
      redirectUris: [...webuiOrigins, uiOrigins[3], lanUrl].map(
        (origin) => `${origin}/auth/callback`,
      ),
      tls,
    }),
  );
  const configFile = join(directory, "config.json");
  await writeFile(
    configFile,
    JSON.stringify({
      issuer: idp.issuer,
      // The API server trusts the issuer certificate through this CA alone.
      issuer_ca_file: tls.caPath,
      ports,
      webui_origins: webuiOrigins,
      alternate_webui_origin: uiOrigins[3],
      lan_webui_origin: lanUrl,
      lan_bundled_webui_origin: lanBundledUrl,
      bundled_issuers: bundledIssuers,
      bundled_private_keys: bundledPrivateKeys,
      directory,
    }),
    { mode: 0o600 },
  );
  ownership.defer(() => rm(configFile, { force: true }));

  const output = await run(
    "cargo",
    [
      "build",
      "-p",
      "recorder",
      "--locked",
      "--features",
      "testcontainers,test-utils",
      "--bin",
      "auth-test-server",
      "--message-format=json",
    ],
    cancellation.signal,
    { capture: true, inspectOutput: compilerMessages },
  );
  const executables: string[] = [];
  for (const line of output.split("\n").filter(Boolean)) {
    const event: unknown = JSON.parse(line);
    if (typeof event !== "object" || event === null || !("reason" in event))
      continue;
    if (
      event.reason === "compiler-message" &&
      "message" in event &&
      typeof event.message === "object" &&
      event.message !== null &&
      "rendered" in event.message &&
      typeof event.message.rendered === "string"
    )
      process.stderr.write(event.message.rendered);
    if (
      event.reason === "compiler-artifact" &&
      "target" in event &&
      typeof event.target === "object" &&
      event.target !== null &&
      "name" in event.target &&
      event.target.name === "auth-test-server" &&
      "executable" in event &&
      typeof event.executable === "string"
    )
      executables.push(event.executable);
  }
  if (executables.length !== 1)
    throw new Error(
      "Cargo did not report exactly one support server executable",
    );
  const log = ownership.adopt(
    createWriteStream(join(directory, "server.log")),
    async (stream) => {
      stream.end();
      await finished(stream);
    },
  );
  const server = spawn(executables[0], [], {
    cwd: root,
    env: { ...process.env, KONOBANGU_AUTH_TEST_CONFIG: configFile },
    stdio: ["ignore", "pipe", "pipe"],
  });
  server.stdout?.pipe(log, { end: false });
  server.stderr?.pipe(log, { end: false });
  ownership.defer(async () => {
    if (server.exitCode !== null || server.signalCode !== null) return;
    const stopped = new Promise<void>((resolve) =>
      server.once("close", () => resolve()),
    );
    server.kill("SIGTERM");
    using _timeout = setTimeout(() => server.kill("SIGKILL"), 15000);
    await stopped;
  });
  const deadline = Date.now() + 90_000;
  for (;;) {
    if (server.exitCode !== null || server.signalCode !== null)
      throw new Error(
        `Support server failed: ${join(directory, "server.log")}`,
      );
    try {
      await stat(join(directory, "server.json"));
      break;
    } catch {
      /* Readiness is published after all production routers are listening. */
    }
    if (Date.now() > deadline)
      throw new Error("Support server readiness timed out");
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  for (const port of ports) {
    const response = await fetch(`http://127.0.0.1:${port}/api/auth/user-info`);
    if (response.status !== 401)
      throw new Error(`Unexpected private readiness status ${response.status}`);
  }
  for (const [index, issuer] of bundledIssuers.entries()) {
    const backend = `http://127.0.0.1:${ports[index === 0 ? 2 : 4]}`;
    const gateway = ownership.use(
      secureServer(
        tls,
        bundledIssuerHandler(issuer, backend, providerChallenges),
      ),
    );
    await listenTls(gateway, issuerPorts[index]);
  }
  const codegenToken = await acquireBundledAccessToken({
    issuer: bundledIssuers[0],
    redirectUri: `${webuiOrigins[2]}/auth/callback`,
    signal: cancellation.signal,
  });
  const codegenEnv = {
    ...process.env,
    KONOBANGU_GRAPHQL_SCHEMA: `http://127.0.0.1:${ports[2]}/api/graphql/introspection`,
    KONOBANGU_CODEGEN_AUTHORIZATION: `Bearer ${codegenToken.access_token}`,
  };
  await run("just", ["dev-codegen"], cancellation.signal, { env: codegenEnv });
  if (!codegenOnly) {
    await run(
      "pnpm",
      [
        "--filter",
        "webui",
        "exec",
        "vite",
        "build",
        "--outDir",
        relative(join(root, "apps/webui"), fixtureWebui),
        "--emptyOutDir",
      ],
      cancellation.signal,
    );
    const inspect = async (folder: string): Promise<void> => {
      for (const entry of await readdir(folder, { withFileTypes: true })) {
        const path = join(folder, entry.name);
        if (entry.isDirectory()) await inspect(path);
        else if (
          /\.(js|html|css|map)$/.test(entry.name) &&
          (await readFile(path, "utf8")).includes("fixture-admin-password")
        )
          throw new Error("Fixture credential leaked into bundle");
      }
    };
    await inspect(fixtureWebui);
    const apiHandler =
      (backend: string, _apiOrigin: string): RequestListener =>
      (request, response) => {
        const proxy = proxyRequest(
          new URL(request.url ?? "/", backend),
          { method: request.method, headers: request.headers },
          (upstream) => {
            response.writeHead(upstream.statusCode ?? 502, upstream.headers);
            upstream.pipe(response);
          },
        );
        proxy.on("error", () => response.writeHead(502).end());
        response.on("close", () => proxy.destroy());
        request.pipe(proxy);
      };
    const uiHandler =
      (
        uiOrigin: string,
        apiOrigin: string,
        noReferrer = false,
      ): RequestListener =>
      async (request, response) => {
        try {
          const url = new URL(request.url ?? "/", uiOrigin);
          if (url.pathname === "/konobangu.config.json") {
            response
              .writeHead(200, {
                "Content-Type": "application/json",
                "Cache-Control": "no-store",
              })
              .end(
                JSON.stringify({ version: 1, apiBaseUrl: `${apiOrigin}/api/` }),
              );
            return;
          }
          const candidate = resolve(
            fixtureWebui,
            `.${decodeURIComponent(url.pathname)}`,
          );
          if (!candidate.startsWith(fixtureWebui + sep)) {
            response.writeHead(400).end();
            return;
          }
          const file = await stat(candidate)
            .then((info) =>
              info.isFile() ? candidate : join(fixtureWebui, "index.html"),
            )
            .catch(() => join(fixtureWebui, "index.html"));
          const mime: Record<string, string> = {
            ".html": "text/html",
            ".js": "text/javascript",
            ".css": "text/css",
            ".json": "application/json",
            ".svg": "image/svg+xml",
          };
          response.writeHead(200, {
            "Content-Type": mime[extname(file)] ?? "application/octet-stream",
            "Cache-Control": "no-store",
            ...(noReferrer && extname(file) === ".html"
              ? { "Referrer-Policy": "no-referrer" }
              : {}),
          });
          createReadStream(file).pipe(response);
        } catch {
          response.writeHead(500).end();
        }
      };
    for (let index = 0; index < 4; index++) {
      const backend = `http://127.0.0.1:${ports[index + 1]}`;
      const api =
        index < 3
          ? ownership.use(
              secureServer(tls, apiHandler(backend, apiOrigins[index])),
            )
          : undefined;
      const ui = ownership.use(
        secureServer(
          tls,
          uiHandler(
            uiOrigins[index],
            apiOrigins[index % apiOrigins.length],
            index === 1,
          ),
        ),
      );
      await Promise.all([
        listenTls(api, apiPorts[index]),
        new Promise<void>((resolve) =>
          ui.listen(uiPorts[index], "127.0.0.1", resolve),
        ),
      ]);
    }
    // Real LAN HTTP pages intentionally lack secure-context APIs. The shared
    // production API and the trusted HTTPS IdP retain their normal protocol paths.
    const lanApi = ownership.use(
      httpServer(apiHandler(`http://127.0.0.1:${ports[1]}`, lanApiUrl)),
    );
    const lanUi = ownership.use(httpServer(uiHandler(lanUrl, lanApiUrl)));
    await Promise.all([
      new Promise<void>((resolve) =>
        lanApi.listen(lanPorts[1], lanAddress, resolve),
      ),
      new Promise<void>((resolve) =>
        lanUi.listen(lanPorts[0], lanAddress, resolve),
      ),
    ]);
    const bundledApi = apiHandler(
      `http://127.0.0.1:${ports[2]}`,
      lanBundledUrl,
    );
    const bundledUi = uiHandler(lanBundledUrl, lanBundledUrl);
    const lanBundled = ownership.use(
      httpServer((request, response) => {
        if (
          new URL(request.url ?? "/", lanBundledUrl).pathname.startsWith(
            "/api/",
          )
        )
          bundledApi(request, response);
        else void bundledUi(request, response);
      }),
    );
    await new Promise<void>((resolve) =>
      lanBundled.listen(lanPorts[2], lanAddress, resolve),
    );
    devSource = ownership.use(await createDevSourceCopy(root));
    ownership.defer(() => devLifetime.disposeAsync());
    await startDev();
  }
  const assetPaths: string[] = [];
  const collectAssets = async (directory: string): Promise<void> => {
    for (const entry of await readdir(directory, { withFileTypes: true })) {
      const path = join(directory, entry.name);
      if (entry.isDirectory()) await collectAssets(path);
      else if (/\.(js|css|wasm)$/.test(entry.name))
        assetPaths.push(
          `/${relative(fixtureWebui, path).split(sep).join("/")}`,
        );
    }
  };
  if (!codegenOnly) await collectAssets(fixtureWebui);
  const descriptor: DeploymentDescriptor = {
    baseUrl,
    crossSiteUrl: uiOrigins[0],
    crossSiteApiUrl: apiOrigins[0],
    lanUrl,
    lanApiUrl,
    lanBundledUrl,
    alternateUrl: uiOrigins[3],
    bundledUrl: uiOrigins[1],
    bundledApiUrl: apiOrigins[1],
    bundledRealm: "Konobangu Provider",
    instanceUrl: uiOrigins[2],
    instanceApiUrl: apiOrigins[2],
    issuer: idp.issuer,
    bundledIssuer: bundledIssuers[0],
    devIssuer: bundledIssuers[1],
    directory,
    devUrl,
    assetPaths: assetPaths.sort(),
    nameProxy: tls.nameProxy,
    devSourceRoot: devSource?.appRoot ?? null,
  };
  const lifetime = ownership.move();
  let baselineBrowser: Promise<string> | undefined;
  const baselineChromiumExecutable = () => {
    cancellation.signal.throwIfAborted();
    if (!baselineBrowser)
      baselineBrowser = (async () => {
        const env = { ...process.env, PLAYWRIGHT_SKIP_BROWSER_GC: "1" };
        const version = (
          await run(
            "just",
            ["--evaluate", "browser_baseline_playwright_version"],
            cancellation.signal,
            { capture: true },
          )
        ).trim();
        const output = await run(
          "pnpm",
          [
            "dlx",
            `playwright@${version}`,
            "install",
            "--dry-run",
            "chromium",
            "--only-shell",
          ],
          cancellation.signal,
          { capture: true, env },
        );
        const location =
          /browser: chromium-headless-shell[^\n]*\n\s*Install location:\s*([^\n]+)/
            .exec(output)?.[1]
            ?.trim();
        if (!location)
          throw new Error(
            "The official baseline Chromium installer did not report its install directory",
          );
        const executable = join(
          location,
          process.platform === "darwin"
            ? "chrome-mac/headless_shell"
            : process.platform === "win32"
              ? "chrome-win/headless_shell.exe"
              : "chrome-linux/headless_shell",
        );
        const present = await stat(executable)
          .then(() => true)
          .catch(() => false);
        if (!present)
          await run(
            "pnpm",
            [
              "dlx",
              `playwright@${version}`,
              "install",
              "chromium",
              "--only-shell",
            ],
            cancellation.signal,
            { env },
          );
        await stat(executable);
        return executable;
      })();
    return baselineBrowser;
  };
  return {
    descriptor,
    rotateSigningKey: () => idp.rotateSigningKey(),
    baselineChromiumExecutable,
    providerLoginChallenges: (): readonly ProviderLoginChallenge[] =>
      providerChallenges.map((event) => ({ ...event })),
    async restartDev() {
      const previous = devLifetime;
      devLifetime = new AsyncDisposableStack();
      await previous.disposeAsync();
      await startDev();
    },
    [Symbol.asyncDispose]: () => {
      cancellation.abort(new Error("Auth fixture ended"));
      return lifetime.disposeAsync();
    },
  };
}

if (import.meta.main) {
  if (!process.argv.includes("--codegen-only"))
    throw new Error(
      "Use Playwright or just test browser; deployment CLI only accepts --codegen-only",
    );
  await using _deployment = await startDeployment({ codegenOnly: true });
}

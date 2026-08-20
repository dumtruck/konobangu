import { spawn } from "node:child_process";
import { mkdtemp, readFile, realpath, rm, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { dirname, join, relative, resolve, sep } from "node:path";
import { pathToFileURL } from "node:url";
import { expect, test } from "@playwright/test";
import { type } from "arktype";

test("local SDL codegen watches schema and documents without an access token", async () => {
  test.setTimeout(60_000);
  await using lifetime = new AsyncDisposableStack();
  // Native filesystem watchers report canonical paths on hosts such as macOS.
  const directory = await realpath(
    await mkdtemp(join(tmpdir(), "konobangu-codegen-")),
  );
  lifetime.defer(() => rm(directory, { recursive: true, force: true }));
  const schema = join(directory, "schema.graphql");
  const document = join(directory, "query.ts");
  const output = join(directory, "generated/");
  const generated = join(output, "graphql.ts");
  const config = join(directory, "codegen.config.ts");
  const ready = join(directory, "watch-ready");
  const webui = resolve(import.meta.dirname, "../..");
  await writeFile(schema, "type Query { marker: String! }\n");
  await writeFile(
    document,
    "const query = gql`query LocalCodegen { marker }`;\n",
  );
  const applicationConfig = pathToFileURL(
    resolve(import.meta.dirname, "../../graphql-codegen.ts"),
  );
  await writeFile(
    config,
    `import { writeFileSync } from "node:fs";
import base from ${JSON.stringify(applicationConfig.href)};
export default { ...base, documents: [${JSON.stringify(relative(webui, document).split(sep).join("/"))}], hooks: {
  onWatchTriggered: () => writeFileSync(${JSON.stringify(ready)}, "ready"),
}, generates: {
  [${JSON.stringify(output)}]: base.generates["./src/infra/graphql/gql/"],
} };
`,
  );
  const env: NodeJS.ProcessEnv = {
    ...process.env,
    KONOBANGU_GRAPHQL_SCHEMA: schema,
  };
  delete env.KONOBANGU_CODEGEN_AUTHORIZATION;
  const metadataPath = createRequire(import.meta.url).resolve(
    "@graphql-codegen/cli/package.json",
  );
  const metadata = type({ bin: { "graphql-codegen": "string" } }).assert(
    JSON.parse(await readFile(metadataPath, "utf8")),
  );
  // Spawn the package's declared executable directly so this owner waits for
  // the actual watcher, without a package-manager process between them.
  const child = spawn(
    process.execPath,
    [
      resolve(dirname(metadataPath), metadata.bin["graphql-codegen"]),
      "--config",
      config,
      "--watch",
    ],
    {
      cwd: webui,
      env,
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  const stopped = new Promise<void>((resolve) =>
    child.once("close", () => resolve()),
  );
  let log = "";
  let failure: Error | undefined;
  child.stdout?.on("data", (part) => {
    log += part;
  });
  child.stderr?.on("data", (part) => {
    log += part;
  });
  child.once("error", (error) => {
    failure = error;
  });
  lifetime.defer(async () => {
    if (child.exitCode === null && child.signalCode === null)
      child.kill("SIGTERM");
    using _timeout = setTimeout(() => child.kill("SIGKILL"), 5_000);
    await stopped;
    expect(child.exitCode !== null || child.signalCode !== null).toBe(true);
    await test
      .info()
      .attach("codegen-watch-log", { body: log, contentType: "text/plain" });
    await readFile(generated, "utf8").then(
      (body) =>
        test.info().attach("codegen-generated-types", {
          body,
          contentType: "text/plain",
        }),
      () => undefined,
    );
  });
  await expect
    .poll(
      () => {
        if (failure) throw failure;
        if (child.exitCode !== null)
          throw new Error(`Local codegen exited (${child.exitCode}): ${log}`);
        return /Watching for changes/i.test(log);
      },
      { timeout: 30_000 },
    )
    .toBe(true);
  expect(await readFile(generated, "utf8")).toContain("LocalCodegenQuery");
  let probe = 0;
  // The CLI prints its banner before subscribing. A real file-event ACK,
  // rather than a sleep or that banner, establishes watcher readiness.
  await expect
    .poll(
      async () => {
        await writeFile(
          schema,
          `type Query { marker: String! }\n# readiness ${probe++}\n`,
        );
        return readFile(ready, "utf8").then(
          (value) => value === "ready",
          () => false,
        );
      },
      { timeout: 15_000 },
    )
    .toBe(true);
  await writeFile(schema, "type Query { marker: Int! changed: Boolean! }\n");
  await expect
    .poll(
      async () =>
        /type LocalCodegenQuery\b[^}]*\bmarker: number\b/s.test(
          await readFile(generated, "utf8"),
        ),
      { timeout: 15_000 },
    )
    .toBe(true);
  await writeFile(
    document,
    "const query = gql`query LocalCodegen { marker changed }`;\n",
  );
  await expect
    .poll(
      async () => {
        if (failure) throw failure;
        return /type LocalCodegenQuery\b[^}]*\bchanged\b/s.test(
          await readFile(generated, "utf8"),
        );
      },
      { timeout: 15_000 },
    )
    .toBe(true);
});

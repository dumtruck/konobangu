import { readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { expect } from "@playwright/test";
import { type } from "arktype";
import { test } from "./support/deployment-fixture.ts";
import { nativeBasicChallenge } from "./support/native-browser.ts";

const WorkerFrames = type({
  url: "string",
  direction: "'request' | 'reply'",
  method: "string | null",
  instance: "number",
  id: "string | null",
  messageType: "number | null",
  errorPresent: "boolean",
}).array();

function pairedOperations(operations: typeof WorkerFrames.infer, name: string) {
  return operations.filter(
    (request) =>
      request.url.includes(name) &&
      request.direction === "request" &&
      request.messageType === 0 &&
      request.method !== null &&
      request.id !== null &&
      !/initialize|ping/i.test(request.method) &&
      operations.some(
        (reply) =>
          reply.instance === request.instance &&
          reply.url === request.url &&
          reply.direction === "reply" &&
          reply.messageType === 1 &&
          reply.id === request.id &&
          !reply.errorPresent,
      ),
  );
}

test("the full development app performs GraphQL/JSON/editor worker operations and preserves editor input during Fast Refresh", async ({
  page,
  context,
  deployment,
}) => {
  test.setTimeout(90_000);
  const { devUrl, devIssuer } = deployment.descriptor;
  const pageErrors: string[] = [];
  page.on("pageerror", (error) => pageErrors.push(error.message));
  await page.addInitScript(() => {
    const frames: {
      url: string;
      direction: "request" | "reply";
      method: string | null;
      instance: number;
      id: string | null;
      messageType: number | null;
      errorPresent: boolean;
    }[] = [];
    let nextInstance = 0;
    Object.defineProperty(window, "__konobanguWorkerProof", { value: frames });
    const NativeWorker = window.Worker;
    window.Worker = class extends NativeWorker {
      readonly proofUrl: string;
      readonly proofInstance: number;
      constructor(url: string | URL, options?: WorkerOptions) {
        super(url, options);
        this.proofUrl = String(url);
        this.proofInstance = nextInstance++;
        this.addEventListener("message", (event: MessageEvent<unknown>) => {
          const data = event.data;
          const object =
            typeof data === "object" && data !== null ? data : null;
          frames.push({
            url: this.proofUrl,
            instance: this.proofInstance,
            direction: "reply",
            method: null,
            id:
              object && "seq" in object && typeof object.seq === "string"
                ? object.seq
                : null,
            messageType:
              object && "type" in object && typeof object.type === "number"
                ? object.type
                : null,
            errorPresent:
              !!object &&
              "err" in object &&
              object.err !== undefined &&
              object.err !== null,
          });
        });
      }
      postMessage(
        message: unknown,
        options?: Transferable[] | StructuredSerializeOptions,
      ): void {
        const method =
          typeof message === "object" &&
          message !== null &&
          "method" in message &&
          typeof message.method === "string"
            ? message.method
            : null;
        const object =
          typeof message === "object" && message !== null ? message : null;
        frames.push({
          url: this.proofUrl,
          instance: this.proofInstance,
          direction: "request",
          method,
          id:
            object && "req" in object && typeof object.req === "string"
              ? object.req
              : null,
          messageType:
            object && "type" in object && typeof object.type === "number"
              ? object.type
              : null,
          errorPresent: false,
        });
        if (Array.isArray(options)) super.postMessage(message, options);
        else super.postMessage(message, options);
      }
    };
  });
  const fontResponse = page.waitForResponse((response) =>
    new URL(response.url()).pathname.endsWith("/codicon.ttf"),
  );
  {
    await using challenge = await nativeBasicChallenge(
      context,
      page,
      devIssuer,
    );
    await page.goto(`${devUrl}/playground/graphql-api`);
    try {
      await expect(
        page.locator('[data-id="graphiql-playground-container"]'),
      ).toBeVisible({ timeout: 30_000 });
    } catch (cause) {
      throw new Error(
        `Full development app did not start: ${pageErrors.join("; ")}`,
        { cause },
      );
    }
    expect(challenge.answered).toBe(1);
  }
  await expect(page.locator("button button")).toHaveCount(0);
  const operationEditor = page.getByRole("region", {
    name: "Operation Editor",
    exact: true,
  });
  const query = operationEditor.locator("textarea:visible");
  try {
    await expect(query).toBeVisible({ timeout: 20_000 });
  } catch (cause) {
    throw new Error(
      `Development editor did not initialize: ${pageErrors.join("; ")}`,
      { cause },
    );
  }
  await query.press("ControlOrMeta+A");
  await page.keyboard.insertText("query WorkerProof { definitelyNotAField }");
  await expect(
    operationEditor.locator(".squiggly-error").first(),
  ).toBeVisible();
  await query.press("End");
  await query.press("Control+Space");
  await page.getByRole("button", { name: "Variables", exact: true }).click();
  const variablesEditor = page.getByRole("region", {
    name: "Variables",
    exact: true,
  });
  const variables = variablesEditor.locator("textarea:visible");
  await variables.press("ControlOrMeta+A");
  await page.keyboard.insertText('{"broken": }');
  await expect(
    variablesEditor.locator(".squiggly-error").first(),
  ).toBeVisible();
  const font = await fontResponse;
  expect(font.status()).toBe(200);
  const fontBytes = (await font.body()).byteLength;
  expect(fontBytes).toBeGreaterThan(0);
  const fontLoaded = await page.evaluate(async () => {
    await document.fonts.ready;
    return [...document.fonts].some(
      (face) =>
        face.family.replaceAll('"', "").replaceAll("'", "") === "codicon" &&
        face.status === "loaded",
    );
  });
  expect(fontLoaded).toBe(true);
  const fontProof = { url: font.url(), bytes: fontBytes, loaded: fontLoaded };
  const frames = async () =>
    WorkerFrames.assert(
      await page.evaluate(() => Reflect.get(window, "__konobanguWorkerProof")),
    );
  await expect
    .poll(async () => {
      const operations = await frames();
      return ["graphql.worker", "json.worker", "editor.worker"].map(
        (name) => pairedOperations(operations, name).length > 0,
      );
    })
    .toEqual([true, true, true]);
  const observations = await frames();
  const proof = ["graphql.worker", "json.worker", "editor.worker"].map(
    (name) => {
      const pairs = pairedOperations(observations, name);
      return {
        worker: name,
        completedOperations: pairs.length,
        requests: pairs.map(({ instance, id, method, url }) => ({
          instance,
          id,
          method,
          url,
        })),
      };
    },
  );
  await writeFile(
    join(deployment.descriptor.directory, "development-worker-proof.json"),
    JSON.stringify({ workers: proof, font: fontProof }, null, 2),
  );
  await test.info().attach("completed-monaco-worker-operations", {
    body: JSON.stringify(proof, null, 2),
    contentType: "application/json",
  });

  const devSource = deployment.descriptor.devSourceRoot;
  if (!devSource)
    throw new Error("The framework must own a development source projection");
  const source = join(devSource, "src/components/layout/nav-user.tsx");
  const original = await readFile(source, "utf8");
  const updated = original.replace(
    '<SidebarMenuButton size="lg">',
    '<SidebarMenuButton size="lg" data-hmr-proof="updated">',
  );
  if (original === updated)
    throw new Error("Development HMR marker target was not found");
  await writeFile(source, updated);
  await expect(page.locator('[data-hmr-proof="updated"]')).toBeVisible();
  await expect(operationEditor).toContainText("WorkerProof");
  await expect(
    page.locator('[data-id="graphiql-playground-container"]'),
  ).toHaveCount(1);
  expect(new URL(page.url()).pathname).toBe("/playground/graphql-api");
  const entry = join(devSource, "src/app/entry.tsx");
  const entrySource = await readFile(entry, "utf8");
  const reloaded = page.waitForEvent("domcontentloaded");
  await writeFile(
    entry,
    `${entrySource}\n// Framework-owned root HMR verification.\n`,
  );
  await reloaded;
  await expect(operationEditor).toContainText("WorkerProof");
  await expect(page.locator("#app")).toHaveCount(1);
  await expect(
    page.locator('[data-id="graphiql-playground-container"]'),
  ).toHaveCount(1);
  await deployment.restartDev();
  await page.reload();
  await expect(operationEditor).toContainText("WorkerProof");
  await expect(page.locator('[data-hmr-proof="updated"]')).toBeVisible();
  await expect(
    page.locator('[data-id="graphiql-playground-container"]'),
  ).toHaveCount(1);
});

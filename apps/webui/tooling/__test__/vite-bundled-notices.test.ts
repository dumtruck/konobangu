import {
  mkdirSync,
  mkdtempDisposableSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { build } from "vite";
import { describe, expect, it } from "vitest";
import { createBundledNotices } from "../vite-bundled-notices.ts";

function dependency(directory: string, name: string, text: string) {
  const output = join(directory, "node_modules", name);
  mkdirSync(output, { recursive: true });
  writeFileSync(
    join(output, "package.json"),
    JSON.stringify({
      name,
      version: "1.0.0",
      main: "index.js",
      private: true,
      license: "MIT",
    }),
  );
  writeFileSync(
    join(output, "index.js"),
    `export const value = ${JSON.stringify(name)};`,
  );
  writeFileSync(join(output, "LICENSE"), `${text} license text`);
  writeFileSync(join(output, "NOTICE"), `${text} copyright notice`);
}

describe("license collection across bundled worker graphs", () => {
  it("merges worker-only private dependencies and complete license/notice text", async () => {
    using temporary = mkdtempDisposableSync(
      join(tmpdir(), "konobangu-notices-"),
    );
    const root = temporary.path;
    dependency(root, "main-license-fixture", "main");
    dependency(root, "worker-only-license-fixture", "worker");
    writeFileSync(
      join(root, "index.ts"),
      'import {value} from "main-license-fixture"; export const marker=value; export function worker(){ return new Worker(new URL("./worker.ts", import.meta.url)); }',
    );
    writeFileSync(
      join(root, "worker.ts"),
      'import {value} from "worker-only-license-fixture"; self.onmessage=()=>self.postMessage(value);',
    );
    const notices = createBundledNotices();
    await build({
      configFile: false,
      root,
      logLevel: "silent",
      plugins: [notices.collector(), notices.finalize()],
      worker: { plugins: () => [notices.collector()] },
      build: {
        lib: { entry: join(root, "index.ts"), formats: ["es"] },
        outDir: join(root, "dist"),
      },
    });
    const report: unknown = JSON.parse(
      readFileSync(join(root, "dist", "THIRD-PARTY-NOTICES.json"), "utf8"),
    );
    expect(report).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          name: "main-license-fixture",
          licenseText: "main license text",
          noticeText: "main copyright notice",
        }),
        expect.objectContaining({
          name: "worker-only-license-fixture",
          licenseText: "worker license text",
          noticeText: "worker copyright notice",
        }),
      ]),
    );
    const markdown = readFileSync(
      join(root, "dist", "THIRD-PARTY-NOTICES.md"),
      "utf8",
    );
    expect(markdown).toContain("worker copyright notice");
    expect(markdown).toContain("main license text");
  });
});

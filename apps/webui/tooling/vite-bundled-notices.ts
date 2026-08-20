import { mkdirSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { join } from "node:path";
import type { Dependency, Options } from "rollup-plugin-license";

// The package runtime is CommonJS, although its declaration uses ESM default syntax.
const license: (
  options: Options,
) => ReturnType<typeof import("rollup-plugin-license").default> = createRequire(
  import.meta.url,
)("rollup-plugin-license");

import type { Plugin } from "vite";

export type BundledNotice = {
  name: string;
  version: string;
  license: string | null;
  licenseText: string | null;
  noticeText: string | null;
  text: string;
};

/** Coordinate mature collectors across main, editor and private-worker builds. */
export function createBundledNotices(): {
  collector(): Plugin;
  finalize(): Plugin;
} {
  const dependencies = new Map<string, BundledNotice>();
  const collect = (items: Dependency[]) => {
    for (const dependency of items) {
      if (!dependency.name || !dependency.version)
        throw new Error("A bundled dependency has no package identity");
      const notice: BundledNotice = {
        name: dependency.name,
        version: dependency.version,
        license: dependency.license,
        licenseText: dependency.licenseText?.replaceAll("\r\n", "\n") ?? null,
        noticeText: dependency.noticeText?.replaceAll("\r\n", "\n") ?? null,
        text: dependency.text().replaceAll("\r\n", "\n"),
      };
      const key = `${notice.name}@${notice.version}`,
        previous = dependencies.get(key);
      if (
        previous &&
        (previous.license !== notice.license ||
          previous.licenseText !== notice.licenseText ||
          previous.noticeText !== notice.noticeText)
      )
        throw new Error(`Conflicting bundled license information: ${key}`);
      dependencies.set(key, notice);
    }
  };
  return {
    collector() {
      const collector = license({
        thirdParty: {
          includePrivate: true,
          multipleVersions: true,
          output: collect,
        },
      });
      const render =
        typeof collector.renderChunk === "function"
          ? collector.renderChunk
          : collector.renderChunk?.handler;
      const generate =
        typeof collector.generateBundle === "function"
          ? collector.generateBundle
          : collector.generateBundle?.handler;
      if (!render || !generate)
        throw new Error(
          "The license collector is missing its documented bundle hooks",
        );
      // The collector uses these two context-free hooks only. Adapt them rather
      // than asserting the entire legacy Rollup plugin into a Rolldown plugin.
      return {
        name: collector.name,
        renderChunk(code, chunk, output) {
          Reflect.apply(render, this, [code, chunk, output]);
          return null;
        },
        generateBundle(output, bundle, writing) {
          Reflect.apply(generate, this, [output, bundle, writing]);
        },
      };
    },
    finalize(): Plugin {
      let output: string;
      return {
        name: "konobangu:bundled-notices",
        apply: "build",
        configResolved(config) {
          output = config.build.outDir;
        },
        // Sequential completion follows PWA's private-worker sub-build.
        closeBundle: {
          order: "post",
          sequential: true,
          handler() {
            const notices = [...dependencies.values()].sort((a, b) =>
              `${a.name}@${a.version}`.localeCompare(
                `${b.name}@${b.version}`,
                "en",
              ),
            );
            if (!notices.length)
              throw new Error(
                "Bundled license collectors produced no dependency notices",
              );
            mkdirSync(output, { recursive: true });
            writeFileSync(
              join(output, "THIRD-PARTY-NOTICES.json"),
              `${JSON.stringify(notices, null, 2)}\n`,
            );
            writeFileSync(
              join(output, "THIRD-PARTY-NOTICES.md"),
              `# Third-party notices\n\n${notices.map((notice) => notice.text).join("\n\n---\n\n")}\n`,
            );
          },
        },
      };
    },
  };
}

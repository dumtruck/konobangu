import {
  cp,
  mkdir,
  mkdtemp,
  readdir,
  readFile,
  rm,
  symlink,
} from "node:fs/promises";
import { basename, dirname, join, relative, resolve } from "node:path";
import { type } from "arktype";

export type DevSourceCopy = {
  readonly repoRoot: string;
  readonly appRoot: string;
  [Symbol.asyncDispose](): Promise<void>;
};

const sourceFiles = [
  "components.json",
  "graphql-codegen.ts",
  "index.html",
  "package.json",
  "playwright.config.ts",
  "postcss.config.mjs",
  "tsconfig.json",
  "tsconfig.scripts.json",
  "tsr.config.json",
  "vite.config.ts",
  "vitest.config.ts",
];

const rootFiles = [
  "biome.jsonc",
  "konobangu-metadata.toml",
  "package.json",
  "pnpm-workspace.yaml",
  "tsconfig.base.json",
  "tsconfig.json",
];

const excludedSourceNames = new Set([
  ".cache",
  ".tanstack",
  "dist",
  "dist-tsc",
  "node_modules",
  "output",
  "secrets",
]);
const RootProjects = type({ references: type({ path: "string" }).array() });

/** Keep source edits and generated WebUI artifacts inside one owned shadow repo. */
export async function createDevSourceCopy(
  projectRoot: string,
): Promise<DevSourceCopy> {
  await using ownership = new AsyncDisposableStack();
  const sourceRoot = resolve(projectRoot);
  const temporary = join(sourceRoot, "temp");
  await mkdir(temporary, { recursive: true });
  const repoRoot = await mkdtemp(join(temporary, "dev-repo-"));
  ownership.defer(() => rm(repoRoot, { recursive: true, force: true }));
  const appRoot = join(repoRoot, "apps/webui");
  const sourceApp = join(sourceRoot, "apps/webui");
  await mkdir(appRoot, { recursive: true });

  for (const directory of ["src", "public"]) {
    await cp(join(sourceApp, directory), join(appRoot, directory), {
      recursive: true,
      verbatimSymlinks: true,
      filter(path) {
        const name = basename(path);
        return !name.startsWith(".env") && !excludedSourceNames.has(name);
      },
    });
  }
  for (const file of sourceFiles)
    await cp(join(sourceApp, file), join(appRoot, file));

  const project = async (path: string, type: "dir" | "file") => {
    const destination = join(repoRoot, path);
    await symlink(
      relative(dirname(destination), join(sourceRoot, path)),
      destination,
      type,
    );
  };
  for (const directory of [
    "packages",
    "node_modules",
    "apps/webui/node_modules",
    "apps/webui/tooling",
    "apps/webui/tests",
  ])
    await project(directory, "dir");
  for (const app of await readdir(join(sourceRoot, "apps"), {
    withFileTypes: true,
  })) {
    if (app.name !== "webui" && (app.isDirectory() || app.isSymbolicLink()))
      await project(join("apps", app.name), "dir");
  }
  const projects = RootProjects.assert(
    JSON.parse(await readFile(join(sourceRoot, "tsconfig.json"), "utf8")),
  );
  const projected = new Set(["apps", "packages", "node_modules"]);
  for (const reference of projects.references) {
    const directory = reference.path.replace(/^\.\//, "").split("/")[0];
    if (!projected.has(directory)) {
      await project(directory, "dir");
      projected.add(directory);
    }
  }
  for (const file of rootFiles) await project(file, "file");

  // Relative links borrow dependencies without granting cleanup ownership of them.
  const owner = ownership.move();
  return {
    repoRoot,
    appRoot,
    [Symbol.asyncDispose]: () => owner.disposeAsync(),
  };
}

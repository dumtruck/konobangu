import { existsSync } from "node:fs";
import { relative, resolve, sep } from "node:path";
import type { CodegenConfig } from "@graphql-codegen/cli";
import { pascalCase } from "change-case-all";

const source =
  process.env.KONOBANGU_GRAPHQL_SCHEMA ??
  resolve(import.meta.dirname, "../recorder/.cache/graphql/schema.graphql");
const remote = /^https?:\/\//i.test(source);
if (!remote && !existsSync(source))
  throw new Error(
    "GraphQL SDL is missing. Start the development recorder to export apps/recorder/.cache/graphql/schema.graphql, or set KONOBANGU_GRAPHQL_SCHEMA to an existing SDL file or explicit HTTP endpoint.",
  );

const config: CodegenConfig = {
  schema: remote
    ? {
        [source]: {
          headers: process.env.KONOBANGU_CODEGEN_AUTHORIZATION
            ? { Authorization: process.env.KONOBANGU_CODEGEN_AUTHORIZATION }
            : {},
        },
      }
    : source,
  documents: ["src/**/*.{ts,tsx}", "!src/**/*.test.{ts,tsx}"],
  hooks: { afterAllFileWrite: ["biome check --write src/infra/graphql/gql/"] },
  generates: {
    "./src/infra/graphql/gql/": {
      // Codegen matches native watch events relative to cwd, even when schema
      // loading uses an absolute path. Keep that matching contract explicit.
      watchPattern: remote
        ? []
        : [relative(process.cwd(), resolve(source)).split(sep).join("/")],
      plugins: [],
      preset: "client",
      presetConfig: {
        gqlTagName: "gql",
      },
      config: {
        enumType: "const",
        useTypeImports: true,
        scalars: {
          SubscriberTaskType: {
            input: "recorder/bindings/SubscriberTaskInput#SubscriberTaskInput",
            output: "recorder/bindings/SubscriberTaskType#SubscriberTaskType",
          },
        },
        namingConvention: (str: string) => {
          if (str === "Array") {
            return "SeaOrmArray";
          }
          return pascalCase(str);
        },
      },
    },
  },
};

export default config;

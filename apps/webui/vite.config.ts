import { tanstackRouter } from "@tanstack/router-plugin/vite";
import react from "@vitejs/plugin-react";
import { defaultClientConditions, defineConfig } from "vite";
import { createBundledNotices } from "./tooling/vite-bundled-notices.ts";

// The login mode is published at runtime through konobangu.config.json and
// GET auth/config, so one build serves every deployment and no auth setting is
// compiled in.
export default defineConfig(() => {
  const notices = createBundledNotices();
  return {
    resolve: {
      tsconfigPaths: true,
      conditions: ["monorepo-tsc", ...defaultClientConditions],
      // Schema objects cross Apollo, GraphiQL and the Monaco language service.
      dedupe: ["graphql", "react", "react-dom"],
      // GraphiQL only imports c; React 19 provides the official compiler runtime.
      alias: { "react-compiler-runtime": "react/compiler-runtime" },
    },
    worker: { plugins: () => [notices.collector()] },
    plugins: [
      notices.collector(),
      tanstackRouter({ target: "react", autoCodeSplitting: true }),
      react(),
      notices.finalize(),
    ],
    optimizeDeps: {
      // Bootstrap and router imports are deferred. Scan their source entries
      // before serving so a cold dev page cannot outlive its dependency hashes.
      entries: [
        "index.html",
        "src/app/entry.tsx",
        "src/presentation/routes/**/*.tsx",
      ],
      // Official worker imports need their ESM source graph. Prebundle its
      // CJS-only leaves as recommended for excluded ESM dependencies by Vite.
      include: [
        "graphql",
        "react-compiler-runtime",
        "graphiql > @graphiql/react > get-value",
        "graphiql > @graphiql/react > set-value",
        "graphiql > @graphiql/react > graphql-language-service > nullthrows",
        "graphiql > @graphiql/react > monaco-graphql > picomatch-browser",
        "graphiql > @graphiql/react > prettier/parser-graphql",
        "graphiql > @graphiql/react > prettier/parser-babel",
        "graphiql > @graphiql/plugin-doc-explorer > @headlessui/react > use-sync-external-store/with-selector",
        "graphiql > @graphiql/react > monaco-graphql/esm/monaco-editor.js",
        "graphiql > @graphiql/react > monaco-graphql/esm/lite.js",
      ],
      // The official worker setup imports `?worker` modules. That suffix is a
      // source-graph feature, so the dependency optimizer cannot resolve it from
      // inside a prebundled dependency; keep these in the source graph instead.
      exclude: ["graphiql", "@graphiql/react"],
    },
    build: {
      target: ["chrome140", "firefox140", "safari18.6"],
      cssTarget: ["chrome140", "firefox140", "safari18.6"],
      // Monaco is a deferred 2.8 MB editor chunk; keep a finite 3 MB raw budget.
      // Cold-start tests separately reject eager Temporal polyfill loading.
      chunkSizeWarningLimit: 3000,
    },
    server: {
      host: "0.0.0.0",
      port: 5000,
      proxy: {
        "/api": { target: "http://127.0.0.1:5001", changeOrigin: false },
      },
    },
  };
});

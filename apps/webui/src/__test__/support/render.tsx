import type { SecuritydeptInjectorTrait } from "@securitydept/client";
import { SecuritydeptProvider } from "@securitydept/client-react";
import { type RenderResult, render } from "@testing-library/react";
import { type ReactNode, StrictMode } from "react";
import { GraphqlProvider } from "@/infra/graphql/provider";

/** Mounts a tree under the same providers the entry installs. */
export function renderInEnvironment(
  app: { readonly injector: SecuritydeptInjectorTrait },
  ui: ReactNode,
  options: { apollo?: boolean; strict?: boolean } = {},
): Omit<RenderResult, "rerender"> & { rerender(next: ReactNode): void } {
  const wrap = (node: ReactNode) => {
    const provided = (
      <SecuritydeptProvider injector={app.injector}>
        {options.apollo ? <GraphqlProvider>{node}</GraphqlProvider> : node}
      </SecuritydeptProvider>
    );
    return options.strict ? <StrictMode>{provided}</StrictMode> : provided;
  };
  const view = render(wrap(ui));
  return { ...view, rerender: (next: ReactNode) => view.rerender(wrap(next)) };
}

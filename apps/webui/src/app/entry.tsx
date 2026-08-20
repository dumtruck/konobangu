import { SecuritydeptProvider } from "@securitydept/client-react";
import { createRouter, RouterProvider } from "@tanstack/react-router";
import { Suspense } from "react";
import { createRoot } from "react-dom/client";
import { AuthProvider } from "@/app/auth/provider";
import type { BootstrapResult } from "@/app/bootstrap";
import { createKonobanguEnvironmentForNativeWeb } from "@/app/environment/native-web";
import { AppNotFoundComponent } from "@/components/layout/app-not-found";
import { AuthService } from "@/domains/auth/auth.service";
import { GraphQLService } from "@/infra/graphql/graphql.service";
import { GraphqlProvider } from "@/infra/graphql/provider";
import { ThemeService } from "@/infra/styles/theme.service";
import { routeTree } from "@/presentation/routeTree.gen";
import "../app.css";

const router = createRouter({
  routeTree,
  defaultPreload: "intent",
  defaultStaleTime: 5000,
  scrollRestoration: true,
  defaultNotFoundComponent: AppNotFoundComponent,
  notFoundMode: "root",
  context: {},
});
declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

export function startApp({ config, auth }: BootstrapResult): Disposable {
  using ownership = new DisposableStack();
  const environment = ownership.use(
    createKonobanguEnvironmentForNativeWeb({
      window,
      config,
      auth,
      appRouter: router,
    }),
  );
  // Resolve the fallible services before anything is mounted or published.
  environment.injector.get(ThemeService);
  environment.injector.get(AuthService);
  environment.injector.get(GraphQLService);
  const rootElement = document.getElementById("app");
  if (!rootElement) throw new Error("Application mount element is missing");
  const root = createRoot(rootElement);
  ownership.defer(() => root.unmount());
  // Publish only after fallible initialization; stopped roots retain no injector.
  router.update({
    context: {
      injector: environment.injector,
      securitydeptInjector: environment.injector,
    },
  });
  ownership.defer(() => router.update({ context: {} }));
  root.render(
    <SecuritydeptProvider injector={environment.injector}>
      <Suspense>
        <AuthProvider router={router}>
          <GraphqlProvider>
            <RouterProvider router={router} />
          </GraphqlProvider>
        </AuthProvider>
      </Suspense>
    </SecuritydeptProvider>,
  );
  return ownership.move();
}

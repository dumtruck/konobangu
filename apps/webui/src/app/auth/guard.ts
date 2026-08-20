import type { ParsedLocation } from "@tanstack/react-router";
import { AuthService } from "@/domains/auth/auth.service";
import {
  type RouterContext,
  requireRouterInjector,
} from "@/infra/routes/traits";

export const beforeLoadGuard = async ({
  context,
  location,
}: {
  context: RouterContext;
  location: ParsedLocation;
}) => {
  const authService = requireRouterInjector(context).get(AuthService);
  // A failed API principal check must surface instead of starting a login.
  await authService.refresh();
  const check = authService.check.get();
  if (check.status === "error") throw check.error;
  if (!authService.isAuthenticated.get()) {
    // The router only produces normalized paths of its own matched routes.
    await authService.login(location.href);
    throw new Error("Authentication redirect started.");
  }
};

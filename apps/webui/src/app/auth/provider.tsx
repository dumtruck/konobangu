import { readErrorPresentationDescriptor } from "@securitydept/client";
import { useSignal } from "@securitydept/client-react";
import type { AnyRouter } from "@tanstack/react-router";
import { Fragment, type ReactNode, useEffect } from "react";
import { AuthService } from "@/domains/auth/auth.service";
import { useInject } from "@/infra/di/inject";
import { AUTH_CALLBACK_PATH } from "./setup";

export function useAuth() {
  const authService = useInject(AuthService);
  return {
    authService,
    authData: useSignal(authService.userData),
    isAuthenticated: useSignal(authService.isAuthenticated),
    check: useSignal(authService.check),
  };
}

/**
 * Everything the authentication domain needs from the React root: the first
 * identity read, route revalidation when the identity changes, and the
 * presentation of a failed identity check.
 */
export function AuthProvider({
  router,
  children,
}: {
  router: AnyRouter;
  children: ReactNode;
}) {
  const auth = useInject(AuthService);
  const check = useSignal(auth.check);
  useEffect(() => {
    // Public callback routes initialize only through the SDK callback hook.
    if (router.state.location.pathname !== AUTH_CALLBACK_PATH)
      void auth.refresh().catch(() => undefined);
    const subscription = auth.identityKeyChanges.subscribe(
      () => void router.invalidate().catch(() => undefined),
    );
    return () => subscription.unsubscribe();
  }, [auth, router]);
  const presentation =
    check.status === "error"
      ? readErrorPresentationDescriptor(check.error)
      : null;
  return (
    <>
      {presentation && (
        <p role="alert">
          {presentation.title}: {presentation.description}
        </p>
      )}
      {children}
    </>
  );
}

/**
 * The private tree owns local form/view state that no service can reset, so it
 * remounts for a new identity while the outer router stays stable.
 */
export function PrivateIdentityView({ children }: { children: ReactNode }) {
  const identity = useSignal(useInject(AuthService).identityKey);
  return identity ? <Fragment key={identity}>{children}</Fragment> : null;
}

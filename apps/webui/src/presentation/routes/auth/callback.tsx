import {
  ResourceStatus,
  RouterNavigationIntent,
  RouterNavigationMode,
  UriReferenceString,
} from "@securitydept/client";
import { describeFrontendOidcModeCallbackError } from "@securitydept/token-set-context-client/frontend-oidc-mode";
import { OidcModeCallbackHandlingKind } from "@securitydept/token-set-context-client/orchestration";
import { useTokenSetFrontendCallback } from "@securitydept/token-set-context-client-react";
import { createFileRoute } from "@tanstack/react-router";
import { useEffect, useRef } from "react";
import { useAuth } from "@/app/auth/provider";
import { OIDC_CLIENT_KEY } from "@/app/auth/setup";
import { APP_ROUTER } from "@/app/environment/traits";
import { useInject } from "@/infra/di/inject";
export const Route = createFileRoute("/auth/callback")({
  component: AuthCallbackRoute,
});
const query = () => ({ clientKey: OIDC_CLIENT_KEY });
export function AuthCallbackRoute() {
  const { state } = useTokenSetFrontendCallback({ clientQuery: query });
  const router = useInject(APP_ROUTER);
  const { authService } = useAuth();
  const handled = useRef(false);
  useEffect(() => {
    if (
      state.status !== ResourceStatus.Resolved ||
      state.value.kind !== OidcModeCallbackHandlingKind.Handled ||
      handled.current ||
      authService.lifetimeSignal.aborted
    )
      return;
    // The SDK returns the value this application stored when it started the
    // flow; it never comes from the callback URL.
    const target = state.value.result.postAuthRedirectUri ?? "/";
    handled.current = true;
    void router.navigate({
      url: UriReferenceString.parse(target),
      intent: RouterNavigationIntent.PostAuthRedirect,
      mode: RouterNavigationMode.Replace,
    });
  }, [router, state, authService]);
  if (
    state.status === ResourceStatus.Error ||
    state.status === ResourceStatus.LoadingError
  ) {
    const descriptor = describeFrontendOidcModeCallbackError(state.error, {
      recoveryLinks: { restart_flow: "/" },
      recoveryLabels: { restart_flow: "Return to the application" },
    });
    return (
      <p role="alert">
        {descriptor.description} <a href="/">Return to the application</a>
      </p>
    );
  }
  if (
    state.status === ResourceStatus.Idle ||
    (state.status === ResourceStatus.Resolved &&
      state.value.kind !== OidcModeCallbackHandlingKind.Handled)
  )
    return (
      <p>
        No authentication response is available.{" "}
        <a href="/">Return to the application</a>.
      </p>
    );
  return <p>Completing sign-in…</p>;
}

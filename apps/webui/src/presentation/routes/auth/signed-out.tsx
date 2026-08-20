import { useSignal } from "@securitydept/client-react";
import { createFileRoute } from "@tanstack/react-router";
import { useEffect } from "react";
import { useAuth } from "@/app/auth/provider";
import { Button } from "@/components/ui/button";

export const Route = createFileRoute("/auth/signed-out")({
  component: SignedOutRoute,
});
const returnPath = "/subscriptions/manage";
/** Shows the authentication domain's sign-in flow while this page is mounted. */
export function SignedOutRoute() {
  const { authService } = useAuth();
  const flow = authService.signIn;
  const { pending, error } = useSignal(flow.state);
  useEffect(() => {
    flow.connect();
    return () => flow.disconnect();
  }, [flow]);
  return (
    <main className="mx-auto max-w-xl p-8">
      <h1>Local access has ended</h1>
      <p>
        Tokens and private application data have been cleared locally. Your
        identity provider may retain its own login credentials.
      </p>
      {error && <p role="alert">{error}</p>}
      <Button disabled={pending} onClick={() => flow.start(returnPath)}>
        Sign in again
      </Button>
    </main>
  );
}

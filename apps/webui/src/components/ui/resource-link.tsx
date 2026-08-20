import { useSignal } from "@securitydept/client-react";
import { type ComponentProps, useEffect, useMemo } from "react";
import { RESOURCE_SERVICE } from "@/app/resource/resource-grants";
import { useInject } from "@/infra/di/inject";
import { Button } from "./button";

/**
 * Renders the resource domain's binding for one locator. This component only
 * connects it while mounted and shows its state; derivation, renewal and
 * retry belong to the binding.
 */
export function ResourceLink({
  href,
  children,
  ...props
}: Omit<ComponentProps<"a">, "href"> & { href: string }) {
  const resources = useInject(RESOURCE_SERVICE);
  const binding = useMemo(() => resources.bind(href), [resources, href]);
  const state = useSignal(binding.state);
  useEffect(() => {
    binding.connect();
    return () => binding.disconnect();
  }, [binding]);
  if (state.status === "failed")
    return (
      <Button variant="link" size="sm" onClick={() => binding.retry()}>
        Retry {children}
      </Button>
    );
  return (
    <a
      {...props}
      href={state.status === "ready" ? state.url : undefined}
      target={props.target ?? "_blank"}
      rel="noopener noreferrer"
      referrerPolicy="no-referrer"
      aria-disabled={state.status !== "ready"}
    >
      {children}
    </a>
  );
}

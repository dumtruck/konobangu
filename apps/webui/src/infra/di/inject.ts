import type { SecuritydeptDependencyToken } from "@securitydept/client";
import { useSecuritydeptContext } from "@securitydept/client-react";
import { useMemo } from "react";

/** The token carries the type, so no consumer casts the resolved value. */
export function useInject<T>(token: SecuritydeptDependencyToken<T>): T {
  const injector = useSecuritydeptContext();
  return useMemo(() => injector.get(token), [injector, token]);
}

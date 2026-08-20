import { ApolloProvider } from "@apollo/client/react";
import { useSignal } from "@securitydept/client-react";
import type { ReactNode } from "react";
import { useInject } from "@/infra/di/inject";
import { GraphQLService } from "./graphql.service";

/** React follows the service's current client; it never decides to replace it. */
export function GraphqlProvider({ children }: { children: ReactNode }) {
  const client = useSignal(useInject(GraphQLService).client);
  return <ApolloProvider client={client}>{children}</ApolloProvider>;
}

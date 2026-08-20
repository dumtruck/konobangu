// @vitest-environment jsdom
import { useApolloClient } from "@apollo/client/react";
import { act, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { createSignedInFixture } from "@/__test__/support/fixture";
import { renderInEnvironment } from "@/__test__/support/render";
import { GraphQLService } from "../graphql.service";

describe("the Apollo client at the React root", () => {
  it("follows the client the service publishes", async () => {
    const app = await createSignedInFixture();
    const service = app.injector.get(GraphQLService);
    const seen: unknown[] = [];
    function Consumer() {
      const client = useApolloClient();
      if (seen.at(-1) !== client) seen.push(client);
      return <output>{seen.length}</output>;
    }
    renderInEnvironment(app, <Consumer />, { apollo: true });
    expect(seen).toEqual([service._apollo]);
    await act(() => app.auth.refresh());
    expect(seen).toHaveLength(1);
    await act(() => app.identity("B"));
    expect(seen).toHaveLength(2);
    expect(seen[1]).toBe(service._apollo);
    expect(screen.getByRole("status").textContent).toBe("2");
  });
});

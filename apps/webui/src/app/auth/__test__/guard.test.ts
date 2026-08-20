// @vitest-environment jsdom

import { describe, expect, it, vi } from "vitest";
import {
  createKonobanguFixture,
  type KonobanguFixtureOptions,
  parsedLocation,
  principalPayload,
  response,
} from "@/__test__/support/fixture";
import { beforeLoadGuard } from "../guard";

function fixture(
  status: number,
  options: Pick<KonobanguFixtureOptions, "auth"> = {},
) {
  const app = createKonobanguFixture({
    ...options,
    handler: () => response(status, status === 200 ? principalPayload() : null),
  });
  return {
    ...app,
    args: { context: { injector: app.injector }, location: parsedLocation() },
  };
}
describe("private route guard", () => {
  it("waits for server authentication before allowing private routes", async () => {
    const { args } = fixture(200);
    await expect(beforeLoadGuard(args)).resolves.toBeUndefined();
  });
  it("redirects only a confirmed 401 and preserves the local return path", async () => {
    const { args, auth } = fixture(401);
    const login = vi.spyOn(auth, "login").mockResolvedValue();
    await expect(beforeLoadGuard(args)).rejects.toThrow("redirect started");
    expect(login).toHaveBeenCalledWith("/subscriptions");
  });
  it.each([403, 500])(
    "does not convert HTTP %i into automatic login",
    async (status) => {
      const { args, auth } = fixture(status);
      const login = vi.spyOn(auth, "login");
      await expect(beforeLoadGuard(args)).rejects.toThrow();
      expect(login).not.toHaveBeenCalled();
    },
  );
  it("does not turn a cancelled logout acquisition into login or private access", async () => {
    const { args, auth } = fixture(200);
    await auth.refresh();
    const login = vi.spyOn(auth, "login");
    const logout = auth.logout();
    await expect(beforeLoadGuard(args)).rejects.toMatchObject({
      name: "AbortError",
    });
    expect(login).not.toHaveBeenCalled();
    await logout;
  });
});

import { describe, expect, it } from "vitest";
import {
  AuthPolicyError,
  assertHostAuthPolicy,
  callbackUrlForOrigin,
} from "../setup";

const callbackUrl = callbackUrlForOrigin("https://ui.konobangu.test");
const safe = {
  clientId: "konobangu-webui",
  issuerUrl: "https://idp.example",
  redirectUrl: callbackUrl,
  pkceEnabled: true,
};

describe("the host safety policy over a valid projection", () => {
  it("accepts a public PKCE client bound to this WebUI callback", () => {
    expect(() => assertHostAuthPolicy(safe, { callbackUrl })).not.toThrow();
    expect(callbackUrl).toBe("https://ui.konobangu.test/auth/callback");
  });

  it.each([
    [{ clientSecret: "s" }, "must be public"],
    [{ claimsCheckScript: "return true" }, "claims-check script"],
    [{ pkceEnabled: false }, "PKCE is required"],
    [{ clientId: "  " }, "clientId"],
    [{ issuerUrl: "" }, "issuerUrl"],
    [
      { redirectUrl: "https://attacker.example/auth/callback" },
      "published callback",
    ],
  ])("refuses %j even though the SDK schema permits it", (patch, message) => {
    expect(() =>
      assertHostAuthPolicy({ ...safe, ...patch }, { callbackUrl }),
    ).toThrow(AuthPolicyError);
    expect(() =>
      assertHostAuthPolicy({ ...safe, ...patch }, { callbackUrl }),
    ).toThrow(message as string);
  });
});

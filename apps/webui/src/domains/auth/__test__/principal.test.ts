import { describe, expect, it } from "vitest";
import {
  ApplicationPrincipalError,
  identityKeyOf,
  parseApplicationPrincipal,
} from "../principal";

const oidc = {
  subject: "resource-account",
  displayName: "Resource Account",
  authType: "oidc",
  issuer: "https://idp.example",
};
describe("the parsed server identity projection", () => {
  it("accepts a verified OIDC identity from either issuer source", () => {
    expect(parseApplicationPrincipal(oidc)).toEqual(oidc);
  });

  it("derives the identity epoch only from verified fields", () => {
    expect(identityKeyOf(parseApplicationPrincipal(oidc))).toBe(
      '["oidc","https://idp.example","resource-account"]',
    );
    expect(
      identityKeyOf(
        parseApplicationPrincipal({
          ...oidc,
          issuer: "https://local.example/auth-provider",
        }),
      ),
    ).not.toBe(identityKeyOf(parseApplicationPrincipal(oidc)));
  });

  it.each([
    ["an empty object", {}],
    ["an array", []],
    ["null", null],
    ["a string", "signed-in"],
    ["a missing subject", { ...oidc, subject: undefined }],
    ["an empty subject", { ...oidc, subject: "" }],
    ["an empty display name", { ...oidc, displayName: "" }],
    ["an unknown auth type", { ...oidc, authType: "session" }],
    ["an OIDC principal without an issuer", { ...oidc, issuer: null }],
    ["an OIDC principal with an empty issuer", { ...oidc, issuer: "  " }],
  ])("refuses %s", (_label, payload) => {
    expect(() => parseApplicationPrincipal(payload)).toThrow(
      ApplicationPrincipalError,
    );
  });

  it("refuses a removed Basic principal even when it carries an issuer", () => {
    expect(() =>
      parseApplicationPrincipal({ ...oidc, authType: "basic" }),
    ).toThrow(ApplicationPrincipalError);
  });
});

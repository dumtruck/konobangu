import { describe, expect, it } from "vitest";
import { createResourceCrypto, ensureCrypto } from "../crypto";

const hex = (bytes: Uint8Array) =>
  [...bytes].map((value) => value.toString(16).padStart(2, "0")).join("");
describe("authentication cryptography capabilities", () => {
  it("preserves secure-context native crypto unchanged", async () => {
    const original = globalThis.crypto.subtle;
    await ensureCrypto(globalThis);
    expect(globalThis.crypto.subtle).toBe(original);
    const crypto = createResourceCrypto(globalThis.crypto);
    expect(
      hex(
        await crypto.hmac(
          new TextEncoder().encode("key"),
          new TextEncoder().encode(
            "The quick brown fox jumps over the lazy dog",
          ),
        ),
      ),
    ).toBe("f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8");
  });
  it("installs only digest while preserving the native random source", async () => {
    const random = globalThis.crypto.getRandomValues.bind(globalThis.crypto);
    const target: {
      crypto: {
        getRandomValues: Crypto["getRandomValues"];
        subtle?: Pick<SubtleCrypto, "digest">;
      };
    } = { crypto: { getRandomValues: random } };
    await ensureCrypto(target);
    expect(target.crypto.getRandomValues).toBe(random);
    expect(Object.keys(target.crypto.subtle!)).toEqual(["digest"]);
    const digest = await target.crypto.subtle!.digest(
      "SHA-256",
      new TextEncoder().encode("abc"),
    );
    expect(hex(new Uint8Array(digest))).toBe(
      "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    );
    await expect(
      target.crypto.subtle!.digest("SHA-1", new Uint8Array()),
    ).rejects.toThrow("Unsupported");
  });
  it("provides the same HMAC through the minimal HTTP capability implementation", async () => {
    const crypto = createResourceCrypto({});
    expect(
      hex(
        await crypto.hmac(
          new TextEncoder().encode("key"),
          new TextEncoder().encode(
            "The quick brown fox jumps over the lazy dog",
          ),
        ),
      ),
    ).toBe("f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8");
  });
  it("fails instead of inventing randomness when the host has no native CSPRNG", async () => {
    await expect(ensureCrypto({})).rejects.toThrow(
      "native cryptographic random",
    );
  });
});

import type { ResourceCryptoTrait } from "./traits";

/** The SDK needs digest and a native CSPRNG before its environment is created. */
export async function ensureCrypto(
  target: {
    readonly crypto?: {
      readonly getRandomValues?: Crypto["getRandomValues"];
      readonly subtle?: { readonly digest?: SubtleCrypto["digest"] };
    };
  } = globalThis,
): Promise<void> {
  const crypto = target.crypto;
  if (typeof crypto?.getRandomValues !== "function")
    throw new Error(
      "Authentication requires a native cryptographic random source",
    );
  if (typeof crypto.subtle?.digest === "function") return;
  const { sha256, sha384, sha512 } = await import("@noble/hashes/sha2.js");
  const digest = async (
    algorithm: AlgorithmIdentifier,
    data: BufferSource,
  ): Promise<ArrayBuffer> => {
    const name = typeof algorithm === "string" ? algorithm : algorithm.name;
    const hash = { "SHA-256": sha256, "SHA-384": sha384, "SHA-512": sha512 }[
      name.toUpperCase()
    ];
    if (!hash)
      throw new DOMException(
        "Unsupported digest algorithm",
        "NotSupportedError",
      );
    const bytes = ArrayBuffer.isView(data)
      ? new Uint8Array(data.buffer, data.byteOffset, data.byteLength)
      : new Uint8Array(data);
    return Uint8Array.from(hash(bytes)).buffer;
  };
  // This deliberately implements only the SDK's digest requirement; it does
  // not claim to provide a complete SubtleCrypto implementation.
  if (crypto.subtle)
    Object.defineProperty(crypto.subtle, "digest", {
      configurable: true,
      value: digest,
    });
  else
    Object.defineProperty(crypto, "subtle", {
      configurable: true,
      value: { digest },
    });
}

export function createResourceCrypto(crypto: {
  readonly subtle?: Pick<SubtleCrypto, "importKey" | "sign">;
}): ResourceCryptoTrait {
  const subtle = crypto.subtle;
  if (
    typeof subtle?.importKey === "function" &&
    typeof subtle.sign === "function"
  )
    return {
      async hmac(key, message) {
        const imported = await subtle.importKey(
          "raw",
          Uint8Array.from(key).buffer,
          { name: "HMAC", hash: "SHA-256" },
          false,
          ["sign"],
        );
        return new Uint8Array(
          await subtle.sign("HMAC", imported, Uint8Array.from(message).buffer),
        );
      },
    };
  let implementation:
    | Promise<
        [
          typeof import("@noble/hashes/hmac.js"),
          typeof import("@noble/hashes/sha2.js"),
        ]
      >
    | undefined;
  return {
    async hmac(key, message) {
      implementation ??= Promise.all([
        import("@noble/hashes/hmac.js"),
        import("@noble/hashes/sha2.js"),
      ]);
      const [{ hmac }, { sha256 }] = await implementation;
      return hmac(sha256, key, message);
    },
  };
}

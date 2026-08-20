import {
  exportJWK,
  generateKeyPair,
  type JWTHeaderParameters,
  SignJWT,
} from "jose";

/**
 * RFC 9068 access tokens for resource-server verification tests, plus the
 * near-miss credentials a resource server must refuse: a foreign or slash-mismatched issuer or
 * audience, an expired or not-yet-valid token, a foreign signature, a missing
 * required scope, an ID token that only differs by its `typ`, and a symmetric
 * signature.
 */
export async function generateAccessTokenVectors(
  issuer: string,
  options: { audience?: string; scope?: string } = {},
) {
  const audience = options.audience ?? "urn:konobangu:api";
  const scope = options.scope ?? "konobangu:api";
  const { privateKey, publicKey } = await generateKeyPair("RS256", {
    extractable: true,
  });
  const foreign = await generateKeyPair("RS256");
  const jwk = await exportJWK(publicKey);
  Object.assign(jwk, { kid: "resource-fixture", alg: "RS256", use: "sig" });
  const symmetric = new Uint8Array(32).fill(7);
  const now = Math.floor(Date.now() / 1000);
  const tokens: Record<string, string> = {};

  for (const name of [
    "valid",
    "issuer",
    "issuerSlashMismatch",
    "audience",
    "expired",
    "notBefore",
    "signature",
    "scope",
    "idToken",
    "missingType",
    "emptySubject",
  ]) {
    const payload: Record<string, unknown> = {
      iss: issuer,
      sub: "resource-account",
      aud: audience,
      iat: now,
      exp: now + 300,
      jti: `fixture-${name}`,
      client_id: "konobangu-webui",
      scope,
    };
    if (name === "issuer") payload.iss = "https://foreign-issuer.example";
    if (name === "issuerSlashMismatch")
      payload.iss = issuer.endsWith("/") ? issuer.slice(0, -1) : `${issuer}/`;
    if (name === "audience") payload.aud = "urn:foreign:api";
    if (name === "expired") payload.exp = now - 300;
    if (name === "notBefore") payload.nbf = now + 600;
    if (name === "scope") payload.scope = "openid profile";
    if (name === "emptySubject") payload.sub = "";
    // An ID token is a validly signed credential for the same issuer; only its
    // declared type says it must not authorize an API call.
    const type = name === "idToken" ? "JWT" : "at+jwt";
    const header: JWTHeaderParameters =
      name === "missingType"
        ? { alg: "RS256", kid: jwk.kid }
        : { alg: "RS256", kid: jwk.kid, typ: type };
    tokens[name] = await new SignJWT(payload)
      .setProtectedHeader(header)
      .sign(name === "signature" ? foreign.privateKey : privateKey);
  }

  tokens.symmetric = await new SignJWT({
    iss: issuer,
    sub: "resource-account",
    aud: audience,
    iat: now,
    exp: now + 300,
    jti: "fixture-symmetric",
    scope,
  })
    .setProtectedHeader({ alg: "HS256", typ: "at+jwt" })
    .sign(symmetric);

  return { jwks: { keys: [jwk] }, tokens };
}

if (import.meta.main) {
  const issuer = process.argv[2];
  if (!issuer) throw new Error("A loopback resource-test issuer is required");
  process.stdout.write(
    JSON.stringify(await generateAccessTokenVectors(issuer)),
  );
}

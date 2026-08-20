import { ArkErrors, type } from "arktype";

/**
 * The verified application identity, as the server projects it. The response
 * is untrusted JSON, so it is parsed rather than asserted: authentication is
 * OIDC, and the verified issuer and subject have to be non-empty. A null field or a
 * separate success boolean must never stand in for "signed in".
 */
/** Whitespace is not an identity, so blank is rejected rather than trimmed. */
const NonBlankString = type("string").narrow(
  (value, ctx) => value.trim().length > 0 || ctx.mustBe("a non-blank string"),
);

export const ApplicationPrincipalSchema = type({
  subject: NonBlankString,
  displayName: NonBlankString,
  authType: '"oidc"',
  issuer: NonBlankString,
});

export type ApplicationPrincipal = typeof ApplicationPrincipalSchema.infer;

export class ApplicationPrincipalError extends Error {
  constructor(reason: string) {
    super(`The server identity projection is unusable: ${reason}`);
    this.name = "ApplicationPrincipalError";
  }
}

/**
 * Both bundled and external issuers supply this same projection. A removed
 * Basic principal cannot become an identity or a private cache key.
 */
export function parseApplicationPrincipal(
  value: unknown,
): ApplicationPrincipal {
  const parsed = ApplicationPrincipalSchema(value);
  if (parsed instanceof ArkErrors)
    throw new ApplicationPrincipalError(parsed.summary);
  return parsed;
}

/** The stable identity epoch key, built only from verified server facts. */
export function identityKeyOf(principal: ApplicationPrincipal): string {
  return JSON.stringify([
    principal.authType,
    principal.issuer,
    principal.subject,
  ]);
}

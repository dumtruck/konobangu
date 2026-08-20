import { describe, expect, it } from "vitest";
import { apolloErrorToMessage, getApolloQueryError } from "../apollo";

describe("apollo error messages", () => {
  it("returns a real string message", () => {
    expect(apolloErrorToMessage({ name: "E", message: "boom" })).toBe("boom");
    expect(apolloErrorToMessage(new Error("thrown"))).toBe("thrown");
    expect(apolloErrorToMessage("plain")).toBe("plain");
  });

  it.each([
    ["a numeric message", { message: 42 }],
    ["a nested object message", { message: { text: "boom" } }],
    ["a null message", { message: null }],
    ["an array message", { message: ["boom"] }],
  ])("does not pass %s through as a string", (_label, error) => {
    const message = apolloErrorToMessage(error);
    expect(typeof message).toBe("string");
    expect(message).toBe("Unknown error");
  });

  it("combines query errors and prefers the single error", () => {
    expect(
      getApolloQueryError({ error: { name: "E", message: "single" } }),
    ).toEqual({ name: "E", message: "single" });
    expect(
      getApolloQueryError({ errors: [{ message: "a" }, { message: "b" }] }),
    ).toEqual({ name: "CombinedError", message: "a; b" });
    expect(getApolloQueryError({})).toBeUndefined();
  });
});

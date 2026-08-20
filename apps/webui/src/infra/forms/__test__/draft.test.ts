import { ArkErrors } from "arktype";
import { describe, expect, it } from "vitest";
import {
  type SubscriptionForm,
  SubscriptionFormSchema,
  validateSubscriptionForm,
} from "@/domains/recorder/schema/subscriptions";
import type { FormDraft } from "../draft";

const empty: FormDraft<SubscriptionForm> = {
  displayName: "",
  category: "",
  enabled: true,
  sourceUrl: "",
  credentialId: Number.NaN,
  year: Number.NaN,
  seasonStr: "",
};

describe("a form draft is not a validated entity", () => {
  it("refuses to become a mutation input while incomplete", () => {
    // The draft type allows this state; the schema is what rejects it.
    expect(SubscriptionFormSchema(empty)).toBeInstanceOf(ArkErrors);
    expect(() => validateSubscriptionForm(empty)).toThrow(
      "The subscription form is incomplete",
    );
  });

  it("validates a completed draft into the form value", () => {
    const value = validateSubscriptionForm({
      ...empty,
      displayName: "Fixture",
      category: "mikan_bangumi",
      sourceUrl:
        "https://mikanani.me/RSS/Bangumi?bangumiId=3600&subgroupid=1207",
    });
    expect(value.category).toBe("mikan_bangumi");
    expect(value.displayName).toBe("Fixture");
  });

  it.each([
    ["an empty display name", { displayName: "" }],
    ["no category", { category: "" }],
    ["a season without a year", { category: "mikan_season", year: Number.NaN }],
  ])("refuses %s", (_label, patch) => {
    expect(() => validateSubscriptionForm({ ...empty, ...patch })).toThrow();
  });
});

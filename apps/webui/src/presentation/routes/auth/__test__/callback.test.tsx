// @vitest-environment jsdom
import { screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { createKonobanguFixture } from "@/__test__/support/fixture";
import { renderInEnvironment } from "@/__test__/support/render";
import { AuthCallbackRoute } from "../callback";

describe("SDK callback route boundaries", () => {
  it("uses the real SDK hook for an empty callback under StrictMode without reading a profile", async () => {
    const app = createKonobanguFixture();
    renderInEnvironment(app, <AuthCallbackRoute />, { strict: true });
    expect(
      await screen.findByText(/No authentication response is available/),
    ).toBeTruthy();
    expect(app.fetch).not.toHaveBeenCalled();
  });
});

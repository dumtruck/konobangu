// @vitest-environment jsdom
import { createCancellationTokenSource } from "@securitydept/client";
import { fireEvent, screen } from "@testing-library/react";
import { toast } from "sonner";
import { describe, expect, it, vi } from "vitest";
import {
  createSignedInFixture,
  deferredReply,
  response,
  type TestRequestHandler,
} from "@/__test__/support/fixture";
import { renderInEnvironment } from "@/__test__/support/render";
import {
  GET_CREDENTIAL_3RD,
  GET_CREDENTIAL_3RD_DETAIL,
} from "@/domains/recorder/schema/credential3rd";
import { GraphQLService } from "@/infra/graphql/graphql.service";
import { Credential3rdCheckAvailableView } from "@/presentation/routes/_app/credential3rd/-check-available";
import { Credential3rdService } from "../credential3rd.service";

const COMMAND = "credential3rd/42/check-available";
async function fixture(command: TestRequestHandler) {
  const app = await createSignedInFixture({ handler: command });
  return {
    ...app,
    service: app.injector.get(Credential3rdService),
    commands: () => app.requests({ path: COMMAND, method: "POST" }),
  };
}
describe("Independent credential command", () => {
  it("uses the shared HTTP client with a JSON body and OIDC credential policy", async () => {
    const app = await fixture(() => response(200, { available: true }));
    expect(await app.service.checkAvailable(42)).toEqual({ available: true });
    const [[, init]] = app.commands();
    expect(new Headers(init?.headers).get("X-Konobangu-CSRF")).toBeNull();
    expect(new Headers(init?.headers).get("Content-Type")).toBe(
      "application/json",
    );
  });
  it.each([403, 409, 502])(
    "preserves HTTP %i as a failure without changing the identity",
    async (status) => {
      const app = await fixture(() =>
        response(status, { message: "command failed" }),
      );
      await expect(app.service.checkAvailable(42)).rejects.toThrow(
        "command failed",
      );
      expect(app.auth.isAuthenticated.get()).toBe(true);
      expect(app.fetch).toHaveBeenCalledTimes(2);
    },
  );
  it.each([null, { available: "true" }])(
    "rejects malformed command success %j",
    async (body) => {
      const app = await fixture(() => response(200, body));
      await expect(app.service.checkAvailable(42)).rejects.toThrow(
        "The API response failed schema validation",
      );
    },
  );
  it.each(["caller", "identity", "root"])(
    "rejects late replies after %s cancellation",
    async (mode) => {
      const reply = deferredReply<Response>();
      const app = await fixture(reply.request);
      const source = createCancellationTokenSource();
      const pending = app.service
        .checkAvailable(42, source.token)
        .catch((error: unknown) => error);
      await reply.requested;
      if (mode === "caller") source.cancel();
      if (mode === "identity") await app.identity("B");
      if (mode === "root") app.environment[Symbol.dispose]();
      const [[, posted]] = app.commands();
      await vi.waitFor(() => expect(posted?.signal?.aborted).toBe(true));
      reply.resolve(response(200, { available: true }));
      expect(await pending).toMatchObject({ name: "AbortError" });
    },
  );
  it("does not re-read the profile or replay a failed command after 401", async () => {
    const app = await fixture(() => response(401));
    await app.identity(null);
    const before = app.fetch.mock.calls.length;
    await expect(app.service.checkAvailable(42)).rejects.toThrow();
    await vi.waitFor(() =>
      expect(app.fetch.mock.calls.length).toBe(before + 1),
    );
    expect(app.auth.isAuthenticated.get()).toBe(false);
  });
});
describe("Credential check button", () => {
  async function mount(command: TestRequestHandler) {
    const app = await fixture(command);
    const refetch = vi
      .spyOn(app.injector.get(GraphQLService)._apollo, "refetchQueries")
      .mockResolvedValue([]);
    const view = renderInEnvironment(
      app,
      <Credential3rdCheckAvailableView id={42} />,
      { apollo: true },
    );
    const button = screen.getByRole("button", { name: "Check Available" });
    return { ...app, ...view, refetch, button };
  }
  it("disables during the command and refreshes the active Apollo credential queries after success", async () => {
    const reply = deferredReply<Response>();
    const success = vi.spyOn(toast, "success");
    const app = await mount(reply.request);
    fireEvent.click(app.button);
    expect(app.button.hasAttribute("disabled")).toBe(true);
    await reply.requested;
    reply.resolve(response(200, { available: true }));
    await vi.waitFor(() =>
      expect(app.button.hasAttribute("disabled")).toBe(false),
    );
    expect(app.refetch).toHaveBeenCalledWith({
      include: [GET_CREDENTIAL_3RD, GET_CREDENTIAL_3RD_DETAIL],
    });
    expect(success).toHaveBeenCalledWith("Credential is available");
    expect(app.commands()).toHaveLength(1);
  });
  it("shows the command error and does not refetch after a conflict", async () => {
    const failure = vi.spyOn(toast, "error");
    const app = await mount(() =>
      response(409, { message: "Credential changed" }),
    );
    fireEvent.click(app.button);
    await vi.waitFor(() =>
      expect(failure).toHaveBeenCalledWith("Failed to check available", {
        description: "Credential changed",
      }),
    );
    expect(app.refetch).not.toHaveBeenCalled();
  });
  it("cancels on unmount and prevents late notification and refetch", async () => {
    const reply = deferredReply<Response>();
    const success = vi.spyOn(toast, "success");
    const app = await mount(reply.request);
    fireEvent.click(app.button);
    await reply.requested;
    app.unmount();
    // Unmounting cancels the command, so the late reply has no reader left.
    const [[, posted]] = app.commands();
    await vi.waitFor(() => expect(posted?.signal?.aborted).toBe(true));
    reply.resolve(response(200, { available: true }));
    await app.fetch.mock.results.at(-1)?.value;
    expect(app.refetch).not.toHaveBeenCalled();
    expect(success).not.toHaveBeenCalled();
  });
});

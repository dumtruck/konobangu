import {
  BOOTSTRAP_DEADLINE_MS,
  createBootstrap,
  createBootstrapStages,
} from "./app/bootstrap";
import { createKonobanguBootstrapEnvironmentForNativeWeb } from "./app/environment/bootstrap";
import { ensurePolyfills } from "./app/polyfills";

// The one place the pre-polyfill environment is resolved; every stage below
// consumes the result instead of reaching for a global of its own.
const environment = createKonobanguBootstrapEnvironmentForNativeWeb({
  window,
  ensurePolyfills: () => ensurePolyfills(),
  deadlineMs: BOOTSTRAP_DEADLINE_MS,
  allowInsecureLoopback: import.meta.env.DEV,
});
const bootstrap = createBootstrap(
  () => import("./app/entry"),
  environment,
  createBootstrapStages(environment),
);
const container = document.getElementById("app");
function stop() {
  window.removeEventListener("pagehide", onPageHide);
  bootstrap.dispose();
}
function onPageHide(event: PageTransitionEvent) {
  if (!event.persisted) stop();
}
window.addEventListener("pagehide", onPageHide);
async function start() {
  try {
    container?.replaceChildren();
    await bootstrap.start();
  } catch {
    if (!container) return;
    const message = document.createElement("p");
    message.textContent =
      "Unable to start Konobangu. Please retry or refresh the page.";
    const retry = document.createElement("button");
    retry.textContent = "Retry";
    // Reload resets a rejected ESM import cache and all composition-root state.
    retry.onclick = () => window.location.reload();
    container.replaceChildren(message, retry);
  }
}
void start();
if (import.meta.hot) {
  // Accept root dependencies without re-evaluating main and starting another root.
  import.meta.hot.accept(["./app/bootstrap", "./app/entry"], () => {
    try {
      stop();
    } finally {
      window.location.reload();
    }
  });
  import.meta.hot.dispose(stop);
}

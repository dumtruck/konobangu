/** Await a borrowed operation without allowing it to outlive its caller scope. */
export async function withinScope<T>(
  operation: Promise<T>,
  signal: AbortSignal,
): Promise<T> {
  const ended = Promise.withResolvers<never>();
  const end = () => ended.reject(signal.reason);
  signal.addEventListener("abort", end, { once: true });
  // Checked after subscribing, so an abort in between is not missed.
  if (signal.aborted) end();
  try {
    const value = await Promise.race([operation, ended.promise]);
    signal.throwIfAborted();
    return value;
  } finally {
    signal.removeEventListener("abort", end);
    // Nothing may be awaiting the scope's side of the race anymore.
    ended.promise.catch(() => undefined);
  }
}

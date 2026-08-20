import { useCallback, useInsertionEffect, useRef } from "react";

/**
 * A stable callback that always calls the latest `fn`. The signature is an
 * explicit Args tuple and Result rather than `any[]`, so a wrong call site is
 * a type error; the returned value is a plain callable and does not promise to
 * preserve overloads or extra properties of the original function.
 */
export function useEvent<TArgs extends readonly unknown[], TResult>(
  fn: (...args: TArgs) => TResult,
): (...args: TArgs) => TResult {
  const ref = useRef(fn);
  useInsertionEffect(() => {
    ref.current = fn;
  }, [fn]);
  return useCallback((...args: TArgs) => ref.current(...args), []);
}

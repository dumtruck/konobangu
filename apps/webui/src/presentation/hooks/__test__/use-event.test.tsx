// @vitest-environment jsdom
import { act, render } from "@testing-library/react";
import { useState } from "react";
import { describe, expect, it } from "vitest";
import { useEvent } from "../use-event";

describe("useEvent", () => {
  it("keeps one identity while always calling the latest closure", () => {
    const seen: number[] = [];
    const identities = new Set<unknown>();
    let bump!: () => void;
    function Probe() {
      const [count, setCount] = useState(0);
      bump = () => setCount((value) => value + 1);
      const handler = useEvent((offset: number) => {
        seen.push(count + offset);
        return count + offset;
      });
      identities.add(handler);
      return (
        <button type="button" onClick={() => handler(10)}>
          run
        </button>
      );
    }
    const view = render(<Probe />);
    act(() => {
      view.getByRole("button").click();
    });
    act(() => bump());
    act(() => {
      view.getByRole("button").click();
    });
    expect(seen).toEqual([10, 11]);
    expect(identities.size).toBe(1);
  });

  it("preserves the argument tuple and the result", () => {
    let result: string | undefined;
    function Probe() {
      const join = useEvent(
        (left: string, right: number) => `${left}:${right}`,
      );
      result = join("a", 1);
      return null;
    }
    render(<Probe />);
    expect(result).toBe("a:1");
  });
});

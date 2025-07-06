type AllKeys<T> = T extends any ? keyof T : never;

type ToDefaultable<T> = Exclude<
  T extends string | undefined
    ? T | ''
    : T extends number | undefined
      ? T | number
      : T extends undefined
        ? T | null
        : T,
  undefined
>;

type PickFieldFormUnion<T, K extends keyof T> = T extends any
  ? T[keyof T & K]
  : never;

// compact more types;
export type FormDefaultValues<T> = {
  -readonly [K in AllKeys<T>]-?: ToDefaultable<PickFieldFormUnion<T, K>>;
};

/**
 * https://github.com/shadcn-ui/ui/issues/427
 */
export function compatFormDefaultValues<T, K extends AllKeys<T> = AllKeys<T>>(
  d: FormDefaultValues<Pick<T, K>>
): T {
  return d as unknown as T;
}

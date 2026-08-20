/**
 * A form draft is the input state a user edits, not a validated entity and not
 * a GraphQL mutation input. An empty text field is `""`, a cleared number is
 * `NaN`, and a not-yet-chosen optional value is `null` — none of which satisfy
 * the validated schema. Keeping the draft its own type means the submit path
 * has to validate before it can build a mutation input.
 *
 * https://github.com/shadcn-ui/ui/issues/427 is why the draft is widened
 * rather than left partial: the controlled inputs need every key present.
 */
type AllKeys<T> = T extends unknown ? keyof T : never;

type DraftValue<T> = Exclude<
  Exclude<T, undefined | null> extends string
    ? T | ""
    : Exclude<T, undefined | null> extends number
      ? T | number
      : undefined extends T
        ? T | null
        : T,
  undefined
>;

type PickFromUnion<T, K extends keyof T> = T extends unknown
  ? T[keyof T & K]
  : never;

export type FormDraft<T, K extends AllKeys<T> = AllKeys<T>> = {
  -readonly [Key in K]-?: DraftValue<PickFromUnion<T, Key>>;
};

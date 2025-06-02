import { ArkErrors, type Traversal } from 'arktype';

export function arkValidatorToTypeNarrower<
  T,
  V extends (input: T) => unknown | ArkErrors,
>(validator: V): (input: T, ctx: Traversal) => boolean {
  return (input, ctx) => {
    const result = validator(input);
    if (result instanceof ArkErrors) {
      ctx.errors.merge(result);
      return false;
    }
    return true;
  };
}

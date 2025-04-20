export class UnreachableError extends Error {
  constructor(detail: string) {
    super(`UnreachableError: ${detail}`);
  }
}

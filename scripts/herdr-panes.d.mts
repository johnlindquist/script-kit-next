/** Operator-only local commands; a result state of `paired` is not a liveness claim. */
export function paneCommand(
  args: string[],
  root: string,
): { state: string; [field: string]: unknown };

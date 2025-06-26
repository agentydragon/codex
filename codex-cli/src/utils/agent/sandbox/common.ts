/**
 * Common writable roots that should always be allowed for write operations,
 * even if not explicitly provided by the user.
 * Without these roots, pyenv rehash may fail if shims isn't writable
 */
export function getCommonRoots(): Array<string> {
  const home = process.env["HOME"]!;
  return [
    `${home}/.pyenv`,
    `${home}/.pyenv/shims`,
  ];
}

/**
 * The key that makes the graph reload: the "refresh" counter and the names switch.
 * Without the switch in the key, turning the names on (or the key being applied right after
 * the tab opens) left the graph with codes until the user pressed "refresh".
 */
export function graphReloadKey(refresh: number, identityPath: string | undefined): string {
  return `${refresh}|${identityPath ?? ""}`;
}

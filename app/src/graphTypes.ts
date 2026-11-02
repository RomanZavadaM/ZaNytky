import type { Strings } from "./i18n";

/** The name of a node type of the channel graphs (channel, site, mention, post). */
export function channelTypeLabel(t: Strings, type: string): string {
  switch (type) {
    case "channel":
      return t.typeChannel;
    case "site":
      return t.typeSite;
    case "mention":
      return t.typeMention;
    case "source":
      return t.typePost;
    default:
      return type;
  }
}

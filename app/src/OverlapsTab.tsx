import { useCallback, useEffect, useState } from "react";
import { api, type Shared } from "./api";
import { KnowledgeGraphPanel } from "./graph/KnowledgeGraphPanel";
import { LanguageContext, type LanguageCode } from "./graph/lang";
import { fill, type Lang, type Strings } from "./i18n";

interface Props {
  t: Strings;
  lang: Lang;
  home: string;
  run: <T>(work: () => Promise<T>) => Promise<T | undefined>;
}

/** What several imported channels have in common: a list and a graph. The finds are hints. */
export function OverlapsTab({ t, lang, home, run }: Props) {
  const [items, setItems] = useState<Shared[] | null>(null);

  useEffect(() => {
    if (!home) return;
    void (async () => {
      const found = await run(() => api.overlaps(home, 100));
      if (found) setItems(found);
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [home]);

  const load = useCallback(
    async () => (await run(() => api.overlapGraph(home, 60))) ?? { nodes: [], edges: [], relations: [] },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [home],
  );

  const kindName = (kind: string) =>
    kind === "forward"
      ? t.overlapsForward
      : kind === "tg"
        ? t.overlapsChannel
        : kind === "site"
          ? t.overlapsSite
          : kind === "mention"
            ? t.overlapsMention
            : t.overlapsText;

  if (items === null) return null;
  if (items.length === 0) return <p className="muted">{t.overlapsNone}</p>;
  return (
    <section>
      <p className="muted">{t.overlapsHint}</p>
      <LanguageContext.Provider value={(lang as LanguageCode) ?? "uk"}>
        <KnowledgeGraphPanel mode="global" notePath="" refreshKey={items.length} load={load} onOpen={() => undefined} />
      </LanguageContext.Provider>
      <table>
        <tbody>
          {items.map((item) => (
            <tr key={`${item.kind}:${item.value}`}>
              <td>{kindName(item.kind)}</td>
              <td>{item.value}</td>
              <td className="muted">
                {fill(t.overlapsIn, { n: item.channels.length })}:{" "}
                {item.channels.map((c) => `${c.name} (${c.count})`).join(", ")}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}

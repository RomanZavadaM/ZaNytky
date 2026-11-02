import { useCallback, useEffect, useMemo, useState } from "react";
import { api, type Shared } from "./api";
import { channelTypeLabel } from "./graphTypes";
import { KnowledgeGraphPanel } from "./graph/KnowledgeGraphPanel";
import { LanguageContext, type LanguageCode } from "./graph/lang";
import { fill, type Lang, type Strings } from "./i18n";

interface Props {
  t: Strings;
  lang: Lang;
  home: string;
  run: <T>(work: () => Promise<T>) => Promise<T | undefined>;
}

// One colour per channel (Okabe-Ito, colour-blind safe).
const COLOURS = ["#E69F00", "#56B4E9", "#009E73", "#CC79A7", "#0072B2", "#D55E00", "#F0E442", "#999999"];
const PAGE = 40;

/** What several imported channels have in common: rows with a bar per channel (the main view)
 * and the graph (on demand). The finds are hints for hypotheses, not proofs. */
export function OverlapsTab({ t, lang, home, run }: Props) {
  const [items, setItems] = useState<Shared[] | null>(null);
  const [kind, setKind] = useState("");
  const [shown, setShown] = useState(PAGE);
  const [graphOpen, setGraphOpen] = useState(false);

  useEffect(() => {
    if (!home) return;
    void (async () => {
      const found = await run(() => api.overlaps(home, 300));
      if (found) setItems(found);
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [home]);

  const load = useCallback(
    async () => (await run(() => api.overlapGraph(home, 60))) ?? { nodes: [], edges: [], relations: [] },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [home],
  );

  const channels = useMemo(() => {
    const seen = new Map<number, string>();
    (items ?? []).forEach((item) => item.channels.forEach((c) => seen.set(c.channelId, c.name)));
    return Array.from(seen.entries());
  }, [items]);
  const maxCount = useMemo(
    () => Math.max(1, ...(items ?? []).flatMap((item) => item.channels.map((c) => c.count))),
    [items],
  );
  const kinds = useMemo(() => {
    const counts = new Map<string, number>();
    (items ?? []).forEach((item) => counts.set(item.kind, (counts.get(item.kind) ?? 0) + 1));
    return Array.from(counts.entries());
  }, [items]);

  const kindName = (value: string) =>
    value === "forward"
      ? t.overlapsForward
      : value === "tg"
        ? t.overlapsChannel
        : value === "site"
          ? t.overlapsSite
          : value === "mention"
            ? t.overlapsMention
            : t.overlapsText;

  if (items === null) return null;
  if (items.length === 0) return <p className="muted">{t.overlapsNone}</p>;
  const visible = items.filter((item) => kind === "" || item.kind === kind);
  const colourOf = (id: number) => COLOURS[channels.findIndex(([cid]) => cid === id) % COLOURS.length];
  const width = (count: number) => (Math.log10(1 + count) / Math.log10(1 + maxCount)) * 100;

  return (
    <section>
      <p className="muted">{t.overlapsHint}</p>
      <p>
        {channels.map(([id, name]) => (
          <span key={id} className="ov-channel">
            <i style={{ background: colourOf(id) }} /> {name}
          </span>
        ))}
      </p>
      <p className="ov-kinds">
        <button className={kind === "" ? "chip on" : "chip"} onClick={() => setKind("")}>
          {t.overlapsAll} <small>{items.length}</small>
        </button>
        {kinds.map(([k, n]) => (
          <button key={k} className={kind === k ? "chip on" : "chip"} onClick={() => setKind(k)}>
            {kindName(k)} <small>{n}</small>
          </button>
        ))}
      </p>
      <div className="ov-rows">
        {visible.slice(0, shown).map((item) => (
          <div className="ov-row" key={`${item.kind}:${item.value}`}>
            <span className="ov-kind">{kindName(item.kind)}</span>
            <span className="ov-label" title={item.value}>
              {item.value}
            </span>
            <span className="ov-bars">
              {item.channels.map((c) => (
                <span className="ov-bar" key={c.channelId} title={`${c.name}: ${c.count}`}>
                  <span style={{ width: `${Math.max(4, width(c.count))}%`, background: colourOf(c.channelId) }} />
                  <b>{c.count}</b>
                </span>
              ))}
            </span>
          </div>
        ))}
      </div>
      {visible.length > shown && (
        <p>
          <button onClick={() => setShown(shown + PAGE)}>
            {fill(t.overlapsMore, { n: visible.length - shown })}
          </button>
        </p>
      )}
      <details onToggle={(e) => setGraphOpen((e.target as HTMLDetailsElement).open)}>
        <summary>{t.overlapsGraphToggle}</summary>
        {graphOpen && (
          <LanguageContext.Provider value={(lang as LanguageCode) ?? "uk"}>
            <KnowledgeGraphPanel
              mode="global"
              notePath=""
              refreshKey={items.length}
              load={load}
              onOpen={() => undefined}
              plain
              typeLabel={(type) => channelTypeLabel(t, type)}
              hint={null}
            />
          </LanguageContext.Provider>
        )}
      </details>
    </section>
  );
}

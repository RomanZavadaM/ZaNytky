import { useCallback, useEffect, useState } from "react";
import { api, type ChannelGaps, type ChannelRow } from "./api";
import { KnowledgeGraphPanel } from "./graph/KnowledgeGraphPanel";
import { LanguageContext, type LanguageCode } from "./graph/lang";
import { fill, type Lang, type Strings } from "./i18n";

interface Props {
  t: Strings;
  lang: Lang;
  home: string;
  run: <T>(work: () => Promise<T>) => Promise<T | undefined>;
}

/** An imported Telegram channel: the numbers missing in the export, and its connections. */
export function ChannelsTab({ t, lang, home, run }: Props) {
  const [channels, setChannels] = useState<ChannelRow[]>([]);
  const [selected, setSelected] = useState<number | null>(null);
  const [gaps, setGaps] = useState<ChannelGaps | null>(null);

  useEffect(() => {
    if (!home) return;
    void (async () => {
      const list = await run(() => api.channels(home));
      if (!list) return;
      setChannels(list);
      setSelected((current) => current ?? list[0]?.channelId ?? null);
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [home]);

  useEffect(() => {
    if (selected === null) {
      setGaps(null);
      return;
    }
    void (async () => {
      const found = await run(() => api.channelGaps(home, selected));
      if (found) setGaps(found);
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [home, selected]);

  const load = useCallback(
    async () =>
      selected === null
        ? { nodes: [], edges: [], relations: [] }
        : ((await run(() => api.channelGraph(home, selected))) ?? { nodes: [], edges: [], relations: [] }),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [home, selected],
  );

  const current = channels.find((c) => c.channelId === selected) ?? null;
  if (channels.length === 0) {
    return <p className="muted">{t.channelsNone}</p>;
  }
  return (
    <section>
      <label>
        {t.channelsSelect}{" "}
        <select value={selected ?? ""} onChange={(e) => setSelected(Number(e.target.value))}>
          {channels.map((c) => (
            <option key={c.channelId} value={c.channelId}>
              {c.name} ({c.posts})
            </option>
          ))}
        </select>
      </label>
      {current && (
        <p className="muted">
          {fill(t.channelsPosts, { n: current.posts, from: current.firstId, to: current.lastId })}
          {current.firstDate ? ` · ${fill(t.channelsPeriod, { from: current.firstDate, to: current.lastDate ?? "" })}` : ""}
        </p>
      )}
      {gaps && (
        <div className="source-box">
          <b>{t.channelsGapsTitle}</b>
          <p className="muted">{fill(t.channelsMissing, { n: gaps.missing })}</p>
          {gaps.ranges.length > 0 && (
            <details>
              <summary>{t.channelsGapsList}</summary>
              <p>
                {gaps.ranges.map((r) => (r.from === r.to ? `${r.from}` : `${r.from}-${r.to}`)).join(", ")}
                {gaps.more > 0 ? ` ... ${fill(t.channelsGapsMore, { n: gaps.more })}` : ""}
              </p>
            </details>
          )}
        </div>
      )}
      <h3>{t.channelsGraphTitle}</h3>
      <p className="muted">{t.channelsRefreshHint}</p>
      <LanguageContext.Provider value={(lang as LanguageCode) ?? "uk"}>
        <KnowledgeGraphPanel mode="global" notePath="" refreshKey={selected ?? 0} load={load} onOpen={() => undefined} />
      </LanguageContext.Provider>
    </section>
  );
}

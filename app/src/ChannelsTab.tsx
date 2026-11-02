import { useCallback, useEffect, useState } from "react";
import { api, type ChannelGaps, type ChannelRow, type MonthRow } from "./api";
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
  const [months, setMonths] = useState<MonthRow[]>([]);
  const [expanded, setExpanded] = useState<string[]>([]);

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

  useEffect(() => {
    setExpanded([]);
    if (selected === null) {
      setMonths([]);
      return;
    }
    void (async () => {
      const found = await run(() => api.channelTimeline(home, selected));
      if (found) setMonths(found);
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [home, selected]);

  const load = useCallback(
    async () =>
      selected === null
        ? { nodes: [], edges: [], relations: [] }
        : ((await run(() => api.channelGraph(home, selected, expanded))) ?? { nodes: [], edges: [], relations: [] }),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [home, selected, expanded],
  );

  const expand = (path: string) => {
    if (path.startsWith("post:") || expanded.includes(path)) return;
    setExpanded([...expanded, path]);
  };

  // A month is a peak when it has clearly more posts than usual (mean + 2 deviations).
  const stats = (() => {
    const values = months.map((m) => m.posts);
    const mean = values.reduce((a, b) => a + b, 0) / (values.length || 1);
    const deviation = Math.sqrt(values.reduce((a, b) => a + (b - mean) ** 2, 0) / (values.length || 1));
    const limit = mean + 2 * deviation;
    return { limit, peak: (m: MonthRow) => values.length >= 6 && m.posts > limit, max: Math.max(1, ...values) };
  })();

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
      {months.length > 0 && (
        <div className="source-box">
          <b>{t.channelsTimelineTitle}</b>
          <p className="muted">{t.channelsTimelineHint}</p>
          <svg viewBox={`0 0 ${Math.max(months.length * 10, 200)} 120`} width="100%" height="140" role="img">
            {months.map((m, i) => {
              const own = ((m.posts - m.forwarded) / stats.max) * 100;
              const fwd = (m.forwarded / stats.max) * 100;
              const x = i * 10 + 1;
              return (
                <g key={m.month}>
                  <rect x={x} y={110 - own - fwd} width={8} height={fwd} fill="#CC79A7" />
                  <rect x={x} y={110 - own} width={8} height={own} fill="#7C8DF0" />
                  {stats.peak(m) && <rect x={x - 0.5} y={108 - own - fwd} width={9} height={own + fwd + 2} fill="none" stroke="#D55E00" />}
                  <title>
                    {m.month}: {m.posts} / {m.forwarded}
                    {m.topOrigin ? ` · ${m.topOrigin} (${m.topCount})` : ""}
                  </title>
                  {(i === 0 || m.month.endsWith("-01")) && (
                    <text x={x} y={119} fontSize="5" fill="currentColor">
                      {m.month.slice(0, 4)}
                    </text>
                  )}
                </g>
              );
            })}
          </svg>
          <p className="muted">
            <span style={{ color: "#7C8DF0" }}>■</span> {t.channelsOwn} <span style={{ color: "#CC79A7" }}>■</span>{" "}
            {t.channelsForwarded}
          </p>
          {months.some((m) => stats.peak(m)) && (
            <>
              <b>{t.channelsPeaks}</b>
              <ul>
                {months
                  .filter((m) => stats.peak(m))
                  .map((m) => (
                    <li key={m.month}>
                      {fill(t.channelsPeakLine, { month: m.month, n: m.posts, f: m.forwarded })}
                      {m.topOrigin ? ` · ${m.topOrigin} (${m.topCount})` : ""}
                    </li>
                  ))}
              </ul>
            </>
          )}
        </div>
      )}
      <h3>{t.channelsGraphTitle}</h3>
      <p className="muted">{t.channelsRefreshHint}</p>
      <p className="muted">{t.channelsExpandHint}</p>
      {expanded.length > 0 && <button onClick={() => setExpanded([])}>{t.channelsCollapse}</button>}
      <LanguageContext.Provider value={(lang as LanguageCode) ?? "uk"}>
        <KnowledgeGraphPanel mode="global" notePath="" refreshKey={(selected ?? 0) * 1000 + expanded.length} load={load} onOpen={expand} />
      </LanguageContext.Provider>
    </section>
  );
}

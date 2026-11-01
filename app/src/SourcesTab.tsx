import { useEffect, useState } from "react";
import { api, pickFolder, type CardSource, type SourceListPage } from "./api";
import { fill, type Lang, type Strings } from "./i18n";
import { kindLabel } from "./kinds";
import { SourceBox } from "./SourceBox";

interface Props {
  t: Strings;
  lang: Lang;
  home: string;
  run: <T>(work: () => Promise<T>) => Promise<T | undefined>;
}

const PAGE = 50;

export function SourcesTab({ t, lang, home, run }: Props) {
  const [input, setInput] = useState("");
  const [query, setQuery] = useState("");
  const [offset, setOffset] = useState(0);
  const [page, setPage] = useState<SourceListPage>({ total: 0, rows: [] });
  const [selected, setSelected] = useState<number | null>(null);
  const [detail, setDetail] = useState<CardSource | null>(null);
  const [tick, setTick] = useState(0);
  const [reliability, setReliability] = useState("F");
  const [importNote, setImportNote] = useState("");

  useEffect(() => {
    if (!home) return;
    void (async () => {
      const found = await run(() => api.sources(home, query, offset, PAGE));
      if (found) setPage(found);
    })();
    // reloaded when the folder, the query, the page or the data change
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [home, query, offset, tick]);

  useEffect(() => {
    if (selected === null) {
      setDetail(null);
      return;
    }
    void (async () => {
      const data = await run(() => api.source(home, selected));
      if (data) setDetail(data);
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [home, selected, tick]);

  const search = () => {
    setOffset(0);
    setQuery(input.trim());
  };

  const importChannel = async () => {
    const folder = await pickFolder();
    if (!folder) return;
    const summary = await run(() => api.importTelegram(home, folder, reliability));
    if (!summary) return;
    setImportNote(
      fill(t.importTelegramDone, {
        channel: summary.channel,
        messages: summary.messages,
        fresh: summary.sourcesNew,
        existing: summary.sourcesExisting,
        texts: summary.textsAdded,
        files: summary.filesAdded,
        missing: summary.filesMissing,
        skipped: summary.skipped,
      }),
    );
    setOffset(0);
    setTick(tick + 1);
  };

  if (detail !== null && selected !== null) {
    return (
      <section>
        <button onClick={() => setSelected(null)}>{t.cardBack}</button>
        <p className="muted">
          {page.rows.find((r) => r.id === selected)?.publishedAt ?? ""} · {kindLabel(lang, detail.kind)}
        </p>
        <SourceBox t={t} home={home} source={detail} run={run} onChanged={() => setTick(tick + 1)} />
      </section>
    );
  }

  return (
    <section>
      <div className="source-box">
        <b>{t.importTelegramTitle}</b>
        <p className="muted">{t.importTelegramHint}</p>
        <label>
          {t.importTelegramReliability}{" "}
          <select value={reliability} onChange={(e) => setReliability(e.target.value)}>
            {["A", "B", "C", "D", "E", "F"].map((grade) => (
              <option key={grade} value={grade}>
                {grade}
              </option>
            ))}
          </select>
        </label>{" "}
        <button disabled={!home} onClick={() => void importChannel()}>
          {t.importTelegramPick}
        </button>
        {importNote ? <p className="ok">{importNote}</p> : null}
      </div>

      <form
        onSubmit={(e) => {
          e.preventDefault();
          search();
        }}
      >
        <input
          type="search"
          value={input}
          placeholder={t.sourcesSearch}
          onChange={(e) => setInput(e.target.value)}
        />{" "}
        <button type="submit">{t.sourcesFind}</button>
      </form>
      <p className="muted">{fill(t.sourcesCount, { n: page.total })}</p>
      {page.rows.length === 0 ? (
        <p className="muted">{t.sourcesNone}</p>
      ) : (
        <ul className="source-list">
          {page.rows.map((row) => (
            <li key={row.id}>
              <button className="chip" onClick={() => setSelected(row.id)}>
                {row.publishedAt ?? ""} [{row.reliability}] {row.title}
              </button>
              <div className="muted">
                {row.hasText ? row.excerpt : t.sourcesNoText}
                {row.files > 0 ? ` · ${fill(t.sourcesFilesCount, { n: row.files })}` : ""}
                {row.statements > 0 ? ` · ${fill(t.sourcesStatementsCount, { n: row.statements })}` : ""}
              </div>
            </li>
          ))}
        </ul>
      )}
      <p>
        <button disabled={offset === 0} onClick={() => setOffset(Math.max(0, offset - PAGE))}>
          {t.sourcesPrev}
        </button>{" "}
        <button disabled={offset + PAGE >= page.total} onClick={() => setOffset(offset + PAGE)}>
          {t.sourcesNext}
        </button>
      </p>
    </section>
  );
}

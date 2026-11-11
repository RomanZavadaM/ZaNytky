import { useState } from "react";
import { api, pickSavePath, type ArticleOutput } from "./api";
import type { Strings } from "./i18n";

interface Props {
  t: Strings;
  home: string;
  identity: string;
  run: <T>(work: () => Promise<T>) => Promise<T | undefined>;
  /** Open this subject's card. */
  onOpen: (id: number) => void;
}

/** The analysis of an investigator's article: who the base knows in it, links, documents, new names. */
export function ArticleTab({ t, home, identity, run, onOpen }: Props) {
  const [title, setTitle] = useState("");
  const [text, setText] = useState("");
  const [result, setResult] = useState<ArticleOutput | null>(null);
  const [saved, setSaved] = useState("");
  const identityPath = identity || undefined;

  const analyse = async () => {
    if (!text.trim()) return;
    const out = await run(() => api.article(home, text, title, identityPath));
    if (out) {
      setResult(out);
      setSaved("");
    }
  };

  const saveReport = async () => {
    if (!result) return;
    const path = await pickSavePath("analiz-statti.md");
    if (!path) return;
    const ok = await run(async () => {
      await api.writeText(path, result.markdown);
      return true;
    });
    if (ok) setSaved(path);
  };

  const saveGraph = async () => {
    if (!result) return;
    const path = await pickSavePath("graf-statti.html");
    if (!path) return;
    const ok = await run(async () => {
      const html = await api.graphHtml(home, {
        identityPath,
        ids: result.analysis.known.map((s) => s.id),
        depth: 1,
        withSources: true,
      });
      await api.writeText(path, html);
      return true;
    });
    if (ok) setSaved(path);
  };

  const analysis = result?.analysis;
  return (
    <section>
      <p className="muted">{t.articleHint}</p>
      <div className="row">
        <input
          value={title}
          placeholder={t.articleTitleLabel}
          onChange={(e) => setTitle(e.target.value)}
          style={{ flex: 1 }}
        />
      </div>
      <textarea
        value={text}
        placeholder={t.articleText}
        onChange={(e) => setText(e.target.value)}
        rows={10}
        style={{ width: "100%" }}
      />
      <div className="row">
        <button className="primary" onClick={() => void analyse()} disabled={!home || !text.trim()}>
          {t.articleRun}
        </button>
        {analysis && <button onClick={() => void saveReport()}>{t.articleSaveReport}</button>}
        {analysis && analysis.known.length > 0 && (
          <button onClick={() => void saveGraph()}>{t.articleSaveGraph}</button>
        )}
      </div>
      {saved && <p className="ok">{saved}</p>}
      {analysis && (
        <>
          <h3>{t.articleKnown}</h3>
          {analysis.known.length === 0 ? (
            <p className="muted">{t.articleNothing}</p>
          ) : (
            <table className="grid">
              <thead>
                <tr>
                  <th>{t.entityCol}</th>
                  <th>{t.colType}</th>
                  <th>{t.articleMentions}</th>
                  <th>{t.colParties}</th>
                </tr>
              </thead>
              <tbody>
                {analysis.known.map((s) => (
                  <tr key={s.id}>
                    <td>
                      <button className="chip" onClick={() => onOpen(s.id)}>
                        {s.label}
                      </button>
                    </td>
                    <td>{s.kind}</td>
                    <td>{s.mentions}</td>
                    <td>
                      {s.statements}
                      {s.confirmed > 0 ? ` (✓ ${s.confirmed})` : ""}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
          {analysis.links.length > 0 && (
            <>
              <h3>{t.articleLinks}</h3>
              {analysis.links.map((link, index) => (
                <div key={index}>
                  <strong>
                    {link.from} ↔ {link.to}
                  </strong>
                  <ul>
                    {link.steps.map((step, i) => (
                      <li key={i}>{step}</li>
                    ))}
                  </ul>
                </div>
              ))}
            </>
          )}
          {analysis.documents.length > 0 && (
            <>
              <h3>{t.articleDocs}</h3>
              <ul>
                {analysis.documents.map((d) => (
                  <li key={d.id}>
                    №{d.id} {d.title} ({d.kind}){d.url ? ` ${d.url}` : ""} — {d.subjects.join(", ")}
                  </li>
                ))}
              </ul>
            </>
          )}
          {analysis.unknown.length > 0 && (
            <>
              <h3>{t.articleNew}</h3>
              <ul>
                {analysis.unknown.slice(0, 60).map((f, i) => (
                  <li key={i}>
                    {f.text} ({f.kind}) — {f.count}
                  </li>
                ))}
              </ul>
            </>
          )}
        </>
      )}
    </section>
  );
}

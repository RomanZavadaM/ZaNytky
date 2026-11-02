import { useState } from "react";
import { api, type Found } from "./api";
import { fill, type Lang, type Strings } from "./i18n";
import { kindLabel } from "./kinds";

interface Props {
  t: Strings;
  lang: Lang;
  home: string;
  /** One source (its text), or the whole base. */
  sourceId: number | null;
  run: <T>(work: () => Promise<T>) => Promise<T | undefined>;
}

/** Finds persons, organisations and registry codes in a text (or in all texts) and adds the
 * confirmed ones to the base. The finds are hints; the investigator decides. */
export function DiscoverPanel({ t, lang, home, sourceId, run }: Props) {
  const [found, setFound] = useState<Found[] | null>(null);
  const [added, setAdded] = useState<Record<string, number>>({});

  const search = async () => {
    const result = await run(() =>
      sourceId === null ? api.discoverAll(home, 100) : api.discoverSource(home, sourceId),
    );
    if (result) setFound(result);
  };

  const add = async (item: Found) => {
    const id = await run(() => api.addFound(home, item.kind, item.text));
    if (id !== undefined) setAdded({ ...added, [`${item.kind}:${item.text}`]: id });
  };

  return (
    <div className="source-box">
      <button onClick={() => void search()}>{sourceId === null ? t.discoverRunAll : t.discoverRun}</button>
      <p className="muted">{t.discoverHint}</p>
      {found !== null && found.length === 0 && <p className="muted">{t.discoverNone}</p>}
      {found !== null && found.length > 0 && (
        <table>
          <tbody>
            {found.map((item) => {
              const key = `${item.kind}:${item.text}`;
              const known = item.known ?? added[key] ?? null;
              return (
                <tr key={key}>
                  <td>{item.kind === "code" ? "№" : kindLabel(lang, item.kind)}</td>
                  <td>{item.text}</td>
                  <td className="muted">{fill(t.discoverCount, { n: item.count, s: item.sources })}</td>
                  <td>
                    {known !== null ? (
                      <span className="ok">{t.discoverKnown}</span>
                    ) : item.kind === "code" ? (
                      <span className="muted">{t.discoverNew}</span>
                    ) : (
                      <button onClick={() => void add(item)}>{t.discoverAdd}</button>
                    )}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      )}
    </div>
  );
}

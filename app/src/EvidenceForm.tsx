import { useState } from "react";
import { api, type EvidenceRequest } from "./api";
import type { Lang, Strings } from "./i18n";
import { kindLabel } from "./kinds";

const KINDS = ["registry", "authority", "media", "testimony", "own"];
const GRADES = ["A", "B", "C", "D", "E", "F"];

interface Props {
  t: Strings;
  lang: Lang;
  home: string;
  statementId: number;
  run: <T>(work: () => Promise<T>) => Promise<T | undefined>;
  /** Called after an evidence was added, so the owner reloads the card. */
  onAdded: () => void;
}

/** Attaches a document to a statement: a registry extract, an official statement, an article.
 * Only a registry or an official statement can confirm; anything can refute. */
export function EvidenceForm({ t, lang, home, statementId, run, onAdded }: Props) {
  const [open, setOpen] = useState(false);
  const [kind, setKind] = useState("registry");
  const [reliability, setReliability] = useState("A");
  const [title, setTitle] = useState("");
  const [url, setUrl] = useState("");
  const [date, setDate] = useState("");
  const [text, setText] = useState("");
  const [note, setNote] = useState("");
  const [done, setDone] = useState("");

  const canConfirm = kind === "registry" || kind === "authority";

  const save = async (status?: "confirmed" | "refuted") => {
    const request: EvidenceRequest = {
      statementId,
      kind,
      reliability,
      title: title.trim(),
      url: url.trim() || undefined,
      publishedAt: date.trim() || undefined,
      text: text.trim() || undefined,
      note: note.trim() || undefined,
      status,
    };
    const id = await run(() => api.addEvidence(home, request));
    if (id === undefined) return;
    setDone(t.evidenceDone);
    setTitle("");
    setUrl("");
    setDate("");
    setText("");
    setNote("");
    onAdded();
  };

  if (!open) {
    return (
      <button onClick={() => setOpen(true)} title={t.evidenceAdd}>
        + {t.evidenceAdd}
      </button>
    );
  }
  return (
    <div className="source-box">
      <b>{t.evidenceAdd}</b>
      <div>
        <label>
          {t.evidenceKind}{" "}
          <select value={kind} onChange={(e) => setKind(e.target.value)}>
            {KINDS.map((k) => (
              <option key={k} value={k}>
                {kindLabel(lang, k)}
              </option>
            ))}
          </select>
        </label>{" "}
        <label>
          {t.evidenceReliability}{" "}
          <select value={reliability} onChange={(e) => setReliability(e.target.value)}>
            {GRADES.map((g) => (
              <option key={g} value={g}>
                {g}
              </option>
            ))}
          </select>
        </label>{" "}
        <label>
          {t.evidenceDate} <input value={date} placeholder="2026-11-03" onChange={(e) => setDate(e.target.value)} />
        </label>
      </div>
      <input placeholder={t.evidenceTitle} value={title} onChange={(e) => setTitle(e.target.value)} />
      <input placeholder={t.evidenceUrl} value={url} onChange={(e) => setUrl(e.target.value)} />
      <textarea rows={3} placeholder={t.evidenceText} value={text} onChange={(e) => setText(e.target.value)} />
      <input placeholder={t.evidenceNote} value={note} onChange={(e) => setNote(e.target.value)} />
      <div>
        <button disabled={!title.trim()} onClick={() => void save()}>
          {t.evidenceSave}
        </button>{" "}
        <button disabled={!title.trim() || !canConfirm} onClick={() => void save("confirmed")}>
          {t.evidenceConfirm}
        </button>{" "}
        <button disabled={!title.trim()} onClick={() => void save("refuted")}>
          {t.evidenceRefute}
        </button>{" "}
        <button onClick={() => setOpen(false)}>{t.cardBack}</button>
      </div>
      {!canConfirm && <p className="muted">{t.evidenceNeedDoc}</p>}
      {done && <p className="ok">{done}</p>}
    </div>
  );
}

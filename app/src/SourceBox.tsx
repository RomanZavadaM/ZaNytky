import { useState } from "react";
import { api, pickAnyFile, type CardSource } from "./api";
import type { Strings } from "./i18n";

interface Props {
  t: Strings;
  home: string;
  source: CardSource;
  run: <T>(work: () => Promise<T>) => Promise<T | undefined>;
  /** Called after a text or a file was added, so the owner reloads the data. */
  onChanged: () => void;
}

/** One source: its text (paste, save, copy) and its files (add, show in the folder). */
export function SourceBox({ t, home, source, run, onChanged }: Props) {
  const [draft, setDraft] = useState("");
  const [previews, setPreviews] = useState<Record<number, string | null>>({});

  const togglePreview = async (id: number, relPath: string) => {
    if (previews[id]) {
      setPreviews({ ...previews, [id]: null });
      return;
    }
    const address = await run(() => api.attachmentPreview(home, relPath));
    if (address) setPreviews({ ...previews, [id]: address });
  };

  const saveText = async () => {
    const text = draft.trim();
    if (!text) return;
    await run(() => api.setSourceText(home, source.id, text));
    setDraft("");
    onChanged();
  };

  const addFile = async () => {
    const path = await pickAnyFile();
    if (!path) return;
    await run(() => api.addSourceFile(home, source.id, path));
    onChanged();
  };

  return (
    <div className="source-box">
      <div>
        <b>
          [{source.reliability}] {source.title}
        </b>
        {source.url ? <span className="muted"> {source.url}</span> : null}
      </div>
      {source.text ? (
        <>
          <pre className="source-text">{source.text}</pre>
          <button onClick={() => void navigator.clipboard.writeText(source.text ?? "")}>{t.sourceCopy}</button>
        </>
      ) : (
        <>
          <textarea
            rows={4}
            value={draft}
            placeholder={t.sourceTextPaste}
            onChange={(e) => setDraft(e.target.value)}
          />
          <button disabled={!draft.trim()} onClick={() => void saveText()}>
            {t.sourceTextSave}
          </button>
        </>
      )}
      {source.files.length > 0 && (
        <ul>
          {source.files.map((file) => (
            <li key={file.id}>
              {file.name}{" "}
              <span className="muted">
                ({Math.max(1, Math.round(file.size / 1024))} KB, {file.sha256.slice(0, 12)})
              </span>{" "}
              <button onClick={() => void run(() => api.revealSourceFile(home, file.relPath))}>
                {t.sourceFileShow}
              </button>{" "}
              {(file.mime ?? "").startsWith("image/") && (
                <button onClick={() => void togglePreview(file.id, file.relPath)}>
                  {previews[file.id] ? t.sourceFileHide : t.sourceFilePreview}
                </button>
              )}
              {previews[file.id] && (
                <div>
                  <img src={previews[file.id] ?? undefined} alt={file.name} style={{ maxWidth: "100%", maxHeight: 480 }} />
                </div>
              )}
            </li>
          ))}
        </ul>
      )}
      <button onClick={() => void addFile()}>{t.sourceFileAdd}</button>
    </div>
  );
}

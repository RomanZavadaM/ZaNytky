import { useState } from "react";
import { api, pickSavePath } from "./api";
import type { Strings } from "./i18n";

interface Props {
  t: Strings;
  home: string;
  run: <T>(work: () => Promise<T>) => Promise<T | undefined>;
}

/** The program goes through the whole base and writes one report for the investigator. */
export function OverviewTab({ t, home, run }: Props) {
  const [text, setText] = useState("");
  const [saved, setSaved] = useState("");

  const build = async () => {
    setSaved("");
    const report = await run(() => api.overview(home));
    if (report) setText(report);
  };

  const save = async () => {
    const path = await pickSavePath("ZaNytky-overview.md");
    if (!path) return;
    const ok = await run(async () => {
      await api.writeText(path, text);
      return true;
    });
    if (ok) setSaved(path);
  };

  return (
    <section>
      <p className="muted">{t.overviewHint}</p>
      <button disabled={!home} onClick={() => void build()}>
        {t.overviewRun}
      </button>{" "}
      {text && (
        <>
          <button onClick={() => void navigator.clipboard.writeText(text)}>{t.overviewCopy}</button>{" "}
          <button onClick={() => void save()}>{t.overviewSave}</button>
          {saved && <span className="ok"> {saved}</span>}
          <pre className="source-text" style={{ maxHeight: "none" }}>
            {text}
          </pre>
        </>
      )}
    </section>
  );
}

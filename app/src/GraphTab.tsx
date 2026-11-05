import { useCallback, useEffect, useRef, useState } from "react";
import { api, pickSavePath } from "./api";
import { KnowledgeGraphPanel } from "./graph/KnowledgeGraphPanel";
import { LanguageContext, type LanguageCode } from "./graph/lang";
import type { Lang, Strings } from "./i18n";
import { kindLabel } from "./kinds";

interface Props {
  t: Strings;
  lang: Lang;
  home: string;
  identity: string;
  run: <T>(work: () => Promise<T>) => Promise<T | undefined>;
  onOpen: (id: number) => void;
}

/** The whole base as a floating graph (the MD Notes graph code, fed by the base). */
export function GraphTab({ t, lang, home, identity, run, onOpen }: Props) {
  const [reveal, setReveal] = useState(false);
  // Names are shown by default as soon as the investigator's key is there (the user can switch them off).
  const nameChoice = useRef(false);
  useEffect(() => {
    if (identity && !nameChoice.current) setReveal(true);
  }, [identity]);
  const [refresh, setRefresh] = useState(0);
  const [around, setAround] = useState("");
  const [depth, setDepth] = useState("1");
  const [confirmedOnly, setConfirmedOnly] = useState(false);
  const [saved, setSaved] = useState("");
  const identityPath = reveal && identity ? identity : undefined;
  const load = useCallback(
    async () => (await run(() => api.graph(home, identityPath))) ?? { nodes: [], edges: [], relations: [] },
    // the graph is rebuilt when the folder or the names switch changes
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [home, identityPath, refresh],
  );
  const saveHtml = async () => {
    const path = await pickSavePath("ZaNytky-graph.html");
    if (!path) return;
    const ok = await run(async () => {
      const id = Number(around);
      const html = await api.graphHtml(home, {
        identityPath,
        around: around.trim() && Number.isFinite(id) ? id : undefined,
        depth: Number(depth),
        confirmedOnly,
      });
      await api.writeText(path, html);
      return true;
    });
    if (ok) setSaved(path);
  };
  return (
    <div>
      <div className="row">
        <label className="check">
          <input type="checkbox" checked={reveal} disabled={!identity} onChange={(e) => {
              nameChoice.current = true;
              setReveal(e.target.checked);
            }} />
          {t.revealNames}
        </label>
        <button onClick={() => setRefresh(refresh + 1)}>{t.graphRefresh}</button>
      </div>
      <div className="row">
        <input
          type="number"
          min={1}
          style={{ width: "9em" }}
          placeholder={t.graphAround}
          value={around}
          onChange={(e) => setAround(e.target.value)}
        />
        <select value={depth} onChange={(e) => setDepth(e.target.value)} title={t.graphDepth}>
          <option value="1">1</option>
          <option value="2">2</option>
          <option value="3">3</option>
        </select>
        <label className="check">
          <input type="checkbox" checked={confirmedOnly} onChange={(e) => setConfirmedOnly(e.target.checked)} />
          {t.graphConfirmedOnly}
        </label>
        <button disabled={!home} onClick={() => void saveHtml()}>
          {t.graphSaveHtml}
        </button>
        {saved && <span className="ok"> {saved}</span>}
      </div>
      <LanguageContext.Provider value={(lang as LanguageCode) ?? "uk"}>
        <KnowledgeGraphPanel
          mode="global"
          notePath=""
          refreshKey={refresh}
          load={load}
          lookByKind
          kindName={(kind) => (kind.startsWith("shared:") ? kind.slice(7) : kindLabel(lang, kind))}
          onOpen={(id) => onOpen(Number(id))}
        />
      </LanguageContext.Provider>
    </div>
  );
}

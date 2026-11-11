import { useCallback, useEffect, useRef, useState } from "react";
import { api, pickSavePath } from "./api";
import { KnowledgeGraphPanel } from "./graph/KnowledgeGraphPanel";
import { graphReloadKey } from "./graphReload";
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
  const [reveal, setReveal] = useState(Boolean(identity));
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
  const [kindsAvailable, setKindsAvailable] = useState<string[]>([]);
  const [kinds, setKinds] = useState<string[]>([]);
  const [exclude, setExclude] = useState("");
  const identityPath = reveal && identity ? identity : undefined;
  const load = useCallback(
    async () => {
      const graph = (await run(() => api.graph(home, identityPath))) ?? { nodes: [], edges: [], relations: [] };
      setKindsAvailable(Array.from(new Set(graph.nodes.map((n) => n.noteType ?? "").filter(Boolean))).sort());
      return graph;
    },
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
        kinds,
        exclude: exclude
          .split(/[^0-9]+/)
          .filter(Boolean)
          .map(Number),
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
        <input
          style={{ width: "16em" }}
          placeholder={t.graphExclude}
          value={exclude}
          onChange={(e) => setExclude(e.target.value)}
        />
        <button disabled={!home} onClick={() => void saveHtml()}>
          {t.graphSaveHtml}
        </button>
        {saved && <span className="ok"> {saved}</span>}
      </div>
      {kindsAvailable.length > 0 && (
        <div className="row">
          <span className="muted">{t.graphKinds}:</span>
          {kindsAvailable.map((kind) => (
            <label key={kind} className="check">
              <input
                type="checkbox"
                checked={kinds.includes(kind)}
                onChange={(e) => setKinds(e.target.checked ? [...kinds, kind] : kinds.filter((k) => k !== kind))}
              />
              {kindLabel(lang, kind)}
            </label>
          ))}
        </div>
      )}
      <LanguageContext.Provider value={(lang as LanguageCode) ?? "uk"}>
        <KnowledgeGraphPanel
          mode="global"
          notePath=""
          refreshKey={graphReloadKey(refresh, identityPath)}
          load={load}
          lookByKind
          kindName={(kind) => (kind.startsWith("shared:") ? kind.slice(7) : kindLabel(lang, kind))}
          onOpen={(id) => onOpen(Number(id))}
        />
      </LanguageContext.Provider>
    </div>
  );
}

import { useCallback, useState } from "react";
import { api } from "./api";
import { KnowledgeGraphPanel } from "./graph/KnowledgeGraphPanel";
import { LanguageContext, type LanguageCode } from "./graph/lang";
import type { Lang, Strings } from "./i18n";

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
  const [refresh, setRefresh] = useState(0);
  const identityPath = reveal && identity ? identity : undefined;
  const load = useCallback(
    async () => (await run(() => api.graph(home, identityPath))) ?? { nodes: [], edges: [], relations: [] },
    // the graph is rebuilt when the folder or the names switch changes
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [home, identityPath, refresh],
  );
  return (
    <div>
      <div className="row">
        <label className="check">
          <input type="checkbox" checked={reveal} disabled={!identity} onChange={(e) => setReveal(e.target.checked)} />
          {t.revealNames}
        </label>
        <button onClick={() => setRefresh(refresh + 1)}>{t.graphRefresh}</button>
      </div>
      <LanguageContext.Provider value={(lang as LanguageCode) ?? "uk"}>
        <KnowledgeGraphPanel
          mode="global"
          notePath=""
          refreshKey={refresh}
          load={load}
          onOpen={(id) => onOpen(Number(id))}
        />
      </LanguageContext.Provider>
    </div>
  );
}

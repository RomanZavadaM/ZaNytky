import { useCallback, useEffect, useState } from "react";
import { api } from "./api";
import { KnowledgeGraphPanel } from "./graph/KnowledgeGraphPanel";
import { LanguageContext, type LanguageCode } from "./graph/lang";
import type { KnowledgeGraph } from "./graph/types";
import type { Lang, Strings } from "./i18n";
import { kindLabel } from "./kinds";
import { loadSelection, onSelectionChanged } from "./windowApi";

interface Props {
  t: Strings;
  lang: Lang;
  home: string;
  identity: string;
  run: <T>(work: () => Promise<T>) => Promise<T | undefined>;
}

/** The selected subjects and what is linked to them within `depth` steps. */
export function cutGraph(graph: KnowledgeGraph, ids: number[], depth: number): KnowledgeGraph {
  const reached = new Set(ids.map(String));
  let frontier = new Set(reached);
  for (let step = 0; step < Math.max(1, depth); step += 1) {
    const next = new Set<string>();
    for (const r of graph.relations) {
      for (const [from, to] of [
        [r.source, r.target],
        [r.target, r.source],
      ]) {
        if (frontier.has(from) && !reached.has(to)) {
          reached.add(to);
          next.add(to);
        }
      }
    }
    frontier = next;
  }
  return {
    nodes: graph.nodes.filter((n) => reached.has(n.path)),
    edges: graph.edges.filter((e) => reached.has(e.source) && reached.has(e.target)),
    relations: graph.relations.filter((r) => reached.has(r.source) && reached.has(r.target)),
  };
}

/** The window with the graph of the subjects ticked in the list. */
export function SelectionGraph({ t, lang, home, identity, run }: Props) {
  const [ids, setIds] = useState<number[]>(() => loadSelection());
  const [depth, setDepth] = useState("1");
  useEffect(() => {
    let stop: (() => void) | undefined;
    void onSelectionChanged(() => setIds(loadSelection())).then((unlisten) => {
      stop = unlisten;
    });
    return () => stop?.();
  }, []);
  const key = `${ids.join(",")}|${depth}|${identity}`;
  const load = useCallback(
    async () => {
      const whole = await run(() => api.graph(home, identity || undefined));
      return cutGraph(whole ?? { nodes: [], edges: [], relations: [] }, ids, Number(depth));
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [home, key],
  );
  if (ids.length === 0) return <p className="muted">{t.selectionEmpty}</p>;
  return (
    <div>
      <div className="row">
        <span>{ids.length}</span>
        <select value={depth} onChange={(e) => setDepth(e.target.value)} title={t.graphDepth}>
          <option value="1">1</option>
          <option value="2">2</option>
          <option value="3">3</option>
        </select>
      </div>
      <LanguageContext.Provider value={(lang as LanguageCode) ?? "uk"}>
        <KnowledgeGraphPanel
          mode="global"
          notePath=""
          refreshKey={key}
          load={load}
          lookByKind
          kindName={(kind) => (kind.startsWith("shared:") ? kind.slice(7) : kindLabel(lang, kind))}
          onOpen={() => undefined}
        />
      </LanguageContext.Provider>
    </div>
  );
}

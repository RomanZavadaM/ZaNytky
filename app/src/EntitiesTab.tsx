import { useEffect, useRef, useState } from "react";
import { api, pickSavePath, type CardSource, type CardStatement, type EntityCardData, type EntityListRow, type PathResultData } from "./api";
import { fill, type Lang, type Strings } from "./i18n";
import { kindLabel } from "./kinds";
import { EvidenceForm } from "./EvidenceForm";
import { SourceBox } from "./SourceBox";
import { openSelectionWindow } from "./windowApi";

interface Props {
  t: Strings;
  lang: Lang;
  home: string;
  identity: string;
  run: <T>(work: () => Promise<T>) => Promise<T | undefined>;
  /** Open this entity's card (e.g. after a click in the graph). */
  focusId?: number | null;
}

function period(statement: CardStatement): string {
  const range = [statement.validFrom, statement.validTo].filter(Boolean).join(" – ");
  if (range) return range;
  return statement.asOf ? `≈ ${statement.asOf}` : "";
}

export function EntitiesTab({ t, lang, home, identity, run, focusId }: Props) {
  const [rows, setRows] = useState<EntityListRow[]>([]);
  const [filter, setFilter] = useState("");
  const [reveal, setReveal] = useState(Boolean(identity));
  // Names are shown by default as soon as the investigator's key is there (the user can switch them off).
  const nameChoice = useRef(false);
  useEffect(() => {
    if (identity && !nameChoice.current) setReveal(true);
  }, [identity]);
  const [picked, setPicked] = useState<number[]>([]);
  const [savedPath, setSavedPath] = useState("");
  const [trail, setTrail] = useState<number[]>([]);
  const [card, setCard] = useState<EntityCardData | null>(null);
  const [tick, setTick] = useState(0);
  const [mergeWith, setMergeWith] = useState("");
  const [pathFrom, setPathFrom] = useState("");
  const [pathTo, setPathTo] = useState("");
  const [pathDepth, setPathDepth] = useState(6);
  const [pathRefuted, setPathRefuted] = useState(false);
  const [pathResult, setPathResult] = useState<PathResultData | null>(null);
  const identityPath = reveal && identity ? identity : undefined;
  const current = trail.length > 0 ? trail[trail.length - 1] : null;

  useEffect(() => {
    if (!home) return;
    void (async () => {
      const list = await run(() => api.entities(home, identityPath));
      if (list) setRows(list);
    })();
    // the list is reloaded when the folder or the names switch changes
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [home, identityPath, tick]);

  useEffect(() => {
    if (current === null) {
      setCard(null);
      return;
    }
    void (async () => {
      const data = await run(() => api.entityCard(home, current, identityPath));
      if (data) setCard(data);
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [home, current, identityPath, tick]);

  useEffect(() => {
    if (focusId !== null && focusId !== undefined) setTrail([focusId]);
  }, [focusId]);

  const findPath = async () => {
    if (!pathFrom || !pathTo) return;
    const found = await run(() =>
      api.paths(home, Number(pathFrom), Number(pathTo), pathDepth, pathRefuted, identityPath),
    );
    if (found) setPathResult(found);
  };

  const togglePick = (id: number) =>
    setPicked((now) => (now.includes(id) ? now.filter((x) => x !== id) : [...now, id]));

  const saveDossier = async (ids: number[]) => {
    if (ids.length === 0) return;
    const path = await pickSavePath(ids.length === 1 ? "dosie.md" : "dosie-dobirky.md");
    if (!path) return;
    const ok = await run(async () => {
      await api.writeText(path, await api.dossier(home, ids, identityPath));
      return true;
    });
    if (ok) setSavedPath(path);
  };

  const saveSelectionGraph = async () => {
    if (picked.length === 0) return;
    const path = await pickSavePath("graf-dobirky.html");
    if (!path) return;
    const ok = await run(async () => {
      const html = await api.graphHtml(home, { identityPath, ids: picked, depth: 1, withSources: true });
      await api.writeText(path, html);
      return true;
    });
    if (ok) setSavedPath(path);
  };

  const doMerge = async () => {
    if (current === null || !mergeWith) return;
    await run(() => api.mergeEntities(home, current, Number(mergeWith)));
    setMergeWith("");
    setTick(tick + 1);
  };

  const doUnmerge = async (drop: number) => {
    if (current === null) return;
    await run(() => api.unmergeEntity(home, current, drop));
    setTick(tick + 1);
  };

  const sourcesOf = (data: EntityCardData): CardSource[] => {
    const seen = new Map<number, CardSource>();
    for (const statement of data.statements) {
      for (const source of statement.sources) if (!seen.has(source.id)) seen.set(source.id, source);
    }
    return Array.from(seen.values());
  };

  const open = (id: number) => setTrail([...trail, id]);
  const back = () => setTrail(trail.slice(0, -1));
  const statusLabel = (status: string) =>
    status === "confirmed" ? t.statusConfirmed : status === "refuted" ? t.statusRefuted : t.statusUnverified;

  const chip = (id: number, label: string) => (
    <button key={id} className="chip" onClick={() => open(id)}>
      {label}
    </button>
  );

  const needle = filter.trim().toLowerCase();
  // Those about whom the base knows most come first.
  const shown = rows
    .filter((row) => !needle || row.label.toLowerCase().includes(needle) || (row.code ?? "").includes(needle))
    .sort((a, b) => b.statements - a.statements || a.label.localeCompare(b.label));

  return (
    <section>
      <label className="check">
        <input
          type="checkbox"
          checked={reveal}
          onChange={(e) => {
            nameChoice.current = true;
            setReveal(e.target.checked);
          }}
          disabled={!identity}
        />
        {t.revealNames}
      </label>
      <p className="muted">{t.revealNote}</p>
      {current === null && (
        <>
          <h3>{t.pathTitle}</h3>
          <div className="row">
            <select value={pathFrom} onChange={(e) => setPathFrom(e.target.value)}>
              <option value="">{t.pathFrom}</option>
              {rows.map((row) => (
                <option key={row.id} value={row.id}>
                  {row.label}
                </option>
              ))}
            </select>
            <select value={pathTo} onChange={(e) => setPathTo(e.target.value)}>
              <option value="">{t.pathTo}</option>
              {rows.map((row) => (
                <option key={row.id} value={row.id}>
                  {row.label}
                </option>
              ))}
            </select>
            <select value={pathDepth} onChange={(e) => setPathDepth(Number(e.target.value))} aria-label={t.pathDepth}>
              {[2, 3, 4, 5, 6, 8].map((n) => (
                <option key={n} value={n}>
                  {t.pathDepth}: {n}
                </option>
              ))}
            </select>
            <button onClick={findPath} disabled={!pathFrom || !pathTo || pathFrom === pathTo}>
              {t.pathFind}
            </button>
          </div>
          <label className="check">
            <input type="checkbox" checked={pathRefuted} onChange={(e) => setPathRefuted(e.target.checked)} />
            {t.pathRefuted}
          </label>
          {pathResult && pathResult.paths.length === 0 && <p className="muted">{t.pathNone}</p>}
          {pathResult && pathResult.paths.length > 0 && (
            <>
              <p>{fill(t.pathFound, { n: pathResult.paths.length, len: pathResult.paths[0].length })}</p>
              {pathResult.paths.map((path, index) => (
                <div key={index} style={{ marginBottom: 12 }}>
                  {chip(path.hops[0].fromId, path.hops[0].from)}
                  {path.hops.map((hop) => (
                    <span key={`${hop.fromId}-${hop.toId}`}>
                      {" "}
                      <span className="muted">
                        —
                        {hop.statements
                          .map(
                            (s) =>
                              ` ${kindLabel(lang, s.kind)}${s.value ? ` ${s.value}` : ""} [${s.grade}, ${statusLabel(s.status)}]`,
                          )
                          .join(";")}
                        {hop.signs.map((s) => ` ⚠ ${fill(t.pathShared, { kind: kindLabel(lang, s.kind) })}${s.text ? `: ${s.text}` : ""}`).join(";")}{" "}
                        →
                      </span>{" "}
                      {chip(hop.toId, hop.to)}
                    </span>
                  ))}
                </div>
              ))}
            </>
          )}
          <div className="row">
            <input
              value={filter}
              placeholder={t.entitiesSearch}
              onChange={(e) => setFilter(e.target.value)}
              style={{ flex: 1 }}
            />
          </div>
          {picked.length > 0 && (
            <div className="row">
              <span>{fill(t.dossierSelected, { count: picked.length })}</span>
              <button onClick={() => void saveDossier(picked)}>{t.dossierMany}</button>
              <button onClick={() => void saveSelectionGraph()}>{t.dossierGraph}</button>
              <button onClick={() => void openSelectionWindow(t.selectionTitle, picked).catch(() => undefined)}>
                {t.selectionWindow}
              </button>
              <button onClick={() => setPicked([])}>{t.dossierClear}</button>
            </div>
          )}
          {savedPath && <p className="ok">{savedPath}</p>}
          {rows.length === 0 ? (
            <p className="muted">{t.entitiesNone}</p>
          ) : (
            <table className="grid">
              <thead>
                <tr>
                  <th title={t.dossierPick}>✓</th>
                  <th>{t.entityCol}</th>
                  <th>{t.colType}</th>
                  <th>{t.cardCode}</th>
                  <th>{t.colParties}</th>
                  <th>{t.sharedCol}</th>
                </tr>
              </thead>
              <tbody>
                {shown.map((row) => (
                  <tr key={row.id}>
                    <td>
                      <input
                        type="checkbox"
                        checked={picked.includes(row.id)}
                        onChange={() => togglePick(row.id)}
                        aria-label={t.dossierPick}
                      />
                    </td>
                    <td>{chip(row.id, row.label)}</td>
                    <td>{kindLabel(lang, row.kind)}</td>
                    <td>{row.code ?? ""}</td>
                    <td>{row.statements}</td>
                    <td>
                      {row.shared > 0 && (
                        <span className="shared-badge" title={t.sharedHint}>
                          ⚠ {row.shared}
                        </span>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </>
      )}
      {current !== null && card && (
        <>
          <div className="row">
            <button onClick={back}>{t.cardBack}</button>
            <button onClick={() => void saveDossier([card.id])}>{t.dossierOne}</button>
          </div>
          {savedPath && <p className="ok">{savedPath}</p>}
          <h3>{card.label}</h3>
          <p className="muted">
            {kindLabel(lang, card.kind)}
            {card.code ? ` · ${t.cardCode}: ${card.code}` : ""}
          </p>
          {card.shared.length > 0 && (
            <div className="shared-box">
              <h3>⚠ {t.sharedTitle}</h3>
              <p className="muted">{t.sharedHint}</p>
              {card.shared.map((sign) => (
                <p key={`${sign.kind}-${sign.value}`}>
                  <strong>{kindLabel(lang, sign.kind)}</strong> <code>{sign.text ?? sign.value}</code>:{" "}
                  {sign.others.map((o) => chip(o.id, o.label))}
                </p>
              ))}
            </div>
          )}
          {card.merged.length > 0 && (
            <div>
              <h3>{t.mergedTitle}</h3>
              {card.merged.map((m) => (
                <p key={m.id}>
                  {m.label}{" "}
                  <button onClick={() => doUnmerge(m.id)}>{t.mergeUndo}</button>
                </p>
              ))}
            </div>
          )}
          {card.kind !== "person" && (
            <div className="row">
              <select value={mergeWith} onChange={(e) => setMergeWith(e.target.value)}>
                <option value="">{t.mergePick}</option>
                {rows
                  .filter((r) => r.kind === card.kind && r.id !== card.id)
                  .map((r) => (
                    <option key={r.id} value={r.id}>
                      {r.label}
                    </option>
                  ))}
              </select>
              <button onClick={doMerge} disabled={!mergeWith}>
                {t.mergeDo}
              </button>
            </div>
          )}
          {card.neighbors.length > 0 && (
            <>
              <h3>{t.cardNeighbors}</h3>
              <p>
                {card.neighbors.map((n) => (
                  <span key={n.id}>
                    {chip(n.id, n.label)} <span className="muted">×{n.count}</span>{" "}
                  </span>
                ))}
              </p>
            </>
          )}
          <h3>{t.cardRelations}</h3>
          {card.statements.length === 0 ? (
            <p className="muted">{t.cardEmpty}</p>
          ) : (
            <div className="stmts">
              {card.statements.map((s) => (
                <div key={s.id} className={`stmt stmt-${s.status}`}>
                  <div className="stmt-head">
                    <b>{kindLabel(lang, s.kind)}</b>
                    <span className="muted">{s.role.split(", ").map((r) => kindLabel(lang, r)).join(", ")}</span>
                    <span className="stmt-status">{statusLabel(s.status)}</span>
                  </div>
                  {s.value && <p className="stmt-value">{s.value}</p>}
                  {s.parties.length > 0 && (
                    <p className="stmt-parties">
                      {s.parties.map((p) => (
                        <span key={`${p.id}-${p.role}`}>
                          {chip(p.id, p.label)} <span className="muted">{kindLabel(lang, p.role)}</span>{" "}
                        </span>
                      ))}
                    </p>
                  )}
                  <p className="muted">
                    #{s.id} · {s.grade} · {s.independent} {t.cardIndependent}
                    {period(s) ? ` · ${period(s)}` : ""}
                  </p>
                  <details>
                    <summary>
                      {t.cardMoreSources} ({s.sources.length})
                    </summary>
                    {s.sources.map((source, index) => (
                      <p key={index} className="stmt-source">
                        [{source.reliability}] {source.title}
                      </p>
                    ))}
                    {s.note ? <p className="muted stmt-source">{s.note}</p> : null}
                    <EvidenceForm
                      t={t}
                      lang={lang}
                      home={home}
                      statementId={s.id}
                      run={run}
                      onAdded={() => setTick(tick + 1)}
                    />
                  </details>
                </div>
              ))}
            </div>
          )}
          {sourcesOf(card).length > 0 && (
            <>
              <h3>{t.cardSources}</h3>
              {sourcesOf(card).map((source) => (
                <SourceBox
                  key={source.id}
                  t={t}
                  home={home}
                  source={source}
                  run={run}
                  onChanged={() => setTick(tick + 1)}
                />
              ))}
            </>
          )}
          {card.owners.length > 0 && (
            <>
              <h3>{t.cardOwners}</h3>
              <ul>
                {card.owners.map((owner) => (
                  <li key={`${owner.statement}-${owner.depth}`}>
                    {chip(owner.ownedId, owner.owned)} ← {chip(owner.ownerId, owner.owner)}{" "}
                    {owner.value ? <b>{owner.value}</b> : null}{" "}
                    <span className="muted">(#{owner.statement}, {owner.depth})</span>
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

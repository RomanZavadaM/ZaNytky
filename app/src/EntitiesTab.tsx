import { useEffect, useState } from "react";
import { api, type CardSource, type CardStatement, type EntityCardData, type EntityListRow, type PathResultData } from "./api";
import { fill, type Lang, type Strings } from "./i18n";
import { kindLabel } from "./kinds";
import { SourceBox } from "./SourceBox";

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
  const [reveal, setReveal] = useState(false);
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
  const shown = rows.filter(
    (row) => !needle || row.label.toLowerCase().includes(needle) || (row.code ?? "").includes(needle),
  );

  return (
    <section>
      <label className="check">
        <input type="checkbox" checked={reveal} onChange={(e) => setReveal(e.target.checked)} disabled={!identity} />
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
          {rows.length === 0 ? (
            <p className="muted">{t.entitiesNone}</p>
          ) : (
            <table className="grid">
              <thead>
                <tr>
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
          </div>
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
          <h3>{t.cardRelations}</h3>
          {card.statements.length === 0 ? (
            <p className="muted">{t.cardEmpty}</p>
          ) : (
            <table className="grid">
              <thead>
                <tr>
                  <th>№</th>
                  <th>{t.colType}</th>
                  <th>{t.cardRoleHere}</th>
                  <th>{t.colParties}</th>
                  <th>{t.colValue}</th>
                  <th>{t.colGrade}</th>
                  <th>{t.colStatus}</th>
                  <th>{t.cardSources}</th>
                </tr>
              </thead>
              <tbody>
                {card.statements.map((s) => (
                  <tr key={s.id}>
                    <td>{s.id}</td>
                    <td>{kindLabel(lang, s.kind)}</td>
                    <td>{s.role.split(", ").map((r) => kindLabel(lang, r)).join(", ")}</td>
                    <td>
                      {s.parties.map((p) => (
                        <span key={`${p.id}-${p.role}`}>
                          {chip(p.id, p.label)} <span className="muted">({kindLabel(lang, p.role)})</span>{" "}
                        </span>
                      ))}
                    </td>
                    <td>
                      {s.value}
                      {period(s) ? <div className="muted">{period(s)}</div> : null}
                    </td>
                    <td>
                      {s.grade}
                      <div className="muted">
                        {s.independent} {t.cardIndependent}
                      </div>
                    </td>
                    <td>{statusLabel(s.status)}</td>
                    <td className="note">
                      {s.sources.map((source, index) => (
                        <div key={index}>
                          [{source.reliability}] {source.title}
                        </div>
                      ))}
                      {s.note ? <div className="muted">{s.note}</div> : null}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
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
        </>
      )}
    </section>
  );
}

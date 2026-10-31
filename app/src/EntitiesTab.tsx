import { useEffect, useState } from "react";
import { api, type CardStatement, type EntityCardData, type EntityListRow } from "./api";
import type { Lang, Strings } from "./i18n";
import { kindLabel } from "./kinds";

interface Props {
  t: Strings;
  lang: Lang;
  home: string;
  identity: string;
  run: <T>(work: () => Promise<T>) => Promise<T | undefined>;
}

function period(statement: CardStatement): string {
  const range = [statement.validFrom, statement.validTo].filter(Boolean).join(" – ");
  if (range) return range;
  return statement.asOf ? `≈ ${statement.asOf}` : "";
}

export function EntitiesTab({ t, lang, home, identity, run }: Props) {
  const [rows, setRows] = useState<EntityListRow[]>([]);
  const [filter, setFilter] = useState("");
  const [reveal, setReveal] = useState(false);
  const [trail, setTrail] = useState<number[]>([]);
  const [card, setCard] = useState<EntityCardData | null>(null);
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
  }, [home, identityPath]);

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
  }, [home, current, identityPath]);

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
                </tr>
              </thead>
              <tbody>
                {shown.map((row) => (
                  <tr key={row.id}>
                    <td>{chip(row.id, row.label)}</td>
                    <td>{kindLabel(lang, row.kind)}</td>
                    <td>{row.code ?? ""}</td>
                    <td>{row.statements}</td>
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

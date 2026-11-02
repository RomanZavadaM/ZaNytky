import { useEffect, useState } from "react";
import { api, type EntityListRow, type PartyRequest } from "./api";
import { fill, type Lang, type Strings } from "./i18n";
import { kindLabel } from "./kinds";

const STATEMENT_KINDS = [
  "owns",
  "beneficiary",
  "director",
  "founder",
  "won",
  "customer",
  "supplies",
  "uses",
  "coordinated",
  "member_of_group",
  "operates_in_market",
  "reported_link",
];
const ENTITY_KINDS = ["person", "org", "group", "asset", "procurement", "market", "event", "document"];
const ROLES = ["owner", "owned", "person", "company", "procurement", "customer", "supplier", "member", "group"];

interface PartyDraft {
  role: string;
  mode: "existing" | "new";
  filter: string;
  entityId: string;
  kind: string;
  label: string;
  code: string;
}

const emptyParty = (): PartyDraft => ({
  role: "",
  mode: "existing",
  filter: "",
  entityId: "",
  kind: "org",
  label: "",
  code: "",
});

interface Props {
  t: Strings;
  lang: Lang;
  home: string;
  identity: string;
  sourceId: number;
  run: <T>(work: () => Promise<T>) => Promise<T | undefined>;
  /** Called after a statement was made, so the owner reloads the data. */
  onCreated: () => void;
}

/** Makes an (unverified) statement from the open source: parties are existing or new entities. */
export function StatementForm({ t, lang, home, identity, sourceId, run, onCreated }: Props) {
  const [reveal, setReveal] = useState(false);
  const [entities, setEntities] = useState<EntityListRow[]>([]);
  const [kind, setKind] = useState("owns");
  const [value, setValue] = useState("");
  const [credibility, setCredibility] = useState(4);
  const [quote, setQuote] = useState("");
  const [note, setNote] = useState("");
  const [parties, setParties] = useState<PartyDraft[]>([emptyParty(), emptyParty()]);
  const [done, setDone] = useState("");
  const [reload, setReload] = useState(0);
  const identityPath = reveal && identity ? identity : undefined;

  useEffect(() => {
    void (async () => {
      const list = await run(() => api.entities(home, identityPath));
      if (list) setEntities(list);
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [home, identityPath, reload]);

  const setParty = (index: number, patch: Partial<PartyDraft>) =>
    setParties(parties.map((p, i) => (i === index ? { ...p, ...patch } : p)));

  const matches = (party: PartyDraft) => {
    const needle = party.filter.trim().toLowerCase();
    const found = entities.filter(
      (e) => !needle || e.label.toLowerCase().includes(needle) || (e.code ?? "").toLowerCase().includes(needle),
    );
    return found.slice(0, 40);
  };

  const ready =
    quote.trim() !== "" &&
    parties.every(
      (p) => p.role.trim() !== "" && (p.mode === "existing" ? p.entityId !== "" : p.label.trim() !== ""),
    );

  const save = async () => {
    const request = {
      sourceId,
      kind,
      value: value.trim() || undefined,
      credibility,
      quote: quote.trim(),
      note: note.trim() || undefined,
      parties: parties.map((p): PartyRequest =>
        p.mode === "existing"
          ? { role: p.role.trim(), entityId: Number(p.entityId) }
          : { role: p.role.trim(), kind: p.kind, label: p.label.trim(), publicCode: p.code.trim() || undefined },
      ),
    };
    const id = await run(() => api.createStatement(home, request));
    if (id === undefined) return;
    setDone(fill(t.statementMade, { id }));
    setQuote("");
    setNote("");
    setValue("");
    setParties([emptyParty(), emptyParty()]);
    setReload(reload + 1);
    onCreated();
  };

  return (
    <div className="source-box">
      <b>{t.statementMakeTitle}</b>
      <p className="muted">{t.statementMakeHint}</p>
      <datalist id="party-roles">
        {ROLES.map((r) => (
          <option key={r} value={r} />
        ))}
      </datalist>
      <div>
        <label>
          {t.statementKind}{" "}
          <select value={kind} onChange={(e) => setKind(e.target.value)}>
            {STATEMENT_KINDS.map((k) => (
              <option key={k} value={k}>
                {kindLabel(lang, k)}
              </option>
            ))}
          </select>
        </label>{" "}
        <label>
          {t.statementValue} <input value={value} onChange={(e) => setValue(e.target.value)} />
        </label>{" "}
        <label>
          {t.statementCredibility}{" "}
          <select value={credibility} onChange={(e) => setCredibility(Number(e.target.value))}>
            {[1, 2, 3, 4, 5, 6].map((n) => (
              <option key={n} value={n}>
                {n}
              </option>
            ))}
          </select>
        </label>
      </div>
      <label>
        <input type="checkbox" checked={reveal} onChange={(e) => setReveal(e.target.checked)} disabled={!identity} />
        {t.revealNames}
      </label>
      {parties.map((party, index) => (
        <div key={index} className="source-box">
          <label>
            {t.statementRole}{" "}
            <input
              list="party-roles"
              value={party.role}
              onChange={(e) => setParty(index, { role: e.target.value })}
            />
          </label>{" "}
          <select value={party.mode} onChange={(e) => setParty(index, { mode: e.target.value as "existing" | "new" })}>
            <option value="existing">{t.partyExisting}</option>
            <option value="new">{t.partyNew}</option>
          </select>
          {party.mode === "existing" ? (
            <div>
              <input
                placeholder={t.partySearch}
                value={party.filter}
                onChange={(e) => setParty(index, { filter: e.target.value, entityId: "" })}
              />{" "}
              <select value={party.entityId} onChange={(e) => setParty(index, { entityId: e.target.value })}>
                <option value="">-</option>
                {matches(party).map((e) => (
                  <option key={e.id} value={e.id}>
                    {e.label} ({kindLabel(lang, e.kind)}
                    {e.code ? `, ${e.code}` : ""})
                  </option>
                ))}
              </select>
            </div>
          ) : (
            <div>
              <select value={party.kind} onChange={(e) => setParty(index, { kind: e.target.value })}>
                {ENTITY_KINDS.map((k) => (
                  <option key={k} value={k}>
                    {kindLabel(lang, k)}
                  </option>
                ))}
              </select>{" "}
              <input
                placeholder={party.kind === "person" ? t.partyNewPerson : t.partyNewName}
                value={party.label}
                onChange={(e) => setParty(index, { label: e.target.value })}
              />{" "}
              {party.kind !== "person" && (
                <input
                  placeholder={t.partyNewCode}
                  value={party.code}
                  onChange={(e) => setParty(index, { code: e.target.value })}
                />
              )}
            </div>
          )}
        </div>
      ))}
      <button onClick={() => setParties([...parties, emptyParty()])} disabled={parties.length >= 4}>
        {t.statementAddParty}
      </button>
      <div>
        <textarea
          rows={3}
          placeholder={t.statementQuote}
          value={quote}
          onChange={(e) => setQuote(e.target.value)}
        />
        <input placeholder={t.statementNote} value={note} onChange={(e) => setNote(e.target.value)} />
      </div>
      <button disabled={!ready} onClick={() => void save()}>
        {t.statementSave}
      </button>
      {done ? <p className="ok">{done}</p> : null}
    </div>
  );
}

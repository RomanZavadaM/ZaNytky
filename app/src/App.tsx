import { useEffect, useState } from "react";
import {
  api,
  errorText,
  pickFile,
  pickFolder,
  pickSavePath,
  type ImportOut,
  type PrepareOutput,
  type StatementRow,
} from "./api";
import { fill, LANGS, PACKS, type Lang, type Strings } from "./i18n";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { notifyDataChanged, onDataChanged, openSectionWindow, windowSection } from "./windowApi";

type Tab = "setup" | "prepare" | "result" | "database" | "report";

/** True when any text box in this window still holds unsaved text. */
function hasUnsavedText(): boolean {
  return Array.from(document.querySelectorAll("textarea")).some((box) => box.value.trim() !== "");
}

function useStored(key: string, initial: string): [string, (value: string) => void] {
  const [value, setValue] = useState(() => {
    try {
      return localStorage.getItem(key) ?? initial;
    } catch {
      return initial;
    }
  });
  // Other windows of the app share the same storage: pick up their changes.
  useEffect(() => {
    const onStorage = (event: StorageEvent) => {
      if (event.key === key && event.newValue !== null) setValue(event.newValue);
    };
    window.addEventListener("storage", onStorage);
    return () => window.removeEventListener("storage", onStorage);
  }, [key]);
  const update = (next: string) => {
    setValue(next);
    try {
      localStorage.setItem(key, next);
    } catch {
      // storage may be unavailable; the value just is not remembered
    }
  };
  return [value, update];
}

interface Shared {
  t: Strings;
  home: string;
  identity: string;
  note: (message: string, isError?: boolean) => void;
  busy: boolean;
  run: <T>(work: () => Promise<T>) => Promise<T | undefined>;
}

function SetupTab({ t, home, identity, note, run, setHome, setIdentity }: Shared & {
  setHome: (v: string) => void;
  setIdentity: (v: string) => void;
}) {
  const [recipients, setRecipients] = useState<number | null>(null);
  const [initialised, setInitialised] = useState<boolean | null>(null);
  const [backupPass, setBackupPass] = useState("");

  const refresh = async (folder: string) => {
    if (!folder) return;
    const status = await run(() => api.homeStatus(folder));
    if (status) {
      setInitialised(status.initialised);
      setRecipients(status.recipients);
    }
  };

  useEffect(() => {
    void refresh(home);
    // the state of the saved folder is checked once at start
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const chooseHome = async () => {
    const folder = await pickFolder();
    if (folder) {
      setHome(folder);
      await refresh(folder);
    }
  };

  const initHome = async () => {
    if (await run(() => api.initHome(home))) await refresh(home);
  };

  const createIdentity = async () => {
    const path = await pickSavePath("investigator-key.txt");
    if (!path) return;
    const created = await run(() => api.newIdentity(home, path));
    if (created) {
      setIdentity(path);
      note(t.identityCreated);
      await refresh(home);
    }
  };

  const chooseIdentity = async () => {
    const path = await pickFile(["txt", "key", "age"]);
    if (!path) return;
    try {
      await api.checkIdentity(path);
      setIdentity(path);
    } catch {
      note(t.notAKeyFile, true);
    }
  };

  const passOk = backupPass.length >= 8;

  const makeBackup = async () => {
    const path = await pickSavePath("zanytky-backup.znbak");
    if (!path) return;
    const done = await run(() => api.backup(home, path, backupPass));
    if (done) note(fill(t.backupDone, { p: done }));
  };

  const restoreBackup = async () => {
    const file = await pickFile(["znbak"]);
    if (!file) return;
    const folder = await pickFolder();
    if (!folder) return;
    const count = await run(() => api.restore(file, folder, backupPass));
    if (count !== undefined) {
      setHome(folder);
      note(fill(t.restoreDone, { n: count }));
      await refresh(folder);
    }
  };

  return (
    <section>
      <p className="muted">{t.privacyNote}</p>
      <h3>{t.homeFolder}</h3>
      <p className="path">{home || t.noFolder}</p>
      <div className="row">
        <button onClick={chooseHome}>{t.chooseFolder}</button>
        <button onClick={initHome} disabled={!home}>
          {t.initHome}
        </button>
      </div>
      {initialised !== null && (
        <p className={initialised ? "ok" : "warn"}>
          {initialised ? t.homeReady : t.homeNotReady}
          {recipients !== null ? ` · ${fill(t.recipientsCount, { n: recipients })}` : ""}
        </p>
      )}
      <h3>{t.identityFile}</h3>
      <p className="path">{identity || t.noIdentity}</p>
      <div className="row">
        <button onClick={chooseIdentity}>{t.chooseIdentity}</button>
        <button onClick={createIdentity} disabled={!initialised}>
          {t.createIdentity}
        </button>
      </div>
      <h3>{t.backupTitle}</h3>
      <p className="muted">{t.backupNote}</p>
      <div className="form">
        <label>
          {t.backupPassphrase}
          <input type="password" value={backupPass} onChange={(e) => setBackupPass(e.target.value)} />
        </label>
        <div className="row">
          <button onClick={makeBackup} disabled={!initialised || !passOk}>
            {t.backupMake}
          </button>
          <button onClick={restoreBackup} disabled={!passOk}>
            {t.backupRestore}
          </button>
        </div>
        {!passOk && <p className="warn">{t.backupNeedPass}</p>}
      </div>
    </section>
  );
}

function Candidates({ t, home, candidates, note, run }: Shared & { candidates: PrepareOutput["candidates"] }) {
  const [open, setOpen] = useState<string | null>(null);
  const [action, setAction] = useState<"same" | "new" | "not_person">("new");
  const [person, setPerson] = useState("");
  const [name, setName] = useState("");
  const [ipn, setIpn] = useState("");
  const [phone, setPhone] = useState("");
  const [email, setEmail] = useState("");

  const apply = async (mention: string) => {
    const done = await run(() =>
      api.resolve({ home, mention, action, person, name, ipn, phone, email }),
    );
    if (done !== undefined) {
      note(t.resolved);
      setOpen(null);
    }
  };

  if (candidates.length === 0) return <p className="muted">{t.noCandidates}</p>;
  return (
    <ul className="candidates">
      {candidates.map((c) => (
        <li key={c.text}>
          <div className="row">
            <strong>{c.text}</strong>
            <span className="muted">{fill(t.mentionedTimes, { n: c.count })}</span>
            <button onClick={() => setOpen(open === c.text ? null : c.text)}>…</button>
          </div>
          {open === c.text && (
            <div className="form">
              <select value={action} onChange={(e) => setAction(e.target.value as typeof action)}>
                <option value="new">{t.actionNew}</option>
                <option value="same">{t.actionSame}</option>
                <option value="not_person">{t.actionNotPerson}</option>
              </select>
              {action === "same" && (
                <input placeholder={t.personToken} value={person} onChange={(e) => setPerson(e.target.value)} />
              )}
              {action === "new" && (
                <>
                  <input placeholder={t.fullName} value={name} onChange={(e) => setName(e.target.value)} />
                  <input placeholder={t.ipn} value={ipn} onChange={(e) => setIpn(e.target.value)} />
                  <input placeholder={t.phone} value={phone} onChange={(e) => setPhone(e.target.value)} />
                  <input placeholder={t.email} value={email} onChange={(e) => setEmail(e.target.value)} />
                </>
              )}
              <button className="primary" onClick={() => apply(c.text)}>
                {t.apply}
              </button>
            </div>
          )}
        </li>
      ))}
    </ul>
  );
}

function PrepareTab(shared: Shared) {
  const { t, home, identity, run } = shared;
  const [title, setTitle] = useState("");
  const [kind, setKind] = useState("media");
  const [url, setUrl] = useState("");
  const [id, setId] = useState("pkg-1");
  const [text, setText] = useState("");
  const [context, setContext] = useState("");
  const [out, setOut] = useState<PrepareOutput | null>(null);
  const [copied, setCopied] = useState("");

  const openFile = async () => {
    const path = await pickFile(["md", "txt", "markdown", "docx"]);
    if (!path) return;
    const content = await run(() => api.readTextFile(path));
    if (content !== undefined) setText(content);
  };

  const make = async () => {
    const result = await run(() =>
      api.prepare({ home, text, id, title, kind, url, context, identityPath: identity }),
    );
    if (result) setOut(result);
  };

  const copy = async (what: string, value: string) => {
    try {
      await navigator.clipboard.writeText(value);
      setCopied(what);
    } catch {
      setCopied("");
    }
  };

  return (
    <section>
      <div className="form">
        <label>
          {t.docTitle}
          <input value={title} onChange={(e) => setTitle(e.target.value)} />
        </label>
        <label>
          {t.sourceKind}
          <select value={kind} onChange={(e) => setKind(e.target.value)}>
            <option value="media">{t.kindMedia}</option>
            <option value="registry">{t.kindRegistry}</option>
            <option value="authority">{t.kindAuthority}</option>
            <option value="testimony">{t.kindTestimony}</option>
            <option value="rumor">{t.kindRumor}</option>
            <option value="own">{t.kindOwn}</option>
          </select>
        </label>
        <label>
          {t.sourceUrl}
          <input value={url} onChange={(e) => setUrl(e.target.value)} />
        </label>
        <label>
          {t.packageId}
          <input value={id} onChange={(e) => setId(e.target.value)} />
        </label>
        <label>
          {t.docText}
          <textarea rows={10} value={text} onChange={(e) => setText(e.target.value)} />
        </label>
        <div className="row">
          <button onClick={openFile}>{t.openTextFile}</button>
        </div>
        <label>
          {t.contextNote}
          <textarea rows={3} value={context} onChange={(e) => setContext(e.target.value)} />
        </label>
        <div className="row">
          <button className="primary" onClick={make} disabled={!home || !text.trim()}>
            {t.makePackage}
          </button>
        </div>
        {!home && <p className="warn">{t.needHome}</p>}
      </div>
      {out && (
        <>
          <h3>
            {t.deidentifiedText} <span className="muted">({fill(t.replacementsCount, { n: out.replacements })})</span>
          </h3>
          <textarea rows={8} readOnly value={out.deidentified} />
          <div className="row">
            <button onClick={() => copy("text", out.deidentified)}>{copied === "text" ? t.copied : t.copy}</button>
          </div>
          <h3>{t.promptText}</h3>
          <textarea rows={8} readOnly value={out.prompt} />
          <div className="row">
            <button onClick={() => copy("prompt", out.prompt)}>{copied === "prompt" ? t.copied : t.copy}</button>
          </div>
          <h3>{t.candidatesTitle}</h3>
          <Candidates {...shared} candidates={out.candidates} />
        </>
      )}
    </section>
  );
}

function ResultTab({ t, home, note, run }: Shared) {
  const [pkg, setPkg] = useState("pkg-1");
  const [json, setJson] = useState("");
  const [review, setReview] = useState("");

  const check = async () => {
    const text = await run(() => api.check(home, json, pkg));
    if (text !== undefined) setReview(text);
  };

  const importNow = async () => {
    const done: ImportOut | undefined = await run(() => api.importResult(home, json, pkg));
    if (done) {
      note(
        fill(t.importedMsg, {
          s: done.sources,
          e: done.entitiesNew,
          x: done.entitiesExisting,
          t: done.statements,
        }),
      );
      void notifyDataChanged();
    }
  };

  return (
    <section>
      <div className="form">
        <label>
          {t.packageId}
          <input value={pkg} onChange={(e) => setPkg(e.target.value)} />
        </label>
        <label>
          {t.pasteResult}
          <textarea rows={12} value={json} onChange={(e) => setJson(e.target.value)} />
        </label>
        {!home && <p className="warn">{t.needHome}</p>}
        <div className="row">
          <button onClick={check} disabled={!json.trim()}>
            {t.check}
          </button>
          <button className="primary" onClick={importNow} disabled={!review}>
            {t.importBtn}
          </button>
        </div>
      </div>
      {review && (
        <>
          <h3>{t.reviewTitle}</h3>
          <pre className="review">{review}</pre>
        </>
      )}
    </section>
  );
}

function DatabaseTab({ t, home, run }: Shared) {
  const [rows, setRows] = useState<StatementRow[]>([]);

  const load = async () => {
    if (!home) return;
    const list = await run(() => api.statements(home));
    if (list) setRows(list);
  };

  const change = async (id: number, status: "confirmed" | "refuted") => {
    if ((await run(() => api.setStatus(home, id, status))) !== undefined) {
      await load();
      void notifyDataChanged();
    }
  };

  useEffect(() => {
    void load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [home]);

  // Another window imported results or changed a status: reload the list.
  useEffect(() => {
    const unlisten = onDataChanged(() => void load());
    return () => {
      void unlisten.then((stop) => stop());
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [home]);

  const statusLabel = (status: string) =>
    status === "confirmed" ? t.statusConfirmed : status === "refuted" ? t.statusRefuted : t.statusUnverified;

  return (
    <section>
      <p className="muted">{t.dbPrivacyNote}</p>
      <div className="row">
        <button onClick={load} disabled={!home}>
          {t.refresh}
        </button>
      </div>
      {rows.length === 0 ? (
        <p className="muted">{t.noStatements}</p>
      ) : (
        <table className="grid">
          <thead>
            <tr>
              <th>№</th>
              <th>{t.colType}</th>
              <th>{t.colParties}</th>
              <th>{t.colValue}</th>
              <th>{t.colGrade}</th>
              <th>{t.colStatus}</th>
              <th>{t.colNote}</th>
              <th />
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr key={r.id}>
                <td>{r.id}</td>
                <td>{r.kind}</td>
                <td>{r.parties.join("; ")}</td>
                <td>{r.value}</td>
                <td>{r.grade}</td>
                <td>{statusLabel(r.status)}</td>
                <td className="note">{r.note}</td>
                <td>
                  <button onClick={() => change(r.id, "confirmed")} disabled={r.status === "confirmed"}>
                    {t.confirmBtn}
                  </button>{" "}
                  <button onClick={() => change(r.id, "refuted")} disabled={r.status === "refuted"}>
                    {t.refuteBtn}
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

function ReportTab({ t, home, identity, note, run }: Shared) {
  const [title, setTitle] = useState("");
  const [format, setFormat] = useState<"md" | "docx" | "pdf">("docx");
  const [passphrase, setPassphrase] = useState("");
  const [plain, setPlain] = useState(false);

  const make = async () => {
    const path = await pickSavePath(`report.${format}`);
    if (!path) return;
    const saved = await run(() =>
      api.report({ home, identityPath: identity, title, format, passphrase, plain, outPath: path }),
    );
    if (saved) note(fill(t.saved, { p: saved }));
  };

  return (
    <section>
      <div className="form">
        <label>
          {t.reportTitle}
          <input value={title} onChange={(e) => setTitle(e.target.value)} />
        </label>
        <label>
          {t.format}
          <select value={format} onChange={(e) => setFormat(e.target.value as typeof format)}>
            <option value="docx">Word (.docx)</option>
            <option value="pdf">PDF</option>
            <option value="md">Markdown</option>
          </select>
        </label>
        <label>
          {t.passphrase}
          <input type="password" value={passphrase} onChange={(e) => setPassphrase(e.target.value)} />
        </label>
        <label className="check">
          <input type="checkbox" checked={plain} onChange={(e) => setPlain(e.target.checked)} />
          {t.plain}
        </label>
        <div className="row">
          <button className="primary" onClick={make} disabled={!home || !identity}>
            {t.makeReport}
          </button>
        </div>
        {!home && <p className="warn">{t.needHome}</p>}
        {home && !identity && <p className="warn">{t.needIdentity}</p>}
      </div>
    </section>
  );
}

export function App() {
  const [lang, setLang] = useStored("lang", "uk");
  const [home, setHome] = useStored("home", "");
  const [identity, setIdentity] = useStored("identity", "");
  const section = windowSection();
  const [tab, setTab] = useState<Tab>(section ?? "setup");
  const [message, setMessage] = useState<{ text: string; error: boolean } | null>(null);
  const [busy, setBusy] = useState(false);
  const t = PACKS[(lang in PACKS ? lang : "uk") as Lang];

  const note = (text: string, error = false) => setMessage({ text, error });

  const run = async <T,>(work: () => Promise<T>): Promise<T | undefined> => {
    setBusy(true);
    setMessage(null);
    try {
      return await work();
    } catch (error) {
      note(`${t.errorPrefix}: ${errorText(error)}`, true);
      return undefined;
    } finally {
      setBusy(false);
    }
  };

  const shared: Shared = { t, home, identity, note, busy, run };
  const tabs: [Tab, string][] = [
    ["setup", t.tabSetup],
    ["prepare", t.tabPrepare],
    ["result", t.tabResult],
    ["database", t.tabDatabase],
    ["report", t.tabReport],
  ];
  const tabTitle = (id: Tab) => tabs.find(([key]) => key === id)?.[1] ?? "";

  const detach = async () => {
    try {
      await openSectionWindow(tab, `ZaNytky - ${tabTitle(tab)}`);
    } catch (error) {
      note(`${t.windowOpenFailed}: ${errorText(error)}`, true);
    }
  };

  // A detached window asks before it closes with unsaved text in it.
  useEffect(() => {
    if (!section) return;
    let stop: (() => void) | undefined;
    getCurrentWindow()
      .onCloseRequested((event) => {
        if (hasUnsavedText() && !window.confirm(t.confirmCloseUnsaved)) event.preventDefault();
      })
      .then((unlisten) => {
        stop = unlisten;
      })
      .catch(() => undefined);
    return () => stop?.();
  }, [section, t]);

  return (
    <div className="app">
      <header>
        <h1>ZaNytky</h1>
        {section ? (
          <nav>
            <span className="tab active">{tabTitle(section)}</span>
          </nav>
        ) : (
          <nav>
            {tabs.map(([id, label]) => (
              <button key={id} className={tab === id ? "tab active" : "tab"} onClick={() => setTab(id)}>
                {label}
              </button>
            ))}
            <button className="tab" title={t.openWindow} aria-label={t.openWindow} onClick={detach}>
              &#10697;
            </button>
          </nav>
        )}
        <select aria-label={t.language} value={lang} onChange={(e) => setLang(e.target.value)}>
          {LANGS.map((l) => (
            <option key={l.code} value={l.code}>
              {l.label}
            </option>
          ))}
        </select>
      </header>
      {busy && <p className="muted">{t.working}</p>}
      {message && <p className={message.error ? "error" : "ok"}>{message.text}</p>}
      <main>
        {tab === "setup" && <SetupTab {...shared} setHome={setHome} setIdentity={setIdentity} />}
        {tab === "prepare" && <PrepareTab {...shared} />}
        {tab === "result" && <ResultTab {...shared} />}
        {tab === "database" && <DatabaseTab {...shared} />}
        {tab === "report" && <ReportTab {...shared} />}
      </main>
    </div>
  );
}

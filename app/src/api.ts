import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";

export interface HomeStatus {
  initialised: boolean;
  recipients: number;
}

export interface Candidate {
  text: string;
  count: number;
}

export interface PrepareOutput {
  deidentified: string;
  prompt: string;
  metaJson: string;
  replacements: number;
  candidates: Candidate[];
}

export interface ImportOut {
  sources: number;
  entitiesNew: number;
  entitiesExisting: number;
  statements: number;
}

export interface PrepareInput {
  home: string;
  text: string;
  id: string;
  title: string;
  kind: string;
  url?: string;
  date?: string;
  context?: string;
  identityPath?: string;
}

export interface ResolveInput {
  home: string;
  mention: string;
  action: "same" | "new" | "not_person";
  person?: string;
  name?: string;
  ipn?: string;
  phone?: string;
  email?: string;
}

export interface ReportInput {
  home: string;
  identityPath: string;
  title: string;
  format: "md" | "docx" | "pdf";
  passphrase?: string;
  plain: boolean;
  outPath: string;
}

export interface StatementRow {
  id: number;
  kind: string;
  parties: string[];
  value: string;
  grade: string;
  status: string;
  note: string;
}

export interface EntityListRow {
  id: number;
  kind: string;
  label: string;
  code: string | null;
  statements: number;
}

export interface CardParty {
  id: number;
  label: string;
  role: string;
}

export interface CardSource {
  title: string;
  kind: string;
  reliability: string;
  url: string | null;
}

export interface CardStatement {
  id: number;
  kind: string;
  role: string;
  value: string;
  validFrom: string | null;
  validTo: string | null;
  asOf: string | null;
  grade: string;
  status: string;
  note: string;
  independent: number;
  parties: CardParty[];
  sources: CardSource[];
}

export interface CardNeighbor {
  id: number;
  label: string;
  kind: string;
  count: number;
}

export interface CardOwner {
  depth: number;
  owner: string;
  ownerId: number;
  owned: string;
  ownedId: number;
  value: string;
  statement: number;
}

export interface EntityCardData {
  id: number;
  kind: string;
  label: string;
  code: string | null;
  statements: CardStatement[];
  neighbors: CardNeighbor[];
  owners: CardOwner[];
}

export const api = {
  entities: (home: string, identityPath?: string) =>
    invoke<EntityListRow[]>("list_entities", { input: { home, identityPath } }),
  entityCard: (home: string, id: number, identityPath?: string) =>
    invoke<EntityCardData>("get_entity_card", { input: { home, id, identityPath } }),
  statements: (home: string) => invoke<StatementRow[]>("list_statements", { input: { home } }),
  setStatus: (home: string, id: number, status: "confirmed" | "refuted" | "unverified") =>
    invoke<void>("set_statement_status", { input: { home, id, status } }),
  homeStatus: (home: string) => invoke<HomeStatus>("home_status", { input: { home } }),
  initHome: (home: string) => invoke<string>("init_home", { input: { home } }),
  newIdentity: (home: string, path: string) =>
    invoke<string>("new_identity", { input: { home, path } }),
  checkIdentity: (path: string) => invoke<void>("check_identity", { input: { path } }),
  readTextFile: (path: string) => invoke<string>("read_text_file", { input: { path } }),
  prepare: (input: PrepareInput) => invoke<PrepareOutput>("prepare_package", { input }),
  resolve: (input: ResolveInput) => invoke<string | null>("resolve_mention", { input }),
  check: (home: string, json: string, pkg: string) =>
    invoke<string>("check_result", { input: { home, json, package: pkg } }),
  importResult: (home: string, json: string, pkg: string) =>
    invoke<ImportOut>("import_result_cmd", { input: { home, json, package: pkg } }),
  report: (input: ReportInput) => invoke<string>("make_report", { input }),
  demo: (folder: string) =>
    invoke<{ home: string; identity: string; statements: number }>("make_demo", { input: { folder } }),
  backup: (home: string, outPath: string, passphrase: string) =>
    invoke<string>("make_backup", { input: { home, outPath, passphrase } }),
  restore: (filePath: string, dest: string, passphrase: string) =>
    invoke<number>("restore_backup", { input: { filePath, dest, passphrase } }),
};

export async function pickFolder(): Promise<string | null> {
  const chosen = await open({ directory: true, multiple: false });
  return typeof chosen === "string" ? chosen : null;
}

export async function pickFile(extensions: string[]): Promise<string | null> {
  const chosen = await open({ multiple: false, filters: [{ name: "files", extensions }] });
  return typeof chosen === "string" ? chosen : null;
}

export async function pickSavePath(defaultPath: string): Promise<string | null> {
  return save({ defaultPath });
}

export function errorText(error: unknown): string {
  return typeof error === "string" ? error : error instanceof Error ? error.message : String(error);
}

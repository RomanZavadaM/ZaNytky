import type { Lang } from "./i18n";

// Readable names of entity kinds, statement kinds and roles. Ukrainian and English;
// other interface languages use English for now (partial translation).
const uk: Record<string, string> = {
  person: "Особа",
  org: "Організація",
  group: "Група",
  asset: "Актив",
  procurement: "Закупівля",
  market: "Ринок",
  event: "Подія",
  document: "Документ",
  owns: "володіє",
  beneficiary: "бенефіціар",
  director: "директор",
  founder: "засновник",
  won: "виграв",
  customer: "замовник",
  supplies: "постачає",
  uses: "використовує",
  coordinated: "узгоджені дії",
  member_of_group: "член групи",
  operates_in_market: "працює на ринку",
  reported_link: "повідомлений зв'язок",
  owner: "власник",
  owned: "об'єкт власності",
  company: "компанія",
  supplier: "постачальник",
  member: "член",
  group_role: "група",
  procurement_role: "закупівля",
};

const en: Record<string, string> = {
  person: "Person",
  org: "Organisation",
  group: "Group",
  asset: "Asset",
  procurement: "Procurement",
  market: "Market",
  event: "Event",
  document: "Document",
  owns: "owns",
  beneficiary: "beneficiary",
  director: "director",
  founder: "founder",
  won: "won",
  customer: "customer",
  supplies: "supplies",
  uses: "uses",
  coordinated: "coordinated",
  member_of_group: "member of group",
  operates_in_market: "operates in market",
  reported_link: "reported link",
  owner: "owner",
  owned: "owned",
  company: "company",
  supplier: "supplier",
  member: "member",
  group_role: "group",
  procurement_role: "procurement",
};

export function kindLabel(lang: Lang, key: string): string {
  const table = lang === "uk" ? uk : en;
  return table[key] ?? key;
}

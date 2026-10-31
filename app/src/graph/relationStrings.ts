// Copied from MD Notes (app/src); ZaNytky modules stay independent of it.
import type { LanguageCode } from "./lang";

export interface RelationStrings {
  rumour: string;
  noGrade: string;
  evidence: string;
  status: Record<string, string>;
  origin: Record<string, string>;
}

export const RELATION_STRINGS: Record<LanguageCode, RelationStrings> = {
  uk: {
    rumour: "ЧУТКА",
    noGrade: "без оцінки",
    evidence: "Докази",
    status: { unverified: "? неперевірено", partly: "~ частково", verified: "✓ підтверджено", disputed: "× спростовано", unknown: "? невідомо" },
    origin: { registry: "реєстр", statement: "заява органу", media: "медіа", testimony: "свідчення", rumor: "чутка", own: "наш аналіз" },
  },
  en: {
    rumour: "RUMOUR",
    noGrade: "ungraded",
    evidence: "Evidence",
    status: { unverified: "? unverified", partly: "~ partly", verified: "✓ verified", disputed: "× disputed", unknown: "? unknown" },
    origin: { registry: "registry", statement: "official statement", media: "media", testimony: "testimony", rumor: "rumour", own: "own analysis" },
  },
  fr: {
    rumour: "RUMEUR",
    noGrade: "sans note",
    evidence: "Preuves",
    status: { unverified: "? non vérifié", partly: "~ en partie", verified: "✓ vérifié", disputed: "× contesté", unknown: "? inconnu" },
    origin: { registry: "registre", statement: "déclaration officielle", media: "médias", testimony: "témoignage", rumor: "rumeur", own: "analyse interne" },
  },
  de: {
    rumour: "GERÜCHT",
    noGrade: "ohne Bewertung",
    evidence: "Belege",
    status: { unverified: "? ungeprüft", partly: "~ teilweise", verified: "✓ bestätigt", disputed: "× bestritten", unknown: "? unbekannt" },
    origin: { registry: "Register", statement: "amtliche Erklärung", media: "Medien", testimony: "Zeugenaussage", rumor: "Gerücht", own: "eigene Analyse" },
  },
  es: {
    rumour: "RUMOR",
    noGrade: "sin valorar",
    evidence: "Pruebas",
    status: { unverified: "? sin verificar", partly: "~ en parte", verified: "✓ verificado", disputed: "× refutado", unknown: "? desconocido" },
    origin: { registry: "registro", statement: "declaración oficial", media: "medios", testimony: "testimonio", rumor: "rumor", own: "análisis propio" },
  },
  ko: {
    rumour: "소문",
    noGrade: "미평가",
    evidence: "근거",
    status: { unverified: "? 미확인", partly: "~ 부분 확인", verified: "✓ 확인됨", disputed: "× 반박됨", unknown: "? 알 수 없음" },
    origin: { registry: "등기부", statement: "공식 발표", media: "언론", testimony: "증언", rumor: "소문", own: "자체 분석" },
  },
  ja: {
    rumour: "噂",
    noGrade: "未評価",
    evidence: "根拠",
    status: { unverified: "? 未確認", partly: "~ 一部確認", verified: "✓ 確認済み", disputed: "× 否定", unknown: "? 不明" },
    origin: { registry: "登記", statement: "公式発表", media: "報道", testimony: "証言", rumor: "噂", own: "独自分析" },
  },
};

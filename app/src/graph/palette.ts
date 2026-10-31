// Copied from MD Notes (app/src); ZaNytky modules stay independent of it.
// Node colours by note type. Okabe-Ito palette (colour-blind safe); the same
// colours are the single source of truth in the project's graph style spec.
// Unknown types get a stable colour derived from the type name, so a vault can
// use its own `type:` values without configuration.

const KNOWN: Record<string, string> = {
  person: "#E69F00",
  company: "#0072B2",
  institution: "#8E7CC3",
  group: "#CC79A7",
  asset: "#009E73",
  tender: "#56B4E9",
  org: "#0072B2",
  procurement: "#56B4E9",
  document: "#6B7280",
  market: "#F0E442",
  case: "#D55E00",
  event: "#999999",
  source: "#6B7280",
  material: "#A8AEB8",
  note: "#7C8DF0",
};

const FALLBACK = ["#E69F00", "#56B4E9", "#009E73", "#F0E442", "#0072B2", "#D55E00", "#CC79A7", "#999999"];
const UNTYPED = "#9AA1AD";

function hash(text: string): number {
  let h = 2166136261;
  for (let i = 0; i < text.length; i += 1) {
    h ^= text.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return h >>> 0;
}

/** Colour for a note type; `null` or empty means a note without a type. */
export function typeColor(type: string | null | undefined): string {
  if (!type) return UNTYPED;
  const known = KNOWN[type];
  if (known) return known;
  return FALLBACK[hash(type) % FALLBACK.length];
}

/** Node size from the number of connections (spec: 8 + 1.3 * min(n, 8)). */
export function nodeSize(degree: number): number {
  return 6 + 1.3 * Math.min(degree, 8) + (degree > 8 ? Math.min(Math.log2(degree - 7), 3) : 0);
}

/** True when the page background is dark (theme "dark" or system dark). */
export function isDarkBackground(): boolean {
  const colour = getComputedStyle(document.body).backgroundColor;
  const match = colour.match(/\d+(\.\d+)?/g);
  if (!match || match.length < 3) return false;
  const [r, g, b] = match.slice(0, 3).map(Number);
  return 0.2126 * r + 0.7152 * g + 0.0722 * b < 128;
}

/** Credibility digit (1-6) of a grade such as "B2"; `undefined` when absent. */
export function gradeDigit(grade: string | null | undefined): number | undefined {
  const match = grade?.match(/([1-6])\s*$/);
  return match ? Number(match[1]) : undefined;
}

export interface EdgeStyle {
  color: string;
  width: number;
  dash: number[];
}

/** Line style by grade: solid green (1-2), dashed amber (3-4), dotted red (5-6). */
export function edgeStyle(grade: string | undefined, dark: boolean): EdgeStyle {
  const digit = gradeDigit(grade);
  if (digit === undefined) return { color: dark ? "#5B6475" : "#A3ABB9", width: 1, dash: [] };
  if (digit <= 2) return { color: dark ? "#66BB6A" : "#2E7D32", width: 2.2, dash: [] };
  if (digit <= 4) return { color: dark ? "#E0A84A" : "#B26A00", width: 1.6, dash: [7, 4] };
  return { color: dark ? "#FF7A70" : "#C62828", width: 1.6, dash: [2, 4] };
}

export interface EvidenceStyle {
  dash: number[];
  rumour: boolean;
}

/** Node outline by the best evidence digit among its relations. */
export function evidenceStyle(best: number | undefined): EvidenceStyle {
  if (best === undefined || best <= 2) return { dash: [], rumour: false };
  if (best <= 4) return { dash: [6, 4], rumour: false };
  return { dash: [1.5, 3], rumour: true };
}

/** Grade chip colours (spec: ok 1-2, mid 3-4, low 5-6, none). */
export function chipColors(grade: string | undefined): { background: string; color: string } {
  const digit = gradeDigit(grade);
  if (digit === undefined) return { background: "#4B5563", color: "#FFFFFF" };
  if (digit <= 2) return { background: "#2E7D32", color: "#FFFFFF" };
  if (digit <= 4) return { background: "#8F5A00", color: "#FFFFFF" };
  return { background: "#B3261E", color: "#FFFFFF" };
}

/** Type glyphs as SVG paths in a 24x24 box centred on the origin; one string feeds Canvas `Path2D` and inline SVG. */
export const GLYPHS: Record<string, string> = {
  person: "M0 -9 a4.2 4.2 0 1 0 0.01 0Z M-8 10 a8 7 0 0 1 16 0 Z",
  company: "M-8 -8 H8 V8 H-8 Z",
  institution: "M0 -9 L8 -4.5 V4.5 L0 9 L-8 4.5 V-4.5 Z",
  case: "M0 -10 L9 0 L0 10 L-9 0 Z",
  material: "M0 -9 L9 8 H-9 Z",
  group: "M-6 -3 a3 3 0 1 0 0.01 0Z M6 -3 a3 3 0 1 0 0.01 0Z M0 6 a3 3 0 1 0 0.01 0Z",
};

const GLYPH_OF: Record<string, string> = {
  person: "person",
  company: "company",
  tender: "company",
  org: "company",
  procurement: "company",
  document: "material",
  institution: "institution",
  market: "institution",
  case: "case",
  asset: "case",
  material: "material",
  source: "material",
  group: "group",
};

/** SVG path of the glyph for a note type, or `null` for types without one. */
export function glyphFor(type: string | null | undefined): string | null {
  const name = type ? GLYPH_OF[type] : undefined;
  return name ? GLYPHS[name] : null;
}

export type NodeState = "strong" | "medium" | "weak" | "none";

/** Reliability state of a node from the best credibility digit among its typed relations. */
export function stateOf(best: number | undefined): NodeState {
  if (best === undefined) return "none";
  if (best <= 2) return "strong";
  if (best <= 4) return "medium";
  return "weak";
}

/** Node fill by state (the same hues as the grade lines; never the only signal: the outline and the rumour badge stay). */
export function stateColor(state: NodeState, dark: boolean): string {
  switch (state) {
    case "strong":
      return dark ? "#66BB6A" : "#2E7D32";
    case "medium":
      return dark ? "#E0A84A" : "#B26A00";
    case "weak":
      return dark ? "#FF7A70" : "#C62828";
    default:
      return dark ? "#9AA1AD" : "#5B6475";
  }
}

/** Glyph colour with enough contrast on the state fill. */
export function glyphColor(dark: boolean): string {
  return dark ? "#111827" : "#FFFFFF";
}

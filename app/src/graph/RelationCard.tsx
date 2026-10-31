// Copied from MD Notes (app/src); ZaNytky modules stay independent of it.
import { useI18n } from "./lang";
import { RELATION_STRINGS } from "./relationStrings";
import { chipColors, gradeDigit } from "./palette";

export interface RelationItem {
  kind: string;
  grade?: string;
  origin?: string;
  status?: string;
  evidence?: string;
  from: string;
  to: string;
}

export interface RelationCardData {
  x: number;
  y: number;
  from: string;
  to: string;
  items: RelationItem[];
}

function Chip({ grade, fallback }: { grade?: string; fallback: string }) {
  const colors = chipColors(grade);
  return (
    <span className="relation-chip" style={{ background: colors.background, color: colors.color }}>
      {grade || fallback}
    </span>
  );
}

/** Pop-up for a line of the graph: who relates to whom, with grade chips and evidence. */
export function RelationCard({ card }: { card: RelationCardData }) {
  const { language } = useI18n();
  const strings = RELATION_STRINGS[language];
  const digits = card.items.map((item) => gradeDigit(item.grade)).filter((d): d is number => d !== undefined);
  const rumourOnly = card.items.length > 0 && digits.length > 0 && Math.min(...digits) >= 5;
  return (
    <div
      className={`relation-card${rumourOnly ? " rumour" : ""}`}
      style={{ left: card.x + 14, top: card.y + 14 }}
      role="tooltip"
    >
      {rumourOnly && <div className="relation-rumour">{strings.rumour}</div>}
      <div className="relation-ends">
        <strong>{card.from}</strong>
        <span aria-hidden="true">→</span>
        <strong>{card.to}</strong>
      </div>
      <ul>
        {card.items.map((item, index) => (
          <li key={`${item.kind}-${index}`}>
            <div className="relation-line">
              <Chip grade={item.grade} fallback={strings.noGrade} />
              <span>{item.kind || "—"}</span>
              <small>{strings.status[item.status ?? "unknown"] ?? item.status}</small>
            </div>
            {(item.from !== card.from || item.to !== card.to) && (
              <small className="relation-dir">
                {item.from} → {item.to}
              </small>
            )}
            <small className="relation-meta">
              {item.origin ? (strings.origin[item.origin] ?? item.origin) : ""}
              {item.evidence ? ` · ${strings.evidence}: ${item.evidence}` : ""}
            </small>
          </li>
        ))}
      </ul>
    </div>
  );
}

// Copied from MD Notes (app/src); ZaNytky modules stay independent of it.
import type Graph from "graphology";
import type Sigma from "sigma";
import { edgeStyle, evidenceStyle, glyphColor } from "./palette";

/** Parts of the panel state the overlay needs to draw. */
export interface OverlayView {
  hovered: string | null;
  neighbours: Set<string>;
  search: string;
  hidden: Set<string>;
  /** Provenance trace: nodes and edge keys kept bright; `null` when off. */
  trace: { nodes: Set<string>; edges: Set<string> } | null;
  /** Draw a glyph for the note type inside every node (node colour then shows reliability). */
  shapes: boolean;
  /** Show only lines whose best credibility digit is at most this (1-6); `null` shows every line. */
  maxDigit: number | null;
}

/** A line under the pointer (graph edge key and pointer position in the container). */
export interface EdgeHit {
  edge: string;
  x: number;
  y: number;
}

interface Segment {
  edge: string;
  x1: number;
  y1: number;
  x2: number;
  y2: number;
}

function fitCanvas(canvas: HTMLCanvasElement, width: number, height: number): CanvasRenderingContext2D | null {
  const ratio = window.devicePixelRatio || 1;
  const w = Math.max(1, Math.round(width * ratio));
  const h = Math.max(1, Math.round(height * ratio));
  if (canvas.width !== w || canvas.height !== h) {
    canvas.width = w;
    canvas.height = h;
  }
  const context = canvas.getContext("2d");
  if (!context) return null;
  context.setTransform(ratio, 0, 0, ratio, 0, 0);
  context.clearRect(0, 0, width, height);
  return context;
}

function layer(): HTMLCanvasElement {
  const canvas = document.createElement("canvas");
  canvas.style.position = "absolute";
  canvas.style.inset = "0";
  canvas.style.width = "100%";
  canvas.style.height = "100%";
  canvas.style.pointerEvents = "none";
  return canvas;
}

function distanceToSegment(px: number, py: number, s: Segment): number {
  const dx = s.x2 - s.x1;
  const dy = s.y2 - s.y1;
  const lengthSquared = dx * dx + dy * dy;
  let t = lengthSquared === 0 ? 0 : ((px - s.x1) * dx + (py - s.y1) * dy) / lengthSquared;
  t = Math.max(0, Math.min(1, t));
  return Math.hypot(px - (s.x1 + t * dx), py - (s.y1 + t * dy));
}

/**
 * Draws what WebGL lines and fills cannot: dashed and dotted lines by grade
 * with direction arrowheads under the nodes, and outlines by evidence (plus a
 * rumour badge) above them. Sigma's own edges are hidden by the panel's edge
 * reducer. Also finds the line under the pointer so the panel can show a card.
 */
export function attachOverlay(
  container: HTMLElement,
  renderer: Sigma,
  graph: Graph,
  view: OverlayView,
  dark: boolean,
  onEdgeHover: (hit: EdgeHit | null) => void,
): () => void {
  const below = layer();
  const above = layer();
  container.insertBefore(below, container.firstChild);
  container.appendChild(above);
  const outline = dark ? "#E5E7EB" : "#1F2937";
  let segments: Segment[] = [];

  const draw = () => {
    const width = container.clientWidth;
    const height = container.clientHeight;
    const lines = fitCanvas(below, width, height);
    const rings = fitCanvas(above, width, height);
    if (!lines || !rings) return;
    const searching = view.search.length > 0;
    const focus = view.hovered;
    segments = [];

    graph.forEachEdge((edge, attributes, source, target) => {
      const sourceData = graph.getNodeAttributes(source);
      const targetData = graph.getNodeAttributes(target);
      if (view.hidden.has(String(sourceData.typeKey)) || view.hidden.has(String(targetData.typeKey))) return;
      if (view.maxDigit !== null) {
        const digit = attributes.digit as number | undefined;
        if (digit === undefined || digit > view.maxDigit) return;
      }
      const style = edgeStyle(attributes.grade as string | undefined, dark);
      const from = renderer.graphToViewport({ x: sourceData.x, y: sourceData.y });
      const to = renderer.graphToViewport({ x: targetData.x, y: targetData.y });
      if (attributes.hasItems === true) {
        segments.push({ edge, x1: from.x, y1: from.y, x2: to.x, y2: to.y });
      }
      const incident = focus !== null && (source === focus || target === focus);
      lines.save();
      const traced = view.trace !== null && view.trace.edges.has(edge);
      lines.globalAlpha =
        view.trace !== null
          ? traced
            ? 1
            : 0.05
          : focus === null
            ? searching
              ? 0.4
              : 0.9
            : incident
              ? 1
              : 0.07;
      lines.strokeStyle = style.color;
      lines.fillStyle = style.color;
      lines.lineWidth = style.width + (incident || traced ? 1.2 : 0);
      lines.setLineDash(style.dash);
      lines.beginPath();
      lines.moveTo(from.x, from.y);
      lines.lineTo(to.x, to.y);
      lines.stroke();

      // Arrowhead at the target end, just outside the node.
      if (attributes.arrow === true) {
        const length = Math.hypot(to.x - from.x, to.y - from.y);
        if (length > 1) {
          const ux = (to.x - from.x) / length;
          const uy = (to.y - from.y) / length;
          const gap = renderer.scaleSize(Number(targetData.size)) + 3;
          const tipX = to.x - ux * gap;
          const tipY = to.y - uy * gap;
          const head = 6 + style.width * 1.5;
          lines.setLineDash([]);
          lines.beginPath();
          lines.moveTo(tipX, tipY);
          lines.lineTo(tipX - ux * head - uy * head * 0.45, tipY - uy * head + ux * head * 0.45);
          lines.lineTo(tipX - ux * head + uy * head * 0.45, tipY - uy * head - ux * head * 0.45);
          lines.closePath();
          lines.fill();
        }
      }
      lines.restore();
    });

    graph.forEachNode((node, attributes) => {
      if (view.hidden.has(String(attributes.typeKey))) return;
      const point = renderer.graphToViewport({ x: attributes.x, y: attributes.y });
      const radius = renderer.scaleSize(Number(attributes.size)) + 1;
      const matches = searching && String(attributes.fullTitle).toLowerCase().includes(view.search);
      const related = focus !== null && (node === focus || view.neighbours.has(node));
      const dimmed =
        (focus !== null && !related) || (searching && !matches) || (view.trace !== null && !view.trace.nodes.has(node));
      const evidence = evidenceStyle(attributes.evidence as number | undefined);
      rings.save();
      rings.globalAlpha = dimmed ? 0.25 : 1;
      rings.strokeStyle = outline;
      rings.lineWidth = 1.5;
      rings.setLineDash(evidence.dash);
      rings.beginPath();
      rings.arc(point.x, point.y, radius, 0, Math.PI * 2);
      rings.stroke();
      if (evidence.rumour) {
        // Badge «Ч»: rumour, never shown by colour alone.
        const badge = Math.max(6, Math.min(9, radius * 0.6));
        const bx = point.x + radius * 0.8;
        const by = point.y - radius * 0.8;
        rings.setLineDash([]);
        rings.fillStyle = dark ? "#FF7A70" : "#B3261E";
        rings.beginPath();
        rings.arc(bx, by, badge, 0, Math.PI * 2);
        rings.fill();
        rings.fillStyle = "#FFFFFF";
        rings.font = `bold ${Math.round(badge * 1.4)}px sans-serif`;
        rings.textAlign = "center";
        rings.textBaseline = "middle";
        rings.fillText("Ч", bx, by + 0.5);
      }
      if (view.shapes && typeof attributes.glyph === "string") {
        rings.save();
        rings.globalAlpha = dimmed ? 0.25 : 1;
        rings.translate(point.x, point.y);
        const scale = (radius * 0.62) / 10;
        rings.scale(scale, scale);
        rings.fillStyle = glyphColor(dark);
        rings.fill(new Path2D(attributes.glyph));
        rings.restore();
      }
      rings.restore();
    });
  };

  // Find the typed line under the pointer (when no node is hovered).
  let pending = false;
  let lastEvent: MouseEvent | null = null;
  const onMove = (event: MouseEvent) => {
    lastEvent = event;
    if (pending) return;
    pending = true;
    requestAnimationFrame(() => {
      pending = false;
      const current = lastEvent;
      if (!current) return;
      if (view.hovered !== null) {
        onEdgeHover(null);
        return;
      }
      const rect = container.getBoundingClientRect();
      const x = current.clientX - rect.left;
      const y = current.clientY - rect.top;
      let best: Segment | null = null;
      let bestDistance = 7;
      for (const segment of segments) {
        const d = distanceToSegment(x, y, segment);
        if (d < bestDistance) {
          bestDistance = d;
          best = segment;
        }
      }
      onEdgeHover(best ? { edge: best.edge, x, y } : null);
    });
  };
  const onLeave = () => {
    lastEvent = null;
    onEdgeHover(null);
  };
  container.addEventListener("mousemove", onMove);
  container.addEventListener("mouseleave", onLeave);

  renderer.on("afterRender", draw);
  draw();
  return () => {
    container.removeEventListener("mousemove", onMove);
    container.removeEventListener("mouseleave", onLeave);
    renderer.off("afterRender", draw);
    below.remove();
    above.remove();
  };
}

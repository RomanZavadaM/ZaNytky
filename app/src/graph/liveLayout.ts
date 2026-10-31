// Copied from MD Notes (app/src); ZaNytky modules stay independent of it.
// Live force simulation for the knowledge graph ("floating" like Obsidian).
//
// Nodes have velocity. Forces: repulsion between nodes (Barnes-Hut quadtree for
// larger graphs), springs along links, a weak pull to the centre. Energy
// ("alpha") decays towards a target: the graph settles by itself; dragging a node
// raises the target so its neighbours follow and flow around it, and releasing it
// lets everything cool down again. With a small non-zero target the graph keeps a
// slow, gentle drift. Pure computation, no DOM: the panel decides when to tick.

export type Edge = readonly [number, number];

export interface Forces {
  /** Pull towards the centre (0 - 0.3). */
  center: number;
  /** Repulsion between nodes (0 - 3). */
  repel: number;
  /** Spring strength of links (0 - 1). */
  link: number;
  /** Natural link length (0.3 - 3). */
  distance: number;
}

export const DEFAULT_FORCES: Forces = { center: 0.04, repel: 1.4, link: 0.3, distance: 1.3 };

const ALPHA_MIN = 0.003;
const ALPHA_DECAY = 0.0228;
const VELOCITY_DECAY = 0.6;
const MAX_FORCE = 4;
const MAX_SPEED = 2;
const EXACT_LIMIT = 220;
const THETA2 = 0.81;
const MAX_DEPTH = 28;

export class LiveLayout {
  readonly x: Float64Array;
  readonly y: Float64Array;
  private readonly vx: Float64Array;
  private readonly vy: Float64Array;
  private readonly fx: Float64Array;
  private readonly fy: Float64Array;
  private readonly pinned: Uint8Array;
  private alpha = 1;
  private alphaTarget = 0;
  private drift = 0;
  forces: Forces;

  // Quadtree storage, reused between ticks.
  private cap = 0;
  private cMass: Float64Array = new Float64Array(0);
  private cX: Float64Array = new Float64Array(0);
  private cY: Float64Array = new Float64Array(0);
  private cHalf: Float64Array = new Float64Array(0);
  private cCx: Float64Array = new Float64Array(0);
  private cCy: Float64Array = new Float64Array(0);
  private cBody: Int32Array = new Int32Array(0);
  private cKids: Int32Array = new Int32Array(0);
  private cells = 0;

  constructor(
    private readonly count: number,
    private readonly edges: readonly Edge[],
    forces: Forces = DEFAULT_FORCES,
  ) {
    this.forces = { ...forces };
    this.x = new Float64Array(count);
    this.y = new Float64Array(count);
    this.vx = new Float64Array(count);
    this.vy = new Float64Array(count);
    this.fx = new Float64Array(count);
    this.fy = new Float64Array(count);
    this.pinned = new Uint8Array(count);
    // Golden-angle spiral: a deterministic, evenly spread start.
    for (let i = 0; i < count; i += 1) {
      const radius = Math.sqrt(i + 0.5);
      const angle = i * 2.399963229728653;
      this.x[i] = Math.cos(angle) * radius;
      this.y[i] = Math.sin(angle) * radius;
    }
  }

  /** True when the graph has cooled down and nothing keeps it moving. */
  get settled(): boolean {
    return this.alpha < ALPHA_MIN && this.alphaTarget <= 0 && this.drift <= 0;
  }

  /** Current energy (1 right after start, falls to 0 when settled). */
  get energy(): number {
    return this.alpha;
  }

  /** Sets the starting position of a node without pinning it (to keep a layout across refreshes). */
  place(index: number, x: number, y: number): void {
    this.x[index] = x;
    this.y[index] = y;
  }

  /** Sets the current energy (lower it when most nodes already have good positions). */
  setEnergy(alpha: number): void {
    this.alpha = Math.max(0, Math.min(1, alpha));
  }

  /** Energy the simulation is cooling towards (0 = settle, 0.02 = gentle drift, 0.3 = while dragging). */
  setTarget(target: number): void {
    this.alphaTarget = Math.max(0, target);
  }

  /** Random push per tick that keeps the graph slowly floating (0 = off). */
  setDrift(amount: number): void {
    this.drift = Math.max(0, amount);
  }

  /** Heats the simulation up again (after changing forces or dragging). */
  reheat(alpha = 0.6): void {
    this.alpha = Math.max(this.alpha, alpha);
  }

  setForces(forces: Forces): void {
    this.forces = { ...forces };
    this.reheat(0.5);
  }

  pin(index: number, x: number, y: number): void {
    this.pinned[index] = 1;
    this.x[index] = x;
    this.y[index] = y;
    this.vx[index] = 0;
    this.vy[index] = 0;
  }

  unpin(index: number): void {
    this.pinned[index] = 0;
  }

  /** Runs one step. */
  tick(): void {
    const n = this.count;
    if (n < 1) return;
    const { x, y, vx, vy, fx, fy, pinned } = this;
    const { repel, link, distance, center } = this.forces;
    fx.fill(0);
    fy.fill(0);

    if (n <= EXACT_LIMIT) {
      for (let i = 0; i < n; i += 1) {
        for (let j = i + 1; j < n; j += 1) {
          let ex = x[i] - x[j];
          let ey = y[i] - y[j];
          let d2 = ex * ex + ey * ey;
          if (d2 < 1e-6) {
            ex = (i - j) * 1e-3;
            ey = 1e-3;
            d2 = ex * ex + ey * ey;
          }
          const f = repel / d2;
          fx[i] += ex * f;
          fy[i] += ey * f;
          fx[j] -= ex * f;
          fy[j] -= ey * f;
        }
      }
    } else {
      this.buildTree();
      for (let i = 0; i < n; i += 1) this.repelFromTree(i, repel);
    }

    for (const [a, b] of this.edges) {
      const ex = x[b] - x[a];
      const ey = y[b] - y[a];
      const d = Math.sqrt(ex * ex + ey * ey) || 1e-3;
      const f = ((d - distance) * link) / d;
      fx[a] += ex * f;
      fy[a] += ey * f;
      fx[b] -= ex * f;
      fy[b] -= ey * f;
    }

    for (let i = 0; i < n; i += 1) {
      fx[i] -= x[i] * center;
      fy[i] -= y[i] * center;
    }

    const alpha = this.alpha;
    for (let i = 0; i < n; i += 1) {
      if (pinned[i] === 1) {
        vx[i] = 0;
        vy[i] = 0;
        continue;
      }
      let gx = fx[i];
      let gy = fy[i];
      const force = Math.sqrt(gx * gx + gy * gy);
      if (force > MAX_FORCE) {
        gx = (gx / force) * MAX_FORCE;
        gy = (gy / force) * MAX_FORCE;
      }
      const noiseX = this.drift > 0 ? (Math.random() - 0.5) * this.drift : 0;
      const noiseY = this.drift > 0 ? (Math.random() - 0.5) * this.drift : 0;
      vx[i] = (vx[i] + gx * alpha + noiseX) * (1 - VELOCITY_DECAY);
      vy[i] = (vy[i] + gy * alpha + noiseY) * (1 - VELOCITY_DECAY);
      const speed = Math.sqrt(vx[i] * vx[i] + vy[i] * vy[i]);
      if (speed > MAX_SPEED) {
        vx[i] = (vx[i] / speed) * MAX_SPEED;
        vy[i] = (vy[i] / speed) * MAX_SPEED;
      }
      x[i] += vx[i];
      y[i] += vy[i];
    }
    this.alpha += (this.alphaTarget - this.alpha) * ALPHA_DECAY;
  }

  // ---- Barnes-Hut quadtree -------------------------------------------------

  private ensure(capacity: number): void {
    if (capacity <= this.cap) return;
    this.cap = capacity;
    this.cMass = new Float64Array(capacity);
    this.cX = new Float64Array(capacity);
    this.cY = new Float64Array(capacity);
    this.cHalf = new Float64Array(capacity);
    this.cCx = new Float64Array(capacity);
    this.cCy = new Float64Array(capacity);
    this.cBody = new Int32Array(capacity);
    this.cKids = new Int32Array(capacity * 4);
  }

  private newCell(cx: number, cy: number, half: number): number {
    const index = this.cells;
    this.cells += 1;
    this.cMass[index] = 0;
    this.cX[index] = 0;
    this.cY[index] = 0;
    this.cCx[index] = cx;
    this.cCy[index] = cy;
    this.cHalf[index] = half;
    this.cBody[index] = -1;
    this.cKids.fill(-1, index * 4, index * 4 + 4);
    return index;
  }

  private buildTree(): void {
    const n = this.count;
    const { x, y } = this;
    let minX = Infinity;
    let minY = Infinity;
    let maxX = -Infinity;
    let maxY = -Infinity;
    for (let i = 0; i < n; i += 1) {
      if (x[i] < minX) minX = x[i];
      if (x[i] > maxX) maxX = x[i];
      if (y[i] < minY) minY = y[i];
      if (y[i] > maxY) maxY = y[i];
    }
    this.ensure(n * 6 + 16);
    this.cells = 0;
    const half = Math.max(maxX - minX, maxY - minY, 1e-6) / 2 + 1e-6;
    this.newCell((minX + maxX) / 2, (minY + maxY) / 2, half);
    for (let i = 0; i < n; i += 1) this.insert(i);
  }

  private childOf(cell: number, px: number, py: number): number {
    const quadrant = (px >= this.cCx[cell] ? 1 : 0) + (py >= this.cCy[cell] ? 2 : 0);
    let child = this.cKids[cell * 4 + quadrant];
    if (child < 0) {
      const h = this.cHalf[cell] / 2;
      const cx = this.cCx[cell] + (quadrant & 1 ? h : -h);
      const cy = this.cCy[cell] + (quadrant & 2 ? h : -h);
      child = this.newCell(cx, cy, h);
      this.cKids[cell * 4 + quadrant] = child;
    }
    return child;
  }

  private insert(body: number): void {
    const px = this.x[body];
    const py = this.y[body];
    let cell = 0;
    for (let depth = 0; depth < MAX_DEPTH; depth += 1) {
      this.cMass[cell] += 1;
      this.cX[cell] += px;
      this.cY[cell] += py;
      const occupant = this.cBody[cell];
      const hasKids = this.cKids[cell * 4] >= 0 || this.cKids[cell * 4 + 1] >= 0 || this.cKids[cell * 4 + 2] >= 0 || this.cKids[cell * 4 + 3] >= 0;
      if (this.cMass[cell] === 1) {
        this.cBody[cell] = body;
        return;
      }
      if (!hasKids && occupant >= 0) {
        // The cell held one body: push it one level down, then continue with the new one.
        this.cBody[cell] = -1;
        const moved = this.childOf(cell, this.x[occupant], this.y[occupant]);
        this.cMass[moved] += 1;
        this.cX[moved] += this.x[occupant];
        this.cY[moved] += this.y[occupant];
        this.cBody[moved] = occupant;
      }
      cell = this.childOf(cell, px, py);
    }
    // Depth limit (coincident points): the last cell just accumulates mass.
    this.cMass[cell] += 1;
    this.cX[cell] += px;
    this.cY[cell] += py;
  }

  private repelFromTree(i: number, repel: number): void {
    const px = this.x[i];
    const py = this.y[i];
    let fxi = 0;
    let fyi = 0;
    const stack: number[] = [0];
    while (stack.length > 0) {
      const cell = stack.pop() as number;
      const mass = this.cMass[cell];
      if (mass === 0) continue;
      if (this.cBody[cell] === i && mass === 1) continue;
      const ex = px - this.cX[cell] / mass;
      const ey = py - this.cY[cell] / mass;
      const d2 = ex * ex + ey * ey;
      const width = this.cHalf[cell] * 2;
      const leaf = this.cBody[cell] >= 0;
      if (leaf || width * width < THETA2 * d2) {
        const safe = d2 < 1e-6 ? 1e-6 : d2;
        const f = (repel * mass) / safe;
        fxi += (d2 < 1e-6 ? 1e-3 : ex) * f;
        fyi += (d2 < 1e-6 ? 1e-3 : ey) * f;
      } else {
        for (let q = 0; q < 4; q += 1) {
          const child = this.cKids[cell * 4 + q];
          if (child >= 0) stack.push(child);
        }
      }
    }
    this.fx[i] += fxi;
    this.fy[i] += fyi;
  }
}

/**
 * Editör yerleşimi: panelin tarafı, genişliği, katlanmışlığı ve görünüm modu.
 *
 * Tarayıcıda saklanır. Öğretmen yerleşimi bir kez kurar; her soruda yeniden
 * kurmak zorunda kalmaz. Saklama başarısız olursa (özel pencere, site verisi
 * kapalı) uygulama varsayılanla çalışmaya devam eder — hatırlamamak,
 * çalışmamaktan iyidir.
 *
 * Önizleme zoom'u BURADA DEĞİL: `SheetPreview` onu kendi anahtarında
 * (`tayan.preview.zoom`) tutar ve o mantık çalışıyor. İkisini birleştirmek
 * çalışan bir şeyi bozma riskidir.
 */

export type PanelSide = "left" | "right";

/** Hangi bölmeler görünür. Kâğıt ve kaynak, tek başına da yan yana da olabilir. */
export type ViewMode = "editor" | "split" | "preview";

export type Layout = {
  side: PanelSide;
  width: number;
  collapsed: boolean;
  mode: ViewMode;
};

const KEY = "tayan.layout.v1";

export const MIN_PANEL_WIDTH = 200;
export const MAX_PANEL_WIDTH = 480;

const DEFAULTS: Layout = {
  side: "left",
  width: 260,
  collapsed: false,
  mode: "split",
};

const SIDES: PanelSide[] = ["left", "right"];
const MODES: ViewMode[] = ["editor", "split", "preview"];

function clampWidth(value: unknown): number {
  const n = Number(value);
  if (!Number.isFinite(n)) return DEFAULTS.width;
  return Math.min(Math.max(n, MIN_PANEL_WIDTH), MAX_PANEL_WIDTH);
}

/**
 * Saklanan değer dış veridir: sürüm atlamış, elle kurcalanmış veya bozulmuş
 * olabilir. Her alan tek tek doğrulanır; tanınmayan değer varsayılana düşer.
 * Tek bir bozuk alan yüzünden yerleşimin tamamını atmak gereksiz olurdu.
 */
function parse(raw: string | null): Layout {
  if (raw === null) return { ...DEFAULTS };

  let data: unknown;
  try {
    data = JSON.parse(raw);
  } catch {
    return { ...DEFAULTS };
  }

  if (data === null || typeof data !== "object") return { ...DEFAULTS };
  const source = data as Record<string, unknown>;

  return {
    side: SIDES.includes(source.side as PanelSide)
      ? (source.side as PanelSide)
      : DEFAULTS.side,
    width: clampWidth(source.width),
    collapsed: typeof source.collapsed === "boolean" ? source.collapsed : DEFAULTS.collapsed,
    mode: MODES.includes(source.mode as ViewMode) ? (source.mode as ViewMode) : DEFAULTS.mode,
  };
}

function load(): Layout {
  // Modül sunucu tarafında da yüklenir (adapter-static ön-derleme yapar);
  // orada localStorage yoktur.
  if (typeof localStorage === "undefined") return { ...DEFAULTS };
  try {
    return parse(localStorage.getItem(KEY));
  } catch {
    return { ...DEFAULTS };
  }
}

export const layout = $state<Layout>(load());

function persist() {
  if (typeof localStorage === "undefined") return;
  try {
    localStorage.setItem(KEY, JSON.stringify({ ...layout }));
  } catch {
    // Hatırlamamak, çalışmamaktan iyidir.
  }
}

export function setSide(side: PanelSide) {
  layout.side = side;
  persist();
}

/** Paneli karşı tarafa taşır. Katlıysa açar — taşınan görünmez panel şaşırtır. */
export function flipSide() {
  layout.side = layout.side === "left" ? "right" : "left";
  layout.collapsed = false;
  persist();
}

export function setWidth(width: number) {
  layout.width = clampWidth(width);
  persist();
}

export function toggleCollapsed() {
  layout.collapsed = !layout.collapsed;
  persist();
}

export function setMode(mode: ViewMode) {
  layout.mode = mode;
  persist();
}

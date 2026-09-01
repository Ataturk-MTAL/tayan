import { EditorView, keymap, lineNumbers, highlightActiveLine, highlightActiveLineGutter, gutter, GutterMarker } from "@codemirror/view";
import { EditorState, StateEffect, StateField, RangeSet, type Extension } from "@codemirror/state";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import { bracketMatching, indentOnInput } from "@codemirror/language";
import { closeBrackets, closeBracketsKeymap, autocompletion, type CompletionContext } from "@codemirror/autocomplete";
import { Decoration, type DecorationSet } from "@codemirror/view";

import { typstSyntax } from "./typst-lang";
import type { TypstDiagnostic } from "./diagnostics";

/**
 * Editörün görünümü, dünyanın kuralına uyar: içerik mürekkep, hata kırmızı.
 * Sözdizimi renklendirmesinde kırmızı yoktur (bkz. typst-lang.ts), bu yüzden
 * editörde görülen her kırmızı tek bir şey demektir: burada bir hata var.
 */
const editorTheme = EditorView.theme({
  "&": {
    color: "#16233f",
    backgroundColor: "transparent",
    height: "100%",
    fontSize: "13.5px",
  },
  ".cm-scroller": {
    fontFamily: '"JetBrains Mono", ui-monospace, monospace',
    lineHeight: "20px",
    overflow: "auto",
  },
  ".cm-content": { padding: "20px 0", caretColor: "#c8102e" },
  ".cm-line": { padding: "0 20px" },

  "&.cm-focused": { outline: "none" },
  ".cm-cursor, .cm-dropCursor": { borderLeft: "2px solid #c8102e" },
  "&.cm-focused .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection": {
    backgroundColor: "#f7e4e7",
  },
  ".cm-activeLine": { backgroundColor: "rgba(22, 35, 63, 0.035)" },

  ".cm-gutters": {
    backgroundColor: "transparent",
    color: "#6e716b",
    border: "none",
    borderRight: "1px solid #c3cec9",
    minWidth: "40px",
  },
  ".cm-activeLineGutter": { backgroundColor: "transparent", color: "#16233f" },
  ".cm-lineNumbers .cm-gutterElement": {
    padding: "0 10px 0 5px",
    fontVariantNumeric: "tabular-nums",
  },

  ".cm-matchingBracket, &.cm-focused .cm-matchingBracket": {
    backgroundColor: "transparent",
    outline: "1px solid #a4b3ad",
  },

  // Kırmızı kalem: hatalı satırın altı çizilir.
  ".tayan-error-line": {
    textDecoration: "underline wavy #c8102e",
    textDecorationSkipInk: "none",
    textUnderlineOffset: "3px",
  },
}, { dark: false });

// ── Hata işaretleri ───────────────────────────────────────────────────────────

export const setDiagnostics = StateEffect.define<TypstDiagnostic[]>();

const errorLineDecoration = Decoration.line({ class: "tayan-error-line" });

const diagnosticField = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(decorations, tr) {
    for (const effect of tr.effects) {
      if (!effect.is(setDiagnostics)) continue;

      const ranges = effect.value
        .filter((d) => d.line !== null && d.line >= 1 && d.line <= tr.state.doc.lines)
        .map((d) => errorLineDecoration.range(tr.state.doc.line(d.line as number).from));

      return Decoration.set(ranges, true);
    }
    return tr.docChanged ? decorations.map(tr.changes) : decorations;
  },
  provide: (field) => EditorView.decorations.from(field),
});

/** Kenar boşluğuna düşülen kırmızı kalem işareti. */
class PenMarker extends GutterMarker {
  toDOM() {
    const el = document.createElement("span");
    el.textContent = "✗";
    el.style.color = "#c8102e";
    el.style.fontWeight = "700";
    el.setAttribute("aria-label", "Bu satırda derleme hatası var");
    return el;
  }
}

const penMarker = new PenMarker();

const errorGutterState = StateField.define<RangeSet<PenMarker>>({
  create: () => RangeSet.empty,
  update(set, tr) {
    for (const effect of tr.effects) {
      if (!effect.is(setDiagnostics)) continue;

      const marks = effect.value
        .filter((d) => d.line !== null && d.line >= 1 && d.line <= tr.state.doc.lines)
        .map((d) => penMarker.range(tr.state.doc.line(d.line as number).from));

      return RangeSet.of(marks, true);
    }
    return tr.docChanged ? set.map(tr.changes) : set;
  },
});

const errorGutter = gutter({
  class: "tayan-pen-gutter",
  markers: (view) => view.state.field(errorGutterState),
  initialSpacer: () => penMarker,
});

// ── Otomatik tamamlama ────────────────────────────────────────────────────────

/**
 * Yalnızca şablonun GERÇEKTEN tanımladığı yardımcılar listelenir.
 * Kaynak: crates/tayan-compiler/src/typst_gen.rs PREAMBLE.
 * Var olmayan bir işlevi önermek, öğretmene çalışmayan kod öğretmek olur.
 *
 * Öneriler TAM İMZAYLA eklenir. Sebebi somut: yalnızca adı öneren bir liste,
 * öğretmeni parametreleri elle yazmaya bırakır ve eksik virgül gibi sözdizimi
 * hataları oradan çıkar. Tam çağrı eklenince kaynak her zaman derlenebilir
 * durumda başlar.
 */
type Snippet = { label: string; detail: string; info?: string; apply: string };

const TAYAN_HELPERS: Snippet[] = [
  {
    label: "#secenekler",
    detail: "Çoktan seçmeli şıklar",
    info: "dogru: doğru şıkkın harfi · karistir: sınavda karıştırılsın mı",
    apply: '#secenekler(dogru: "A", karistir: false,\n  [],\n  [],\n  [],\n  [],\n  [],\n)',
  },
  {
    label: "#dogru-yanlis",
    detail: "Doğru / yanlış kutucukları",
    info: "dogru: true ya da false. Cevap kâğıda basılmaz.",
    apply: "#dogru-yanlis(dogru: true)",
  },
  {
    label: "#bosluk",
    detail: "Boşluk doldurma",
    info: "cevap: kabul edilenler | ile ayrılır",
    apply: '#bosluk(cevap: "", width: 4cm)',
  },
  {
    label: "#cevap-alani",
    detail: "Klasik soru cevap çizgileri",
    apply: "#cevap-alani(satir: 6)",
  },
  { label: "#cb", detail: "Kutucuk", apply: "#cb()" },
  { label: "#tik", detail: "Tik işareti", apply: "#tik()" },
  { label: "#blank", detail: "Düz boşluk çizgisi", apply: "#blank(width: 4cm)" },
];

const TYPST_BASICS: Snippet[] = [
  { label: "#image", detail: "Görsel", apply: '#image("", width: 60%)' },
  { label: "#table", detail: "Tablo", apply: "#table(columns: 2, [], [])" },
  { label: "#underline", detail: "Altı çizili", apply: "#underline[]" },
  { label: "#text", detail: "Metin biçimi", apply: "#text(size: 10pt)[]" },
  { label: "#v", detail: "Dikey boşluk", apply: "#v(0.5cm)" },
  { label: "#h", detail: "Yatay boşluk", apply: "#h(1em)" },
];

/**
 * Kalıpların parametreleri. İmleç bir kalıp çağrısının İÇİNDEYSE ad yerine
 * parametre önerilir — yanlış parametre adı yazmak, bu ekranda en sık yapılan
 * hatalardan biri.
 */
/**
 * Matematik parçacıkları. "$" yazınca listelenir.
 *
 * Hepsi satır içi biçimde: dolarların yanında boşluk yok. Boşluk koymak Typst'te
 * bloğa geçirir ve formül cümlenin ortasından kendi satırına düşer.
 */
const MATH_SNIPPETS: Snippet[] = [
  { label: "$ satır içi", detail: "Cümlenin içinde", apply: "$$" },
  { label: "$ blok", detail: "Kendi satırında, ortalı", apply: "$  $" },
  { label: "$ türev", detail: "dy/dx", apply: "$(dif y)/(dif x)$" },
  { label: "$ ikinci türev", detail: "d²y/dx²", apply: "$(dif^2 y)/(dif x^2)$" },
  { label: "$ kısmi türev", detail: "∂f/∂x", apply: "$(diff f)/(diff x)$" },
  { label: "$ integral", detail: "Belirsiz", apply: "$integral f(x) dif x$" },
  { label: "$ belirli integral", detail: "Sınırlı", apply: "$integral_0^1 f(x) dif x$" },
  { label: "$ limit", detail: "lim", apply: "$lim_(x -> 0) $" },
  { label: "$ toplam", detail: "sigma", apply: "$sum_(i=1)^n $" },
  { label: "$ kesir", detail: "a/b", apply: "$(a)/(b)$" },
  { label: "$ kök", detail: "√", apply: "$sqrt(x)$" },
  { label: "$ nabla", detail: "∇", apply: "$nabla f$" },
];

/**
 * Yunan harfleri. Typst'te adıyla yazılır; büyük harf için baş harf büyük.
 * "$" yazınca matematik parçacıklarıyla birlikte listelenir.
 */
const GREEK: Snippet[] = [
  ["alpha", "α"], ["beta", "β"], ["gamma", "γ"], ["delta", "δ"],
  ["epsilon", "ε"], ["zeta", "ζ"], ["eta", "η"], ["theta", "θ"],
  ["iota", "ι"], ["kappa", "κ"], ["lambda", "λ"], ["mu", "μ"],
  ["nu", "ν"], ["xi", "ξ"], ["pi", "π"], ["rho", "ρ"],
  ["sigma", "σ"], ["tau", "τ"], ["upsilon", "υ"], ["phi", "φ"],
  ["chi", "χ"], ["psi", "ψ"], ["omega", "ω"],
  ["Gamma", "Γ"], ["Delta", "Δ"], ["Theta", "Θ"], ["Lambda", "Λ"],
  ["Xi", "Ξ"], ["Pi", "Π"], ["Sigma", "Σ"], ["Phi", "Φ"],
  ["Psi", "Ψ"], ["Omega", "Ω"],
].map(([ad, glif]) => ({
  label: `$${ad}`,
  detail: glif,
  info: `$${ad}$ olarak yazılır`,
  apply: `$${ad}$`,
}));

const PARAMS: Record<string, Snippet[]> = {
  secenekler: [
    { label: "dogru", detail: "Doğru şıkkın harfi", apply: 'dogru: "A", ' },
    { label: "karistir", detail: "Sınavda karıştırılsın mı", apply: "karistir: true, " },
  ],
  "dogru-yanlis": [{ label: "dogru", detail: "true / false", apply: "dogru: true" }],
  bosluk: [
    { label: "cevap", detail: "Kabul edilen cevaplar, | ile", apply: 'cevap: "", ' },
    { label: "width", detail: "Çizgi genişliği", apply: "width: 4cm" },
  ],
  "cevap-alani": [{ label: "satir", detail: "Çizgi sayısı", apply: "satir: 6" }],
};

/** İmlecin içinde bulunduğu, henüz kapanmamış kalıp çağrısının adı. */
function enclosingCall(text: string): string | null {
  let depth = 0;
  for (let i = text.length - 1; i >= 0; i -= 1) {
    const ch = text[i];
    if (ch === ")") depth += 1;
    else if (ch === "(") {
      if (depth === 0) {
        const before = text.slice(0, i);
        const match = before.match(/#([a-zA-Z][\w-]*)$/);
        return match ? match[1] : null;
      }
      depth -= 1;
    }
  }
  return null;
}

function tayanCompletions(context: CompletionContext) {
  const upto = context.state.sliceDoc(0, context.pos);
  const call = enclosingCall(upto);

  if (call && PARAMS[call]) {
    const word = context.matchBefore(/[a-zA-Z-]*/);
    if (!word) return null;
    return {
      from: word.from,
      options: PARAMS[call].map((p) => ({ ...p, type: "property" })),
    };
  }

  const word = context.matchBefore(/[#$][\w-]*/);
  if (!word || (word.from === word.to && !context.explicit)) return null;

  return {
    from: word.from,
    options: [
      ...TAYAN_HELPERS.map((o) => ({ ...o, type: "function" })),
      ...TYPST_BASICS.map((o) => ({ ...o, type: "function" })),
      ...MATH_SNIPPETS.map((o) => ({ ...o, type: "keyword" })),
      ...GREEK.map((o) => ({ ...o, type: "constant" })),
    ],
  };
}

// ── Kurulum ───────────────────────────────────────────────────────────────────

export function typstEditorExtensions(
  onChange: (value: string) => void,
  onPaste?: (event: ClipboardEvent) => boolean,
): Extension[] {
  return [
    lineNumbers(),
    errorGutter,
    highlightActiveLine(),
    highlightActiveLineGutter(),
    history(),
    indentOnInput(),
    bracketMatching(),
    closeBrackets(),
    autocompletion({ override: [tayanCompletions] }),
    keymap.of([...closeBracketsKeymap, ...defaultKeymap, ...historyKeymap, indentWithTab]),
    typstSyntax,
    diagnosticField,
    errorGutterState,
    editorTheme,
    EditorView.lineWrapping,
    EditorState.tabSize.of(2),
    EditorView.updateListener.of((update) => {
      if (update.docChanged) onChange(update.state.doc.toString());
    }),
    // Görsel yapıştırma. false dönerse CodeMirror normal metin yapıştırmasını yapar.
    EditorView.domEventHandlers({
      paste: (event) => (onPaste ? onPaste(event) : false),
    }),
  ];
}

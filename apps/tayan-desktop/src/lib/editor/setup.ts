import { EditorView, keymap, lineNumbers, highlightActiveLine, highlightActiveLineGutter, gutter, GutterMarker, tooltips } from "@codemirror/view";
import { EditorState, StateEffect, StateField, RangeSet, type Extension } from "@codemirror/state";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import { bracketMatching, indentOnInput } from "@codemirror/language";
import { closeBrackets, closeBracketsKeymap, autocompletion, type CompletionContext } from "@codemirror/autocomplete";
import { Decoration, type DecorationSet } from "@codemirror/view";

import { typstSyntax } from "./typst-lang";
import { typstSymbols, inMathMode, localDefinitions, type TypstSymbol } from "./symbols";
import { tinymistComplete, toCodeMirror } from "./tinymist";
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

  // ── Tamamlama kutusu ────────────────────────────────────────────────────
  // Genişlik sınırlı: uzun imzalar (tinymist bazen tam fonksiyon imzası
  // döndürüyor) kutuyu ekran genişliğine kadar şişiriyordu.
  ".cm-tooltip.cm-tooltip-autocomplete": {
    border: "1px solid #a4b3ad",
    background: "#fbfbf8",
    boxShadow: "0 1px 2px rgba(22,35,63,0.08), 0 4px 12px rgba(22,35,63,0.10)",
    maxWidth: "420px",
  },
  ".cm-tooltip-autocomplete > ul": {
    maxHeight: "220px",
    fontFamily: "'JetBrains Mono', ui-monospace, monospace",
    fontSize: "12px",
    lineHeight: "20px",
  },
  ".cm-tooltip-autocomplete > ul > li": {
    padding: "1px 8px",
    // Uzun satır kutuyu genişletmesin: kesilir, tamamı seçilince yazılır.
    overflow: "hidden",
    textOverflow: "ellipsis",
    whiteSpace: "nowrap",
  },
  ".cm-tooltip-autocomplete > ul > li[aria-selected]": {
    background: "#e9ebe4",
    color: "#96061f",
  },
  ".cm-completionLabel": { color: "inherit" },
  ".cm-completionDetail": {
    color: "#6e716b",
    fontStyle: "normal",
    marginLeft: "0.6em",
  },
  // Sağdaki ayrıntı paneli de sınırlı; tinymist uzun belge döndürebiliyor.
  ".cm-completionInfo": {
    maxWidth: "320px",
    maxHeight: "220px",
    overflow: "auto",
    border: "1px solid #a4b3ad",
    background: "#fbfbf8",
    padding: "4px 8px",
    fontSize: "12px",
    lineHeight: "18px",
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
 * Kalıp parçacıkları. Bunlar sembol dökümünde de var ama burada TAM ÇAĞRI ve
 * öğretici ipucuyla duruyorlar: sadece adı önermek, öğretmeni parametreleri
 * elle yazmaya bırakır ve eksik virgül gibi hatalar oradan çıkar.
 *
 * Geri kalan her şey — 554 Typst sembolü — elle yazılmıyor, Rust tarafında
 * kütüphanenin kendisi taranıyor (bkz. symbols.ts).
 */
type Snippet = { label: string; detail: string; info?: string; apply: string };

const TEMPLATES: Snippet[] = [
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
  {
    label: "#image",
    detail: "Görsel — ortalı",
    info: "Yapıştırmak (⌘V) daha hızlı: dosya kendiliğinden kopyalanır.",
    apply: '#align(center)[#image("images/", width: 60%)]',
  },
  {
    label: "#figure",
    detail: "Numaralı şekil, altyazılı",
    apply: '#figure(image("images/", width: 60%), caption: [])',
  },
];

/**
 * Sembolü tamamlama girdisine çevirir.
 *
 * Matematik kipinde `#` YOK: $alpha$ doğru, $#alpha$ hata. Kip dışında işlev
 * çağrıları `#` ile başlar. Bu ayrım yapılmazsa öneriler derlenmeyen kod üretir.
 */
function toCompletion(sym: TypstSymbol, math: boolean) {
  const detail =
    sym.params.length > 0 ? `(${sym.params.slice(0, 5).join(", ")})` : sym.kind;

  if (math) {
    return {
      label: sym.name,
      detail,
      info: sym.summary || undefined,
      type: sym.kind === "function" ? "function" : "constant",
      apply: sym.kind === "function" ? `${sym.name}()` : sym.name,
    };
  }

  return {
    label: `#${sym.name}`,
    detail,
    info: sym.summary || undefined,
    type: sym.kind === "tayan" ? "keyword" : "function",
    apply: sym.params.length > 0 ? `#${sym.name}()` : `#${sym.name}`,
  };
}

/** İmlecin içinde bulunduğu, henüz kapanmamış çağrının adı. */
function enclosingCall(text: string): string | null {
  let depth = 0;
  for (let i = text.length - 1; i >= 0; i -= 1) {
    const ch = text[i];
    if (ch === ")") depth += 1;
    else if (ch === "(") {
      if (depth === 0) {
        const before = text.slice(0, i);
        const match = before.match(/#?([a-zA-Z][\w-]*)$/);
        return match ? match[1] : null;
      }
      depth -= 1;
    }
  }
  return null;
}

async function tayanCompletions(context: CompletionContext) {
  const upto = context.state.sliceDoc(0, context.pos);

  // Pahalı işten ÖNCE ucuz eleme.
  //
  // Bu sıra ters yazılmıştı: her tuş vuruşunda — silme dahil — tinymist'e tam
  // belge gönderilip yanıt bekleniyor, sonra "aslında tamamlanacak kelime yok"
  // diye atılıyordu. İmlecin yazının gerisinde kalmasının ikinci sebebi buydu.
  const prefix = context.matchBefore(/[#$]?[\w.-]*/);
  if (!prefix || (prefix.from === prefix.to && !context.explicit)) return null;

  // 0) Önce tinymist. Typst'in tamamını, içe aktarılan paketleri ve belgedeki
  //    kendi tanımlarımızı bilir. Yoksa veya hata verirse null döner ve
  //    aşağıdaki kendi dökümümüze düşeriz.
  const pos = context.state.doc.lineAt(context.pos);
  const lsp = await tinymistComplete(
    context.state.doc.toString(),
    pos.number - 1,
    context.pos - pos.from,
  );

  if (lsp !== null && lsp.length > 0) {
    return {
      from: prefix.from,
      options: [
        // Kalıplarımız üstte kalır: tam çağrı ve öğretici ipucu taşıyorlar,
        // tinymist bunları sade birer işlev olarak önerir.
        ...TEMPLATES.map((t) => ({ ...t, type: "keyword", boost: 99 })),
        ...lsp.map(toCodeMirror),
      ],
    };
  }

  // Buradan itibaren yedek yol: kendi sembol dökümümüz.
  const symbols = [
    ...(await typstSymbols()),
    ...localDefinitions(context.state.doc.toString()),
  ];

  // 1) Bir çağrının içindeysek PARAMETRE öner — yanlış parametre adı bu
  //    ekranda en sık yapılan hatalardan biri.
  const call = enclosingCall(upto);
  if (call) {
    const target = symbols.find((s) => s.name === call);
    if (target && target.params.length > 0) {
      const word = context.matchBefore(/[a-zA-Z-]*/);
      if (word) {
        return {
          from: word.from,
          options: target.params.map((p) => ({
            label: p,
            type: "property",
            detail: `${call} parametresi`,
            apply: `${p}: `,
          })),
        };
      }
    }
  }

  const math = inMathMode(upto);

  // 2) Matematik kipinde # yok, sade ad yazılır.
  if (math) {
    const word = context.matchBefore(/[a-zA-Z][\w.-]*/);
    if (!word || (word.from === word.to && !context.explicit)) return null;
    return {
      from: word.from,
      options: symbols.filter((s) => s.math).map((s) => toCompletion(s, true)),
    };
  }

  // 3) Kip dışında: önce kalıplar, sonra kütüphanenin tamamı.
  const word = context.matchBefore(/#[\w-]*/);
  if (!word || (word.from === word.to && !context.explicit)) return null;

  return {
    from: word.from,
    options: [
      ...TEMPLATES.map((t) => ({ ...t, type: "keyword", boost: 99 })),
      ...symbols
        .filter((s) => !s.math && s.kind !== "value" && s.kind !== "module")
        .map((s) => toCompletion(s, false)),
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
    /**
     * Tamamlama kutusu editör bölmesinin DIŞINA taşıyordu.
     *
     * Varsayılan `position: "absolute"` kutuyu editörün kaydırıcısına göre
     * konumlandırır; imleç sağ kenara ya da alta yakınken kutu bölmeden dışarı
     * çıkıyor ve kırpılıyordu. `fixed` görüntü alanına göre konumlandırır:
     * CodeMirror kutuyu ekran içinde tutar ve altta yer yoksa YUKARI çevirir —
     * VS Code'un yaptığı da budur.
     *
     * Bedeli: kutu artık document.body'ye asılıyor, editörle birlikte
     * kaydırılmıyor. Kaydırma sırasında zaten kapanıyor, sorun olmuyor.
     */
    tooltips({ position: "fixed" }),

    autocompletion({
      override: [tayanCompletions],
      // 560 sembolün tamamını çizmek hem yavaş hem okunmaz. VS Code da
      // listeyi kırpar; aradığın ilk harflerden sonra zaten daralıyor.
      maxRenderedOptions: 60,
    }),
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

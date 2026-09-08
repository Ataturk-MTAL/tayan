import { StreamLanguage, type StringStream } from "@codemirror/language";
import { tags } from "@lezer/highlight";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";

/**
 * Typst için akış tabanlı sözdizimi modu.
 *
 * Tam bir Typst ayrıştırıcısı değildir ve olmaya çalışmaz. Amacı öğretmene
 * yapıyı göstermektir: neyin matematik, neyin komut, neyin düz metin olduğu.
 * Kaynak görünür kalsın ki neden-sonuç öğrenilebilsin — kaynağı gizlemek
 * öğretmeyi de gizler.
 */

type TypstState = {
  inMath: boolean;
  inBlockComment: boolean;
};

const FUNCTION_START = /[#][a-zA-Z_][a-zA-Z0-9_-]*/;
const KEYWORDS = new Set([
  "let", "set", "show", "import", "include", "if", "else", "for", "while",
  "return", "none", "auto", "true", "false", "context",
]);

export const typstMode = StreamLanguage.define<TypstState>({
  name: "typst",

  startState: () => ({ inMath: false, inBlockComment: false }),

  token(stream: StringStream, state: TypstState) {
    // ── Blok yorum ────────────────────────────────────────────────────────────
    if (state.inBlockComment) {
      if (stream.skipTo("*/")) {
        stream.match("*/");
        state.inBlockComment = false;
      } else {
        stream.skipToEnd();
      }
      return "comment";
    }
    if (stream.match("/*")) {
      state.inBlockComment = true;
      return "comment";
    }
    if (stream.match("//")) {
      stream.skipToEnd();
      return "comment";
    }

    // ── Matematik kipi ────────────────────────────────────────────────────────
    if (state.inMath) {
      if (stream.match("$")) {
        state.inMath = false;
        return "meta";
      }
      stream.next();
      return "atom";
    }
    if (stream.match("$")) {
      state.inMath = true;
      return "meta";
    }

    // ── Komut / kod ifadesi ───────────────────────────────────────────────────
    // stream.match düzenli ifadeyle çağrıldığında eşleşme dizisi döner, ama
    // tip imzası boolean'ı da kapsıyor; daraltmadan indekslenemez.
    const fn = stream.match(FUNCTION_START);
    if (fn && typeof fn !== "boolean") {
      const word = fn[0].slice(1);
      return KEYWORDS.has(word) ? "keyword" : "variableName.function";
    }

    // ── Dize ──────────────────────────────────────────────────────────────────
    if (stream.match(/"(?:[^"\\]|\\.)*"/)) return "string";

    // ── Ham kod ───────────────────────────────────────────────────────────────
    if (stream.match(/`[^`]*`/)) return "monospace";

    // ── Satır başı yapıları ───────────────────────────────────────────────────
    if (stream.sol()) {
      if (stream.match(/^\s*=+\s/)) return "heading";
      if (stream.match(/^\s*[-+]\s/)) return "list";
    }

    // ── Vurgu ─────────────────────────────────────────────────────────────────
    if (stream.match(/\*[^*\n]+\*/)) return "strong";
    if (stream.match(/_[^_\n]+_/)) return "emphasis";

    // ── Etiket / gönderme ─────────────────────────────────────────────────────
    if (stream.match(/[@][a-zA-Z_][a-zA-Z0-9_-]*/)) return "labelName";
    if (stream.match(/<[a-zA-Z_][a-zA-Z0-9_-]*>/)) return "labelName";

    if (stream.match(/\d+(\.\d+)?(pt|mm|cm|in|em|%|fr)?/)) return "number";

    stream.next();
    return null;
  },

  languageData: {
    commentTokens: { line: "//", block: { open: "/*", close: "*/" } },
  },
});

/**
 * Renk kanalı kuralı burada da geçerlidir: kırmızı yalnızca değerlendirme
 * kanalıdır, bu yüzden sözdizimi renklendirmesinde kırmızı KULLANILMAZ.
 * Editörde kırmızı gördüğün an bir hata vardır.
 */
export const typstHighlight = HighlightStyle.define([
  // Renkler CSS değişkeninden: vurgu tanımı derleme anında sabitleniyor ve
  // koyu kipte yeniden kurulmuyor. Sabit hex yazsaydık koyu zeminde lacivert
  // anahtar kelimeler okunmaz olurdu.
  { tag: tags.comment, color: "var(--cm-syn-comment)", fontStyle: "italic" },
  { tag: tags.meta, color: "var(--cm-syn-meta)", fontWeight: "700" },
  { tag: tags.atom, color: "var(--cm-syn-atom)" },
  { tag: tags.keyword, color: "var(--cm-syn-keyword)", fontWeight: "700" },
  { tag: tags.function(tags.variableName), color: "var(--cm-syn-function)", fontWeight: "500" },
  { tag: tags.string, color: "var(--cm-syn-string)" },
  { tag: tags.monospace, color: "var(--cm-syn-mono)" },
  { tag: tags.heading, color: "var(--cm-syn-keyword)", fontWeight: "700" },
  { tag: tags.list, color: "var(--cm-syn-mono)" },
  { tag: tags.strong, fontWeight: "700" },
  { tag: tags.emphasis, fontStyle: "italic" },
  { tag: tags.labelName, color: "var(--cm-syn-label)" },
  { tag: tags.number, color: "var(--cm-syn-atom)" },
]);

export const typstSyntax = [typstMode, syntaxHighlighting(typstHighlight)];

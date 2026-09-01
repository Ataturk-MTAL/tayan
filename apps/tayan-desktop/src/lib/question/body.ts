import type { ContentNode } from "$lib/types";

/**
 * Yeni editör gövdeyi tek bir `typst_raw` düğümü olarak saklar. Ama bankada
 * eski zengin metin editörüyle yazılmış sorular var; onlar `text`/`math`/`blank`
 * düğümlerinden oluşuyor.
 *
 * Bu iki yön, o soruları kaybetmeden açabilmek için var. Eski bir soruyu açıp
 * kaydeden öğretmen onu Typst kaynağına çevirmiş olur — tek yönlü ve bilinçli.
 */

export function typstBody(code: string): ContentNode[] {
  return [{ type: "typst_raw", code }];
}

/** Typst kaynağında özel anlamı olan karakterleri kaçır. */
function escapeText(text: string): string {
  return text.replace(/([#@$*_`~<>\\])/g, "\\$1");
}

function styledText(node: Extract<ContentNode, { type: "text" }>): string {
  let out = escapeText(node.text);
  if (node.style.bold) out = `*${out}*`;
  if (node.style.italic) out = `_${out}_`;
  if (node.style.underline) out = `#underline[${out}]`;
  if (node.style.strikethrough) out = `#strike[${out}]`;
  return out;
}

export function bodySource(body: ContentNode[]): string {
  if (body.length === 1 && body[0].type === "typst_raw") {
    return body[0].code;
  }

  return body
    .map((node) => {
      switch (node.type) {
        case "text":
          return styledText(node);
        case "math":
          return node.display === "block" ? `\n$ ${node.raw} $\n` : `$${node.raw}$`;
        case "typst_raw":
          return node.code;
        case "chem":
          return `$${node.raw}$`;
        case "image": {
          const width = node.width ? `, width: ${node.width}` : "";
          return `#image("${node.src}"${width})`;
        }
        case "blank":
          return `#blank(width: ${node.width ?? "4cm"})`;
        case "newline":
          return "\n";
        default:
          return "";
      }
    })
    .join("");
}

/** Eski düğümlerden dönüştürülmüş bir gövde mi? Öğretmene söylemek gerekir. */
export function isLegacyBody(body: ContentNode[]): boolean {
  return body.length > 0 && !(body.length === 1 && body[0].type === "typst_raw");
}

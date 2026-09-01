import { invoke } from "@tauri-apps/api/core";

/**
 * tinymist dil sunucusundan tamamlama.
 *
 * Ayrı süreçte çalışıyor (yayımlanan tinymist kütüphaneleri typst'in yamalanmış
 * bir çatalına göre yazılmış, crates.io'dan derlenmiyor). Ölçüm: initialize
 * 87 ms, tamamlama 6 ms, ~50 MB RSS.
 */
export type LspCompletion = {
  label: string;
  detail: string | null;
  documentation: string | null;
  /** LSP CompletionItemKind */
  kind: number | null;
  insert_text: string | null;
  /** 2 = snippet */
  insert_format: number | null;
};

/**
 * LSP CompletionItemKind -> CodeMirror tipi.
 * Sayılar LSP belirtiminden; ikon seçimi için.
 */
const KIND: Record<number, string> = {
  3: "function",
  5: "property",
  6: "variable",
  7: "class",
  9: "namespace",
  10: "property",
  14: "keyword",
  15: "text",
  21: "constant",
  22: "type",
  25: "type",
};

/**
 * Snippet yer tutucularını temizler.
 *
 * LSP snippet biçimi $1, ${2:ad}, $0 kullanır. CodeMirror'ın snippet motoruna
 * bağlamadan bunları olduğu gibi eklemek, öğretmenin gövdesine "${1:body}" gibi
 * derlenmeyen metin bırakır.
 */
function plainInsert(item: LspCompletion): string {
  const raw = item.insert_text ?? item.label;
  if (item.insert_format !== 2) return raw;

  return raw
    .replace(/\$\{\d+:([^}]*)\}/g, "$1")
    .replace(/\$\{\d+\}/g, "")
    .replace(/\$\d+/g, "");
}

let unavailable = false;

/**
 * tinymist'ten tamamlama ister. Sunucu yoksa veya hata verirse null döner ve
 * çağıran kendi sembol dökümüne düşer — tamamlama uğruna yazmayı bloklamayız.
 *
 * Bir kez başarısız olursa bir daha denenmez: her tuş vuruşunda çalışmayan bir
 * süreci yeniden başlatmaya çalışmak, yazmayı yavaşlatmaktan başka işe yaramaz.
 */
export async function tinymistComplete(
  body: string,
  line: number,
  character: number,
): Promise<LspCompletion[] | null> {
  if (unavailable) return null;

  try {
    return await invoke<LspCompletion[]>("lsp_complete", { body, line, character });
  } catch {
    unavailable = true;
    return null;
  }
}

export function toCodeMirror(item: LspCompletion) {
  return {
    label: item.label,
    detail: item.detail ?? undefined,
    info: item.documentation ?? undefined,
    type: item.kind !== null ? (KIND[item.kind] ?? "variable") : "variable",
    apply: plainInsert(item),
  };
}

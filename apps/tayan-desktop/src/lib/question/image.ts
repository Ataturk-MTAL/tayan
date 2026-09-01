import { api } from "$lib/api";

/**
 * Görselin uygulama veri klasörüne yazılması ve Typst çağrısına çevrilmesi.
 *
 * Saklama kararı: görsel veritabanıyla AYNI klasöre yazılır ve kaynakta GÖRELİ
 * yolla anılır. Mutlak yol kullanıcı adını gömer; veri başka bir makineye ya da
 * başka bir kullanıcıya taşındığında sınav görselsiz basılır ve bunu fark etmek
 * zordur.
 */

const MAX_BYTES = 8 * 1024 * 1024;

const EXT_BY_TYPE: Record<string, string> = {
  "image/png": "png",
  "image/jpeg": "jpg",
  "image/gif": "gif",
  "image/webp": "webp",
  // SVG vektörel kalır, baskıda her ölçekte keskin çıkar. Typst onu resvg ile
  // çizer; script çalıştırmaz, ayrıca önizleme çıktısı ayrıca temizlenir.
  "image/svg+xml": "svg",
};

export function isSupportedImage(file: File | null): boolean {
  return file !== null && file.type in EXT_BY_TYPE;
}

function readAsBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result));
    reader.onerror = () => reject(new Error("Dosya okunamadı."));
    reader.readAsDataURL(file);
  });
}

/** Görseli kaydeder ve gövdeye eklenecek Typst çağrısını döndürür. */
export async function saveImageAsTypst(file: File, width = "60%"): Promise<string> {
  const ext = EXT_BY_TYPE[file.type];
  if (!ext) {
    throw new Error(`Desteklenmeyen görsel biçimi: ${file.type || "bilinmiyor"}`);
  }
  if (file.size > MAX_BYTES) {
    throw new Error(
      `Görsel çok büyük (${Math.round(file.size / 1024 / 1024)} MB). En fazla 8 MB.`,
    );
  }

  const dataUrl = await readAsBase64(file);
  const relative = await api.images.save(dataUrl, ext);
  return `#image("${relative}", width: ${width})`;
}

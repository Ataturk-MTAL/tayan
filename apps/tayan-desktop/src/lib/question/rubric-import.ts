import type { RubricItem } from "$lib/types";

/**
 * Soru gövdesindeki `#rubrik((...))` bloğunu panele taşınabilir veriye çevirir.
 *
 * NE OLDUĞU KADAR NE OLMADIĞI DA ÖNEMLİ.
 *
 * Bu bir Typst çözümleyicisi DEĞİL ve asla not vermez. Tek işi, öğretmenin
 * elindeki eski cevap anahtarı dosyasından yapıştırdığı ölçütleri BİR KEZ,
 * gözünün önünde, panele taşımak. Taşındıktan sonra hüküm panelin ve
 * ClassicQuestion::validate()'in.
 *
 * Bu ayrım güvenliğin kendisi: kutucuklar not veriyor. Kutucukların kaynağı
 * bir metin çözümleyicisi olsaydı, yarım yazılmış bir makro yüzünden verilen
 * not yanlış olurdu. Burada en kötü ihtimalle "ayrıştıramadım" denir ve
 * öğretmen ölçütleri elle girer.
 *
 * YALNIZ DÜZ-DEĞİŞMEZ ALT KÜME kabul edilir:
 *
 *     #rubrik((
 *       ([ölçüt metni], 6),
 *       ([başka ölçüt], 4),
 *     ))
 *
 * Değişken (`#rubrik(olcutler)`), koşul, döngü ve hesaplanmış puan KABUL
 * EDİLMEZ. Typst tam bir dil; bunları doğru okumak derlemek demektir ve
 * derlemenin ürettiği şey kaynakta yazandan başka olabilir. Sınırın burada
 * olması bilinçli.
 */

export type RubricImport =
  | { ok: true; items: RubricItem[]; from: number; to: number }
  | { ok: false; reason: string; from: number; to: number };

const CAGRI = "#rubrik(";

/**
 * Dengeli ayraç tarayıcı. `konum` açılışı gösterir; kapanışın indisini döndürür.
 * Kapanmıyorsa -1 — yarım yazılmış makro sessizce kabul edilmemeli.
 */
function ayraciKapat(kaynak: string, konum: number): number {
  let derinlik = 0;
  let dizede: '"' | null = null;

  for (let i = konum; i < kaynak.length; i++) {
    const c = kaynak[i];

    if (c === "\\") {
      i++;
      continue;
    }
    if (dizede) {
      if (c === dizede) dizede = null;
      continue;
    }
    if (c === '"') {
      dizede = '"';
      continue;
    }
    if (c === "(" || c === "[") derinlik++;
    else if (c === ")" || c === "]") {
      derinlik--;
      if (derinlik === 0) return i;
      if (derinlik < 0) return -1;
    }
  }
  return -1;
}

/** Gövdede `#rubrik(` var mı? Kaydetme kapısı ve uyarı şeridi bunu sorar. */
export function hasRubricCall(source: string): boolean {
  return source.includes(CAGRI);
}

/**
 * `([içerik], puan)` demetlerini ayıklar.
 *
 * İçerik ham bırakılır: matematik ve biçimlendirme ölçüt metninin parçası.
 */
function demetleriOku(govde: string): RubricItem[] | string {
  const items: RubricItem[] = [];
  let i = 0;

  while (i < govde.length) {
    const c = govde[i];
    if (c === "," || /\s/.test(c)) {
      i++;
      continue;
    }
    if (c !== "(") {
      return `Beklenmeyen karakter: "${c}". Yalnız ([ölçüt], puan) demetleri okunabiliyor.`;
    }

    const kapanis = ayraciKapat(govde, i);
    if (kapanis === -1) return "Ayraçlar kapanmamış.";

    const demet = govde.slice(i + 1, kapanis);
    i = kapanis + 1;

    // İçerik bloğu ile puanı ayır: [..] , sayı
    const icerikBas = demet.indexOf("[");
    if (icerikBas === -1) return "Ölçüt metni [ ] içinde değil.";

    const icerikSon = ayraciKapat(demet, icerikBas);
    if (icerikSon === -1) return "Ölçüt metninin köşeli parantezi kapanmamış.";

    const criterion = demet.slice(icerikBas + 1, icerikSon).trim();
    if (criterion === "") return "Boş ölçüt metni.";

    const kalan = demet.slice(icerikSon + 1).replace(/^\s*,\s*/, "").trim();
    // Hesaplanmış puan (5 + 5, degisken) KABUL EDİLMEZ: doğru okuduğumuzu
    // ancak tam sayıda garanti edebiliriz.
    if (!/^\d+$/.test(kalan)) {
      return `Puan tam sayı olmalı, okunan: "${kalan}".`;
    }

    items.push({ criterion, points: Number(kalan) });
  }

  return items.length > 0 ? items : "Hiç ölçüt bulunamadı.";
}

/**
 * Gövdedeki İLK `#rubrik(...)` bloğunu okur.
 *
 * Blok yoksa null döner — bu bir hata değil, olağan durum.
 */
export function importRubric(source: string): RubricImport | null {
  const bas = source.indexOf(CAGRI);
  if (bas === -1) return null;

  const acilis = bas + CAGRI.length - 1;
  const kapanis = ayraciKapat(source, acilis);
  if (kapanis === -1) {
    return {
      ok: false,
      reason: "#rubrik( ayracı kapanmamış.",
      from: bas,
      to: source.length,
    };
  }

  const to = kapanis + 1;
  let ic = source.slice(acilis + 1, kapanis).trim();

  // `goster: true` bizim ürettiğimiz çağrıda var; içe aktarırken anlamı yok.
  ic = ic.replace(/,\s*goster\s*:\s*(true|false)\s*$/, "").trim();

  // Dış demet parantezi: ((..), (..)) → (..), (..)
  if (ic.startsWith("(") && ayraciKapat(ic, 0) === ic.length - 1) {
    ic = ic.slice(1, -1);
  }

  const sonuc = demetleriOku(ic);
  if (typeof sonuc === "string") {
    return { ok: false, reason: sonuc, from: bas, to };
  }
  return { ok: true, items: sonuc, from: bas, to };
}

/** Bloğu gövdeden çıkarır. Taşıma tek yönlü: panel artık sahibi. */
export function removeRange(source: string, from: number, to: number): string {
  return (source.slice(0, from) + source.slice(to)).replace(/\n{3,}/g, "\n\n").trim();
}

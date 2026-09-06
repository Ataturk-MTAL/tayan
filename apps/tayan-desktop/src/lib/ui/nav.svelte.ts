/**
 * Uygulamanın bölüm listesi — çekmecenin ve başlık şeridinin ortak kaynağı.
 *
 * TEK KAYNAK, İKİ TÜKETİCİ. Bölümler `+layout.svelte` içinde sabit bir dizide
 * duruyordu; oradan yalnız çekmece okuyabiliyordu. Başlık şeridi
 * (`PageShell.svelte`) araya `{@render children()}` sınırı girdiği için o
 * diziyi göremiyor, aynı bilgiyi ikinci kez tanımlamak gerekiyordu — ve iki
 * kopya er geç birbirinden ayrı düşer. Liste `$state` olarak burada durunca
 * bir bölüm eklemek, silmek ya da yeniden adlandırmak İKİ yeri birden
 * güncelliyor.
 *
 * `$state` KULLANILIYOR, sabit dizi değil: bölümlerin çalışma anında
 * değişebilmesi (role göre bölüm gizleme, sonradan eklenecek bölümler)
 * hedeflenen davranış. Sabit diziyle bu, iki tüketicinin de elle yeniden
 * çizilmesini gerektirirdi.
 */

import {
  ChartPieOutline,
  ClipboardListOutline,
  FileLinesOutline,
  InfoCircleOutline,
  QuestionCircleOutline,
  UsersGroupOutline,
} from "flowbite-svelte-icons";

export type NavItem = {
  href: string;
  label: string;
  /** Bölümün ikonu. `ThemeToggle`daki `typeof SunOutline` deseniyle aynı. */
  icon: typeof FileLinesOutline;
};

class NavState {
  /** Uygulamanın asıl bölümleri — çekmecenin üst kümesi. */
  main = $state<NavItem[]>([
    { href: "/questions", label: "Sorular", icon: FileLinesOutline },
    { href: "/exams", label: "Sınavlar", icon: ClipboardListOutline },
    { href: "/students", label: "Öğrenciler", icon: UsersGroupOutline },
    { href: "/analysis", label: "Analiz", icon: ChartPieOutline },
  ]);

  /**
   * Yardımcı bölümler — çekmecenin altında, ayraçtan sonra.
   *
   * AYRI LİSTE, `ana`nın devamı değil: bunlar uygulamanın işi değil, uygulama
   * HAKKINDA. Tek listede olsalardı "Analiz"den sonra "Yardım" gelirdi ve
   * öğretmen için ikisi eşit ağırlıkta görünürdü.
   */
  secondary = $state<NavItem[]>([
    { href: "/yardim", label: "Yardım", icon: QuestionCircleOutline },
    { href: "/hakkinda", label: "Hakkında", icon: InfoCircleOutline },
  ]);

  /** İki kümenin birleşimi — "bu yol hangi bölüme ait" araması için. */
  get all(): NavItem[] {
    return [...this.main, ...this.secondary];
  }
}

export const nav = new NavState();

/**
 * Yol bu bölümün içinde mi.
 *
 * `/questions` ile `/questions/new` aynı bölüm. `startsWith` TEK BAŞINA
 * yetmez: `/exams` öneki ileride eklenecek bir `/exams-arsiv` yoluyla da
 * eşleşir ve yanlış bölüm etkin görünürdü. Ayraç şartı bunu keser.
 */
export function isInSection(href: string, path: string): boolean {
  return path === href || path.startsWith(`${href}/`);
}

/**
 * Yolun ait olduğu bölüm, yoksa `null`.
 *
 * Kök yol (`/`) hiçbir bölüme ait değil — ana sayfa bölümlerin ÜSTÜNDE durur,
 * içlerinden biri değildir. Bu yüzden `null` dönmesi bir eksiklik değil doğru
 * cevap; çağıran taraf bölüm etiketini o durumda hiç göstermez.
 */
export function activeSection(path: string): NavItem | null {
  return nav.all.find((item) => isInSection(item.href, path)) ?? null;
}

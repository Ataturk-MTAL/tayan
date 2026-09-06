/**
 * Tema seçimi — açık, koyu ya da sistemin kararı.
 *
 * SEÇİM KALICI. Öğretmen temayı bir kez seçer, uygulama her açılışta onu
 * hatırlar. `prefers-color-scheme`'i dayatmak, projeksiyonda sunum yapan bir
 * öğretmeni akşam saatinde koyu temaya mahkûm ederdi.
 *
 * "system" ayrı bir seçenek olarak duruyor: kullanıcı açıkça "işletim
 * sistemim ne diyorsa" diyebilmeli ve bu, o an koyu olmasıyla aynı şey değil —
 * sistem gündüz açığa dönünce uygulama da dönmeli.
 */

const STORAGE_KEY = "tayan.theme.v1";

export type ThemeChoice = "light" | "dark" | "system";

const CHOICES: ThemeChoice[] = ["light", "dark", "system"];

function readStoredChoice(): ThemeChoice {
  if (typeof localStorage === "undefined") return "system";
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return CHOICES.includes(raw as ThemeChoice) ? (raw as ThemeChoice) : "system";
  } catch {
    // Gizli pencere ya da kapatılmış site verisi: varsayılana düş, çökme.
    return "system";
  }
}

function isSystemDark(): boolean {
  if (typeof window === "undefined" || !window.matchMedia) return false;
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

class ThemeState {
  /** Kullanıcının seçimi. */
  choice = $state<ThemeChoice>("system");
  /** Sistemin o anki hâli — "system" seçiliyken hangi temanın uygulandığını belirler. */
  systemDark = $state(false);

  /** Ekranda gerçekten hangi tema var. */
  get isDark(): boolean {
    return this.choice === "dark" || (this.choice === "system" && this.systemDark);
  }
}

export const theme = new ThemeState();

/**
 * Temayı belgeye uygular ve sistem değişikliklerini izler.
 *
 * `+layout.svelte` bunu bir kez çağırıyor. Dönen fonksiyon dinleyiciyi
 * kaldırır; düzen bileşeni ömür boyu ayakta kaldığı için pratikte
 * çağrılmıyor, yine de sızıntı bırakmamak için var.
 */
export function initTheme(): () => void {
  theme.choice = readStoredChoice();
  theme.systemDark = isSystemDark();
  applyTheme();

  if (typeof window === "undefined" || !window.matchMedia) return () => {};

  const mq = window.matchMedia("(prefers-color-scheme: dark)");
  const onSystemChange = (e: MediaQueryListEvent) => {
    theme.systemDark = e.matches;
    applyTheme();
  };
  mq.addEventListener("change", onSystemChange);
  return () => mq.removeEventListener("change", onSystemChange);
}

export function setTheme(next: ThemeChoice) {
  theme.choice = next;
  try {
    localStorage.setItem(STORAGE_KEY, next);
  } catch {
    // Yazamamak temayı uygulamayı engellemez; yalnız hatırlanmaz.
  }
  applyTheme();
}

/** Açık ↔ koyu arasında gidip gelir. "system" seçiliyken karşıtına geçer. */
export function toggleTheme() {
  setTheme(theme.isDark ? "light" : "dark");
}

function applyTheme() {
  if (typeof document === "undefined") return;
  document.documentElement.classList.toggle("dark", theme.isDark);
  // Tarayıcının kendi çizdiği alanlar (kaydırma çubuğu, form denetimleri)
  // temayla uyumlu olsun; olmazsa koyu sayfada beyaz kaydırma çubuğu kalır.
  document.documentElement.style.colorScheme = theme.isDark ? "dark" : "light";
}

<script lang="ts">
  import { selectedLabel } from "$lib/ui/select-label";
  /**
   * SelectBox — kapalı liste ya da serbest girişli (combobox) seçim kutusu.
   *
   * NEDEN yerel `<select>` yetmedi:
   *
   * 1. `Ders` serbest metindi ve "Matematik" / "matematik" / "MATEMATİK" üç
   *    ayrı ders oluyordu. Kazanım kodu (MAT.9.1.2) derse bağlı; ders dizesi
   *    tutarsızsa kazanım eşleşmesi baştan çürük olur. Bu bileşen bilinen
   *    dersleri ÖNERİR ama listede olmayanı da yazdırır — meslek liselerinde
   *    her okulun kendi dersi olabilir, kapalı liste onları dışarıda bırakırdı.
   *
   * 2. Yerel `<select>` macOS'ta kendi çerçevesini ve okunu çiziyor; ne eski
   *    Kırmızı Kalem'in cetvelli diline ne de şimdiki Flowbite girdi
   *    diline (kenarlık, yuvarlak köşe, odak halkası) uyuyordu.
   *
   * `allowCustom: false` verilirse kapalı liste gibi davranır: yazılan metin
   * yalnız süzme yapar, değer olarak kaydedilmez.
   *
   * GÖRÜNÜM KENDİ SINIFLARINDA (RuledField'e bağımlı DEĞİL): önceki sürüm
   * stilini sarmalayan RuledField'in `:global()` seçicisinden alıyordu.
   * Burası artık Flowbite'ın `Input` temasındaki `md` boyutunu birebir
   * taşıyor — RuledField dışında kullanılsa bile görünüm bozulmaz. RuledField
   * de bu girdiyi `:not([role="combobox"])` ile kendi kurallarının dışında
   * bırakıyor; yoksa katmansız CSS'i buradaki `pr-8` ve `border-red-500`
   * sınıflarını eziyor, ok metnin üstüne biniyordu.
   */
  type Option = { value: string; label: string };

  type Props = {
    value: string;
    options: Option[];
    /** true ise listede olmayan bir değer de yazılabilir. */
    allowCustom?: boolean;
    placeholder?: string;
    invalid?: boolean;
    /** Boş değer satırı. null ise boş seçilemez. */
    emptyLabel?: string | null;
    onchange: (value: string) => void;
  };

  let {
    value,
    options,
    allowCustom = false,
    placeholder = "",
    invalid = false,
    emptyLabel = null,
    onchange,
  }: Props = $props();

  let open = $state(false);
  let query = $state("");
  let active = $state(0);
  let root = $state<HTMLDivElement | undefined>(undefined);
  let input = $state<HTMLInputElement | undefined>(undefined);

  /**
   * LİSTE `fixed` KONUMLANIYOR, BU YÜZDEN ÖLÇÜLÜYOR.
   *
   * Liste `absolute` iken QuestionInspector'daki kutular panelin alt
   * yarısındayken listenin yalnız ilk bir iki satırı görünüyordu: DockPanel'in
   * `min-h-0 flex-1 overflow-auto` kaydırma kabı, `overflow: auto` olan bir ata
   * olarak mutlak konumlu torununu KIRPIYOR (`z-20` yığın sırasını değiştirir,
   * kırpmayı engellemez). `fixed` öğe ise ata `overflow`una takılmaz — atalarda
   * `transform`/`filter`/`contain` da yok, dolayısıyla kapsayan blok viewport.
   * Bedeli: konum artık CSS'ten değil, girdinin ölçüsünden geliyor.
   */
  const MENU_MAX_HEIGHT = 240; /* 15rem — eskiden `max-h-60` sınıfıydı */
  const MENU_GAP = 4;
  /**
   * YALNIZ İLK KARE TAHMİNİ — liste henüz DOM'da yokken kullanılır. 32 px =
   * text-sm 20px satır + py-1.5 2×6px, ve bu ancak satır TEK SATIRSA doğru.
   * `<li>` bu yüzden `truncate` taşıyor (bkz. şablon): etiket ham öğretmen
   * metni (uzun sınav/ders adı) ve sarmaya izin verilse satır 2-3 katına çıkıp
   * bu sabiti sessizce yanlışa çevirirdi. Sarma serbest bırakılacaksa bu sabit
   * de gitmeli.
   */
  const ROW_HEIGHT = 32;
  /** ul kenarlığı: üst+alt 1px. `scrollHeight` kenarlığı SAYMAZ, elle eklenir. */
  const MENU_BORDER = 2;
  /** ul: py-1 2×4px + MENU_BORDER — yalnız ilk kare tahmininde kullanılır. */
  const MENU_CHROME = 8 + MENU_BORDER;

  /** Ölçüm için liste elemanı. Liste kapalıyken yok; o an tahmine düşülür. */
  let menu = $state<HTMLUListElement | null>(null);
  let menuLeft = $state(0);
  let menuWidth = $state(0);
  /** Yukarı çevrildi mi? Çevrildiyse liste ALT kenarından çıpalanır. */
  let menuUp = $state(false);
  /** `menuUp` ise viewport'un altından `bottom`, değilse üstünden `top` (px). */
  let menuAnchor = $state(0);
  let menuMaxHeight = $state(MENU_MAX_HEIGHT);

  let menuStyle = $derived(
    `left: ${menuLeft}px; width: ${menuWidth}px; max-height: ${menuMaxHeight}px; ` +
      (menuUp ? `bottom: ${menuAnchor}px` : `top: ${menuAnchor}px`),
  );

  const id = `cb-${Math.random().toString(36).slice(2, 9)}`;

  /** Kapalıyken seçili etiketi, açıkken yazılanı gösterir. */
  let shown = $derived(open ? query : selectedLabel(options, value, emptyLabel));

  /** Türkçe küçültme: "I" → "ı", "İ" → "i". toLowerCase() bunu bozar. */
  function fold(s: string): string {
    return s.toLocaleLowerCase("tr");
  }

  let filtered = $derived.by(() => {
    const q = fold(query.trim());
    const base = q === "" ? options : options.filter((o) => fold(o.label).includes(q));
    return emptyLabel === null ? base : [{ value: "", label: emptyLabel }, ...base];
  });

  /**
   * Listeyi girdinin altına, yer yoksa üstüne hizalar.
   *
   * DÜZELTME — eski hesap listeyi HER ZAMAN 240 px yüksek sayıyordu:
   * `above = rect.top - MENU_MAX_HEIGHT - MENU_GAP` ile üstten çıpalıyordu,
   * oysa `max-h-60` yalnız ÜST SINIR. QuestionInspector'daki "Zorluk" listesi
   * 3 seçenek + emptyLabel = 4 satır → 4 × 32 + 10 = 138 px. Yukarı çevrilen
   * böyle bir liste girdinin 244 px yukarısına konuyor, alt kenarıyla girdi
   * arasında 244 − 138 = 106 px boşluk kalıyordu: liste havada asılı duruyor,
   * hangi alana ait olduğu anlaşılmıyordu. "Eşleşme yok" tek satırında
   * (28 + 10 = 38 px) boşluk 206 px'e çıkıyordu.
   *
   * Üç değişiklik:
   * • Yukarı çevrildiğinde `top` değil `bottom` veriliyor. Alt kenar girdiye
   *   yapışık kaldığı için KONUMLAMA gerçek yüksekliği bilmek zorunda değil —
   *   liste kaç satır olursa olsun girdinin 4 px üstünde bitiyor. (Çevirme
   *   KARARI ayrı mesele, bir alttaki madde.)
   * • `max-height` o yöndeki gerçek boşluğa kelepçeleniyor; hiçbir yönde
   *   viewport dışına taşmıyor, sığmayan liste kendi kaydırıcısını açıyor.
   * • ÇEVİRME KARARI da 240 px varsayımını bıraktı: liste çizildikten sonra
   *   `scrollHeight` ile GERÇEK yükseklik okunuyor. Yukarıdaki 138 px'lik
   *   "Zorluk" listesi, altında 138 px'e yetecek yer varken 240 px sanıldığı
   *   için gereksiz yere yukarı çevriliyordu. ROW_HEIGHT tahmini yalnız liste
   *   henüz DOM'da yokken devrede; efekt çizimin ardından ölçüp yönü düzeltiyor.
   */
  function measureMenu() {
    const rect = input?.getBoundingClientRect();
    if (!rect) return;
    menuLeft = rect.left;
    menuWidth = rect.width;

    /*
     * `scrollHeight` `max-height` kelepçesinden ETKİLENMEZ — içeriğin doğal
     * yüksekliğini verir, bu yüzden ölç → kelepçele → yeniden ölç salınımı
     * doğmuyor. Kenarlığı saymadığı için MENU_BORDER ekleniyor.
     */
    const rows = Math.max(filtered.length, 1); /* boşken "Eşleşme yok" satırı */
    const natural = menu ? menu.scrollHeight + MENU_BORDER : rows * ROW_HEIGHT + MENU_CHROME;
    const wanted = Math.min(MENU_MAX_HEIGHT, natural);
    const roomBelow = window.innerHeight - rect.bottom - MENU_GAP;
    const roomAbove = rect.top - MENU_GAP;

    // Aşağı sığmıyorsa VE yukarıda daha çok yer varsa çevrilir. İkisi de
    // yetmediğinde geniş olan yön seçiliyor ve liste oraya kelepçeleniyor.
    menuUp = wanted > roomBelow && roomAbove > roomBelow;
    menuAnchor = menuUp ? window.innerHeight - rect.top + MENU_GAP : rect.bottom + MENU_GAP;
    menuMaxHeight = Math.max(0, Math.min(MENU_MAX_HEIGHT, menuUp ? roomAbove : roomBelow));
  }

  /**
   * Yazdıkça `filtered` daralıyor: 12 satırlık liste tek satıra düşünce hem
   * çevirme kararı hem `max-height` geçersizleşiyor. `measureMenu()` eskiden
   * yalnız açılışta çağrılıyordu, bu yüzden daralan liste açılış anındaki
   * yönde ve kelepçede kalıyordu. Efekt `measureMenu()` içindeki
   * `filtered.length` okuması sayesinde her süzmede yeniden koşuyor.
   *
   * GERÇEK YÜKSEKLİK ÖLÇÜMÜNÜN TEK YERİ DE BURASI: efekt DOM güncellendikten
   * SONRA koşar, yani `menu` bağlanmıştır. `openList()` içindeki çağrı liste
   * daha çizilmemişken olduğu için kararı ROW_HEIGHT tahminiyle verir; buradaki
   * çağrı ölçer ve gerekiyorsa yönü düzeltir.
   */
  $effect(() => {
    if (!open) return;
    measureMenu();
  });

  function openList() {
    if (open) return;
    query = "";
    const i = options.findIndex((o) => o.value === value);
    active = i < 0 ? 0 : i + (emptyLabel === null ? 0 : 1);
    measureMenu();
    open = true;
  }

  function close() {
    open = false;
    query = "";
  }

  function pick(option: Option) {
    onchange(option.value);
    close();
    input?.focus();
  }

  function onInput(event: Event) {
    const text = (event.currentTarget as HTMLInputElement).value;
    query = text;
    active = 0;
    // `openList()` DEĞİL: o `query`yi sıfırlar ve yazılan metni siler. Yalnız
    // ölçüp açıyoruz.
    if (!open) {
      measureMenu();
      open = true;
    }
    // Serbest girişte yazılan metin DOĞRUDAN değerdir. Listeden seçmeyi
    // beklemek, listede olmayan bir dersi girmeyi imkânsız kılardı.
    if (allowCustom) onchange(text);
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      openList();
      active = Math.min(active + 1, filtered.length - 1);
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      active = Math.max(active - 1, 0);
    } else if (event.key === "Enter") {
      if (open && filtered[active]) {
        event.preventDefault();
        pick(filtered[active]);
      }
    } else if (event.key === "Escape") {
      // Yalnız liste AÇIKKEN yutulur. Kapalıyken Esc üst katmana geçmeli —
      // yoksa çekmece ve yardım paneli kapanamaz (docs/UI-IA.md §9.5).
      if (open) {
        event.stopPropagation();
        close();
      }
    } else if (event.key === "Tab") {
      close();
    }
  }

  function onWindowPointerDown(event: PointerEvent) {
    if (!open || !root) return;
    if (!root.contains(event.target as Node)) close();
  }

  /**
   * `fixed` liste sayfayla birlikte KAYMAZ; panel kaydırılınca havada asılı
   * kalırdı. Kaydırma olayı KABARMADIĞI için DockPanel'in kaydırma kabından
   * haberdar olmanın tek yolu yakalama (capture) aşamasında dinlemek.
   *
   * DÜZELTME (gerileme geri alındı): bu dinleyici eskiden `close()` çağırıyor,
   * yalnız olayın hedefi listenin İÇİNDEYSE geçiyordu. Kısa bir liste (Zorluk:
   * 4 satır, 138 px) kendi içinde kaydırılamaz olduğu için üzerinde tekerlek
   * çevirmek DockPanel'i kaydırır; olayın hedefi liste olmadığından kutu
   * kapanırdı. `absolute` sürümde liste panelle birlikte kayıp AÇIK kalıyordu,
   * yani kapanmak bir gerilemeydi. Artık kapatmıyoruz, yeniden ölçüyoruz:
   * liste girdiyi takip ediyor — eski `absolute` davranışının aynısı.
   */
  $effect(() => {
    if (!open) return;

    window.addEventListener("scroll", measureMenu, true);
    window.addEventListener("resize", measureMenu);
    return () => {
      window.removeEventListener("scroll", measureMenu, true);
      window.removeEventListener("resize", measureMenu);
    };
  });
</script>

<svelte:window onpointerdown={onWindowPointerDown} />

<div class="relative" bind:this={root}>
  <input
    bind:this={input}
    type="text"
    role="combobox"
    aria-expanded={open}
    aria-controls={id}
    aria-autocomplete={allowCustom ? "list" : "none"}
    aria-invalid={invalid}
    readonly={!allowCustom}
    value={shown}
    {placeholder}
    oninput={onInput}
    onkeydown={onKeydown}
    onmousedown={() => {
      if (!allowCustom) openList();
    }}
    class="block w-full rounded-lg border bg-gray-50 px-2.5 py-2.5 pr-8 text-sm text-gray-900
           placeholder-gray-400 focus:outline-none focus:ring-1 dark:bg-gray-700 dark:text-white
           dark:placeholder-gray-400"
    class:cursor-pointer={!allowCustom}
    class:border-gray-300={!invalid}
    class:dark:border-gray-600={!invalid}
    class:focus:border-primary-500={!invalid}
    class:focus:ring-primary-500={!invalid}
    class:border-red-500={invalid}
    class:dark:border-red-500={invalid}
    class:focus:border-red-500={invalid}
    class:focus:ring-red-500={invalid}
  />

  <button
    type="button"
    class="absolute inset-y-0 right-0 flex w-8 items-center justify-center text-gray-400
           transition-colors hover:text-primary-600 dark:text-gray-500 dark:hover:text-primary-400"
    tabindex="-1"
    aria-label={open ? "Listeyi kapat" : "Listeyi aç"}
    onclick={() => {
      if (open) {
        close();
      } else {
        input?.focus();
        openList();
      }
    }}
  >
    {open ? "▴" : "▾"}
  </button>

  {#if open}
    <!--
      `max-h-60` SINIFI YOK: üst sınır artık `menuStyle` içinde ve açılan yöndeki
      gerçek boşluğa kelepçeli. Satır içi stil sınıfı zaten ezeceği için sınıf
      olarak bırakmak, okuyanı 240 px'in her koşulda geçerli olduğuna inandıran
      ölü koddu. `overflow-y-auto` kalıyor: kelepçe devreye girince liste kendi
      kaydırıcısını açıyor.
    -->
    <ul
      bind:this={menu}
      {id}
      role="listbox"
      class="fixed z-20 overflow-y-auto rounded-lg border border-gray-200 bg-white
             py-1 shadow-lg dark:border-gray-600 dark:bg-gray-700"
      style={menuStyle}
    >
      {#if filtered.length === 0}
        <li class="px-2.5 py-1.5 text-xs text-gray-400 dark:text-gray-500">Eşleşme yok</li>
      {:else}
        {#each filtered as option, i (option.value)}
          <!--
            `truncate` (+ `title`): etiket ham öğretmen metni — uzun sınav/ders
            adı gelebilir ve `ul` genişliği girdiye kilitli. Sarmaya izin
            verilirse satır 2-3 satıra çıkar: hem ROW_HEIGHT ilk kare tahmini
            yanlışa döner, hem kelepçe devreye girip kaydırıcı çıktığında daralan
            genişlik sarmayı yeniden hesaplatıp yükseklik oynar. Metin
            KISALTILMIYOR, yalnız görsel olarak kesiliyor; tamamı `title`da
            duruyor — kesilen metne erişim böyle sağlanıyor.
          -->
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <li
            role="option"
            aria-selected={option.value === value}
            title={option.label}
            class="cursor-pointer truncate px-2.5 py-1.5 text-sm text-gray-900 dark:text-gray-50"
            class:bg-primary-50={i === active}
            class:text-primary-700={i === active}
            class:dark:bg-gray-600={i === active}
            class:dark:text-white={i === active}
            onpointerenter={() => (active = i)}
            onpointerdown={(e) => {
              // preventDefault: girdinin odağı kaybetmesini engeller, yoksa
              // dışarı-tık dinleyicisi listeyi seçim olmadan kapatır.
              e.preventDefault();
              pick(option);
            }}
          >
            {option.label}
          </li>
        {/each}
      {/if}
    </ul>
  {/if}
</div>

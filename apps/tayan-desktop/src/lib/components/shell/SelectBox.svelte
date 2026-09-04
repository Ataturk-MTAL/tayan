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
   * taşıyor — RuledField dışında kullanılsa bile görünüm bozulmaz.
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

  function openList() {
    if (open) return;
    query = "";
    const i = options.findIndex((o) => o.value === value);
    active = i < 0 ? 0 : i + (emptyLabel === null ? 0 : 1);
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
    if (!open) open = true;
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
    <ul
      {id}
      role="listbox"
      class="absolute inset-x-0 top-full z-20 mt-1 max-h-60 overflow-y-auto rounded-lg border
             border-gray-200 bg-white py-1 shadow-lg dark:border-gray-600 dark:bg-gray-700"
    >
      {#if filtered.length === 0}
        <li class="px-2.5 py-1.5 text-xs text-gray-400 dark:text-gray-500">Eşleşme yok</li>
      {:else}
        {#each filtered as option, i (option.value)}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <li
            role="option"
            aria-selected={option.value === value}
            class="cursor-pointer px-2.5 py-1.5 text-sm text-gray-900 dark:text-gray-50"
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

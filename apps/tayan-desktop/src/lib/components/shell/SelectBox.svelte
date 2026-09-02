<script lang="ts">
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
   * 2. Yerel `<select>` macOS'ta kendi çerçevesini ve okunu çiziyor; Kırmızı
   *    Kalem'in cetvelli, köşesiz diline uymuyordu.
   *
   * `allowCustom: false` verilirse kapalı liste gibi davranır: yazılan metin
   * yalnız süzme yapar, değer olarak kaydedilmez.
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
  let shown = $derived.by(() => {
    if (open) return query;
    const hit = options.find((o) => o.value === value);
    return hit ? hit.label : value;
  });

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
  />

  <button
    type="button"
    class="chevron"
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
    <ul {id} role="listbox" class="panel">
      {#if filtered.length === 0}
        <li class="pencil px-half py-quarter text-[12px]">Eşleşme yok</li>
      {:else}
        {#each filtered as option, i (option.value)}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <li
            role="option"
            aria-selected={option.value === value}
            class="row"
            class:is-active={i === active}
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

<style>
  /*
    Girdinin kendi stilini RuledField veriyor (alt çizgi, saydam zemin).
    Burada yalnız açılan liste biçimlendiriliyor: köşesiz, cetvelli kâğıt.
  */
  .chevron {
    position: absolute;
    right: 0;
    top: 0;
    padding: 0 2px;
    font-size: 11px;
    line-height: 20px;
    color: var(--color-pencil);
    background: transparent;
  }
  .chevron:hover {
    color: var(--color-red-deep);
  }

  .panel {
    position: absolute;
    z-index: 20;
    top: 100%;
    left: 0;
    right: 0;
    max-height: 220px;
    overflow-y: auto;
    margin-top: 2px;
    border: 1px solid var(--color-rule-strong);
    background: var(--color-paper-lift);
    box-shadow:
      0 1px 2px rgba(22, 35, 63, 0.08),
      0 4px 12px rgba(22, 35, 63, 0.1);
  }

  .row {
    padding: 3px 10px;
    font-size: 13px;
    line-height: 20px;
    cursor: pointer;
    color: var(--color-ink);
  }
  .row.is-active {
    background: var(--color-paper-sunk);
    color: var(--color-red-deep);
  }
</style>

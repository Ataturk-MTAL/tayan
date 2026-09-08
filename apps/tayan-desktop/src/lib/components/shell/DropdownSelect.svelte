<script lang="ts">
  /**
   * Tek seçimli, NATIVE OLMAYAN seçici — Flowbite `Button` + `Dropdown`.
   *
   * NEDEN `Select` DEĞİL. `Select` gerçek bir native `<select>` çiziyor ve
   * macOS onun için işletim sisteminin kendi seçim penceresini açıyor —
   * uygulamanın içinde kalmıyor, temayla ilgisi yok.
   *
   * DÜZELTME: burada eskiden "native `<select>` üreten iki bileşen var:
   * `Select` ve `MultiSelect`" yazıyordu. `MultiSelect` için bu YANLIŞTI —
   * o da bir `<select multiple>` çiziyor ama `hidden` özniteliğiyle; yalnız
   * form gönderimi için var, hiç etkileşime girilmiyor ve native pencere
   * açmıyor. `MultiSelect`ten kaçınmanın gerçek gerekçeleri başka:
   * `role="listbox"` kabının içindeki satırlar `role="presentation"`
   * (yani ekran okuyucuya sıfır seçenek), ok tuşları DOM odağını taşımadığı
   * için panel kendi kendine kaymıyor, ve yüksekliği seçim yapıldıkça
   * değişiyor (43,2 px → ~46 px), yani şeritteki 42 px'e hiç oturmuyor.
   *
   * İKİNCİ VE DAHA SERT SEBEP: native `<select>`in yüksekliği HİÇBİR sınıfla
   * ayarlanamıyor. `appearance: auto` iken Chrome'un kendi stil sayfası
   * `line-height: normal` dayatıyor; `leading-5` de satır içi
   * `style="line-height:20px"` de yok sayılıyor. Yükseklik yazı tipinin
   * metriklerinden çıkıyor ve font YÜKLENDİKÇE DEĞİŞİYOR — aynı işaretleme
   * Public Sans yüklenmeden 41 px, yüklendikten sonra 40 px ölçüldü. Yani
   * yanındaki 42 px'lik arama kutusuyla hiçbir zaman tam hizalanamaz.
   * `appearance-none` bunu çözer ama native oku da siler; o da elde ok
   * çizmek demek, yani daha çok bespoke CSS.
   *
   * KABUL EDİLEN BEDEL: native `<select>`in bedava ok tuşu gezinmesi ve
   * yazarak-bulma özelliği kayboluyor. Bu panel Sekme / Shift+Sekme / Enter /
   * Esc ile kullanılıyor — Flowbite'ın kendi erişilebilirlik notu da bunu
   * söylüyor. 1.33.1'de ok tuşlarını uygulayan tek iki bileşen
   * `CommandPalette` (kalıp pencere, Cmd+K'yı gasbediyor, kalıcı bir değeri
   * yok) ve `MultiSelect` (dizi değerli, ARIA'sı daha kötü) — ikisi de bu iş
   * için daha kötü uyum.
   *
   * `SelectBox.svelte` İLE KARIŞTIRILMAMALI: o, soru formundaki SERBEST
   * METİN girilebilen ders alanı; kapalı listeye çevrilirse bankada olmayan
   * meslek dersleri yazılamaz. Bu bileşen kapalı liste içindir.
   */
  import { Button, Dropdown, DropdownDivider, DropdownGroup, DropdownItem } from "flowbite-svelte";
  import { CheckOutline, ChevronDownOutline } from "flowbite-svelte-icons";

  export type DropdownSelectOption = { name: string; value: string };

  type Props = {
    /** Seçili değer. `""` = hiçbiri, yani `placeholder` durumu. */
    value: string;
    options: DropdownSelectOption[];
    /**
     * Tetikleyicinin `id`si. HER ÖRNEK İÇİN BENZERSİZ olmalı: `Dropdown`
     * tetikleyicisini `querySelectorAll` ile buluyor, aynı `id` iki kez
     * geçerse tek panel iki düğmeye birden bağlanır.
     */
    id: string;
    /** `<Label for=…>` ile eşleşen etiketin `id`si. */
    labelId?: string;
    /** Seçim yokken görünen metin. */
    placeholder?: string;
    /** Panelin ve tetikleyicinin genişlik sınıfı — ikisi tek yerden. */
    width?: string;
    onchange: (value: string) => void;
  };

  let {
    value,
    options,
    id,
    labelId = undefined,
    placeholder = "Tümü",
    width = "w-56",
    onchange,
  }: Props = $props();

  let isOpen = $state(false);

  let label = $derived(options.find((option) => option.value === value)?.name ?? placeholder);

  /**
   * Seçimi uygular ve paneli kapatır.
   *
   * ODAK TETİKLEYİCİYE GERİ VERİLMİYOR — bilerek. Flowbite'ın `Popper`ı
   * `focusable` bayrağını kaynakta sabit `true` yazmış ve `focusin` olayını
   * paneli AÇMAK için dinliyor; seçimden sonra tetikleyiciyi odaklamak paneli
   * anında yeniden açardı. 1.33.1'de bunu kapatan bir prop yok.
   */
  function choose(next: string) {
    onchange(next);
    isOpen = false;
  }

  /**
   * Tetikleyici odaktayken Backspace/Delete seçimi temizler; ok tuşları da
   * buradan panele giriyor.
   *
   * TETİKLEYİCİDE DE GEREKLİ: panel odak alınca kendiliğinden açılıyor ama
   * odak hâlâ düğmede; o anda basılan Aşağı ok olayı panele değil düğmeye
   * düşüyor.
   */
  function onTriggerKeydown(event: KeyboardEvent) {
    if (event.key === "Backspace" || event.key === "Delete") {
      event.preventDefault();
      choose("");
      return;
    }
    onArrowKeys(event);
  }

  let host = $state<HTMLElement | null>(null);

  /**
   * Panel içinde ok tuşlarıyla gezinme.
   *
   * NEDEN ELDE YAZILDI. `role="menu"` ve `menuitemradio` ekran okuyucuya
   * "ok tuşlarıyla gezilir" sözü veriyor; flowbite-svelte'in `Dropdown`u ise
   * yalnız Sekme/Enter/Esc uyguluyor. Söz verip tutmamak, hiç söz vermemekten
   * kötü: kullanıcı Aşağı oka basıyor, hiçbir şey olmuyor ve on iki dersin
   * arasından geçmek için on iki kez Sekme'ye basmak zorunda kalıyor.
   *
   * Dinleyici hem tetikleyicide hem panelde. `Dropdown`a verilen `onkeydown`
   * `Popper`ın `{...restProps}` yayılımıyla `role="menu"` taşıyan panel
   * öğesine iniyor; kaba konsaydı Svelte haklı olarak "rolü olmayan `<div>`
   * klavye dinleyemez" diye uyarırdı. Öğeleri bulmak için yine host
   * kullanılıyor — panel DOM ağacında onun içinde.
   */
  function onArrowKeys(event: KeyboardEvent) {
    if (!isOpen || !host) return;
    const step = { ArrowDown: 1, ArrowUp: -1 }[event.key];
    const edge = { Home: 0, End: -1 }[event.key];
    if (step === undefined && edge === undefined) return;

    const items = [...host.querySelectorAll<HTMLElement>('[role="menuitemradio"]')];
    if (items.length === 0) return;

    event.preventDefault();

    if (edge !== undefined) {
      items.at(edge)?.focus();
      return;
    }

    const current = items.indexOf(document.activeElement as HTMLElement);
    // Odak henüz panelde değilse (tetikleyicideyken Aşağı ok) baştan başla.
    const next =
      current === -1
        ? step === 1
          ? 0
          : items.length - 1
        : (current + step! + items.length) % items.length;
    items[next]?.focus();
  }
</script>

<!--
  YÜKSEKLİK 42 px, arama kutusuyla BİREBİR.
  `Button` varsayılan `size="md"` → `px-5 py-2.5 text-sm` = 20 px satır + 20 px
  dolgu + 2 px kenarlık. `Search size="md"` → `text-sm p-2.5 ps-10 pe-10` =
  aynı 42 px. Düğmenin yatay dolgusu `px-2.5` ile arama kutusununkine
  çekiliyor; bu yüksekliği etkilemiyor.

  RENKLER DE ARAMA KUTUSUNDAN: `color="light"` `bg-white` veriyor, arama
  kutusu ise `bg-gray-50 dark:bg-gray-700`. İkisi yan yana duracağı için
  düğme arama kutusunun zeminine çekiliyor.

  CHEVRON'A BOYUT VERİLMİYOR. Flowbite'ın kendi doküman örnekleri ikona
  `h-6 w-6` yazıyor; o 24 px, düğmeyi 46 px'e çıkarıp hizayı sessizce bozar.
  İkonun kendi varsayılanı (`w-5 h-5`) doğru olan.
-->
<div bind:this={host} class={width}>
  <Button
    {id}
    color="light"
    class="w-full justify-between bg-gray-50 px-2.5 font-normal dark:bg-gray-700"
    aria-haspopup="menu"
    aria-expanded={isOpen}
    aria-labelledby={labelId ? `${labelId} ${id}` : undefined}
    onkeydown={onTriggerKeydown}
  >
    <span class="truncate {value === '' ? 'text-gray-500 dark:text-gray-400' : ''}">
      {label}
    </span>
    <ChevronDownOutline class="ms-2 shrink-0" />
  </Button>

  <!--
    `simple` KULLANILMIYOR: o kip, prop geçilemeyen bir iç `DropdownGroup`
    üretiyor; `role="none"` verilemiyor ve menü ARIA'sı eksik kalıyor.

    `mt-0`: panelin tema tabanında `mt-2` var; boşluğu iki yerden birden
    almasın diye iptal edildi, aralık yalnız `offset` ile veriliyor.

    GENİŞLİK AÇIKÇA VERİLİYOR. Panel floating-ui ile konumlanıyor ve yalnız
    `left`/`top` yazılıyor — tetikleyicinin genişliğini MİRAS ALMIYOR.
  -->
  <Dropdown
    bind:isOpen={isOpen}
    triggeredBy={`#${id}`}
    role="menu"
    placement="bottom-start"
    offset={4}
    onkeydown={onArrowKeys}
    class="mt-0 max-h-64 overflow-y-auto {width}"
  >
    <DropdownGroup role="none">
      <DropdownItem
        role="menuitemradio"
        aria-checked={value === ""}
        class="flex items-center justify-between gap-2"
        onclick={() => choose("")}
      >
        <span class="text-gray-500 dark:text-gray-400">{placeholder}</span>
        {#if value === ""}
          <CheckOutline class="h-4 w-4 shrink-0 text-primary-600 dark:text-primary-500" />
        {/if}
      </DropdownItem>

      {#if options.length > 0}
        <DropdownDivider />
      {/if}

      {#each options as option (option.value)}
        <DropdownItem
          role="menuitemradio"
          aria-checked={value === option.value}
          class="flex items-center justify-between gap-2"
          onclick={() => choose(option.value)}
        >
          <span class="truncate">{option.name}</span>
          {#if value === option.value}
            <CheckOutline class="h-4 w-4 shrink-0 text-primary-600 dark:text-primary-500" />
          {/if}
        </DropdownItem>
      {/each}
    </DropdownGroup>
  </Dropdown>
</div>

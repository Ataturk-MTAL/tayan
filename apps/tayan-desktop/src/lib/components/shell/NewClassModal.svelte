<script lang="ts">
  /**
   * "Yeni sınıf" kalıbı — iki yol: elle ve Excel'den (e-Okul listesi).
   *
   * NEDEN `Modal`ın `form` + `onaction` DESENİ KULLANILMIYOR. Dokümanın
   * öne çıkardığı kalıp `_onsubmit` içinde `const r = onaction(...); if (r ===
   * false) return; close();` yapıyor. Bir `Promise` `false` DEĞİLDİR — yani
   * `async` bir işleyicide kalıp daha `await` bitmeden kapanır ve API hatası,
   * öğretmenin artık göremediği bir ekranda belirir. Bunun yerine gövdede düz
   * bir `<form onsubmit>` var ve kalıp yalnız BAŞARILI `await`ten sonra
   * kapanıyor.
   *
   * NEDEN `title` DEĞİL `header`. `title` propu bileşenin kendi kapatma
   * düğmesini çiziyor ve o düğmenin `aria-label`i kaynağa gömülü İNGİLİZCE
   * "Close"; propuna dışarıdan erişilemiyor. `dismissable={false}` ile o düğme
   * hiç çizilmiyor, başlığı ve Türkçe kapatma düğmesini biz veriyoruz.
   *
   * ESC MERDİVENİNE KAYITLI. Flowbite'ın `Modal`ı `<dialog>` + `showModal()`
   * üstüne kurulu, yani tarayıcının kendi Esc'i var. Uygulamanın da tek
   * pencere dinleyicili bir Esc merdiveni var (`escape-stack.ts`) ve o
   * `preventDefault()` çağırıyor — kayıt olmazsa Esc, kalıbı değil ALTTAKİ
   * katmanı kapatabilir. Kayıtla merdivenin "en içteki önce kapanır" kuralı
   * korunuyor; iki yol da `open = false`ta buluşuyor, tekrarı zararsız.
   */
  import { Alert, Button, Input, Label, Modal } from "flowbite-svelte";
  import { CloseOutline, CloudArrowUpOutline } from "flowbite-svelte-icons";
  import DropdownSelect from "$lib/components/shell/DropdownSelect.svelte";
  import { pushEscapeLayer } from "$lib/ui/escape-stack";
  import { api } from "$lib/api";
  import { errorText } from "$lib/editor/diagnostics";

  export type ParsedRosterStudent = { number: string; first: string; last: string };
  export type NewClassInput = {
    name: string;
    grade: number;
    branch: string;
    students: ParsedRosterStudent[];
  };

  type Props = {
    open: boolean;
    busy: boolean;
    /** Sunucudan dönen hata — kalıbın İÇİNDE gösteriliyor, arkasında değil. */
    error: string | null;
    /** `true` dönerse kalıp kapanır. Hata durumunda `false` dönmeli. */
    oncreate: (input: NewClassInput) => Promise<boolean>;
  };

  let { open = $bindable(), busy, error, oncreate }: Props = $props();

  let mode = $state<"manual" | "excel">("manual");

  // ————— Elle —————
  let name = $state("");
  let grade = $state(9);
  let branch = $state("A");

  // ————— Excel —————
  let fileName = $state("");
  let headers = $state<string[]>([]);
  let rows = $state<string[][]>([]);
  let fileError = $state<string | null>(null);
  let reading = $state(false);

  /** Hangi başlık hangi alana denk geliyor. `""` = seçilmedi. */
  let numberColumn = $state("");
  let firstNameColumn = $state("");
  let lastNameColumn = $state("");

  /** Ad ve soyad tek sütunda mı (e-Okul'un "Adı Soyadı" biçimi). */
  const COMBINED = "__birlikte__";

  $effect(() => {
    if (!open) return;
    return pushEscapeLayer(() => {
      if (!busy && !reading) open = false;
    });
  });

  /** Türkçe normalize: karşılaştırma için küçült ve boşlukları sadeleştir. */
  function normalize(text: string): string {
    return text.toLocaleLowerCase("tr").replace(/\s+/g, " ").trim();
  }

  /*
    Başlık takma adları. e-Okul çıktısının sütun adları elimizde YOK; bu liste
    tahmin, tek doğruluk kaynağı değil — bu yüzden eşleme ekranı her zaman
    görünüyor ve otomatik bulunan değer değiştirilebiliyor. Tanıma başarısız
    olursa özellik çalışmaz hâle gelmiyor, yalnız elle eşleme gerekiyor.
  */
  const NUMBER_ALIASES = ["okul no", "öğrenci no", "no", "numara", "okul numarası", "öğrenci numarası"];
  const FULL_NAME_ALIASES = ["adı soyadı", "ad soyad", "adı ve soyadı", "öğrenci adı soyadı"];
  const FIRST_NAME_ALIASES = ["adı", "ad", "isim", "öğrenci adı"];
  const LAST_NAME_ALIASES = ["soyadı", "soyad"];

  function matchHeader(aliases: string[]): string {
    return headers.find((b) => aliases.includes(normalize(b))) ?? "";
  }

  /**
   * Başlık satırını bulur.
   *
   * İLK SATIR OLMAYABİLİR: e-Okul çıktıları başta okul adı, dönem gibi
   * serbest satırlar taşıyabiliyor. İlk 15 satır içinde bilinen takma adlara
   * EN ÇOK eşleşen satır başlık kabul ediliyor; hiç eşleşme yoksa ilk dolu
   * satıra düşülüyor ve eşleme tamamen kullanıcıya kalıyor.
   */
  function findHeaderRow(data: string[][]): number {
    const allAliases = [...NUMBER_ALIASES, ...FULL_NAME_ALIASES, ...FIRST_NAME_ALIASES, ...LAST_NAME_ALIASES];
    let bestRow = -1;
    let bestScore = 0;

    for (let i = 0; i < Math.min(data.length, 15); i++) {
      const score = data[i].filter((h) => allAliases.includes(normalize(h))).length;
      if (score > bestScore) {
        bestScore = score;
        bestRow = i;
      }
    }

    if (bestRow !== -1) return bestRow;
    return data.findIndex((r) => r.some((h) => h.trim() !== ""));
  }

  /**
   * Dosyayı okur — girdiden mi sürüklemeden mi geldiği umurunda değil.
   *
   * İKİ GİRİŞ, TEK YOL. Dosya seçiciden gelen `File` ile bırakılan `File` aynı
   * şey; ayrıştırma, başlık bulma, otomatik sütun tanıma ve hata metinleri
   * ikisinde de birebir aynı olmalı. İki ayrı işleyici yazsaydık biri
   * güncellenip diğeri unutulurdu — sürükleyen öğretmen, tıklayanın gördüğü
   * hatayı görmezdi.
   */
  async function readRoster(file: File) {
    reading = true;
    fileError = null;
    headers = [];
    rows = [];

    try {
      /*
        AYRIŞTIRMA RUST'TA. Baytı IPC ile `parse_roster`a geçiriyoruz; dosya
        seçici webview'de kaldığı için ne `plugin-fs` ne de `dialog:allow-open`
        izni gerekiyor. Rust tarafı `.xlsx` yanında `.xls`, `.xlsb` ve `.ods`
        da okuyor ve biçimi uzantıdan değil İÇERİKTEN anlıyor.
      */
      const { rows: data } = await api.students.parseRoster(
        new Uint8Array(await file.arrayBuffer()),
      );
      const headerIndex = findHeaderRow(data);
      if (headerIndex === -1) {
        fileError = "Dosyada okunabilir satır bulunamadı.";
        return;
      }

      headers = data[headerIndex].map((h, i) => (h.trim() === "" ? `Sütun ${i + 1}` : h.trim()));
      rows = data.slice(headerIndex + 1).filter((r) => r.some((h) => h.trim() !== ""));
      fileName = file.name;

      // Otomatik tanı — bulunamayan alan boş kalır, kullanıcı seçer.
      numberColumn = matchHeader(NUMBER_ALIASES);
      const combinedColumn = matchHeader(FULL_NAME_ALIASES);
      if (combinedColumn !== "") {
        firstNameColumn = combinedColumn;
        lastNameColumn = COMBINED;
      } else {
        firstNameColumn = matchHeader(FIRST_NAME_ALIASES);
        lastNameColumn = matchHeader(LAST_NAME_ALIASES);
      }

      if (name.trim() === "") name = file.name.replace(/\.[^.]+$/, "");
    } catch (err: unknown) {
      // Rust tarafı hatayı zaten Türkçe ve okunur veriyor ("Dosya açılamadı: …").
      fileError = errorText(err);
    } finally {
      reading = false;
    }
  }

  async function onFileSelected(event: Event) {
    const el = event.target as HTMLInputElement;
    const file = el.files?.[0];
    if (!file) return;

    await readRoster(file);

    // Aynı dosyayı tekrar seçebilmek için girdiyi sıfırla: `change` yalnız
    // DEĞER değişince atıyor, aynı yolu ikinci kez seçmek olay üretmezdi.
    el.value = "";
  }

  /**
   * Sürükleme alanın üstünde mi — yalnız görsel geri bildirim için.
   *
   * `dragover` alanın İÇİNDEKİ çocuklar üzerinde de atıyor ve her geçişte bir
   * `dragleave` geliyor; tek bir bayrağı `dragleave`de körlemesine kapatmak
   * çerçeveyi titretirdi. Bu yüzden çocuklar `pointer-events-none`: tarayıcı
   * sürükleme olaylarını yalnız kabın kendisinde görüyor.
   */
  let dragOver = $state(false);

  function onDragOver(event: DragEvent) {
    // Varsayılan davranış "bırakmayı reddet"; engellenmezse `drop` hiç atmıyor.
    event.preventDefault();
    dragOver = true;
  }

  function onDragLeave() {
    dragOver = false;
  }

  async function onDrop(event: DragEvent) {
    event.preventDefault();
    dragOver = false;

    const file = event.dataTransfer?.files?.[0];
    if (!file) return;
    await readRoster(file);
  }

  let columnOptions = $derived(headers.map((b) => ({ name: b, value: b })));

  let lastNameOptions = $derived([
    { name: "Ad sütununda birlikte", value: COMBINED },
    ...columnOptions,
  ]);

  function cell(row: string[], header: string): string {
    const i = headers.indexOf(header);
    if (i === -1) return "";
    return (row[i] ?? "").trim();
  }

  /** Ad/soyad ayrıştırması: son kelime soyad, kalanı ad. */
  function splitName(full: string): { first: string; last: string } {
    const words = full.split(/\s+/).filter(Boolean);
    const last = words.length > 1 ? (words.pop() as string) : "";
    return { first: words.join(" "), last };
  }

  let parsedStudents = $derived.by((): ParsedRosterStudent[] => {
    if (numberColumn === "" || firstNameColumn === "") return [];
    return rows
      .map((row) => {
        const number = cell(row, numberColumn);
        if (lastNameColumn === COMBINED) {
          const { first, last } = splitName(cell(row, firstNameColumn));
          return { number, first, last };
        }
        return {
          number,
          first: cell(row, firstNameColumn),
          last: lastNameColumn === "" ? "" : cell(row, lastNameColumn),
        };
      })
      .filter((o) => o.number !== "" && o.first !== "");
  });

  let canSubmit = $derived(
    name.trim() !== "" && !busy && !reading && (mode === "manual" || parsedStudents.length > 0),
  );

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!canSubmit) return;

    const created = await oncreate({
      name: name.trim(),
      grade: grade,
      branch: branch.trim(),
      students: mode === "excel" ? parsedStudents : [],
    });

    if (created) {
      open = false;
      name = "";
      fileName = "";
      headers = [];
      rows = [];
      fileError = null;
      mode = "manual";
    }
  }

  const TAB = "rounded-lg px-3 py-1.5 text-sm transition-colors";
  const TAB_ACTIVE = "bg-primary-100 font-semibold text-primary-800 dark:bg-primary-900/40 dark:text-primary-200";
  const TAB_IDLE = "text-gray-600 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700/60";
</script>

<Modal bind:open={open} size="lg" dismissable={false}>
  {#snippet header()}
    <div class="flex w-full items-center justify-between">
      <h3 class="text-lg font-semibold text-gray-900 dark:text-white">Yeni sınıf</h3>
      <Button
        color="alternative"
        class="border-0 p-1.5"
        aria-label="Kapat"
        disabled={busy}
        onclick={() => (open = false)}
      >
        <CloseOutline class="h-5 w-5" />
      </Button>
    </div>
  {/snippet}

  <div class="flex gap-1" role="tablist" aria-label="Sınıf ekleme yolu">
    <button
      type="button"
      role="tab"
      aria-selected={mode === "manual"}
      class="{TAB} {mode === 'manual' ? TAB_ACTIVE : TAB_IDLE}"
      onclick={() => (mode = "manual")}
    >
      Elle
    </button>
    <button
      type="button"
      role="tab"
      aria-selected={mode === "excel"}
      class="{TAB} {mode === 'excel' ? TAB_ACTIVE : TAB_IDLE}"
      onclick={() => (mode = "excel")}
    >
      Excel'den (e-Okul)
    </button>
  </div>

  <form id="new-class-form" class="mt-4 space-y-4" onsubmit={submit}>
    <!--
      ÜÇ ALAN KABIN GENİŞLİĞİNİ UMURSAMIYOR.

      Buradaki ızgara `grid-cols-[1fr_100px_100px]` idi ve KUSURLUYDU. Kalıp
      `size="lg"`, yani Flowbite temasında `max-w-4xl` = 896 px; `1fr` "makul
      genişlik" değil "ARTAN NE VARSA HEPSİ" demek, dolayısıyla "9-A" yazan
      alan ~600 px'e çıkıyor, 100 px'e çivilenmiş Seviye ve Şube de sağ kenara
      sürülüyordu. Kalıp bu kadar geniş, çünkü aşağıdaki sütun eşleme tablosu
      geniş; bu üç alanın onunla bir işi yok.

      `flex` doğru primitif: `flex-1` basis'i SIFIR yapıp büyütüyor, `max-w-xs`
      büyümenin nerede duracağını söylüyor; iki küçük alan `flex-none`, yani ne
      büyüyor ne küçülüyor — 96 px'lik kutu font ölçeğiyle birlikte değişiyor,
      sabit piksel gibi donmuyor. `flex-wrap`, dar bir kapta alanların ezilmek
      yerine alt satıra inmesi için.

      Not: ızgarada kalınsaydı bile `1fr` yanlış yazımdı — `1fr` aslında
      `minmax(auto, 1fr)` ve `auto` minimumu içeriğin min-content'i olduğu için
      uzun bir değer ızgarayı taşırır; güvenli form `minmax(0, 1fr)`.
    -->
    <div class="flex flex-wrap items-end gap-3">
      <div class="min-w-48 max-w-xs flex-1">
        <Label for="nc-name" class="mb-1.5">Sınıf adı</Label>
        <Input id="nc-name" bind:value={name} placeholder="9-A" required />
      </div>
      <div class="w-24 flex-none">
        <Label for="nc-grade" class="mb-1.5">Seviye</Label>
        <Input id="nc-grade" type="number" min="1" max="12" bind:value={grade} />
      </div>
      <div class="w-24 flex-none">
        <Label for="nc-branch" class="mb-1.5">Şube</Label>
        <Input id="nc-branch" bind:value={branch} />
      </div>
    </div>

    {#if mode === "excel"}
      <!--
        DIŞ KART KALDIRILDI. Burada `rounded-lg border p-3` taşıyan bir kap
        vardı ve içindeki kesikli bırakma alanı da yuvarlak köşeli bir kutu:
        iç içe iki çerçeve, ikincisi hiçbir şey söylemiyordu. Bırakma alanının
        kesikli kenarı zaten "buraya bırak" sınırını çiziyor.
      -->
      <div class="space-y-3">
        <!--
          DOSYA `<input type="file">` İLE ALINIYOR, Tauri dialog'uyla değil.
          `@tauri-apps/plugin-dialog`ın `open()`i yalnız YOL döndürüyor, bayt
          döndürmüyor; baytı okumak için `plugin-fs` gerekirdi — kurulu değil,
          `fs:*` izinleri de üretilmiş şemada hiç yok. Üstelik capabilities'te
          `dialog:allow-open` de verilmemiş (yalnız `dialog:allow-save`).
          Webview'in kendi dosya seçici de native pencereyi açıyor ve doğrudan
          `File` veriyor — sıfır yeni bağımlılık, sıfır yeni izin.
        -->
        <div>
          <Label for="nc-file" class="mb-1.5">Excel dosyası (.xlsx, .xls, .ods)</Label>
          <!--
            NATIVE DÜĞME YOK; ALANIN KENDİSİ HEM TIKLANIYOR HEM BIRAKILIYOR.

            `<input type="file">`in kendi görünümü iki yerden kusurlu. DİL:
            "Choose File" ve "no file selected" WebKit'in içinden geliyor;
            `::file-selector-button` biçimlendirilebiliyor ama METNİ sabit —
            tamamen Türkçe bir ekranda iki İngilizce parça kalıyordu. GEOMETRİ:
            o düğme girdinin içerik kutusunun sol üst köşesinden başlıyor ve
            KENDİ köşeleri kare; girdiye `rounded-lg` verilince kare köşe
            yuvarlatılmış kenarın içinden taşıyor, çerçevenin solunda dışarı
            çıkan gri bir dikdörtgen görünüyordu.

            FLOWBITE'IN `Fileupload`I BU KUSURU ÇÖZMÜYOR, TAŞIYOR: temasında
            (`forms/fileupload/theme.js`, 1.33.1) `rounded-lg border` var ama
            tek bir `file:` sınıfı yok — aynı kare köşe, aynı İngilizce metin.
            `Dropzone` metni bize bırakıyor ama girdisine `class="hidden"`
            DAYATIYOR ve o sınıf `restProps`tan sonra yazıldığı için dışarıdan
            geçersiz kılınamıyor: `display:none` bir girdi sekme sırasında
            yoktur, yani klavyeyle dosya seçme yolu tümüyle kapanır. Bu yüzden
            görsel dili (kesikli çerçeve, yuvarlak köşe, hover) alındı,
            işaretleme burada yazıldı.

            `<label for>` iki işi birden yapıyor: üstüne tıklamak dosya
            seçiciyi açıyor (bunun için JavaScript gerekmiyor) ve bırakma
            hedefi de o. Girdi `sr-only`, `hidden` DEĞİL — gizli ama
            odaklanabilir; `peer` olduğu için odak halkası alanın kendisinde
            beliriyor. Sekmeyle gelen kullanıcı nereye geldiğini görüyor.
          -->
          <input
            id="nc-file"
            type="file"
            accept=".xlsx,.xls,.xlsb,.ods"
            class="peer sr-only"
            onchange={onFileSelected}
          />
          <label
            for="nc-file"
            ondragover={onDragOver}
            ondragleave={onDragLeave}
            ondrop={onDrop}
            class="flex cursor-pointer flex-col items-center justify-center gap-1 rounded-lg
                   border-2 border-dashed px-4 py-6 text-center transition-colors
                   peer-focus-visible:ring-2 peer-focus-visible:ring-primary-500
                   {dragOver
              ? 'border-primary-500 bg-primary-50 dark:border-primary-500 dark:bg-primary-900/20'
              : 'border-gray-300 bg-gray-50 hover:bg-gray-100 dark:border-gray-600 dark:bg-gray-700 dark:hover:bg-gray-600'}"
          >
            <!--
              ÇOCUKLAR OLAY GEÇİRMİYOR. `pointer-events-none` olmasaydı ikon ile
              yazı arasındaki her geçiş kaba bir `dragleave` yollar ve kesikli
              çerçeve sürükleme boyunca yanıp sönerdi.
            -->
            <div class="pointer-events-none flex flex-col items-center gap-1">
              <CloudArrowUpOutline class="h-7 w-7 text-gray-400 dark:text-gray-500" />
              {#if fileName === ""}
                <p class="text-sm text-gray-700 dark:text-gray-200">
                  <span class="font-semibold">Dosya seçmek için tıkla</span>
                  ya da e-Okul listesini buraya sürükle
                </p>
                <p class="text-xs text-gray-500 dark:text-gray-400">.xlsx · .xls · .xlsb · .ods</p>
              {:else}
                <p class="text-sm font-semibold text-gray-900 dark:text-white">{fileName}</p>
                <p class="text-xs text-gray-500 dark:text-gray-400">
                  {rows.length} satır okundu · değiştirmek için tıkla
                </p>
              {/if}
            </div>
          </label>
        </div>

        {#if reading}
          <p class="text-sm text-gray-500 dark:text-gray-400">Dosya okunuyor…</p>
        {/if}

        {#if fileError}
          <Alert color="red">{fileError}</Alert>
        {/if}

        {#if headers.length > 0}
          <!--
            EŞLEME EKRANI HER ZAMAN GÖRÜNÜR, yalnız tanıma başarısız olunca
            değil: otomatik tanı bir tahmin ve sessizce yanlış sütunu seçmesi,
            hiç seçmemesinden daha kötü. Öğretmen ne eşlendiğini görüp
            düzeltebiliyor.
          -->
          <div class="grid grid-cols-3 gap-3">
            <div>
              <Label id="nc-number-label" for="nc-number" class="mb-1.5">Numara sütunu</Label>
              <DropdownSelect
                id="nc-number"
                labelId="nc-number-label"
                width="w-full"
                value={numberColumn}
                options={columnOptions}
                placeholder="Seç"
                onchange={(v) => (numberColumn = v)}
              />
            </div>
            <div>
              <Label id="nc-firstname-label" for="nc-firstname-col" class="mb-1.5">Ad sütunu</Label>
              <DropdownSelect
                id="nc-firstname-col"
                labelId="nc-firstname-label"
                width="w-full"
                value={firstNameColumn}
                options={columnOptions}
                placeholder="Seç"
                onchange={(v) => (firstNameColumn = v)}
              />
            </div>
            <div>
              <Label id="nc-lastname-label" for="nc-lastname-col" class="mb-1.5">Soyad sütunu</Label>
              <DropdownSelect
                id="nc-lastname-col"
                labelId="nc-lastname-label"
                width="w-full"
                value={lastNameColumn}
                options={lastNameOptions}
                placeholder="Seç"
                onchange={(v) => (lastNameColumn = v)}
              />
            </div>
          </div>

          {#if parsedStudents.length === 0}
            <p class="text-sm text-gray-500 dark:text-gray-400">
              Numara ve ad sütunlarını seç — önizleme burada görünecek.
            </p>
          {:else}
            <div>
              <p class="mb-1.5 text-xs font-medium text-gray-500 uppercase dark:text-gray-400">
                Önizleme — ilk 5 satır, toplam {parsedStudents.length} öğrenci
              </p>
              <ul class="space-y-1 text-sm">
                {#each parsedStudents.slice(0, 5) as o (o.number + o.first + o.last)}
                  <li class="flex gap-3">
                    <span class="tnum w-12 shrink-0 text-gray-500 dark:text-gray-400">{o.number}</span>
                    <span class="truncate">{o.first} {o.last}</span>
                  </li>
                {/each}
              </ul>
            </div>
          {/if}
        {/if}
      </div>
    {/if}

    {#if error}
      <Alert color="red">{error}</Alert>
    {/if}
  </form>

  {#snippet footer()}
    <div class="flex w-full justify-end gap-2">
      <Button color="alternative" disabled={busy} onclick={() => (open = false)}>İptal</Button>
      <Button type="submit" form="new-class-form" disabled={!canSubmit}>
        {mode === "excel" && parsedStudents.length > 0
          ? `${parsedStudents.length} öğrenciyle oluştur`
          : "Oluştur"}
      </Button>
    </div>
  {/snippet}
</Modal>

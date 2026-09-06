<script lang="ts">
  /**
   * Sınav listesi — Flowbite süzgeçleri + datatable eklentisinde sıralama ve
   * sayfalama.
   *
   * ARAMA KUTUSU EKLENTİDEN ALINDI. `simple-datatables` kendi `<input>`ünü
   * üretiyordu; o kutu Flowbite'ın `Search` bileşeni DEĞİL, yalnız ona
   * benzeyen ham bir input — odaklanınca uygulamanın geri kalanıyla uyuşmayan
   * bir çerçeve çiziyordu. `searchable: false` ile eklentinin kutusu kapatıldı;
   * arama artık gerçek `<Search>` bileşeninde ve süzme Svelte tarafında.
   *
   * SÜZME NEDEN SVELTE'TE, KÜTÜPHANEDE DEĞİL. Eklentinin arama API'si metin
   * eşleşmesi üzerine kurulu; "şu tarihten sonra" gibi bir aralık sorgusu için
   * sütun başına özel `searchMethod` yazmak gerekiyordu. Üç süzgeci de düz
   * JavaScript'te tutmak hem okunur hem de tek doğruluk kaynağı bırakıyor.
   *
   * EKLENTİNİN ÜÇ SERT KISITI — kod bunlara göre şekillendi:
   *
   * 1. `items` YALNIZ DÜZ DEĞER ALIR (`Record<string, string|number|boolean>`).
   *    Hücreye Svelte bileşeni konamıyor; durum rozeti bu yüzden
   *    `columns[].render` ile elde çiziliyor.
   *
   * 2. SÜTUN BAŞLIĞI NESNENİN ANAHTARIDIR. Eklenti başlıkları
   *    `Object.keys(items[0])` ile üretiyor, ayrı bir başlık seçeneği yok —
   *    bu yüzden anahtarlar Türkçe ve göründükleri gibi yazıldı.
   *
   * 3. TABLO YALNIZ `onMount`TA KURULUYOR. `simple-datatables` kaynak
   *    `<table>`ı bir kez okuyup DOM'u kendi sanal DOM'uyla değiştiriyor;
   *    sonradan değişen satırları görmüyor. Bu yüzden tablo `{#key}` ile
   *    sarıldı: süzgeç değişince bileşen yeniden kuruluyor. Bedeli, süzgeç
   *    değiştiğinde sıralamanın sıfırlanması — sayfa numarasının sıfırlanması
   *    zaten istenen davranış.
   */
  import { onMount } from "svelte";
  import { Alert, Button, Datepicker, Label, Search, Spinner } from "flowbite-svelte";
  import DropdownSelect from "$lib/components/shell/DropdownSelect.svelte";
  import { Table as DataTable } from "@flowbite-svelte-plugins/datatable";
  /*
    TİPLER DERİN YOLDAN ALINIYOR — mecburen. `simple-datatables`ın paket
    girişi (`dist/index.d.ts`) sonunda yalnız DEĞERLERİ dışa veriyor
    (`export { DataTable, exportCSV, … }`); `DataTableOptions`,
    `elementNodeType` ve `nodeType` o dosyanın içinde tanımlı ama export
    listesinde yok.

    Eklentinin dokümanı bunun için `app.d.ts`e bir `declare module` shim'i
    koymayı söylüyor. O shim İŞE YARAMIYOR: `export {}` yüzünden dosya modül
    sayılıyor, bildirim de paketi değiştirmek yerine augmentation'a dönüşüyor
    ve tipler sessizce `any` oluyor (ölçüldü: `DataTableOptions` olarak
    işaretlenmiş bir nesneye uydurma anahtar eklemek hata VERMİYORDU;
    `skipLibCheck: true` de `.d.ts` içindeki sorunu gizliyor). Derin yol
    paketin genel API'si değil, ama tipler orada gerçekten dışa veriliyor ve
    paketin `exports` haritası olmadığı için çözümleniyor.
  */
  import type {
    DataTableOptions,
    elementNodeType,
    nodeType,
  } from "simple-datatables/dist/dts/types";
  import type { DataTable as SimpleDataTable } from "simple-datatables";
  import PageShell from "$lib/components/shell/PageShell.svelte";
  import { api } from "$lib/api";
  import { errorText } from "$lib/editor/diagnostics";
  import { EXAM_STATUS_LABELS, type Exam } from "$lib/types";
  import { goto } from "$app/navigation";

  let exams = $state<Exam[]>([]);
  let loading = $state(true);
  let loadError = $state<string | null>(null);

  onMount(load);

  async function load() {
    loading = true;
    try {
      exams = await api.exams.list();
      loadError = null;
    } catch (err: unknown) {
      loadError = errorText(err);
    } finally {
      loading = false;
    }
  }

  // ————— Süzgeçler —————

  let searchInput = $state("");
  let subject = $state<string>("");
  let startDate = $state<Date | undefined>(undefined);
  let endDate = $state<Date | undefined>(undefined);

  /**
   * Ders listesi verinin kendisinden geliyor; sabit bir liste tutulmuyor.
   *
   * BOŞ DERS ELENİYOR. Süzgeçte `""` "tümü" anlamına gelen sentinel; dersi
   * boş bırakılmış bir sınav listeye girseydi düğmede "Tüm dersler" yazısı
   * kaybolur, panelde adsız bir satır belirir ve o satır hiçbir şeyi
   * süzmezdi — iki farklı anlam tek değere biniyordu.
   */
  let subjectOptions = $derived(
    [...new Set(exams.map((exam) => exam.meta.subject))]
      .filter((name) => name !== "")
      .sort((a, b) => a.localeCompare(b, "tr"))
      .map((name) => ({ name: name, value: name })),
  );

  /**
   * `Date` → `"YYYY-MM-DD"`, YEREL saatle.
   *
   * `toISOString()` KULLANILMIYOR: o UTC'ye çevirir ve UTC+3'te seçilen
   * 1 Ocak gecesi bir önceki güne kayabilir. Takvimden seçilen gün neyse
   * karşılaştırma da o gün üzerinden yapılmalı.
   */
  function isoDay(date: Date): string {
    const pad = (value: number) => String(value).padStart(2, "0");
    return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
  }

  /**
   * Türkçe küçültme.
   *
   * `toLowerCase()` "İSTANBUL"u bozar; Türkçede I → ı ve İ → i. Öğretmenin
   * yazdığı da, sınav adı da Türkçe.
   */
  function lower(text: string): string {
    return text.toLocaleLowerCase("tr");
  }

  /**
   * Aramanın geciktirilmiş hâli.
   *
   * NEDEN GEREKLİ. Tablo `{#key}` ile sarılı ve anahtar aramayı içeriyor;
   * her tuş vuruşu tabloyu baştan kuruyordu. Bu yalnız yavaş değil, VERİ
   * KAYBETTİRİYOR: yeniden kurulan tablo "kayıt/sayfa" seçimini de sıfırlıyor,
   * yani 50'ye çeken öğretmen ilk harfte 10'a düşüyordu.
   *
   * Etki `arama`yı OKUYUP `debouncedSearch`ye YAZIYOR, tersini yapmıyor —
   * kendi kendini tetikleyen bir döngü kurulmuyor.
   */
  const SEARCH_DEBOUNCE_MS = 250;

  let debouncedSearch = $state("");

  $effect(() => {
    const rawSearch = searchInput;
    const timer = setTimeout(() => (debouncedSearch = rawSearch), SEARCH_DEBOUNCE_MS);
    return () => clearTimeout(timer);
  });

  let searchNeedle = $derived(lower(debouncedSearch.trim()));

  let filtered = $derived(
    exams.filter((exam) => {
      if (subject && exam.meta.subject !== subject) return false;

      // `meta.date` ISO ("YYYY-MM-DD") olduğu için dizge karşılaştırması
      // kronolojik karşılaştırmayla aynı — tarih ayrıştırmaya gerek yok.
      if (startDate && exam.meta.date < isoDay(startDate)) return false;
      if (endDate && exam.meta.date > isoDay(endDate)) return false;

      if (!searchNeedle) return true;
      return [exam.meta.title, exam.meta.subject, exam.meta.classroom].some((field) =>
        lower(field).includes(searchNeedle),
      );
    }),
  );

  let hasFilter = $derived(Boolean(searchNeedle || subject || startDate || endDate));

  /**
   * Tablonun yeniden kurulmasını tetikleyen anahtar.
   *
   * Eklenti satırları yalnız kurulurken okuyor; süzgeç değişince bileşenin
   * kendisi yeniden yaratılmalı. Anahtar süzgeçlerin metin hâli.
   */
  let tableKey = $derived(
    `${searchNeedle}|${subject}|${startDate ? isoDay(startDate) : ""}|${endDate ? isoDay(endDate) : ""}`,
  );

  /**
   * Tarih alanını 42 px'e ve doğru köşe yarıçapına çeken sınıflar.
   *
   * `Datepicker`ın `size` propu YOK; girdisi temada `px-4 py-2 text-sm
   * rounded-md` ile sabit — 38 px ve köşesi komşularından keskin.
   * `py-2.5` yüksekliği 42 px'e, `rounded-lg` yarıçapı arama kutusununkine
   * eşitliyor; `ps-2.5` metin başlangıcını hizalıyor; `pe-10` sağdaki takvim
   * düğmesine yer açıyor (o düğme `absolute right-0 px-3` + 16 px ikon = 40 px).
   *
   * `classes={{ input: … }}` DEĞİL: bileşen kaynağında yalnız `inputClass`
   * okunuyor (`class={input({ color, class: clsx(theme?.input, inputClass) })}`).
   * `classes.input` tipte VAR ama çalışma anında sessizce yok sayılıyor —
   * dokümanın yanlış olduğu bir nokta daha.
   */
  const DATE_FIELD = "rounded-lg py-2.5 ps-2.5 pe-10";

  function clearFilters() {
    searchInput = "";
    subject = "";
    startDate = undefined;
    endDate = undefined;
  }

  // ————— Tablo —————

  /** `"2026-01-15"` → `"15.01.2026"`. */
  function formatDate(iso: string): string {
    const [yil, ay, gun] = iso.split("-");
    return `${gun}.${ay}.${yil}`;
  }

  /**
   * Satırlar. Anahtar sırası sütun sırası, anahtar metni sütun başlığı.
   *
   * TARİH ISO OLARAK DURUYOR, `gg.aa.yyyy` OLARAK DEĞİL. Sıralama hücrenin
   * ham değerine bakıyor; `"15.01.2026"` biçiminde saklansaydı gün gün
   * sıralanır, yıl hiç dikkate alınmazdı. ISO'da dizge sırası = zaman sırası.
   * Okunur biçim yalnız `render` ile GÖRÜNÜME ekleniyor. Aynı gerekçeyle
   * `Soru` ve `Süre` de sayı olarak duruyor ("40 dk" değil, 40).
   */
  let items = $derived(
    filtered.map((exam) => ({
      Sınav: exam.meta.title,
      Ders: exam.meta.subject,
      Sınıf: exam.meta.classroom,
      Tarih: exam.meta.date,
      Soru: exam.questions.length,
      Süre: exam.meta.duration_min,
      Durum: EXAM_STATUS_LABELS[exam.status],
      /*
        GİZLİ KİMLİK SÜTUNU — satırı güvenilir biçimde sınava bağlayan tek şey.
        Görünmüyor (`columns` içinde `hidden: true`) ama `simple-datatables`
        gizli sütunları da OKUYOR: `hidden` yalnız çizim anında kontrol
        ediliyor (`virtual_dom.ts`), okuma anında değil. En sona konuyor ki
        önündeki sütunların indeksleri değişmesin.
      */
      Id: exam.id,
    })),
  );

  /** `render`ın metin düğümü — kullanıcı metnini KAÇIRMADAN geçiren tek yol. */
  function textNode(value: string): nodeType {
    return { nodeName: "#text", data: value };
  }

  /**
   * Hücrenin içini değiştirir ve hücrenin KENDİSİNİ döndürür.
   *
   * `render` bir nesne döndürdüğünde `simple-datatables` onu `<td>`NİN YERİNE
   * koyuyor (kaynakta `if (typeof i !== "string") return i` doğrudan hücre
   * map'inden çıkıyor). Doğrudan bir `<span>` döndürmek `<td>`yi yok eder ve
   * `<tr>` içinde geçersiz bir düğüm bırakırdı. Hazır hücreyi alıp içini
   * değiştirmek, kütüphanenin ona yazdığı genişlik ve sınıfları da koruyor.
   *
   * STRING DÖNDÜRÜLMÜYOR, BİLEREK: string dönüşü `<td>${i}</td>` olarak
   * AYRIŞTIRILIYOR. Sınav adı, ders ve sınıf öğretmenin serbest metni;
   * string birleştirmek doğrudan bir XSS yolu açardı.
   */
  function fillCell(td: object, children: nodeType[]): elementNodeType {
    const cell = td as elementNodeType;
    cell.childNodes = children;
    return cell;
  }

  /**
   * Durum rozeti.
   *
   * Yayına çıkmış sınav artık düzenlenemez; kırmızı rozet bu tekliği
   * hatırlatıyor — uygulamanın her yerinde kırmızı yalnız "dikkat" demek.
   */
  const BADGE = "inline-flex items-center rounded px-2.5 py-0.5 text-xs font-medium";
  const BADGE_PUBLISHED = `${BADGE} bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300`;
  const BADGE_DRAFT = `${BADGE} bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300`;

  const tableOptions: DataTableOptions = {
    /* Arama Flowbite'ın `<Search>` bileşeninde; eklentininki kapalı. */
    searchable: false,
    sortable: true,
    paging: true,
    perPage: 10,
    perPageSelect: [10, 25, 50],
    /*
      KLAVYEYLE SATIR AÇMA.

      DÜZELTME: burada eskiden yalnız `tabIndex: 0` vardı ve yorumu
      "`rowNavigation` ve `rowSelectionKeys` zaten açık" diyordu. İKİSİ DE
      YANLIŞTI. `config.ts`te `rowNavigation: false` varsayılan; kapalıyken
      `datatable.ts` keydown dinleyicisini HİÇ kurmuyor, yani ok tuşları da
      Enter/boşluk da ölüydü. `tabIndex: 0` ise tabloyu sekme sırasına
      sokuyordu ama içinde yapılacak bir şey yoktu — klavyeyle sınav açmanın
      hiçbir yolu yoktu (WCAG 2.1.1).

      `rowNavigation: true` hem dinleyiciyi kuruyor hem tabloyu kendiliğinden
      odaklanabilir yapıyor; `rowSelectionKeys` varsayılanı `["Enter", " "]`
      zaten istenen.
    */
    rowNavigation: true,
    /*
      Sıralama Türkçe harmanlamayla. Varsayılan `"en-US"` ile "Çevre" C'lerin
      önüne, "İnkılap" "Işık"ın önüne düşüyordu; sayfanın geri kalanı zaten
      `localeCompare(…, "tr")` ve `toLocaleLowerCase("tr")` kullanıyor.
    */
    locale: "tr",
    /*
      Kalan etiketler de İngilizce geliyor ("entries per page",
      "Showing {start} to {end} of {rows} entries"). Uygulamanın geri kalanı
      Türkçe; çevrilmeseydi tablonun etrafı tek İngilizce ada olurdu.
    */
    labels: {
      placeholder: "",
      searchTitle: "",
      searchLabel: "",
      perPage: "kayıt/sayfa",
      pageTitle: "Sayfa {page}",
      noRows: "Sınav yok",
      noResults: "Eşleşen sınav yok",
      info: "{rows} sınavın {start}–{end} arası",
      sortHint: "Sıralamak için etkinleştir",
    },
    columns: [
      { select: 0, cellClass: "font-semibold text-gray-900 dark:text-white" },
      {
        select: 3,
        /*
          `type: "string"` ŞART. `simple-datatables`ın varsayılan hücre tipi
          `"html"` ve o tipte `cell.data` bir dizge değil, DÜĞÜM DİZİSİ
          (`[{ nodeName: "#text", data: "2026-01-15" }]`). Tipsiz bırakıldığında
          `render`a gelen değer diziydi; `String(...)` onu
          "[object Object],[object Object]" yapıyor, `split("-")` parça
          bulamıyor ve sütun "undefined.undefined.[object Object]" basıyordu.
          `Soru`/`Süre` sütunları bu tuzağa düşmedi çünkü onlarda `type:
          "number"` var.

          Sıralama yine ISO değeri üzerinden, yani kronolojik.
        */
        type: "string",
        render: (value, td) => fillCell(td, [textNode(formatDate(String(value)))]),
      },
      { select: 4, type: "number" },
      {
        select: 5,
        type: "number",
        render: (value, td) => fillCell(td, [textNode(`${value} dk`)]),
      },
      {
        select: 6,
        /*
          BADGE SATIRIN KENDİ HÜCRESİNDEN — dış diziye indekslenmiyor.

          DÜZELTME: burada eskiden `filtered[satir]` okunuyordu ve yorumu
          "`satir` kararlı indeks, sıralama onu değiştirmiyor" diyordu. Bu
          YANLIŞTI. `simple-datatables` sıralamayı diziyi YENİDEN KURARAK
          yapıyor (`columns.ts`: `this.dt.data.data = sortedData`) ve
          `render`a geçen indeks o yeni diziden türüyor
          (`_paginate()`: `this.data.data.map((row, index) => …)`). Yani bir
          kez başlığa tıklandıktan sonra `filtered[satir]` BAŞKA sınavı
          veriyordu: taslak bir sınavın satırında kırmızı "Yayında" rozeti
          çıkıyordu.

          `type: "string"` sayesinde `deger` doğrudan hücrenin metni; rozet o
          metinden üretiliyor ve indeksle hiç ilgisi kalmıyor.
        */
        type: "string",
        render: (value, td) => {
          const label = String(value);
          const isPublished = label === EXAM_STATUS_LABELS.Published;
          return fillCell(td, [
            {
              nodeName: "SPAN",
              attributes: { class: isPublished ? BADGE_PUBLISHED : BADGE_DRAFT },
              childNodes: [textNode(label)],
            },
          ]);
        },
      },
      /*
        Kimlik sütunu: çizilmiyor ama okunuyor. `type: "string"` veriliyor ki
        `cell.data` düğüm dizisi değil düz dizge olsun — `onRowSelected` onu
        doğrudan kullanıyor.
      */
      { select: 7, hidden: true, type: "string" },
    ],
  };

  /**
   * Satıra tıklamak ya da Enter/boşluğa basmak sınavı açar.
   *
   * KİMLİK SATIRIN KENDİ VERİSİNDEN OKUNUYOR, `filtered[satir]`DEN DEĞİL.
   *
   * DÜZELTME — bu bir veri bütünlüğü hatasıydı: `simple-datatables` sıralamayı
   * `data.data` dizisini YENİDEN KURARAK yapıyor (`columns.ts`:
   * `this.dt.data.data = sortedData`), `data-index` de o yeni diziden
   * yazılıyor (`virtual_dom.ts`) ve tıklama olayı tam olarak onu geçiriyor
   * (`datatable.ts`: `emit("datatable.selectrow", parseInt(row.dataset.index))`).
   * `filtered` ise hiç sıralanmayan özgün dizi. Yani bir kez başlığa
   * tıklandıktan sonra satıra tıklamak BAŞKA sınavı açıyordu — öğretmen
   * açtığını sandığından farklı bir sınavı düzenlemeye başlıyordu.
   *
   * `dt.data.data[satir]` ile `satir` AYNI dizinin indeksi olduğu için bu
   * okuma sıralamadan etkilenmiyor.
   *
   * SAĞ VE ORTA TIK ELENİYOR: dinleyici `mousedown` üzerinde ve düğme ayrımı
   * yapmıyordu; sağ tık bağlam menüsü yerine sayfayı değiştiriyordu.
   */
  const ID_COLUMN = 7;

  function onRowSelected(rowIndex: number, event: Event, dt: SimpleDataTable) {
    if (event instanceof MouseEvent && event.button !== 0) return;

    const cell = dt?.data?.data?.[rowIndex]?.cells?.[ID_COLUMN];
    const id = typeof cell?.data === "string" ? cell.data : "";
    if (id) goto(`/exams/${id}`);
  }
</script>

<!--
  Takvimin içindeki temizleme düğmesi.

  `showActionButtons` KULLANILMIYOR: o kip "Today" / "Clear" / "Apply"
  metinlerini kaynağa gömülü İNGİLİZCE basıyor, çeviri kancası yok.

  BU DÜĞME ŞART, süs değil: alanın metnini silip Enter'a basmak tarihi
  TEMİZLEMİYOR — bileşen `handleInputChange` içinde yalnız `rangeFrom`/
  `rangeTo`yu sıfırlıyor, `value`ya dokunmuyor, sonra `value={formatDate(value)}`
  yeniden çizilince tarih geri geliyor. `handleClear` ise üçünü birden
  sıfırlayan tek iç yol.
-->
{#snippet dateClearAction(handleClear: () => void, close: () => void)}
  <div class="mt-3 flex justify-end">
    <Button
      size="xs"
      color="alternative"
      onclick={() => {
        handleClear();
        close();
      }}
    >
      Temizle
    </Button>
  </div>
{/snippet}

<PageShell
  title="Sınavlar"
  subtitle={loading
    ? null
    : hasFilter
      ? `${filtered.length} / ${exams.length} sınav`
      : `${exams.length} sınav`}
>
  {#snippet actions()}
    <Button size="sm" onclick={() => goto("/exams/new")}>Yeni sınav</Button>
  {/snippet}

  {#if loading}
    <div class="flex items-center gap-2 text-gray-500 dark:text-gray-400">
      <Spinner size="5" />
      Sınavlar okunuyor…
    </div>
  {:else if loadError}
    <Alert color="red">{loadError}</Alert>
  {:else if exams.length === 0}
    <p class="text-gray-500 dark:text-gray-400">Henüz sınav yok.</p>
  {:else}
    <!--
      SÜZGEÇ ŞERİDİ. Üçü de Flowbite bileşeni: `Search`, `Select`, `Datepicker`
      — odak halkası, yarıçap ve koyu kip davranışı uygulamanın geri kalanıyla
      aynı yerden geliyor.

      `items-end`: etiketler farklı yükseklikte olsa da alanların ALT kenarı
      hizalı kalıyor.
    -->
    <!--
      SÜZGEÇ ŞERİDİ — üç denetim de 42 px, birebir aynı yükseklikte.

      YÜKSEKLİKLER KENDİLİĞİNDEN UYMUYOR, üç bileşenin ortak bir boyut sistemi
      yok ve varsayılanları birbirini tutmuyor:
        • `Search` varsayılanı `size="lg"` → `sm:text-base p-3` = 50 px.
          Ayrıca `lg`deki `sm:text-base` bir kırılma noktası taşıyor: 640 px
          altında 46 px'e düşüyor. `md` düz `text-sm` kullanıyor, o uçurum yok.
        • `Button` varsayılanı `md` → `px-5 py-2.5 text-sm` = 42 px.
        • `Datepicker`ın `size` propu HİÇ YOK; girdisi `px-4 py-2 text-sm
          rounded-md` ile sabitlenmiş = 38 px, üstelik köşe yarıçapı da
          diğerlerinden küçük.
      Bu yüzden `Search`e açıkça `size="md"` veriliyor ve `Datepicker`
      `inputClass` ile 42 px'e çekiliyor. `items-end` yalnız alt kenarları
      hizalar; kutuların KENDİ boyutlarını eşitlemez.
    -->
    <div class="mb-4 flex flex-wrap items-end gap-3">
      <!--
        `flex-1` YOK — kaldırıldı. Varken arama alanı şeritteki artan tüm
        genişliği yutuyordu: diğer üç alan sabit genişlikte olduğu için
        1280 px'lik pencerede arama kutusu tek başına ~600 px oluyor ve
        şeridin yarısını kaplıyordu. Alan genişliği içeriğine göre seçilir,
        artan boşluğa göre değil: serbest metin girildiği için Ders'ten (w-56)
        geniş, ama sabit.
      -->
      <div class="w-72">
        <Label for="exam-search" class="mb-1.5">Ara</Label>
        <!--
          `required={false}`: bileşen girdiye `required` yazıyor ve ekran
          okuyucu isteğe bağlı bir süzgeci "gerekli" diye duyuruyordu; alan
          boşken de geçersiz sayılıyordu. Sondaki `{...restProps}` yayılımı
          bileşenin kendi özniteliğini eziyor.
        -->
        <Search
          id="exam-search"
          size="md"
          required={false}
          bind:value={searchInput}
          placeholder="Sınav, ders veya sınıf…"
        />
      </div>

      <div>
        <Label id="subject-filter-label" for="subject-filter" class="mb-1.5">Ders</Label>
        <DropdownSelect
          id="subject-filter"
          labelId="subject-filter-label"
          value={subject}
          options={subjectOptions}
          placeholder="Tüm dersler"
          onchange={(next) => (subject = next)}
        />
      </div>

      <!--
        TEK "ARALIK" KUTUSU DEĞİL, İKİ AYRI TARİH — mecburen.

        `<Datepicker range>` girdisinin değerini kaynakta
        `` `${formatDate(rangeFrom)} - ${formatDate(rangeTo)}` `` diye
        üretiyor. İki uç da boşken bu " - " oluyor; boş olmayan bir değer
        olduğu için tarayıcı `placeholder`ı HİÇ çizmiyor. Ekranda yalnız bir
        tire kalıyordu — kullanıcının "tarih aralığı tek tarihle mi oluyor?"
        sorusunun sebebi tam olarak buydu. Bu, dışarıdan düzeltilebilir bir
        şey değil: değer `{...inputProps}`ten SONRA yazılıyor, ezilemiyor.

        İki ayrı alanda `formatDate(value)` boşken "" döndüğü için placeholder
        çalışıyor. Üstüne kütüphane bedava çapraz doğrulama veriyor:
        `availableTo`/`availableFrom` takvimde geçersiz günleri `disabled`
        yapıyor, yani ters aralık seçilemiyor. Tek kutulu `range` bunu
        doğrulamıyor, sessizce iki ucu takas ediyordu.

        Kaybedilen: takvimdeki aralık vurgusu. Kazanılan: anlaşılır boş
        durum ve ters aralığın imkânsızlığı.
      -->
      <div class="w-44">
        <Label for="date-start" class="mb-1.5">Başlangıç</Label>
        <Datepicker
          bind:value={startDate}
          availableTo={endDate}
          locale="tr"
          firstDayOfWeek={1}
          placeholder="Tümü"
          inputClass={DATE_FIELD}
          inputProps={{ id: "date-start" }}
        >
          {#snippet actionSlot({ handleClear, close })}
            {@render dateClearAction(handleClear, close)}
          {/snippet}
        </Datepicker>
      </div>

      <div class="w-44">
        <Label for="date-end" class="mb-1.5">Bitiş</Label>
        <Datepicker
          bind:value={endDate}
          availableFrom={startDate}
          locale="tr"
          firstDayOfWeek={1}
          placeholder="Tümü"
          inputClass={DATE_FIELD}
          inputProps={{ id: "date-end" }}
        >
          {#snippet actionSlot({ handleClear, close })}
            {@render dateClearAction(handleClear, close)}
          {/snippet}
        </Datepicker>
      </div>

      {#if hasFilter}
        <Button color="alternative" onclick={clearFilters}>Süzgeçleri temizle</Button>
      {/if}
    </div>

    {#if filtered.length === 0}
      <!--
        Eklenti boş `items` ile hiç `<thead>`/`<tbody>` çizmiyor; kendi
        "sonuç yok" mesajı da o durumda görünmüyor. Mesaj bu yüzden burada.
      -->
      <p class="text-gray-500 dark:text-gray-400">Bu süzgeçlerle eşleşen sınav yok.</p>
    {:else}
      <!--
        `{#key}`: eklenti satırları yalnız kurulurken okuduğu için süzgeç
        değişince tablo yeniden kurulmalı.

        `selectable` burada "seçim" için değil, AÇMAK için: eklentinin satır
        etkinleştirmeyi bildirdiği tek geri çağrısı `onSelectRow` ve fareyle
        tıklamayı da Enter/boşluk tuşunu da o veriyor.
      -->
      {#key tableKey}
        <DataTable
          {items}
          dataTableOptions={tableOptions}
          selectable
          multiSelect={false}
          onSelectRow={onRowSelected}
        />
      {/key}
    {/if}
  {/if}
</PageShell>

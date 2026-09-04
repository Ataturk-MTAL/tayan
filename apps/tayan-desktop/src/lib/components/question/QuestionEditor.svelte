<script lang="ts">
  import { Spinner } from "flowbite-svelte";
  import TypstSource from "./TypstSource.svelte";
  import SheetPreview from "./SheetPreview.svelte";
  import FloatingPalette from "./FloatingPalette.svelte";
  import QuestionInspector from "./QuestionInspector.svelte";
  import DockPanel from "../shell/DockPanel.svelte";
  import PenButton from "../shell/PenButton.svelte";
  import { layout, setMode, type ViewMode } from "$lib/ui/layout.svelte";
  import { api } from "$lib/api";
  import { parseDiagnostics, errorText, type TypstDiagnostic } from "$lib/editor/diagnostics";
  import type { QuestionMeta, QuestionStats, Question, RubricItem } from "$lib/types";

  type QuestionType = Question["question_type"];

  type Props = {
    body: string;
    questionType: QuestionType;
    outcomeText: string;
    points: number;
    rubric: RubricItem[];
    sampleAnswer: string;
    stats?: QuestionStats | null;
    structureError: string | null;
    meta: QuestionMeta;
    subjectOptions: string[];
    bank: Question[];
    onmetachange: (next: QuestionMeta) => void;
    saving: boolean;
    saveLabel: string;
    onbodychange: (body: string) => void;
    onquestiontypechange: (value: QuestionType) => void;
    onoutcometextchange: (value: string) => void;
    onpointschange: (value: number) => void;
    onrubricchange: (next: RubricItem[]) => void;
    onsampleanswerchange: (next: string) => void;
    onsave: () => void;
    /** Cevap bölgesi. Kırmızı cetvelin sağı yalnızca ölçümündür, cevabın değil. */
    answer?: import("svelte").Snippet;
  };

  let {
    body,
    questionType,
    outcomeText,
    points,
    rubric,
    sampleAnswer,
    stats = null,
    structureError,
    meta,
    subjectOptions,
    bank,
    onmetachange,
    saving,
    saveLabel,
    onbodychange,
    onquestiontypechange,
    onoutcometextchange,
    onpointschange,
    onrubricchange,
    onsampleanswerchange,
    onsave,
    answer,
  }: Props = $props();

  /**
   * Derleme her tuş vuruşunda değil, yazma durunca çalışır.
   *
   * 120 ms, ölçüme dayanıyor: font kaydı ısındıktan sonra tek bir sorunun
   * derlenmesi 4-30 ms sürüyor (debug profilinde ölçüldü). Bu hızda daha uzun
   * bir bekleme yalnızca yapay gecikme olur.
   *
   * Sıfır değil, çünkü derlemeler sıraya alınsa da her tuş vuruşunda IPC turu
   * yapmanın anlamı yok; 120 ms yazma ritmindeki vuruşları toparlar.
   */
  const DEBOUNCE_MS = 120;

  let pages = $state<string[]>([]);
  let compileError = $state<string | null>(null);
  let diagnostics = $state<TypstDiagnostic[]>([]);
  let compiling = $state(false);

  /**
   * Derleme göstergesi yalnızca uzun süren derlemede görünür.
   *
   * Tipik derleme 5-30 ms. Her derlemede bir gösterge yakıp söndürmek, yazma
   * hızında saniyede birkaç kez yanıp sönen bir arayüz demek — bilgi vermiyor,
   * yalnızca göz yoruyor.
   */
  const SLOW_COMPILE_MS = 400;
  let slowCompile = $state(false);
  let slowTimer: ReturnType<typeof setTimeout> | null = null;

  let sourceRef = $state<ReturnType<typeof TypstSource> | null>(null);

  /**
   * Kaynak bölmesinde hangi metin düzenleniyor.
   *
   * KALICI DEĞİL, bilerek. `layout.svelte.ts` kalıcı YERLEŞİM tercihini
   * tutuyor; aktif sekme ise soruya özgü geçici durum. Kalıcılaştırılsaydı
   * çoktan seçmeli bir soru açıldığında "cevap" sekmesi anlamsız kalırdı.
   */
  let sourceTab = $state<"question" | "answer">("question");

  /** Cevap sekmesi yalnız açık uçlu soruda var: sample_answer orada. */
  let hasAnswerTab = $derived(questionType === "classic");

  /**
   * Sekme şeridindeki ipucu. Tek yerde tutuluyor çünkü aynı metin hem şeritte
   * görünüyor hem de `title` olarak veriliyor: dar bölmede metin `truncate` ile
   * tek satıra kırpıldığında tamamı fare üstündeyken hâlâ okunabilmeli.
   * İki ayrı ternary yazılsaydı biri değişip diğeri unutulabilirdi.
   */
  let sourceHint = $derived(
    sourceTab === "answer" ? "Yalnız cevap anahtarına basılır" : "Öğrenci kâğıdına basılır",
  );

  // Soru tipi klasikten çıkarsa cevap sekmesi kaybolur; üstünde kalmamalı.
  $effect(() => {
    if (!hasAnswerTab && sourceTab === "answer") sourceTab = "question";
  });

  /**
   * Aynı anda en fazla bir derleme. Üst üste binen derlemeler paralel
   * TayanWorld örneği demektir; her biri kendi belleğini tutar ve hızlı yazan
   * bir öğretmen süreci kolayca şişirir. Sıraya alınan yalnızca EN SON kaynak
   * tutulur — aradakiler zaten ekranda görünmeyecek.
   */
  /** Koşum sürerken gelen son iş. Dize değil KAPANIŞ: hangi varyantın
   * derleneceği bilgisi de taşınmalı. */
  let pendingJob: (() => Promise<string[]>) | null = null;

  let imageError = $state<string | null>(null);

  const MODES: Array<{ id: ViewMode; label: string; title: string }> = [
    { id: "editor", label: "Editör", title: "Yalnızca kaynak" },
    { id: "split", label: "Yan yana", title: "Kaynak ve kâğıt yan yana" },
    { id: "preview", label: "Kâğıt", title: "Yalnızca basılacak sayfa" },
  ];

  /**
   * Önizleme SEKMEYİ İZLER: Soru sekmesinde öğrenci nüshası, Cevap sekmesinde
   * cevap anahtarı. Böylece öğretmen anahtarın nasıl basılacağını — rubrik
   * tablosu dahil — kaydetmeden önce görüyor.
   *
   * Tek derleme yapılıyor: yalnız görünen varyant. İkisini birden derlemek
   * her tuş vuruşunda iki Typst koşusu demekti.
   */
  $effect(() => {
    // Bağımlılıklar açıkça okunuyor: cevap sekmesindeyken rubrik ya da örnek
    // cevap değişince de önizleme tazelenmeli.
    const current = body;
    const tab = sourceTab;
    const cevap = sampleAnswer;
    const olcutler = rubric;
    const puan = points;

    const timer = setTimeout(() => {
      if (tab === "answer") void compileAnswer(current, cevap, olcutler, puan);
      else void compile(current);
    }, DEBOUNCE_MS);
    return () => clearTimeout(timer);
  });

  /**
   * Tek koşum yolu; iki varyant da buradan geçer.
   *
   * KUYRUK ARTIK KAYNAK DİZESİNE DEĞİL İŞE BAKIYOR. Önceden bekleyen iş
   * `pendingSource: string` olarak tutuluyordu ve hangi komutun çağrılacağı
   * bilgisi yoktu; iki varyant gelince aynı dizeyle yanlış önizleme
   * derlenebilirdi. Bekleyen iş artık kapanışın kendisi.
   */
  async function run(is: () => Promise<string[]>) {
    if (compiling) {
      pendingJob = is;
      return;
    }

    compiling = true;
    slowTimer = setTimeout(() => (slowCompile = true), SLOW_COMPILE_MS);

    try {
      pages = await is();
      compileError = null;
      diagnostics = [];
    } catch (err: unknown) {
      // Önceki sayfalar ekranda kalır: hata anında kâğıdı boşaltmak,
      // öğretmenin neyi bozduğunu görmesini engeller.
      const message = errorText(err);
      compileError = message;
      diagnostics = parseDiagnostics(message);
    } finally {
      compiling = false;
      if (slowTimer !== null) {
        clearTimeout(slowTimer);
        slowTimer = null;
      }
      slowCompile = false;

      const queued = pendingJob;
      pendingJob = null;
      if (queued !== null) void run(queued);
    }
  }

  async function compileAnswer(
    source: string,
    cevap: string,
    olcutler: RubricItem[],
    puan: number,
  ) {
    await run(() =>
      api.compiler.previewAnswerKey(source, cevap.trim() === "" ? null : cevap, olcutler, puan),
    );
  }

  async function compile(source: string) {
    await run(() => api.compiler.previewQuestion(source));
  }

  /** Tek editör var; hangi belge açıksa oraya yazar. */
  function handleInsert(snippet: string) {
    sourceRef?.insert(snippet);
  }
</script>

{#snippet inspector()}
  <DockPanel title="Soru">
    <QuestionInspector
      {questionType}
      {outcomeText}
      {points}
      {rubric}
      {stats}
      {structureError}
      {meta}
      {subjectOptions}
      {bank}
      {onmetachange}
      {onquestiontypechange}
      {onoutcometextchange}
      {onpointschange}
      {onrubricchange}
    />
  </DockPanel>
{/snippet}

<div class="flex h-full min-h-0 flex-col">
  <!--
    Tek ince araç satırı. Öncesinde burada iki şerit vardı: form başlığı
    (tip + kazanım + Kaydet) ve künye (tip + puan + kazanım). İkisi de aynı
    gerçekleri iki kez basıyordu; ikisi de panele indi. Kalan yalnızca
    görünüm modu ve kaydetme — ikisi de bir bölmeye ait değil.
  -->
  <div
    class="flex shrink-0 items-center gap-2 border-b border-gray-200 bg-white px-3 py-2
           dark:border-gray-700 dark:bg-gray-800"
  >
    <!-- Görünüm modu: ThemeToggle ile aynı kalıp — kapalı gri iz, aktif beyaz/koyu segment. -->
    <div
      class="flex items-center gap-1 rounded-lg bg-gray-100 p-1 dark:bg-gray-700/60"
      role="group"
      aria-label="Görünüm modu"
    >
      {#each MODES as m (m.id)}
        <button
          type="button"
          class="rounded-md px-3 py-1 text-xs font-medium transition-colors"
          class:bg-white={layout.mode === m.id}
          class:shadow-sm={layout.mode === m.id}
          class:text-primary-700={layout.mode === m.id}
          class:dark:bg-gray-800={layout.mode === m.id}
          class:dark:text-primary-300={layout.mode === m.id}
          class:text-gray-500={layout.mode !== m.id}
          class:hover:text-gray-900={layout.mode !== m.id}
          class:dark:text-gray-400={layout.mode !== m.id}
          class:dark:hover:text-white={layout.mode !== m.id}
          aria-pressed={layout.mode === m.id}
          title={m.title}
          onclick={() => setMode(m.id)}
        >
          {m.label}
        </button>
      {/each}
    </div>

    <!--
      Görsel hatası satırı ÇIPLAK BIRAKILAMAZ.

      Flex öğesinin varsayılanı `min-width: auto`: min-content genişliğinin —
      yani en uzun kelimenin — altına inmeyi reddeder. Kısa mesajlarda bu taban
      metni kelimelerinden sarmalayıp araç satırını ~37 px'ten ~70 px'e
      çıkarıyor, altındaki editörle kâğıdı aşağı itiyordu. Mesaj boşluksuz uzun
      bir belirteç içerdiğinde (image.rs'ten gelen `e.to_string()` çıktısı bir
      dosya yolu ya da uzun dosya adı taşıyabiliyor) taban yüzlerce piksele
      çıkıyor ve satırın son öğesini — "Kaydet" düğmesini — pencerenin sağından
      dışarı itiyordu; app.css'teki html,body{overflow:hidden} yüzünden kaydırma
      çubuğu da olmadığı için kaydetme eylemi ULAŞILAMAZ hâle geliyordu.

      `min-w-0` kabı gerçekten küçülebilir yapıyor (onsuz `truncate` hiç
      devreye girmez), `flex-1` artan yeri hataya veriyor, `truncate` da metni
      tek satırda tutuyor. Metin KISALTILMIYOR: tamamı `title` ile erişilebilir.
    -->
    {#if imageError}
      <span
        class="min-w-0 flex-1 truncate text-xs text-red-600 dark:text-red-400"
        title={imageError}>{imageError}</span
      >
    {/if}

    <span
      class="ml-auto flex items-center gap-1.5 text-xs text-gray-500 dark:text-gray-400"
      class:invisible={!slowCompile}
    >
      <Spinner size="4" />
      derleniyor…
    </span>

    <!--
      Kaydet, satırdaki tek geri alınamaz eylem: yer daralınca en son o feda
      edilmeli. PenButton'ın içindeki Flowbite `Button` taban sınıfında
      `shrink-0` yok, yani flex öğesi olarak min-content'e ("Kaydet" /
      "Güncelle") kadar ezilebiliyor. Sarmalayıcı bu ezilmeyi kapatıyor;
      bileşenin kendisine sınıf geçirmek PenButton'ın API'sini değiştirmek
      olurdu ve o bileşeni başka altı rota daha çağırıyor.
    -->
    <div class="shrink-0">
      <PenButton kind="ink" disabled={saving || structureError !== null} onclick={onsave}>
        {saveLabel}
      </PenButton>
    </div>
  </div>

  <div class="flex min-h-0 flex-1">
    {#if layout.side === "left"}{@render inspector()}{/if}

    <!--
      İki bölmenin genişlik kısıtı: önizleme KÜÇÜLEBİLİR (`min-w-0`), editörün
      tabanı ise KOŞULLU (`min-w-[min(320px,60%)]`).

      `SheetPage` kâğıda açık piksel genişliği verir (794 × zoom): %159'da
      1263 px'lik gerçek bir öğe. Flex öğelerinin varsayılanı `min-width: auto`,
      yani içeriğinin min-content genişliğinin altına inmeyi REDDEDER; önizleme
      bölmesindeki `min-w-0` o tabanı kaldırıyor, taşmayı SheetPreview'ın kendi
      `overflow-auto` kabı üstleniyor.

      EDİTÖRÜN TABANI ÖNCEDEN SABİT 320 px'Tİ VE ÖNİZLEMEYİ ULAŞILAMAZ
      YAPIYORDU. Ölçüm (tauri.conf.json minWidth = 1024; kenarlıklar
      `box-sizing: border-box` ile genişliğin İÇİNDE, ayrıca sayılmaz):
      1024 − 224 (w-56 çekmece) = 800 px içerik. Bu satıra kalan =
      800 − panel genişliği. Panel `MAX_PANEL_WIDTH` = 480'e çekildiğinde
      (sürükleyerek ya da tutamaçta ArrowRight ile ulaşılıyor) satır 320 px:
      sabit taban editörü tam 320'ye kilitliyor, `flex-[1.15] basis-0 min-w-0`
      önizleme bölmesine 0 px kalıyordu. PreviewZoom'un HER öğesi `shrink-0`
      olduğundan düğmeler 0 px'lik bölmenin dışına boyanıyor, app.css'teki
      html,body{overflow:hidden} da kaydırıcı bırakmıyordu: "Sığdır" / "100%" /
      zoom düğmeleri ve kâğıdın kendisi erişilemez oluyordu. Kırılma yalnız
      480'de de değil — önizlemeye kalan 480 − panel genişliği px, yani panel
      440'ı geçtiği anda bölme 40 px'in altına iniyor.

      `min(320px, 60%)` tabanı KORUYOR ama küçülebilir yapıyor: 320 px yalnız
      satırın %60'ını aşmadığı sürece, yani satır ≥ 533 px iken geçerli.
      Varsayılan 260 px panelde (1024 px pencere → satır 540 px) editör yine
      tam 320 px alıyor, önizlemeye 220 px kalıyor — SheetPreview'ın `fit()`
      için verdiği sayının aynısı, yani alışılmış yerleşim değişmiyor. Panel
      genişledikçe taban orantılı geriliyor ve önizlemeye her koşulda satırın
      %40'ı kalıyor (en dar hâlde 320 × 0,4 = 128 px). Yüzde, kapsayıcının
      içerik genişliğine göre çözülür; kap belirsiz sayılıp yüzde düşse bile editör grow oranından
      payını alır (1 / 2,15 ≈ %46,5) — iki bölmeden hiçbiri sıfıra inmez.

      Bedeli kabul edildi: dar satırda FloatingPalette'in 300 px'lik kutusu
      (`right-4`) editör bölmesinden taşıp panelin üstüne biniyor. `absolute` +
      `z-10` olduğu için okunur ve tıklanabilir kalıyor; 0 px'lik ULAŞILAMAZ
      bir önizlemeye yeğlendi.
    -->
    <div class="flex min-h-0 min-w-0 flex-1">
      {#if layout.mode !== "preview"}
        <!--
          Palet kaynağın ÜZERİNDE yüzer; `relative` bu yüzden burada. Kâğıdın
          üstüne taşmaz — basılacak sayfayı hiçbir şey örtmez.
        -->
        <section
          class="relative flex min-h-0 min-w-[min(320px,60%)] flex-1 basis-0 flex-col"
          class:border-r={layout.mode === "split"}
          class:border-gray-200={layout.mode === "split"}
          class:dark:border-gray-700={layout.mode === "split"}
        >
          <!--
            Sekmeler YALNIZ açık uçlu soruda. Örnek cevap alanı yalnız orada
            var; şıklı soruda doğru cevap zaten şık verisinde ve boş bir
            "Cevap" sekmesi öğretmeni yanıltırdı.
          -->
          {#if hasAnswerTab}
            <div
              class="flex shrink-0 gap-0 border-b border-gray-200 px-3 dark:border-gray-700"
            >
              {#each [{ id: "question" as const, ad: "Soru" }, { id: "answer" as const, ad: "Cevap" }] as sekme (sekme.id)}
                <button
                  type="button"
                  class="shrink-0 border-b-2 px-3 py-1.5 text-sm font-medium transition-colors"
                  class:border-primary-600={sourceTab === sekme.id}
                  class:text-primary-700={sourceTab === sekme.id}
                  class:dark:border-primary-400={sourceTab === sekme.id}
                  class:dark:text-primary-400={sourceTab === sekme.id}
                  class:border-transparent={sourceTab !== sekme.id}
                  class:text-gray-500={sourceTab !== sekme.id}
                  class:hover:text-gray-700={sourceTab !== sekme.id}
                  class:dark:text-gray-400={sourceTab !== sekme.id}
                  class:dark:hover:text-gray-200={sourceTab !== sekme.id}
                  onclick={() => (sourceTab = sekme.id)}
                >
                  {sekme.ad}
                </button>
              {/each}
              <!--
                İpucu, şeridin ezilebilen tek öğesi olmalı.

                Editör bölmesi 320 px tabanına oturduğunda — 1024 px pencere +
                varsayılan 260 px panel + yan yana mod; taban artık
                `min(320px,60%)` ve bu satır genişliğinde (540 px) hâlâ 320'de
                kalıyor — bu span'a ~183 px kalıyor, en uzun Türkçe metnin
                istediği ise ~180 px: sınırda. Panel daha da genişletilince
                taban yüzdeye devrolup bölme daralıyor, yani buradaki pay
                büsbütün eriyor.
                Sistem yazı tipi bir tık geniş çizince ya da kullanıcı yazı
                boyutunu büyütünce span min-content'ine ("anahtarına" ≈ 55 px)
                doğru ezilirken metni sarmalıyor, şerit iki kata çıkıyor ve
                altındaki editör aşağı kayıyordu.

                `min-w-0` + `shrink` küçülmeyi serbest bırakıyor, `truncate` da
                sarmalama yerine tek satırda üç nokta koyuyor — `min-w-0`
                olmadan `truncate` hiç çalışmazdı, çünkü öğe küçülemediği için
                taşma da oluşmuyor. `self-center` şeridin yüksekliğini
                içeriğine göre büyütmeden hizalıyor. Metin kısaltılmıyor:
                tamamı `title` ile erişilebilir kalıyor.
              -->
              <span
                class="ml-auto min-w-0 shrink self-center truncate ps-2 text-xs text-gray-400
                       dark:text-gray-500"
                title={sourceHint}
              >
                {sourceHint}
              </span>
            </div>
          {/if}

          <!--
            TEK EDİTÖR, İKİ BELGE.
            Sekme başına ayrı bir TypstSource monte etmek iki imleç çiziyordu
            ve gizlenen CodeMirror ölçüm yapamadığı için göründüğünde bozuk
            çizebiliyordu. Artık tek görünüm var; `docId` değişince editör
            durumu değiştiriyor. Geri alma geçmişi durumun içinde yaşadığı
            için sekmeler arası geçişte kaybolmuyor.
          -->
          <div class="relative min-h-0 flex-1">
            <TypstSource
              bind:this={sourceRef}
              docId={sourceTab}
              value={sourceTab === "answer" ? sampleAnswer : body}
              diagnostics={sourceTab === "answer" ? [] : diagnostics}
              onchange={(v, doc) => (doc === "answer" ? onsampleanswerchange(v) : onbodychange(v))}
              onimageerror={(m) => (imageError = m)}
            />
          </div>

          <FloatingPalette {questionType} oninsert={handleInsert} />
        </section>
      {/if}

      {#if layout.mode !== "editor"}
        <!--
          `min-w-0`: kâğıt ne kadar büyürse büyüsün bu bölme küçülebilmeli.
          İçindeki kaydırıcı taşmayı zaten üstleniyor; bu sınıf olmadan kâğıdın
          piksel genişliği yerleşimi dışarı iter ve editörü ekrandan siler.
        -->
        <section class="min-h-0 min-w-0 flex-[1.15] basis-0">
          <SheetPreview {pages} stale={slowCompile} error={compileError} />
        </section>
      {/if}
    </div>

    {#if layout.side === "right"}{@render inspector()}{/if}
  </div>

  {#if answer}
    <div
      class="max-h-[240px] shrink-0 overflow-auto border-t border-gray-200 bg-white px-4 py-3
             dark:border-gray-700 dark:bg-gray-800"
    >
      {@render answer()}
    </div>
  {/if}
</div>

<script lang="ts">
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
    class="ruled-bottom paper-plain flex shrink-0 items-center gap-half bg-paper-lift
           px-half py-quarter"
  >
    <div class="flex items-stretch border border-rule-strong">
      {#each MODES as m (m.id)}
        <button
          type="button"
          class="border-r border-rule px-half py-quarter text-[12px] leading-rule
                 transition-colors last:border-r-0 hover:text-red-deep"
          class:bg-paper-sunk={layout.mode === m.id}
          class:font-semibold={layout.mode === m.id}
          class:text-ink={layout.mode === m.id}
          class:text-pencil={layout.mode !== m.id}
          aria-pressed={layout.mode === m.id}
          title={m.title}
          onclick={() => setMode(m.id)}
        >
          {m.label}
        </button>
      {/each}
    </div>

    {#if imageError}
      <span class="annot">{imageError}</span>
    {/if}

    <span class="annot ml-auto" class:invisible={!slowCompile}>derleniyor…</span>

    <PenButton kind="ink" disabled={saving || structureError !== null} onclick={onsave}>
      {saveLabel}
    </PenButton>
  </div>

  <div class="flex min-h-0 flex-1">
    {#if layout.side === "left"}{@render inspector()}{/if}

    <!--
      `min-w-0` ve `min-w-[320px]` SÜS DEĞİL, yerleşimin taşıyıcı kısıtı.

      `SheetPage` kâğıda açık piksel genişliği verir (794 × zoom): %159'da
      1263 px'lik gerçek bir öğe. Flex öğelerinin varsayılanı `min-width: auto`,
      yani içeriğinin min-content genişliğinin altına inmeyi REDDEDER — önizleme
      1263 px talep eder ve editörü sıfıra ezer.

      Eski grid bunu `minmax(320px,1fr)_minmax(360px,1.15fr)` ile karşılıyordu.
      Flex'e geçerken o taban düştü; buradaki iki sınıf onu geri koyuyor:
      editörün tabanı var, önizleme küçülüp KENDİ içinde kaydırıyor.
    -->
    <div class="flex min-h-0 min-w-0 flex-1">
      {#if layout.mode !== "preview"}
        <!--
          Palet kaynağın ÜZERİNDE yüzer; `relative` bu yüzden burada. Kâğıdın
          üstüne taşmaz — basılacak sayfayı hiçbir şey örtmez.
        -->
        <section
          class="relative flex min-h-0 min-w-[320px] flex-1 basis-0 flex-col"
          class:border-r={layout.mode === "split"}
          class:border-rule-strong={layout.mode === "split"}
        >
          <!--
            Sekmeler YALNIZ açık uçlu soruda. Örnek cevap alanı yalnız orada
            var; şıklı soruda doğru cevap zaten şık verisinde ve boş bir
            "Cevap" sekmesi öğretmeni yanıltırdı.
          -->
          {#if hasAnswerTab}
            <div class="ruled-bottom flex shrink-0 gap-0 px-rule">
              {#each [{ id: "question" as const, ad: "Soru" }, { id: "answer" as const, ad: "Cevap" }] as sekme (sekme.id)}
                <button
                  type="button"
                  class="stamp border-b-2 px-half py-quarter leading-rule transition-colors"
                  class:border-red={sourceTab === sekme.id}
                  class:text-red-deep={sourceTab === sekme.id}
                  class:border-transparent={sourceTab !== sekme.id}
                  class:text-ink-mid={sourceTab !== sekme.id}
                  onclick={() => (sourceTab = sekme.id)}
                >
                  {sekme.ad}
                </button>
              {/each}
              <span class="pencil ml-auto self-center">
                {sourceTab === "answer"
                  ? "Yalnız cevap anahtarına basılır"
                  : "Öğrenci kâğıdına basılır"}
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
    <div class="ruled-top paper-plain max-h-[240px] shrink-0 overflow-auto bg-paper px-rule py-half">
      {@render answer()}
    </div>
  {/if}
</div>

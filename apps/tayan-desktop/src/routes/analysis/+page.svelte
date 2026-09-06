<script lang="ts">
  import { onMount } from "svelte";
  import { Alert, Button, ButtonGroup, Label } from "flowbite-svelte";
  import PageShell from "$lib/components/shell/PageShell.svelte";
  import DropdownSelect from "$lib/components/shell/DropdownSelect.svelte";
  import ScoreDistribution from "$lib/components/measure/ScoreDistribution.svelte";
  import ItemAnalysis from "$lib/components/measure/ItemAnalysis.svelte";
  import ItemMap from "$lib/components/measure/ItemMap.svelte";
  import OutcomeChart from "$lib/components/measure/OutcomeChart.svelte";
  import AnswerGrid from "$lib/components/measure/AnswerGrid.svelte";
  import ResultEntry from "$lib/components/measure/ResultEntry.svelte";
  import { api } from "$lib/api";
  import { errorText } from "$lib/editor/diagnostics";
  import { bodyPreview } from "$lib/types";
  import { itemStats, spread } from "$lib/analysis/item-stats";
  import { clearSelection, selection } from "$lib/ui/analysis-selection.svelte";
  import { buildReport } from "$lib/analysis/report";
  import { examFileName } from "$lib/exam/filename";
  import { save } from "@tauri-apps/plugin-dialog";
  import type { Classroom, Exam, ExamResult, Question, Student } from "$lib/types";

  let exams = $state<Exam[]>([]);
  let classrooms = $state<Classroom[]>([]);
  let students = $state<Student[]>([]);
  let results = $state<ExamResult[]>([]);
  /** Banka; sınavın soru atıflarını çözmek için gerekli. */
  let bank = $state<Question[]>([]);

  type AnalysisTab = "giris" | "analiz";
  let activeTab = $state<AnalysisTab>("giris");

  let examId = $state<string>("");
  let classroomId = $state<string>("");

  let loading = $state(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      [exams, classrooms, bank] = await Promise.all([
        api.exams.list(),
        api.students.listClassrooms(),
        api.questions.list(),
      ]);
      const published = exams.filter((e) => e.status !== "Draft");
      if (published.length > 0) examId = published[0].id;
      if (classrooms.length > 0) classroomId = classrooms[0].id;
      error = null;
    } catch (err: unknown) {
      error = errorText(err);
    } finally {
      loading = false;
    }
  });

  $effect(() => {
    if (examId) void loadResults(examId);
  });

  $effect(() => {
    if (classroomId) void loadStudents(classroomId);
  });

  async function loadResults(id: string) {
    try {
      results = await api.results.getByExam(id);
      error = null;
    } catch (err: unknown) {
      error = errorText(err);
    }
  }

  async function loadStudents(id: string) {
    try {
      students = await api.students.listByClassroom(id);
      error = null;
    } catch (err: unknown) {
      error = errorText(err);
    }
  }

  /*
    SINAV YA DA SINIF DEĞİŞİNCE SEÇİM TEMİZLENİYOR. Seçim soru ve öğrenci
    KİMLİĞİ tutuyor; başka bir sınava geçildiğinde o kimlikler artık hiçbir
    şeye karşılık gelmiyor ve ekran sebepsiz yere yarı soluk açılırdı.
  */
  $effect(() => {
    void examId;
    void classroomId;
    clearSelection();
  });

  /*
    BALON ETİKETLERİ. Grafikler soru/öğrenci KİMLİĞİ taşıyor; adı çözmek için
    bankaya ve öğrenci listesine erişim gerekiyor. Bileşenlere bütün listeleri
    prop olarak vermek yerine tek bir çözücü işlev veriliyor — grafik neyin
    adını gösterdiğini bilmek zorunda değil.
  */
  function soruAdi(questionId: string): string {
    const q = bank.find((b) => b.id === questionId);
    if (!q) return "bankada yok";
    const baslik = q.meta.title.trim();
    return baslik !== "" ? baslik : bodyPreview(q.body, 40);
  }

  function ogrenciAdi(studentId: string): string {
    const s = students.find((x) => x.id === studentId);
    return s ? `${s.number} ${s.first_name} ${s.last_name}` : "öğrenci";
  }

  let selectedExam = $derived(exams.find((e) => e.id === examId) ?? null);

  let questionIds = $derived(
    selectedExam
      ? selectedExam.questions
          .slice()
          .sort((a, b) => a.display_order - b.display_order)
          .map((q) => q.question_id)
      : [],
  );

  let classResults = $derived(
    results.filter((r) => students.some((s) => s.id === r.student_id)),
  );

  /*
    PUANLAR VE KİMLİKLER AYNI SIRADAN TÜRETİLİYOR. Dağılımdaki fırça, seçilen
    aralığın hangi öğrencilere karşılık geldiğini indeks eşleşmesiyle buluyor;
    iki liste ayrı ayrı süzülseydi sıralar kayar ve fırça YANLIŞ öğrencileri
    seçerdi — sessizce, hiçbir hata vermeden.
  */
  let scored = $derived(classResults.filter((r) => r.total_points_max > 0));

  let percentages = $derived(
    scored.map((r) => (r.total_points_earned / r.total_points_max) * 100),
  );

  let scoredStudentIds = $derived(scored.map((r) => r.student_id));

  /** Geçme eşiği. Şimdilik sabit; sınav ayarına bağlanana kadar tek yerde. */
  const PASSING_THRESHOLD = 50;

  let isSavingReport = $state(false);
  let reportStatus = $state<string | null>(null);

  /**
   * Analiz raporunu PDF olarak kaydeder.
   *
   * ÖLÇÜLER EKRANDAN GİDER. Rust ikinci bir hesap yapmıyor; öğretmenin veliye
   * gösterdiği kâğıt ile ekranda gördüğü aynı sayıları taşımak zorunda.
   */
  async function saveReport() {
    if (!selectedExam) return;

    const report = buildReport({
      exam: selectedExam,
      items: items,
      bank,
      results: classResults,
      students,
      threshold: PASSING_THRESHOLD,
    });
    if (report === null) {
      reportStatus = "Sonuç girilmemiş; rapor alınamaz.";
      return;
    }

    const targetPath = await save({
      defaultPath: examFileName(selectedExam, {
        answerKey: false,
        booklet: null,
        extension: "pdf",
        suffix: "analiz",
      }),
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });
    if (!targetPath) return; // vazgeçildi

    isSavingReport = true;
    reportStatus = null;
    try {
      const savedPath = await api.compiler.exportAnalysisPdf(report, targetPath);
      reportStatus = `Rapor kaydedildi: ${savedPath}`;
    } catch (err: unknown) {
      reportStatus = errorText(err);
    } finally {
      isSavingReport = false;
    }
  }

  /** Yayılım ölçüleri: ortalama, ortanca, çeyrekler. */
  let distribution = $derived(spread(percentages));

  /** Soru soru madde analizi — bu sınavın kendi sonuçlarından. */
  let items = $derived(itemStats(classResults, questionIds, bank));

  let summary = $derived.by(() => {
    if (percentages.length === 0) return null;
    const sorted = [...percentages].sort((a, b) => a - b);
    return {
      count: sorted.length,
      mean: sorted.reduce((sum, p) => sum + p, 0) / sorted.length,
      min: sorted[0],
      max: sorted[sorted.length - 1],
      median: sorted[Math.floor(sorted.length / 2)],
      failing: sorted.filter((p) => p < 50).length,
    };
  });
</script>

<PageShell title="Sınav analizi" subtitle={selectedExam?.meta.title ?? null} scroll={false}>
  {#snippet actions()}
    <!--
      PDF düğmesi başlığa taşındı (PageShell'in ortak eylem yuvası). Görünürlük
      eski davranışla aynı: yalnız analiz sekmesinde VE sınıfta sonuç varken.
    -->
    {#if activeTab === "analiz" && classResults.length > 0}
      <Button size="sm" disabled={isSavingReport} onclick={saveReport}>
        {isSavingReport ? "Yazılıyor…" : "Analiz PDF"}
      </Button>
    {/if}
  {/snippet}

  <div class="flex h-full min-h-0 flex-col">
    <!--
      ARAÇ ÇUBUĞU FLOWBITE'A GEÇTİ. Eskiden burada üç ayrı geometri yan yana
      duruyordu: `SelectBox` 42px yüksek ve `rounded-lg` (bu projede 16px),
      elle yazılmış sekme grubu 32px ve KÖŞESİZ, "Kaydet" ise Flowbite
      `Button` olduğu için yine 16px. `items-end` yalnız altlarını hizaladığı
      için üst kenarlar tırtıklı çıkıyordu. Artık üçü de aynı bileşen
      ailesinden ve aynı yükseklikte.

      Renkler anlamsal değişkenlerden: `border-default-medium`,
      `bg-neutral-primary-medium`, `text-body-subtle`, `text-heading` kendi
      koyu kip karşılıklarını taşıyor (flowbite/src/themes/default.css bunları
      `.dark` altında yeniden tanımlıyor), o yüzden tek bir `dark:` yok.
    -->
    <div
      class="flex shrink-0 flex-wrap items-end gap-5 border-b border-default-medium
             bg-neutral-primary-medium px-5 py-2.5"
    >
      <div>
        <Label id="exam-filter-label" for="exam-filter" class="mb-1.5">Sınav</Label>
        <DropdownSelect
          id="exam-filter"
          labelId="exam-filter-label"
          value={examId}
          options={exams.map((e) => ({ name: e.meta.title, value: e.id }))}
          placeholder="— seç —"
          width="w-64"
          onchange={(v) => (examId = v)}
        />
      </div>

      <div>
        <Label id="class-filter-label" for="class-filter" class="mb-1.5">Sınıf</Label>
        <DropdownSelect
          id="class-filter"
          labelId="class-filter-label"
          value={classroomId}
          options={classrooms.map((c) => ({ name: c.name, value: c.id }))}
          placeholder="— seç —"
          width="w-40"
          onchange={(v) => (classroomId = v)}
        />
      </div>

      <!--
        `ButtonGroup` + `Button`: bitişik köşeler (`first:rounded-s` /
        `last:rounded-e`) ve tek kenarlık bileşenden geliyor, elle yazılmış
        `border-r … last:border-r-0` numarasına gerek kalmıyor.

        `py-2.5` AÇIKÇA VERİLİYOR. `ButtonGroup` içindeki her `Button`
        boyutunu `sm`e ZORLUYOR (Button.svelte: `actualSize = group ? "sm" :
        size`), o da `py-2` → 38px; yanındaki `DropdownSelect` ise 42px
        (`py-2.5` + `text-sm`). 4px'lik fark aynı satırda göze çarpıyordu.

        Hover rengi kırmızı DEĞİL: kırmızı bu üründe yalnız bir değerlendirme
        sinyali (yanlış cevap, eşiğin altı, hata) ve aynı ekranda gerçek
        kırmızı — "Eşiğin altında" sayısı — duruyor.

        `aria-pressed` korunuyor: bu iki düğme bir görünüm anahtarı.
        Flowbite'ın `ButtonToggleGroup`u burada kullanılamaz — `value`
        prop'unu bir kez okuyup kendi `$state`ine tohumluyor ve bir daha
        okumuyor, yani dışarıdan gelen değişimle sessizce ayrışır.
      -->
      <ButtonGroup>
        {#each [["giris", "Sonuç girişi"], ["analiz", "Analiz"]] as [id, label] (id)}
          <Button
            color={activeTab === id ? "primary" : "alternative"}
            class="py-2.5"
            aria-pressed={activeTab === id}
            onclick={() => (activeTab = id as AnalysisTab)}
          >
            {label}
          </Button>
        {/each}
      </ButtonGroup>

      {#if summary && activeTab === "analiz"}
        <dl class="ml-auto flex items-baseline gap-5">
          <div>
            <dt class="text-xs font-semibold uppercase tracking-wider text-body-subtle">
              Ortalama
            </dt>
            <dd class="tnum text-lg font-bold leading-tight text-heading">
              {summary.mean.toFixed(0)}%
            </dd>
          </div>
          <div>
            <dt class="text-xs font-semibold uppercase tracking-wider text-body-subtle">
              Ortanca
            </dt>
            <dd class="tnum text-lg font-bold leading-tight text-heading">
              {summary.median.toFixed(0)}%
            </dd>
          </div>
          <div>
            <dt class="text-xs font-semibold uppercase tracking-wider text-body-subtle">
              En düşük / yüksek
            </dt>
            <dd class="tnum text-lg font-bold leading-tight text-heading">
              {summary.min.toFixed(0)} / {summary.max.toFixed(0)}
            </dd>
          </div>
          <div>
            <dt class="text-xs font-semibold uppercase tracking-wider text-body-subtle">
              Eşiğin altında
            </dt>
            <!-- Eşiğin altındaki sayı gerçek bir değerlendirme bulgusu: kırmızı burada doğru yerinde. -->
            <dd class="tnum text-lg font-bold leading-tight text-red-600 dark:text-red-400">
              {summary.failing} / {summary.count}
            </dd>
          </div>
        </dl>
      {/if}
    </div>

    {#if error}
      <Alert color="red" rounded={false} class="shrink-0 border-b border-default-medium text-xs">
        {error}
      </Alert>
    {/if}

    <div class="flex min-h-0 flex-1 flex-col">
      {#if loading}
        <p class="px-5 py-5 text-xs text-body-subtle">Okunuyor…</p>
      {:else if !examId || !classroomId}
        <p class="px-5 py-5 text-xs text-body-subtle">Bir sınav ve bir sınıf seç.</p>
      {:else if selectedExam === null}
        <p class="px-5 py-5 text-xs text-body-subtle">Sınav bulunamadı.</p>
      {:else if activeTab === "giris"}
        <ResultEntry
          exam={selectedExam}
          {students}
          {results}
          {bank}
          onsaved={() => {
            // Ölçüm her kayıtta yeniden hesaplanıyor; listeyi de tazele ki
            // öğrencinin yanındaki işaret hemen görünsün.
            void loadResults(examId);
          }}
        />
      {:else if classResults.length === 0}
        <p class="px-5 py-5 text-xs text-body-subtle">
          Bu sınav için bu sınıfta girilmiş sonuç yok. Sonuç girişi sekmesinden başla.
        </p>
      {:else}
        <!--
          SEÇİM ŞERİDİ. Fırça ya da kazanım tıklaması bütün kartları
          etkiliyor; etkinin NEREDEN geldiği ve nasıl geri alınacağı tek bir
          yerde yazmalı, yoksa öğretmen "ekranın yarısı neden soluk"
          sorusuyla baş başa kalır.
        -->
        {#if selection.isActive}
          <div
            class="flex shrink-0 flex-wrap items-center gap-2.5 border-b border-default-medium
                   bg-primary-50 px-5 py-1 text-xs dark:bg-primary-900/30"
          >
            <span class="text-body">
              {#if selection.questionIds.length > 0}
                {selection.questionIds.length} soru
              {/if}
              {#if selection.questionIds.length > 0 && selection.studentIds.length > 0}·{/if}
              {#if selection.studentIds.length > 0}
                {selection.studentIds.length} öğrenci
              {/if}
              seçili{#if selection.source}
                — {selection.source}{/if}
            </span>
            <Button size="xs" color="alternative" onclick={clearSelection}>Seçimi temizle</Button>
          </div>
        {:else}
          <p class="border-b border-default-medium px-5 py-1 text-xs text-body-subtle">
            {#if reportStatus}{reportStatus}{:else}Grafiklerde bölge seçebilir, kazanıma tıklayabilirsin — diğer kartlar birlikte vurgulanır.{/if}
          </p>
        {/if}

        <!--
          KAYDIRMA BURADA. Belge kaydırması ana menüyü de yukarı taşıyordu;
          pencere sabit kalmalı, yalnız bu bölge kaymalı.
        -->
        <div class="min-h-0 flex-1 overflow-auto">
          <!--
            TEK IZGARA, ÜÇ AYRI KAP DEĞİL.

            Eskiden panel üç bağımsız blok hâlindeydi: (dağılım + cevap
            ızgarası) bir grid, madde haritası ayrı bir div, soru soru tablosu
            ayrı bir div. Kartlar birbirini görmediği için biri bittiğinde
            altında ölü boşluk kalıyor, sonraki kart tek başına geniş bir
            satıra oturuyordu. Hepsi TEK ızgarada olunca kartlar birlikte
            akıyor ve boşluk kalmıyor.

            `repeat(auto-fit, minmax(20rem, 40rem))` — kırılma noktası YOK.
            `xl:` gibi bir eşik, pencere genişliğini bilir ama KARTA KALAN yeri
            bilmez; çekmece açılınca ya da pencere yandan bölününce eşik yanlış
            tarafta kalıyordu. `auto-fit` doğrudan kalan yere bakıp sütun
            sayısını kendisi seçiyor: dar pencerede 1, tipik pencerede 2, geniş
            ekranda 3.

            ÜST SINIR 40rem KASITLI. Sınırsız `1fr` verseydik geniş ekranda
            kart 800px'e çıkar, içindeki SVG ise viewBox oranıyla birlikte
            BÜTÜN iç ölçüleriyle (8px eksen yazısı dâhil) ölçeklenirdi —
            grafik büyümez, ZOOMLANIR. 40rem tavanı en fazla ~1.9 kat büyümeye
            izin veriyor; artan yer `justify-center` ile iki yana eşit
            dağılıyor, sağda tek taraflı bir boşluk bırakmıyor.

            `items-stretch` (varsayılan) + sarmalayıcıda `h-full`: aynı satırda
            iki kart eşit yükseklikte kalıyor. Eskiden kısa kart yukarıda asılı
            duruyordu.
          -->
          <div
            class="grid items-stretch justify-center gap-5 px-5 py-2.5
                   [grid-template-columns:repeat(auto-fit,minmax(20rem,40rem))]"
          >
            <!--
              min-w-0 ŞART: ızgara öğesinin varsayılan `min-width:auto`su onu
              içeriğinin min-content'inin altına indirmez. SVG `w-full` +
              viewBox olduğu için kap küçülebilince grafik de sorunsuz küçülür.
            -->
            <div class="h-full min-w-0">
              <ScoreDistribution
                {percentages}
                studentIds={scoredStudentIds}
                studentLabel={ogrenciAdi}
                stats={distribution}
                threshold={PASSING_THRESHOLD}
              />
            </div>

            <!--
              MADDE HARİTASI DAĞILIMIN YANINDA. İkisi de bir sınav sonrası ilk
              bakılan şeyler ama farklı soruya cevap veriyor: dağılım "sınıf
              nerede", harita "hangi soru sorunlu". Yan yana durmaları
              öğretmenin ikisini karşılaştırmasını sağlıyor — düşük ortalama
              zayıf sınıftan mı, bozuk sorudan mı geliyor?
            -->
            <div class="h-full min-w-0">
              <ItemMap items={items} studentCount={classResults.length} label={soruAdi} />
            </div>

            <!--
              KAZANIM GRAFİĞİ ÜÇÜNCÜ KART. Dağılım "sınıf nerede", harita
              "hangi soru bozuk", kazanım "hangi KONU tutmadı" diyor. Üçü
              birlikte bir sınav sonrası kararın tamamını veriyor: not ver,
              soruyu düzelt, konuyu tekrar işle.
            -->
            <div class="h-full min-w-0">
              <OutcomeChart
                {items}
                {bank}
                studentCount={classResults.length}
                threshold={PASSING_THRESHOLD}
              />
            </div>

            <!--
              CEVAP IZGARASI TÜM SATIRI ALIYOR. Genişliği soru sayısına bağlı
              (yaklaşık 255 + 20 × soru); 20 soruda ~655px ve dar bir izde kendi
              yatay kaydırıcısına düşerdi. Öğrenci × soru ızgarasını yatay
              kaydırarak okumak, tablonun bütün faydasını götürür.

              min-w-0 burada da: olmadan tablonun min-content genişliği izin
              TABAN boyutuna geçip ızgarayı dışarı iterdi. Böyle taşma
              AnswerGrid'in KENDİ overflow-auto kabına düşüyor, sayfayı yatay
              kaydırmıyor.
            -->
            <div class="h-full min-w-0 [grid-column:1/-1]">
              <AnswerGrid results={classResults} {students} {questionIds} />
            </div>

            <!--
              SORU SORU TABLOSU DA TAM GENİŞLİK: satır başına soru metni +
              yığılmış çubuk + iki sayı sütunu var, dar izde metin kırpılmaktan
              başka bir şey yapamaz. Haritanın ALTINDA duruyor çünkü sıra
              böyle: harita "hangi sorular sorunlu" (öbek, tek bakış), tablo
              "bu soru tam olarak ne yaptı" (satır satır, kesin sayı).
            -->
            <div class="h-full min-w-0 [grid-column:1/-1]">
              <ItemAnalysis items={items} {bank} studentCount={classResults.length} />
            </div>
          </div>
        </div>
      {/if}
    </div>
  </div>
</PageShell>

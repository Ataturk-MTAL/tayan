<script lang="ts">
  /**
   * Sınav sonucu girişi — öğrenci öğrenci.
   *
   * Izgara (öğrenci × soru) daha hızlı görünür ama fiziksel işe uymaz:
   * öğretmenin elinde 30 kâğıtlık bir deste var, birini alıp cevaplarını girip
   * sonrakine geçiyor. Öğrenci başına form bu akışın birebir karşılığı; ızgarada
   * gözün satır kaydırması yanlış öğrenciye yazma riski üretir.
   *
   * CEVAP KODLAMASI doğrudan ScoringService::score_answer'a gider ve birebir
   * uymak zorundadır — yanlış kodlama sessizce yanlış not demektir:
   *   Çoktan seçmeli : şık kimliği, "A" / "B" / ...
   *   Doğru-yanlış   : "true" / "false"  (Rust tarafı parse::<bool>() yapar)
   *   Boşluk doldurma: JSON eşlemesi, {"b1": "180", "b2": "..."}
   *   Klasik         : cevap yok; puan ELLE girilir ve olduğu gibi kaydedilir
   */
  import PenButton from "../shell/PenButton.svelte";
  import { Alert, Badge, Card, Checkbox, Input, Table, TableBody, TableBodyCell, TableBodyRow, TableHead, TableHeadCell } from "flowbite-svelte";
  import { api } from "$lib/api";
  import { errorText } from "$lib/editor/diagnostics";
  import { bodyPreview } from "$lib/types";
  import type {
    Exam,
    ExamResult,
    Question,
    QuestionAnswerInput,
    RubricItem,
    Student,
  } from "$lib/types";

  type Props = {
    exam: Exam;
    students: Student[];
    results: ExamResult[];
    /** Bankadaki tüm sorular; sınavın atıfları buradan çözülür. */
    bank: Question[];
    onsaved: () => void;
  };

  let { exam, students, results, bank, onsaved }: Props = $props();

  let studentId = $state("");
  let saving = $state(false);
  let saveError = $state<string | null>(null);
  let saved = $state<string | null>(null);

  /** Girilen cevaplar: anahtar → ham metin. Kodlama kaydederken yapılır. */
  let answers = $state<Record<string, string>>({});
  /** Klasik sorular için elle girilen puan. */
  let manualPoints = $state<Record<string, number>>({});

  /**
   * Klasik soruda karşılanan rubrik ölçütleri: soru kimliği → ölçüt sıraları.
   *
   * Ölçüt işaretlendikçe puan kendiliğinden toplanır. Puanın KAYNAĞI yine
   * `manualPoints`; buradan hesaplanıp oraya yazılır. Böylece rubriksiz soru,
   * rubrikli soru ve öğretmenin elle düzelttiği puan tek yoldan kaydedilir.
   */
  let rubricMet = $state<Record<string, number[]>>({});

  function rubrikOf(q: Question): RubricItem[] {
    return q.question_type === "classic" ? q.rubric : [];
  }

  /** Ölçütü işaretle/kaldır ve puanı yeniden topla. */
  function toggleCriterion(q: Question, index: number) {
    const mevcut = rubricMet[q.id] ?? [];
    const sonraki = mevcut.includes(index)
      ? mevcut.filter((i) => i !== index)
      : [...mevcut, index].sort((a, b) => a - b);

    rubricMet = { ...rubricMet, [q.id]: sonraki };

    const rubrik = rubrikOf(q);
    const toplam = sonraki.reduce((sum, i) => sum + (rubrik[i]?.points ?? 0), 0);
    manualPoints = { ...manualPoints, [q.id]: toplam };
  }

  /**
   * Sınavın soruları, display_order sırasında ve bankada BULUNANLAR.
   *
   * Bankada olmayan atıf sessizce atlanmıyor: öğretmen kâğıtta o soruyu görüyor
   * ve puanının nereye gittiğini sorar. Aşağıda açıkça uyarılıyor.
   */
  let sorular = $derived.by(() => {
    const sirali = [...exam.questions].sort((a, b) => a.display_order - b.display_order);
    return sirali.map((ref) => ({
      ref,
      question: bank.find((q) => q.id === ref.question_id) ?? null,
    }));
  });

  let eksikSoru = $derived(sorular.filter((s) => s.question === null).length);

  /** Sınavın toplam puanı: sınava özgü puan varsa o, yoksa sorunun kendi puanı. */
  let toplamPuan = $derived(
    sorular.reduce((sum, s) => {
      const q = s.question;
      if (q === null) return sum;
      if (s.ref.points_override !== null) return sum + s.ref.points_override;
      if (q.question_type === "fill_in_blank")
        return sum + q.blanks.reduce((a, b) => a + b.points, 0);
      return sum + q.points;
    }, 0),
  );

  let girilmis = $derived(new Set(results.map((r) => r.student_id)));

  function selectStudent(id: string) {
    studentId = id;
    answers = {};
    manualPoints = {};
    rubricMet = {};
    saved = null;
    saveError = null;
  }

  function setAnswer(key: string, value: string) {
    answers = { ...answers, [key]: value };
  }

  function setPoints(qid: string, value: number) {
    manualPoints = { ...manualPoints, [qid]: value };
  }

  function maxPoints(ref: { points_override: number | null }, q: Question): number {
    if (ref.points_override !== null) return ref.points_override;
    return q.question_type === "fill_in_blank"
      ? q.blanks.reduce((a, b) => a + b.points, 0)
      : q.points;
  }

  /** Ham girdileri ScoringService'in beklediği biçime çevirir. */
  function buildPayload(): QuestionAnswerInput[] {
    const out: QuestionAnswerInput[] = [];

    for (const { question } of sorular) {
      if (question === null) continue;
      const qid = question.id;

      if (question.question_type === "classic") {
        out.push({
          question_id: qid,
          given_answer: null,
          points_earned: manualPoints[qid] ?? 0,
          is_correct: null,
          rubric_met: rubricMet[qid] ?? [],
        });
        continue;
      }

      if (question.question_type === "fill_in_blank") {
        const map: Record<string, string> = {};
        for (const b of question.blanks) {
          const v = answers[`${qid}::${b.id}`];
          if (v !== undefined && v !== "") map[b.id] = v;
        }
        out.push({
          question_id: qid,
          given_answer: JSON.stringify(map),
          points_earned: 0,
          is_correct: null,
          rubric_met: [],
        });
        continue;
      }

      // Çoktan seçmeli ve doğru-yanlış. Boş bırakılan soru CEVAPSIZ sayılır:
      // null gönderilir, yanlış bir şık uydurulmaz.
      const raw = answers[qid];
      out.push({
        question_id: qid,
        given_answer: raw === undefined || raw === "" ? null : raw,
        points_earned: 0,
        is_correct: null,
        rubric_met: [],
      });
    }

    return out;
  }

  async function save() {
    if (studentId === "") {
      saveError = "Önce öğrenci seç.";
      return;
    }

    saving = true;
    saveError = null;
    saved = null;
    try {
      await api.results.enter({
        examId: exam.id,
        studentId,
        answers: buildPayload(),
        totalMax: toplamPuan,
      });
      const s = students.find((x) => x.id === studentId);
      saved = s ? `${s.first_name} ${s.last_name}` : "Sonuç";
      onsaved();
    } catch (err: unknown) {
      saveError = errorText(err);
    } finally {
      saving = false;
    }
  }
</script>

<div class="grid min-h-0 flex-1 grid-cols-[220px_1fr]">
  <!-- Öğrenci listesi. Girilmiş olanlar işaretli: deste ilerledikçe kalan görünür. -->
  <nav class="min-h-0 overflow-auto border-r border-gray-300 dark:border-gray-600">
    {#if students.length === 0}
      <p class="p-2.5 text-[12px] leading-5 text-gray-500 dark:text-gray-400">Bu sınıfta öğrenci yok.</p>
    {:else}
      <ul>
        {#each students as s (s.id)}
          <li>
            <button
              type="button"
              class="flex w-full items-center gap-2.5 border-b border-gray-200 px-2.5 py-[5px]
                     text-left text-[13px] leading-5 transition-colors hover:bg-gray-50
                     dark:border-gray-700 dark:hover:bg-gray-700
                     {s.id === studentId ? 'bg-primary-50 font-semibold dark:bg-primary-900/30' : ''}"
              onclick={() => selectStudent(s.id)}
            >
              <!-- shrink-0: numara sütunu 28px'te sabit kalmalı. Uzun adlarda flex sıkıştırması
                   sırayla her öğeyi eziyordu; numara okunmaz hale geliyordu. -->
              <span class="tnum w-[28px] shrink-0 text-gray-500 dark:text-gray-400">{s.number}</span>
              <!-- Ad span'i bir flex öğesi ve flex öğesinin varsayılan min-width'i auto: kendi
                   min-content genişliğinin — yani en uzun kelimesinin, ör. "Küçükçalışkanoğlu" —
                   ALTINA inemiyordu. Buton w-full ama 220px'lik sabit ızgara sütununda olduğu için
                   flex satırı butonun dışına taşıyor, sağdaki ✓ rozeti nav'ın overflow-auto'suna
                   kaçıyordu (macOS'ta overlay kaydırıcı görünmediği için kayıp sessizdi).
                   min-w-0 küçülmenin önünü açar; truncate ancak ebeveyn küçülebildiğinde çalışır,
                   bu yüzden ikisi birlikte veriliyor. Tam ad title ile erişilebilir kalıyor —
                   metni kısaltmıyoruz, yalnız görüntüde kırpıyoruz. -->
              <span class="min-w-0 flex-1 truncate" title={`${s.first_name} ${s.last_name}`}>{s.first_name} {s.last_name}</span>
              {#if girilmis.has(s.id)}
                <!-- Sonucu girilmiş: yeşil "tamam" rozeti. Bu bir "doğru cevap" değil,
                     bir iş durumu — kırmızı/gri değerlendirme ekseninin dışında.
                     shrink-0: rozet, öğretmenin destede hangi kâğıdı girdiğini gördüğü TEK işaret;
                     dar sütunda ezilip kaybolmamalı, yeri adın kırpılmasından önce gelir. -->
                <Badge color="green" class="shrink-0 px-1.5 py-0 text-[11px]">✓</Badge>
              {/if}
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </nav>

  <div class="min-h-0 overflow-auto px-5 py-2.5">
    {#if eksikSoru > 0}
      <Alert color="red" class="mb-2.5 text-[12px] leading-5">
        Bu sınavın {eksikSoru} sorusu bankada bulunamadı. O sorular puanlanamaz;
        toplam puan onlar hariç hesaplandı.
      </Alert>
    {/if}

    {#if studentId === ""}
      <p class="text-[12px] leading-5 text-gray-500 dark:text-gray-400">Soldan bir öğrenci seç.</p>
    {:else}
      <div class="mb-2.5 flex flex-wrap items-center gap-2.5 border-b border-gray-300 pb-[5px] dark:border-gray-600">
        <span class="text-[11px] font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">
          Toplam
        </span>
        <span class="tnum font-bold text-gray-900 dark:text-white">{toplamPuan}</span>
        <span class="text-[12px] text-gray-500 dark:text-gray-400">puan</span>
        {#if girilmis.has(studentId)}
          <!-- Uyarı, hata değil: amber. Kırmızı yalnız değerlendirme/yanlış içindir. -->
          <span class="text-[12px] leading-5 text-amber-600 dark:text-amber-400">
            Bu öğrencinin sonucu daha önce girilmiş; kaydetmek üzerine yazar.
          </span>
        {/if}
        <span class="ml-auto"></span>
        <PenButton kind="ink" disabled={saving} onclick={save}>
          {saving ? "Kaydediliyor…" : "Kaydet"}
        </PenButton>
      </div>

      {#if saveError}
        <Alert color="red" class="mb-2.5 text-[12px] leading-5">{saveError}</Alert>
      {/if}
      {#if saved}
        <p class="mb-2.5 text-[12px] leading-5 text-gray-500 dark:text-gray-400">
          {saved} kaydedildi. Ölçüm yeniden hesaplandı.
        </p>
      {/if}

      <Card size="xl" class="p-0">
        <Table>
          <TableHead>
            <!-- flowbite'ın tableHeadCell/tableBodyCell base'i px-6 (48px yatay dolgu) taşıyor;
                 üç sütunda 144px salt dolgu ediyor. Tailwind preflight box-sizing: border-box
                 verdiği için bildirilen w-[2rem] (32px) dolgunun (48px) altına inemiyor ve
                 sessizce yok sayılıyordu — sütun gerçekte ~60px kalıyordu. Dolgu daralınca hem
                 genişlik bildirimi geçerli oluyor hem de tabloya ~90px yatay yer geri dönüyor. -->
            <TableHeadCell class="w-[2.5rem] px-2 py-2">#</TableHeadCell>
            <TableHeadCell class="px-3 py-2">Soru</TableHeadCell>
            <TableHeadCell class="px-3 py-2">Cevap</TableHeadCell>
          </TableHead>
          <TableBody>
            {#each sorular as { ref, question }, i (ref.question_id)}
              {#if question !== null}
                <TableBodyRow>
                  <!-- flowbite tableBodyCell base'i: "px-6 py-4 whitespace-nowrap font-medium".
                       white-space KALITIMLI bir özellik olduğu için hücrenin nowrap'i içindeki her
                       span/label/Checkbox etiketine iniyordu. tv() içeride tailwind-merge kullandığı
                       için whitespace-normal aynı gruptan gelip nowrap'i düşürür. flex-wrap bunu
                       kurtaramazdı: o yalnız flex ÖĞELERİNİ alt satıra atar, öğenin içindeki metni
                       sarmaz. px-2 py-2 ise başlıktaki dar dolguyla hizalı kalsın diye. -->
                  <TableBodyCell class="tnum w-[2.5rem] whitespace-normal px-2 py-2 align-top text-gray-500 dark:text-gray-400">
                    {i + 1}.
                  </TableBodyCell>
                  <!-- ASIL TAŞMA KAYNAĞI. 90 karaktere kadar olan soru önizlemesi, kalıtılan
                       whitespace-nowrap yüzünden tek satırda ~560-600px genişlik istiyordu.
                       Tablonun min-content genişliği kartı aşınca Table'ın sardığı
                       div.relative.overflow-x-auto içeriği sağ kenarda KESİYORDU; macOS'ta overlay
                       kaydırma çubuğu görünmediğinden bu "içerik kartın sağına taşıyor" olarak
                       görünüyordu. Metni kısaltmıyoruz: whitespace-normal sarmayı açar, w-[45%]
                       sütunu payla sınırlar.

                       break-words DEĞİL wrap-anywhere: overflow-wrap: break-word hücrenin
                       min-content katkısını DÜŞÜRMEZ — min-content hesabında kelime bölünmemiş
                       sayılır (aynı ölçüm ItemAnalysis.svelte'de Soru sütunu yorumunda
                       yazılı). bodyPreview çıktısı
                       boşluksuz, bölünemez tek bir Typst belirteci olabiliyor ($R=(V_("pin")-V_F)/I
                       gibi); o durumda hücrenin min-content'i belirtecin tam genişliği kalıyor,
                       tablo yine kartı aşıp yatay kaydırıcı açıyordu. wrap-anywhere yalnız satıra
                       sığmayan dizgeyi kırar, Türkçe cümleyi ortadan bölmez (break-all bölerdi).

                       title ŞART: bodyPreview METNİ 90 karakterde JS'te KESİYOR ve sonuna "…"
                       koyuyor — CSS kırpması değil, gerçek kayıp. Uzun bir klasik soruda ilk 90
                       karakter çoğu zaman hangi soru olduğunu ayırt etmeye yetmiyor; öğretmen
                       yanlış soruyu puanlar. Infinity ile çağrılan bodyPreview hiç kesmiyor
                       (`text.length > maxLen` sağlanmaz), yani tam düz metin title'da duruyor. -->
                  <TableBodyCell class="w-[45%] whitespace-normal px-3 py-2 align-top wrap-anywhere">
                    <span
                      class="text-[13px] leading-5 text-gray-700 dark:text-gray-300"
                      title={bodyPreview(question.body, Infinity)}
                    >
                      {bodyPreview(question.body, 90)}
                    </span>
                    <span class="tnum block text-[12px] text-gray-500 dark:text-gray-400">
                      {maxPoints(ref, question)} p
                    </span>
                  </TableBodyCell>
                  <!-- Aynı kalıtılan whitespace-nowrap burada da "boş bırakılırsa cevapsız sayılır"
                       yardım metnini ve rubrik ölçütlerini tek satıra kilitliyordu. whitespace-normal
                       sarmayı geri verir. wrap-anywhere (break-words değil): rubrik ölçütüne
                       yapıştırılmış bir formül ya da boşluksuz uzun bir belirteç, break-word
                       altında hücrenin min-content'ini hâlâ tam kelime genişliğinde bırakıyor
                       ve tabloyu kartın dışına itiyordu. overflow-wrap KALITSAL olduğu için
                       aşağıdaki ölçüt metinleri, etiketler ve yardım metni de kapsanıyor —
                       alt öğelere ayrıca yazmak gerekmiyor, yazılırsa geri çevirir. -->
                  <TableBodyCell class="whitespace-normal px-3 py-2 align-top wrap-anywhere">
                    {#if question.question_type === "multiple_choice"}
                      <div class="flex flex-wrap items-center gap-[5px]">
                        <!-- hover çerçevesi primary-*, kırmızı DEĞİL: bir şıkkın üzerine gelmek
                             ne yanlış cevap ne hata, yalnız imlecin nereye geldiğini gösteren
                             dekoratif bir vurgu. Kırmızı bu uygulamada tek anlam taşır —
                             değerlendirme (yanlış cevap, eşiğin altı, hata); hover'da kullanmak
                             öğretmene şıkkı seçmenin yanlış olduğunu söyler. Seçili durum da
                             zaten primary-50 / primary-900/30 ile aynı eksende. -->
                        {#each question.options as opt (opt.id)}
                          <button
                            type="button"
                            class="w-[30px] border border-gray-300 bg-white py-[5px] text-[12px]
                                   leading-5 transition-colors hover:border-primary-600
                                   dark:border-gray-600 dark:bg-gray-800 dark:hover:border-primary-400
                                   {answers[question.id] === opt.id
                                     ? 'bg-primary-50 font-semibold dark:bg-primary-900/30'
                                     : ''}"
                            onclick={() =>
                              setAnswer(question.id, answers[question.id] === opt.id ? "" : opt.id)}
                          >
                            {opt.id}
                          </button>
                        {/each}
                        <span class="ml-2.5 text-[12px] text-gray-500 dark:text-gray-400">
                          boş bırakılırsa cevapsız sayılır
                        </span>
                      </div>
                    {:else if question.question_type === "true_false"}
                      <div class="flex flex-wrap items-center gap-[5px]">
                        <!-- İki düğmede de hover primary-*: "Yanlış" düğmesinin üzerine gelmek
                             öğrencinin yanlış cevapladığı anlamına gelmez — düğme öğretmenin
                             GİRDİSİ, sonucu değil. Yanlışı düğmenin ETİKETİ söyler, çerçevesi
                             değil; kırmızı çerçeve "Doğru" düğmesinde de belirir ve iki düğmeyi
                             aynı uyarı rengiyle boyayarak ayrımı büsbütün siler. -->
                        <button
                          type="button"
                          class="border border-gray-300 bg-white px-2.5 py-[5px] text-[12px]
                                 leading-5 transition-colors hover:border-primary-600
                                 dark:border-gray-600 dark:bg-gray-800 dark:hover:border-primary-400
                                 {answers[question.id] === 'true'
                                   ? 'bg-primary-50 font-semibold dark:bg-primary-900/30'
                                   : ''}"
                          onclick={() =>
                            setAnswer(question.id, answers[question.id] === "true" ? "" : "true")}
                        >
                          Doğru
                        </button>
                        <button
                          type="button"
                          class="border border-gray-300 bg-white px-2.5 py-[5px] text-[12px]
                                 leading-5 transition-colors hover:border-primary-600
                                 dark:border-gray-600 dark:bg-gray-800 dark:hover:border-primary-400
                                 {answers[question.id] === 'false'
                                   ? 'bg-primary-50 font-semibold dark:bg-primary-900/30'
                                   : ''}"
                          onclick={() =>
                            setAnswer(question.id, answers[question.id] === "false" ? "" : "false")}
                        >
                          Yanlış
                        </button>
                      </div>
                    {:else if question.question_type === "fill_in_blank"}
                      <div class="flex flex-wrap items-center gap-2.5">
                        {#each question.blanks as b (b.id)}
                          <!-- flex-wrap yoktu: dar üçüncü sütunda boşluk kimliği + 110px'lik girdi
                               tek satırda kalıp hücreden taşıyordu — PUAN satırıyla birebir aynı
                               yapı. label'da shrink-0: etiket+girdi ikilisi ortadan bölünmek yerine
                               bir bütün olarak dış sarmalayıcıda alt satıra insin. -->
                          <label class="flex shrink-0 flex-wrap items-center gap-[5px]">
                            <!-- shrink-0: sıkışmada önce kimlik etiketi eziliyordu; hangi boşluğa
                                 yazıldığı görünmeden girdi işe yaramaz. -->
                            <span class="shrink-0 text-[11px] font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">
                              {b.id}
                            </span>
                            <!-- flowbite Input sarmalayıcısız çizildiğinde class'ı DİZİ olarak
                                 kuruyor: class={[wrapped || base(), inputCls({…})]}. base() =
                                 "relative w-full" ve dizi elemanı olduğundan tailwind-merge'e
                                 GİRMİYOR; <input> üzerinde w-full ile w-[110px] yan yana duruyor,
                                 kazanan Tailwind'in çıktı sırasına kalıyordu. max-w-[110px] rakipsiz
                                 bir özellik grubunda olduğu için genişliği kesin sınırlar; shrink-0
                                 girdinin sarma sırasında okunmaz kalınlığa ezilmesini önler. -->
                            <Input
                              type="text"
                              size="sm"
                              class="w-[110px] max-w-[110px] shrink-0"
                              value={answers[`${question.id}::${b.id}`] ?? ""}
                              oninput={(e) =>
                                setAnswer(
                                  `${question.id}::${b.id}`,
                                  (e.currentTarget as HTMLInputElement).value,
                                )}
                            />
                          </label>
                        {/each}
                      </div>
                    {:else}
                      <!--
                        Açık uçlu soru. Rubrik varsa ölçüt ölçüt işaretlenir ve puan
                        kendiliğinden toplanır: hangi ölçütün verilmediği kayda geçer,
                        itiraz geldiğinde gerekçe elde durur.

                        Puan kutusu YİNE DE düzenlenebilir kalıyor. Ölçüte tam
                        uymayan ama karşılığı olan bir cevabı öğretmen takdir
                        edebilmeli; rubrik yardımcıdır, kelepçe değil.
                      -->
                      {#if rubrikOf(question).length > 0}
                        <ul class="mb-[5px] space-y-[2px]">
                          {#each rubrikOf(question) as olcut, oi (oi)}
                            <li>
                              <!--
                                labelProps.class ÖLÜ KOD'du. Checkbox.svelte etiketi
                                `<Label {...labelProps} class={divStyle(...)}>` diye çiziyor ve
                                Svelte'de yayılımdan SONRA gelen açık class niteliği yayılımı eziyor;
                                istenen items-start hiç uygulanmıyor, temanın "flex items-center"ı
                                kalıyordu. Bu sürümde etiket sarmalayıcısına ulaşmanın tek yolu
                                classes.div — kaynakta styling = classes ?? { div: divClass } ve
                                divStyle({ class: clsx(theme?.div, styling.div) }), yani
                                tailwind-merge'den geçip items-center'ı items-start ile değiştirir.
                                Ölçüt metni artık sardığı için kutu metnin ORTASINA değil ÜSTÜNE
                                hizalanmalı; yoksa iki-üç satırlık ölçütte hangi kutunun hangi
                                ölçüte ait olduğu belirsizleşiyor.
                              -->
                              <Checkbox
                                checked={(rubricMet[question.id] ?? []).includes(oi)}
                                onchange={() => toggleCriterion(question, oi)}
                                classes={{ div: "flex w-full items-start gap-[5px]" }}
                              >
                                <!-- flex-1 = flex: 1 1 0%, ama flex öğesinin min-width'i varsayılan
                                     auto: öğe kendi min-content'inin altına inemiyor. Kalıtılan
                                     whitespace-nowrap bu min-content'i TÜM CÜMLE yaptığı için ölçüt
                                     satırı kabın dışına taşıyor, sağdaki shrink-0'lı puanı satırdan
                                     atıyordu — öğretmen hangi ölçüte kaç puan verdiğini göremiyordu.
                                     min-w-0 küçülmeyi açar; sarmayı hücredeki whitespace-normal,
                                     bölünemez uzun dizgeyi de hücreden KALITILAN wrap-anywhere
                                     halleder. Buraya ayrıca break-words yazmak zararlıydı: kalıtılan
                                     `anywhere`ı `break-word`e geri çevirip min-content'i yine tam
                                     kelime yapıyordu, o yüzden kaldırıldı. -->
                                <span class="min-w-0 flex-1 text-[12px] leading-5 text-gray-700 dark:text-gray-300">
                                  {olcut.criterion}
                                </span>
                                <span class="tnum shrink-0 text-[12px] text-gray-500 dark:text-gray-400">
                                  {olcut.points}
                                </span>
                              </Checkbox>
                            </li>
                          {/each}
                        </ul>
                      {/if}

                      <!-- flex-wrap yoktu: "PUAN" + sayı girdisi + "/ N" dar üçüncü sütunda alt
                           satıra geçmek yerine tek satırda kalıp hücrenin sağından dışarı çıkıyordu.
                           "/ N" ve girdinin sağ kenarı görünmez oluyor, öğretmen puanı kaç üzerinden
                           verdiğini göremiyordu. flex-wrap üçlüyü sığdığı yerden bölerek sarar. -->
                      <label class="flex flex-wrap items-center gap-[5px]">
                        <!-- shrink-0: sıkışmada esnemesi gereken şey etiket değil satırın kendisi.
                             Olmadan "Puan" harf harf eziliyor, girdi ise 70px'ini koruyordu. -->
                        <span class="shrink-0 text-[11px] font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">
                          Puan
                        </span>
                        <!-- flowbite Input, left/right/clearable/data verilmediğinde sarmalayıcısız
                             çiziliyor ve class'ı dizi olarak kuruyor:
                             class={[wrapped || base(), inputCls({…})]}. base() = "relative w-full"
                             ve dizi elemanı olduğundan tailwind-merge'e GİRMİYOR — <input> üzerinde
                             w-full ile w-[70px] aynı anda duruyor, kazanan Tailwind'in çıktı sırasına
                             kalıyor, yani genişlik güvenilmez. max-w-[70px] rakipsiz bir özellik
                             grubunda olduğu için birleşmemiş w-full'ü kesin sınırlar; shrink-0 ise
                             sarma sırasında girdinin okunmaz kalınlığa ezilmesini önler. -->
                        <Input
                          type="number"
                          size="sm"
                          min="0"
                          max={maxPoints(ref, question)}
                          class="tnum w-[70px] max-w-[70px] shrink-0"
                          value={manualPoints[question.id] ?? 0}
                          oninput={(e) =>
                            setPoints(question.id, Number((e.currentTarget as HTMLInputElement).value))}
                        />
                        <!-- shrink-0: üst sınır ("/ N") puanın anlamını taşıyor; ezilirse öğretmen
                             10 üzerinden mi 25 üzerinden mi verdiğini bilemez. -->
                        <span class="shrink-0 text-[12px] text-gray-500 dark:text-gray-400">
                          / {maxPoints(ref, question)}
                        </span>
                      </label>
                    {/if}
                  </TableBodyCell>
                </TableBodyRow>
              {/if}
            {/each}
          </TableBody>
        </Table>
      </Card>
    {/if}
  </div>
</div>

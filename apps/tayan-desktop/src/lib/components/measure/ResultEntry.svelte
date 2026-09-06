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

  function rubricOf(q: Question): RubricItem[] {
    return q.question_type === "classic" ? q.rubric : [];
  }

  /** Ölçütü işaretle/kaldır ve puanı yeniden topla. */
  function toggleCriterion(q: Question, index: number) {
    dirty = true;
    const current = rubricMet[q.id] ?? [];
    const next = current.includes(index)
      ? current.filter((i) => i !== index)
      : [...current, index].sort((a, b) => a - b);

    rubricMet = { ...rubricMet, [q.id]: next };

    const rubric = rubricOf(q);
    const total = next.reduce((sum, i) => sum + (rubric[i]?.points ?? 0), 0);
    manualPoints = { ...manualPoints, [q.id]: total };
  }

  /**
   * Sınavın soruları, display_order sırasında ve bankada BULUNANLAR.
   *
   * Bankada olmayan atıf sessizce atlanmıyor: öğretmen kâğıtta o soruyu görüyor
   * ve puanının nereye gittiğini sorar. Aşağıda açıkça uyarılıyor.
   */
  let examQuestions = $derived.by(() => {
    const ordered = [...exam.questions].sort((a, b) => a.display_order - b.display_order);
    return ordered.map((ref) => ({
      ref,
      question: bank.find((q) => q.id === ref.question_id) ?? null,
    }));
  });

  let missingQuestionCount = $derived(examQuestions.filter((s) => s.question === null).length);

  /** Sınavın toplam puanı: sınava özgü puan varsa o, yoksa sorunun kendi puanı. */
  let totalPoints = $derived(
    examQuestions.reduce((sum, s) => {
      const q = s.question;
      if (q === null) return sum;
      if (s.ref.points_override !== null) return sum + s.ref.points_override;
      if (q.question_type === "fill_in_blank")
        return sum + q.blanks.reduce((a, b) => a + b.points, 0);
      return sum + q.points;
    }, 0),
  );

  let enteredStudentIds = $derived(new Set(results.map((r) => r.student_id)));

  /**
   * Form öğretmen tarafından ELLE değiştirildi mi?
   *
   * Geç gelen sonuçların üzerine yazmamak için: aşağıdaki etki, sonuç listesi
   * sonradan tazelendiğinde formu doldurmaya çalışıyor; öğretmen o sırada
   * yazmaya başlamışsa girdiği değerleri EZMEMELİ.
   */
  let dirty = $state(false);

  /**
   * Kaydedilmiş sonucu forma geri yükler.
   *
   * BU EKSİKTİ VE SESSİZ VERİ KAYBI ÜRETİYORDU. `selectStudent` formu
   * temizliyor ama kaydı okumuyordu: puanlanmış bir öğrenciye dönen öğretmen
   * BOŞ bir form görüyor, üstünde "sonucu daha önce girilmiş; kaydetmek
   * üzerine yazar" uyarısı duruyordu — ve Kaydet'e basmak tam bir sonucu boş
   * sonuçla eziyordu. Veri zaten burada: `results` prop'u sınavın tüm
   * sonuçlarını taşıyor, yalnız hiç okunmuyordu.
   *
   * Sonuç yoksa hiçbir şey yapmaz; çağıran zaten formu temizlemiş olur.
   */
  function hydrate(id: string) {
    const existing = results.find((r) => r.student_id === id);
    if (!existing) return;

    const nextAnswers: Record<string, string> = {};
    const nextPoints: Record<string, number> = {};
    const nextRubric: Record<string, number[]> = {};

    for (const answer of existing.answers) {
      const question = bank.find((q) => q.id === answer.question_id);
      if (question === undefined) continue;

      // Klasik soruda cevap metni yok; puan ve rubrik kanıtı saklanıyor.
      if (question.question_type === "classic") {
        nextPoints[question.id] = answer.points_earned;
        nextRubric[question.id] = answer.rubric_met;
        continue;
      }

      // `null` = cevapsız. Boş şıkkı "seçilmiş" göstermemek için atlanıyor.
      if (answer.given_answer === null) continue;

      if (question.question_type === "fill_in_blank") {
        /*
          `given_answer` burada bir JSON eşlemesi: {"b1": "180", …} —
          `buildPayload` böyle yazıyor. Bozuk bir kayıt tüm formu
          çökertmemeli: ayrıştırma başarısızsa o soru boş kalır, diğerleri
          yüklenmeye devam eder.
        */
        let map: unknown;
        try {
          map = JSON.parse(answer.given_answer);
        } catch {
          continue;
        }
        if (typeof map !== "object" || map === null) continue;
        for (const [blankId, value] of Object.entries(map as Record<string, unknown>)) {
          if (typeof value === "string") nextAnswers[`${question.id}::${blankId}`] = value;
        }
        continue;
      }

      // Çoktan seçmeli ve doğru-yanlış: ham dize olduğu gibi geri konur.
      nextAnswers[question.id] = answer.given_answer;
    }

    answers = nextAnswers;
    manualPoints = nextPoints;
    rubricMet = nextRubric;
    dirty = false;
  }

  function selectStudent(id: string) {
    studentId = id;
    answers = {};
    manualPoints = {};
    rubricMet = {};
    saved = null;
    saveError = null;
    dirty = false;
    hydrate(id);
  }

  /**
   * Sonuçlar SEÇİMDEN SONRA gelirse formu yine doldur.
   *
   * `results` ana sayfada bir `$effect` ile yükleniyor. Öğretmen o IPC gidiş
   * dönüşü bitmeden bir öğrenciye tıklarsa `selectStudent` içindeki `hydrate`
   * boş listede arar ve hiçbir şey bulamaz; sonuç sonradan gelir, ✓ rozeti ve
   * uyarı belirir ama form BOŞ kalırdı — yani aynı üzerine yazma tuzağının
   * dar bir penceresi. Kaydetme sonrası tazelemede de bu etki çalışır ve
   * formu sunucudaki kaydın son hâline eşitler.
   *
   * `dirty` koruması şart: öğretmen yazmaya başladıysa girdiği değerler
   * korunur.
   */
  $effect(() => {
    void results;
    if (studentId === "" || dirty) return;
    hydrate(studentId);
  });

  function setAnswer(key: string, value: string) {
    dirty = true;
    answers = { ...answers, [key]: value };
  }

  function setPoints(qid: string, value: number) {
    dirty = true;
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

    for (const { question } of examQuestions) {
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

  /*
    CEVAP DÜĞMELERİ TEK GEOMETRİDEN. Çoktan seçmeli şıklar sabit `w-[30px]`,
    Doğru/Yanlış ise `px-2.5` ile otomatik genişlikteydi: aynı işi yapan iki
    kontrol, iki ayrı boyutlandırma stratejisi ve gözle görülür yükseklik
    farkı. Artık ikisi de aynı dizeyi kullanıyor; şıklar yalnız `min-w` ile
    eşit genişliğe zorlanıyor, böylece A/B/C/D bir ızgara gibi hizalı kalıyor
    ama uzun bir şık kimliği de taşabilirse sığıyor.

    `rounded-lg`: sayfadaki Flowbite kontrolleriyle (DropdownSelect, Button)
    aynı yarıçap. Eskiden bu düğmelerin hiç yarıçapı yoktu ve yuvarlak
    komşularının yanında köşeli duruyorlardı.
  */
  const CHOICE_BUTTON =
    "rounded-lg border border-default-medium bg-neutral-primary-medium px-2.5 py-1 " +
    "text-xs leading-5 text-heading transition-colors hover:border-primary-600 " +
    "dark:hover:border-primary-400";

  const CHOICE_SELECTED =
    "border-primary-600 bg-primary-50 font-semibold text-primary-800 " +
    "dark:border-primary-400 dark:bg-primary-900/30 dark:text-primary-200";

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
        totalMax: totalPoints,
      });
      const s = students.find((x) => x.id === studentId);
      saved = s ? `${s.first_name} ${s.last_name}` : "Sonuç";
      /*
        `dirty` sıfırlanıyor: `onsaved` sonuç listesini tazeliyor ve yukarıdaki
        etki formu SUNUCUDAKİ kayıttan yeniden dolduruyor. Böylece ekranda
        görünen, kaydedilenin ta kendisi olur — puanlamanın sunucu tarafında
        normalize edildiği durumlar da dahil.
      */
      dirty = false;
      onsaved();
    } catch (err: unknown) {
      saveError = errorText(err);
    } finally {
      saving = false;
    }
  }
</script>

<!--
  SOL SÜTUN 240px VE `px-5`. İki hizasızlık vardı: sütun 220px'ti (öğrenciler
  sayfasındaki sol sütun 240px) ve satırlar `px-2.5` (10px) ile başlıyordu,
  oysa hemen ÜSTÜNDEKİ araç çubuğu `px-5` (20px). Öğrenci numarası ile "Sınav"
  etiketi aynı dikeyde durmuyordu.
-->
<div class="grid min-h-0 flex-1 grid-cols-[240px_1fr]">
  <!-- Öğrenci listesi. Girilmiş olanlar işaretli: deste ilerledikçe kalan görünür. -->
  <nav class="min-h-0 overflow-auto border-r border-default-medium">
    {#if students.length === 0}
      <p class="p-2.5 text-xs text-body-subtle">Bu sınıfta öğrenci yok.</p>
    {:else}
      <ul>
        {#each students as s (s.id)}
          <li>
            <button
              type="button"
              class="flex w-full items-center gap-2.5 border-b border-default-medium px-5 py-1
                     text-left text-sm leading-5 transition-colors
                     hover:bg-neutral-tertiary-medium
                     {s.id === studentId ? 'bg-primary-50 font-semibold dark:bg-primary-900/30' : ''}"
              onclick={() => selectStudent(s.id)}
            >
              <!-- shrink-0: numara sütunu 28px'te sabit kalmalı. Uzun adlarda flex sıkıştırması
                   sırayla her öğeyi eziyordu; numara okunmaz hale geliyordu. -->
              <span class="tnum w-[28px] shrink-0 text-body-subtle">{s.number}</span>
              <!-- Ad span'i bir flex öğesi ve flex öğesinin varsayılan min-width'i auto: kendi
                   min-content genişliğinin — yani en uzun kelimesinin, ör. "Küçükçalışkanoğlu" —
                   ALTINA inemiyordu. Buton w-full ama 220px'lik sabit ızgara sütununda olduğu için
                   flex satırı butonun dışına taşıyor, sağdaki ✓ rozeti nav'ın overflow-auto'suna
                   kaçıyordu (macOS'ta overlay kaydırıcı görünmediği için kayıp sessizdi).
                   min-w-0 küçülmenin önünü açar; truncate ancak ebeveyn küçülebildiğinde çalışır,
                   bu yüzden ikisi birlikte veriliyor. Tam ad title ile erişilebilir kalıyor —
                   metni kısaltmıyoruz, yalnız görüntüde kırpıyoruz. -->
              <span class="min-w-0 flex-1 truncate" title={`${s.first_name} ${s.last_name}`}>{s.first_name} {s.last_name}</span>
              {#if enteredStudentIds.has(s.id)}
                <!-- Sonucu girilmiş: yeşil "tamam" rozeti. Bu bir "doğru cevap" değil,
                     bir iş durumu — kırmızı/gri değerlendirme ekseninin dışında.
                     shrink-0: rozet, öğretmenin destede hangi kâğıdı girdiğini gördüğü TEK işaret;
                     dar sütunda ezilip kaybolmamalı, yeri adın kırpılmasından önce gelir. -->
                <Badge color="green" class="shrink-0 px-1.5 py-0 text-xs">✓</Badge>
              {/if}
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </nav>

  <div class="min-h-0 overflow-auto px-5 py-2.5">
    {#if missingQuestionCount > 0}
      <Alert color="red" class="mb-2.5 text-xs">
        Bu sınavın {missingQuestionCount} sorusu bankada bulunamadı. O sorular puanlanamaz;
        toplam puan onlar hariç hesaplandı.
      </Alert>
    {/if}

    {#if studentId === ""}
      <p class="text-xs text-body-subtle">Soldan bir öğrenci seç.</p>
    {:else}
      <div class="mb-2.5 flex flex-wrap items-center gap-2.5 border-b border-default-medium pb-1">
        <!--
          "TAM PUAN", "TOPLAM" DEĞİL. `totalPoints` sınavın TAM puanı — soru
          soru en yüksek değerlerin toplamı — ve API'ye de `totalMax` olarak
          gidiyor. Etiket "Toplam" olduğunda, hemen yanında "Kaydet" düğmesi
          ve "bu öğrencinin sonucu daha önce girilmiş" uyarısı dururken,
          öğretmen bunu ÖĞRENCİNİN ALDIĞI puan olarak okuyordu: hiçbir şık
          seçili değilken bile 4 × 25 = 100 yazıyordu.

          Alınan puanı burada CANLI hesaplamıyoruz. Puanlama sunucuda
          yapılıyor; istemci tarafında ikinci bir hesap kurmak, kaydedilenle
          ekranda görünenin ayrışabileceği anlamına gelir — belirsiz bir
          etiketten daha kötü bir tutarsızlık olurdu.
        -->
        <span class="text-xs font-semibold uppercase tracking-wider text-body-subtle">
          Tam puan
        </span>
        <span class="tnum font-bold text-heading">{totalPoints}</span>
        <span class="text-xs text-body-subtle">puan üzerinden</span>
        {#if enteredStudentIds.has(studentId)}
          <!-- Uyarı, hata değil: amber. Kırmızı yalnız değerlendirme/yanlış içindir. -->
          <span class="text-xs text-amber-600 dark:text-amber-400">
            Aşağıdaki değerler daha önce kaydedilmiş sonuçtan geldi; Kaydet üzerine yazar.
          </span>
        {/if}
        <span class="ml-auto"></span>
        <PenButton kind="ink" disabled={saving} onclick={save}>
          {saving ? "Kaydediliyor…" : "Kaydet"}
        </PenButton>
      </div>

      {#if saveError}
        <Alert color="red" class="mb-2.5 text-xs">{saveError}</Alert>
      {/if}
      {#if saved}
        <p class="mb-2.5 text-xs text-body-subtle">
          {saved} kaydedildi. Ölçüm yeniden hesaplandı.
        </p>
      {/if}

      <!--
        `overflow-hidden` ŞART, süs değil. `Card`ın base'i `rounded-lg` (bu
        projede 16px) + `border` taşıyor ama KIRPMA yok; içindeki `TableHead`
        kendi zeminini KARE köşeyle boyayıp kartın yuvarlak köşesinin üstüne
        biniyordu. Sonuç üst iki köşede çentik: kart yuvarlak, başlık şeridi
        köşeli. Diğer üç panelde (`ItemAnalysis`, `ScoreDistribution`,
        `AnswerGrid`) görünmüyor çünkü onların `p-4`ü çocuğu köşeden uzak
        tutuyor; burada `p-0` var, tablo doğrudan kenara dayanıyor.

        Tablonun yatay kaydırması etkilenmiyor: kaydırma kabı `Table`ın
        sardığı `div.overflow-x-auto` ve bu kırpma onun DIŞINDA.
      -->
      <Card size="xl" class="overflow-hidden p-0">
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
            {#each examQuestions as { ref, question }, i (ref.question_id)}
              {#if question !== null}
                <TableBodyRow>
                  <!-- flowbite tableBodyCell base'i: "px-6 py-4 whitespace-nowrap font-medium".
                       white-space KALITIMLI bir özellik olduğu için hücrenin nowrap'i içindeki her
                       span/label/Checkbox etiketine iniyordu. tv() içeride tailwind-merge kullandığı
                       için whitespace-normal aynı gruptan gelip nowrap'i düşürür. flex-wrap bunu
                       kurtaramazdı: o yalnız flex ÖĞELERİNİ alt satıra atar, öğenin içindeki metni
                       sarmaz. px-2 py-2 ise başlıktaki dar dolguyla hizalı kalsın diye. -->
                  <TableBodyCell class="tnum w-[2.5rem] whitespace-normal px-2 py-2 align-top text-body-subtle">
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
                      class="text-sm leading-5 text-body"
                      title={bodyPreview(question.body, Infinity)}
                    >
                      {bodyPreview(question.body, 90)}
                    </span>
                    <span class="tnum block text-xs leading-5 text-body-subtle">
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
                      <div class="flex flex-wrap items-center gap-1">
                        <!-- hover çerçevesi primary-*, kırmızı DEĞİL: bir şıkkın üzerine gelmek
                             ne yanlış cevap ne hata, yalnız imlecin nereye geldiğini gösteren
                             dekoratif bir vurgu. Kırmızı bu uygulamada tek anlam taşır —
                             değerlendirme (yanlış cevap, eşiğin altı, hata); hover'da kullanmak
                             öğretmene şıkkı seçmenin yanlış olduğunu söyler. Seçili durum da
                             zaten primary-50 / primary-900/30 ile aynı eksende. -->
                        {#each question.options as opt (opt.id)}
                          <button
                            type="button"
                            class="{CHOICE_BUTTON} min-w-[2.25rem] {answers[question.id] === opt.id
                                     ? CHOICE_SELECTED
                                     : ''}"
                            onclick={() =>
                              setAnswer(question.id, answers[question.id] === opt.id ? "" : opt.id)}
                          >
                            {opt.id}
                          </button>
                        {/each}
                        <span class="ml-2.5 text-xs text-body-subtle">
                          boş bırakılırsa cevapsız sayılır
                        </span>
                      </div>
                    {:else if question.question_type === "true_false"}
                      <div class="flex flex-wrap items-center gap-1">
                        <!-- İki düğmede de hover primary-*: "Yanlış" düğmesinin üzerine gelmek
                             öğrencinin yanlış cevapladığı anlamına gelmez — düğme öğretmenin
                             GİRDİSİ, sonucu değil. Yanlışı düğmenin ETİKETİ söyler, çerçevesi
                             değil; kırmızı çerçeve "Doğru" düğmesinde de belirir ve iki düğmeyi
                             aynı uyarı rengiyle boyayarak ayrımı büsbütün siler. -->
                        <button
                          type="button"
                          class="{CHOICE_BUTTON} {answers[question.id] === 'true'
                                   ? CHOICE_SELECTED
                                   : ''}"
                          onclick={() =>
                            setAnswer(question.id, answers[question.id] === "true" ? "" : "true")}
                        >
                          Doğru
                        </button>
                        <button
                          type="button"
                          class="{CHOICE_BUTTON} {answers[question.id] === 'false'
                                   ? CHOICE_SELECTED
                                   : ''}"
                          onclick={() =>
                            setAnswer(question.id, answers[question.id] === "false" ? "" : "false")}
                        >
                          Yanlış
                        </button>
                        <!--
                          YARDIM METNİ BURADA DA VAR. Çoktan seçmelide yazılı
                          olan bu cümle doğru-yanlış dalında YOKTU, oysa kural
                          birebir aynı: iki düğme de `setAnswer(id, "")` ile
                          boşa düşüyor ve `buildPayload` boş cevabı `null`
                          gönderiyor, yani cevapsız sayılıyor. Aynı kuralın
                          yalnız yarısını yazmak, öğretmene doğru-yanlışta
                          boş bırakmanın başka bir anlama geldiğini düşündürür.
                        -->
                        <span class="ml-2.5 text-xs text-body-subtle">
                          boş bırakılırsa cevapsız sayılır
                        </span>
                      </div>
                    {:else if question.question_type === "fill_in_blank"}
                      <div class="flex flex-wrap items-center gap-2.5">
                        {#each question.blanks as b (b.id)}
                          <!-- flex-wrap yoktu: dar üçüncü sütunda boşluk kimliği + 110px'lik girdi
                               tek satırda kalıp hücreden taşıyordu — PUAN satırıyla birebir aynı
                               yapı. label'da shrink-0: etiket+girdi ikilisi ortadan bölünmek yerine
                               bir bütün olarak dış sarmalayıcıda alt satıra insin. -->
                          <label class="flex shrink-0 flex-wrap items-center gap-1">
                            <!-- shrink-0: sıkışmada önce kimlik etiketi eziliyordu; hangi boşluğa
                                 yazıldığı görünmeden girdi işe yaramaz. -->
                            <span class="shrink-0 text-xs font-semibold uppercase tracking-wider text-body-subtle">
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
                      {#if rubricOf(question).length > 0}
                        <ul class="mb-[5px] space-y-[2px]">
                          {#each rubricOf(question) as criterion, oi (oi)}
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
                                classes={{ div: "flex w-full items-start gap-1" }}
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
                                <span class="min-w-0 flex-1 text-xs leading-5 text-body">
                                  {criterion.criterion}
                                </span>
                                <span class="tnum shrink-0 text-xs leading-5 text-body-subtle">
                                  {criterion.points}
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
                      <label class="flex flex-wrap items-center gap-1">
                        <!-- shrink-0: sıkışmada esnemesi gereken şey etiket değil satırın kendisi.
                             Olmadan "Puan" harf harf eziliyor, girdi ise 70px'ini koruyordu. -->
                        <span class="shrink-0 text-xs font-semibold uppercase tracking-wider text-body-subtle">
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
                        <span class="shrink-0 text-xs text-body-subtle">
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

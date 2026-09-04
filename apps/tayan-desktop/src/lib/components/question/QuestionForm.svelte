<script lang="ts">
  import { Alert, Button } from "flowbite-svelte";
  import QuestionEditor from "./QuestionEditor.svelte";
  import { bodySource, typstBody } from "$lib/question/body";
  import type { RubricItem } from "$lib/types";
  import { hasRubricCall, importRubric, removeRange } from "$lib/question/rubric-import";
  import { STARTER_SUBJECTS, subjectSuggestions } from "$lib/question/subjects";
  import { splitOutcomes } from "$lib/question/outcomes";
  import { onMount } from "svelte";
  import type { ContentNode } from "$lib/types";
  import {
    parseOptions,
    parseTrueFalse,
    parseBlanks,
    parseAnswerLines,
    OPTION_LETTERS,
  } from "$lib/question/templates";
  import { api } from "$lib/api";
  import { errorText } from "$lib/editor/diagnostics";
  import {
  MAX_GRADE,
  MIN_GRADE,
  questionPoints,
  type Question,
  type QuestionMeta,
  type QuestionStats,
} from "$lib/types";
  import { goto } from "$app/navigation";

  type QuestionType = Question["question_type"];

  type Props = {
    initialType?: QuestionType;
    initialBody?: string;
    stats?: QuestionStats | null;
    legacyWarning?: boolean;
    /**
     * Var olan bir soru düzenleniyorsa o soru. null ise yeni soru.
     *
     * Bu ayrım olmadan düzenleme her kaydedişte YENİ soru üretiyordu: banka
     * kopyalarla doluyor, sınavlar eski kimliğe atıf yapmaya devam ediyor ve
     * yapılan değişiklik sınavda hiç görünmüyordu.
     */
    existing?: Question | null;
  };

  let {
    initialType = "multiple_choice",
    initialBody = "",
    stats = null,
    legacyWarning = false,
    existing = null,
  }: Props = $props();

  let questionType = $state<QuestionType>(initialType);
  let body = $state(initialBody);
  /**
   * Sorunun kendi puanı artık editörde düzenlenmiyor: puan soruya değil,
   * sorunun SINAVDAKİ kullanımına aittir. Aynı soru bir yazılıda 5, başkasında
   * 10 puan edebilir; sınav ekranında belirlenir.
   *
   * Bu alan yalnızca sınavda puan verilmediğinde kullanılan yedektir. Var olan
   * soruda korunur, yenide varsayılana düşer.
   */
  const DEFAULT_POINTS = 5;
  let points = $state(existing ? questionPoints(existing) : DEFAULT_POINTS);
  let outcomeText = $state(existing ? existing.outcomes.join(" ") : "");

  /**
   * Açık uçlu sorunun puanlama ölçütleri.
   *
   * Bir zamanlar burada `rubric: []` SABİTİ vardı: alan Rust'ta, doğrulaması
   * Rust'ta, cevap anahtarına dizgisi Rust'ta hazırdı ama arayüz her kayıtta
   * boş yazıyordu — öğretmen rubriği hiç oluşturamıyordu.
   */
  let rubric = $state<RubricItem[]>(
    existing?.question_type === "classic" ? existing.rubric : [],
  );

  /**
   * Örnek çözüm — YALNIZ cevap anahtarına basılır.
   *
   * Alan uçtan uca ölüydü: komut yapısında yoktu, Rust işleyicisi `None`
   * yazıyordu, form `null` gönderiyordu. Cevap anahtarı "Örnek cevap:"
   * basmaya hazır olduğu hâlde öğretmen hiçbir yerden giremiyordu.
   */
  let sampleAnswer = $state(
    existing?.question_type === "classic" && existing.sample_answer
      ? bodySource(existing.sample_answer)
      : "",
  );

  /**
   * Gövdeye yazılmış `#rubrik(...)` bloğu.
   *
   * Öğretmenin elindeki eski cevap anahtarı dosyalarında ölçütler böyle
   * yazılı; yapıştırıp panele taşıyabilmeli. Taşıma TEK YÖNLÜ ve BİR KEZ:
   * sonrasında hüküm panelin. Çözümleyici asla not vermez, yalnız veri taşır.
   */
  /**
   * KAYNAK İKİ TANE. Öğretmen eski cevap anahtarı dosyasını büyük ihtimalle
   * CEVAP sekmesine yapıştırır — ölçütler orada. Yalnız gövdeye bakmak, aynı
   * sessiz kaybı öbür kapıdan içeri alırdı.
   */
  let govdedekiRubrik = $derived.by(() => {
    if (questionType !== "classic") return null;
    const govde = importRubric(body);
    if (govde) return { kaynak: "body" as const, sonuc: govde };
    const cevap = importRubric(sampleAnswer);
    if (cevap) return { kaynak: "sample" as const, sonuc: cevap };
    return null;
  });

  function rubrigiPaneleTasi() {
    const bulunan = govdedekiRubrik;
    if (!bulunan?.sonuc.ok) return;
    const { from, to, items } = bulunan.sonuc;
    rubric = items;
    if (bulunan.kaynak === "body") {
      body = removeRange(body, from, to);
    } else {
      sampleAnswer = removeRange(sampleAnswer, from, to);
    }
  }

  /** Rust'taki ClassicQuestion::validate ile aynı kural. */
  let rubricError = $derived.by(() => {
    if (questionType !== "classic" || rubric.length === 0) return null;
    if (rubric.some((r) => r.criterion.trim() === "")) {
      return "Rubrikte boş ölçüt var.";
    }
    const toplam = rubric.reduce((sum, r) => sum + r.points, 0);
    if (toplam !== points) {
      return `Rubrik toplamı (${toplam}) soru puanıyla (${points}) eşleşmiyor.`;
    }
    return null;
  });

  /**
   * Sorunun künyesi. Var olan soruda kayıtlı değer; yenide boş.
   *
   * Eski kayıtlarda bu alan yoktu ve Rust tarafı serde(default) ile
   * subject: "", grade: 0 döndürür. Böyle bir soru açıldığında alanlar boş
   * görünür ve kaydetme kilitlenir — öğretmen o an doldurur. Uydurma bir
   * varsayılan koymak, yanlış kazanım eşleşmesi üretmekten kötüdür.
   */
  let meta = $state<QuestionMeta>(
    existing?.meta ?? { subject: "", grade: 0, difficulty: null, title: "" },
  );

  /** Ders ve sınıf seviyesi zorunlu; Rust tarafındaki kuralın aynısı. */
  let metaError = $derived.by(() => {
    if (meta.subject.trim() === "") return "Ders alanı boş olamaz.";
    if (!Number.isFinite(meta.grade) || meta.grade < MIN_GRADE || meta.grade > MAX_GRADE)
      return `Sınıf seviyesi ${MIN_GRADE} ile ${MAX_GRADE} arasında olmalı.`;

    // Biçimsiz kazanım kaydetmede Rust tarafından reddedilir. Burada yakalamak,
    // öğretmenin hatayı formu doldurduktan SONRA öğrenmesini önler.
    const { invalid } = splitOutcomes(outcomeText);
    if (invalid.length > 0)
      return `Kazanım kodu biçimsiz: ${invalid.join(", ")} — DERS.SINIF.ÜNİTE.KAZANIM bekleniyor.`;

    return null;
  });

  /**
   * Ders önerileri: bankada gerçekten kullanılanlar önce, sonra başlangıç
   * listesi. Banka okunamazsa yalnız başlangıç listesiyle devam edilir —
   * öneri listesi çalışmazsa soru yazmak durmamalı.
   */
  let subjectOptions = $state<string[]>(STARTER_SUBJECTS);
  /** Kazanım önerileri bankanın kendisinden türetiliyor. */
  let bank = $state<Question[]>([]);

  onMount(async () => {
    try {
      bank = await api.questions.list();
      subjectOptions = subjectSuggestions(bank);
    } catch {
      // Başlangıç listesi zaten yüklü.
    }
  });

  let saving = $state(false);
  let saveError = $state<string | null>(null);

  let outcomes = $derived(
    outcomeText
      .split(/[,\s]+/)
      .map((o) => o.trim())
      .filter(Boolean),
  );

  /**
   * Sorunun yapısı gövdenin KENDİSİNDEN okunur; ayrı bir form paneli yok.
   * Bu, öğretmenin tek yere bakmasını sağlar ve cevap anahtarının kâğıtta
   * görünenden ayrı düşmesini imkânsız kılar.
   */
  let parsed = $derived.by(() => {
    if (body.trim() === "") return "Soru gövdesi boş.";

    if (questionType === "multiple_choice") return parseOptions(body);
    if (questionType === "true_false") return parseTrueFalse(body);
    if (questionType === "fill_in_blank") return parseBlanks(body);
    return parseAnswerLines(body);
  });

  /** Kalıp hatası yazarken de görünür; kaydetmeyi beklemeye gerek yok. */
  let structureError = $derived(typeof parsed === "string" ? parsed : null);

  /**
   * Var olan soruyu, kimliğini ve istatistiğini KORUYARAK günceller.
   *
   * stats korunmak zorunda: madde analizi geçmişi soruya bağlıdır. Yeni bir
   * nesne üretip stats'ı sıfırlamak, o sorunun ölçülmüş tarihini silmek olur.
   */
  function buildUpdated(base: Question, bodyNodes: ContentNode[]): Question {
    const common = {
      id: base.id,
      meta,
      outcomes,
      body: bodyNodes,
      stats: base.stats,
    };

    if (questionType === "multiple_choice") {
      const { options, correctIndex, shuffle } = parsed as {
        options: string[];
        correctIndex: number;
        shuffle: boolean;
      };
      return {
        ...common,
        question_type: "multiple_choice",
        points,
        options: options.map((source, i) => ({
          id: OPTION_LETTERS[i],
          body: typstBody(source),
          correct: i === correctIndex,
        })),
        shuffle,
      };
    }

    if (questionType === "true_false") {
      return {
        ...common,
        question_type: "true_false",
        points,
        correct_answer: parsed as boolean,
      };
    }

    if (questionType === "fill_in_blank") {
      const blanks = parsed as Array<{ accepted: string[] }>;
      return {
        ...common,
        question_type: "fill_in_blank",
        blanks: blanks.map((b, i) => ({
          id: `b${i + 1}`,
          accepted_answers: b.accepted,
          points,
          case_sensitive: false,
        })),
      };
    }

    return {
      ...common,
      question_type: "classic",
      points,
      sample_answer: sampleAnswer.trim() === "" ? null : typstBody(sampleAnswer),
      rubric,
      answer_space: { Lines: parsed as number },
    };
  }

  async function save() {
    if (metaError !== null) {
      saveError = metaError;
      return;
    }

    if (structureError !== null) {
      saveError = structureError;
      return;
    }

    // Rust ClassicQuestion::validate gövdedeki #rubrik( yüzünden reddediyor.
    // Sebebi burada söylemek, öğretmeni ham hata mesajıyla baş başa bırakmıyor.
    if (questionType === "classic" && (hasRubricCall(body) || hasRubricCall(sampleAnswer))) {
      saveError =
        "Kaynakta #rubrik( var. Panele taşımadan kaydedilemez: kaynağa " +
        "yazılan rubrik ne cevap anahtarına ne de sonuç girişine yansır.";
      return;
    }

    // Rust zaten reddediyor; burada durdurmak sebebi ANLAŞILIR kılıyor.
    if (rubricError !== null) {
      saveError = rubricError;
      return;
    }

    saving = true;
    saveError = null;

    try {
      const bodyNodes = typstBody(body);

      if (existing) {
        await api.questions.update(buildUpdated(existing, bodyNodes));
        await goto("/questions");
        return;
      }

      if (questionType === "multiple_choice") {
        const { options, correctIndex, shuffle } = parsed as {
          options: string[];
          correctIndex: number;
          shuffle: boolean;
        };
        await api.questions.addMultipleChoice({
          meta,
          points,
          outcomes,
          body: bodyNodes,
          options: options.map((source, i) => ({
            id: OPTION_LETTERS[i],
            body: typstBody(source),
            correct: i === correctIndex,
          })),
          shuffle,
        });
      } else if (questionType === "true_false") {
        await api.questions.addTrueFalse({
          meta,
          points,
          outcomes,
          body: bodyNodes,
          correct_answer: parsed as boolean,
        });
      } else if (questionType === "fill_in_blank") {
        const blanks = parsed as Array<{ accepted: string[] }>;
        await api.questions.addFillInBlank({
          meta,
          outcomes,
          body: bodyNodes,
          blanks: blanks.map((b, i) => ({
            id: `b${i + 1}`,
            accepted_answers: b.accepted,
            points,
          })),
        });
      } else {
        await api.questions.addClassic({
          meta,
          points,
          outcomes,
          body: bodyNodes,
          sample_answer: sampleAnswer.trim() === "" ? null : typstBody(sampleAnswer),
          rubric,
          answer_space: { Lines: parsed as number },
        });
      }

      await goto("/questions");
    } catch (err: unknown) {
      saveError = errorText(err);
    } finally {
      saving = false;
    }
  }
</script>

<div class="flex h-full min-h-0 flex-col">
  {#if legacyWarning}
    <div class="shrink-0 px-4 pt-3">
      <Alert color="amber">
        Bu soru eski zengin metin editörüyle yazılmış. Typst kaynağına çevrildi;
        kaydedersen bu çeviri kalıcı olur.
      </Alert>
    </div>
  {/if}

  <!--
    Kalıp hatası (structureError) artık burada BASILMAZ: panelde, kazanım ve
    puanın hemen altında duruyor. Kaydetme hatası ise ayrı bir şey — ağa/diske
    giden bir çağrının başarısızlığı — ve üstte kalır.
  -->
  <!--
    Gövdeye yazılmış rubrik SESSİZ KALMAZ. Kaydetme zaten kilitli; burada
    öğretmen ya tek tıkla panele taşır ya da neden okunamadığını görür.
  -->
  {#if govdedekiRubrik}
    <div class="shrink-0 px-4 pt-3">
      {#if govdedekiRubrik.sonuc.ok}
        <Alert color="amber">
          <p>
            {govdedekiRubrik.kaynak === "body" ? "Soru gövdesinde" : "Örnek cevapta"}
            {govdedekiRubrik.sonuc.items.length} ölçütlük bir
            <span class="font-mono">#rubrik(…)</span> bloğu var. Ölçütler panelden
            yönetilir; kaynakta kalırsa ne cevap anahtarına ne sonuç girişine yansır.
          </p>
          <Button size="xs" color="light" class="mt-2" onclick={rubrigiPaneleTasi}>
            Panele taşı ve gövdeden kaldır
          </Button>
        </Alert>
      {:else}
        <Alert color="red">
          Kaynaktaki <span class="font-mono">#rubrik(…)</span> okunamadı:
          {govdedekiRubrik.sonuc.reason} Yalnız düz
          <span class="font-mono">([ölçüt], puan)</span> demetleri taşınabiliyor —
          değişken, hesaplanmış puan ve döngü okunmuyor. Ölçütleri panele elle
          gir ve bloğu kaynaktan sil.
        </Alert>
      {/if}
    </div>
  {/if}

  {#if saveError}
    <div class="shrink-0 px-4 pt-3">
      <Alert color="red">{saveError}</Alert>
    </div>
  {/if}

  <div class="min-h-0 flex-1">
    <QuestionEditor
      {body}
      {questionType}
      {outcomeText}
      {points}
      {rubric}
      {sampleAnswer}
      {stats}
      structureError={structureError ?? metaError}
      {meta}
      {subjectOptions}
      {bank}
      onmetachange={(next) => (meta = next)}
      {saving}
      saveLabel={saving ? "Kaydediliyor…" : existing ? "Güncelle" : "Kaydet"}
      onbodychange={(next) => (body = next)}
      onquestiontypechange={(next) => (questionType = next)}
      onoutcometextchange={(next) => (outcomeText = next)}
      onpointschange={(next) => (points = next)}
      onrubricchange={(next) => (rubric = next)}
      onsampleanswerchange={(next) => (sampleAnswer = next)}
      onsave={save}
    />
  </div>
</div>

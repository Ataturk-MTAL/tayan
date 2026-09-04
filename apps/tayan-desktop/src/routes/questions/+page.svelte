<script lang="ts">
  import { onMount } from "svelte";
  import { Alert, Badge, Button, Spinner } from "flowbite-svelte";
  import PageShell from "$lib/components/shell/PageShell.svelte";
  import QuestionCard from "$lib/components/question/QuestionCard.svelte";
  import { api } from "$lib/api";
  import { errorText } from "$lib/editor/diagnostics";
  import { bodySource } from "$lib/question/body";
  import {
    QUESTION_TYPE_LABELS,
    questionPoints,
    scoreBadge,
    type Question,
    type ScoreBadge,
  } from "$lib/types";
  import { goto } from "$app/navigation";

  let questions = $state<Question[]>([]);
  let loading = $state(true);
  let loadError = $state<string | null>(null);
  let filter = $state<"all" | "untested" | "weak">("all");

  const BADGE_LABEL: Record<ScoreBadge, string> = {
    excellent: "Çok iyi",
    good: "İyi",
    fair: "Orta",
    poor: "Zayıf",
    untested: "Denenmemiş",
  };
  /*
   * Ölçüm skoru düştükçe rozet kırmızıya kayar — kırmızı bu uygulamada yalnız
   * değerlendirme rengi (bkz. app.css), skorun kötü olduğunu göz gezdirerek
   * anlatması gereken tek yer burası.
   */
  const BADGE_COLOR: Record<ScoreBadge, "green" | "lime" | "yellow" | "red" | "gray"> = {
    excellent: "green",
    good: "lime",
    fair: "yellow",
    poor: "red",
    untested: "gray",
  };

  onMount(load);

  async function load() {
    loading = true;
    try {
      questions = await api.questions.list();
      loadError = null;
    } catch (err: unknown) {
      loadError = errorText(err);
    } finally {
      loading = false;
    }
  }

  let shown = $derived(
    questions.filter((q) => {
      if (filter === "untested") return q.stats.times_used === 0;
      if (filter === "weak")
        return q.stats.times_used > 0 && q.stats.discrimination_index < 0.2;
      return true;
    }),
  );
</script>

<PageShell
  title="Soru bankası"
  subtitle={loading ? null : `${shown.length} / ${questions.length} soru`}
>
  {#snippet actions()}
    <Button color="alternative" size="sm" onclick={() => (filter = "all")}>Tümü</Button>
    <Button color="alternative" size="sm" onclick={() => (filter = "untested")}>Denenmemiş</Button>
    <Button color="red" size="sm" onclick={() => (filter = "weak")}>Zayıf ayırt edici</Button>
    <Button size="sm" onclick={() => goto("/questions/new")}>Soru yaz</Button>
  {/snippet}

  {#if loading}
    <div class="flex items-center gap-2 text-gray-500 dark:text-gray-400">
      <Spinner size="5" />
      Banka okunuyor…
    </div>
  {:else if loadError}
    <Alert color="red">{loadError}</Alert>
  {:else if shown.length === 0}
    <p class="text-gray-500 dark:text-gray-400">
      {questions.length === 0
        ? "Bankada henüz soru yok."
        : "Bu süzgece uyan soru yok."}
    </p>
  {:else}
    <!--
      Fiş dizimi: her soru dizilmiş hâliyle bir kâğıt parçası. Tablo, soruyu
      ham Typst kaynağı olarak gösteriyordu; öğretmen kendi bankasında
      `#secenekler(dogru:` görüyordu.
    -->
    <div class="grid gap-4" style="grid-template-columns: repeat(auto-fill, minmax(260px, 1fr))">
      {#each shown as q (q.id)}
        {@const badge = scoreBadge(q.stats)}
        <button
          type="button"
          class="flex flex-col overflow-hidden rounded-lg border border-gray-200 bg-white
                 text-left shadow-sm transition-shadow hover:shadow-md
                 dark:border-gray-700 dark:bg-gray-800"
          onclick={() => goto(`/questions/${q.id}`)}
        >
          <QuestionCard body={bodySource(q.body)} />

          <div
            class="flex flex-col gap-1 border-t border-gray-200 bg-gray-50 px-3 py-2
                   dark:border-gray-700 dark:bg-gray-900/40"
          >
            <div class="flex items-baseline gap-2">
              <span
                class="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400"
              >
                {QUESTION_TYPE_LABELS[q.question_type]}
              </span>
              <span class="ml-auto tnum text-sm text-gray-500 dark:text-gray-400">
                {questionPoints(q)} p
              </span>
            </div>

            <!--
              Uzun bir ders adı ("Bilişim Teknolojileri ve Yazılım Geliştirme")
              ya da boşluksuz uzun bir belirteç burada sessizce yok oluyordu: ne
              üç nokta çıkıyordu ne kaydırma. İki span flex öğesi ve varsayılan
              min-width:auto ile min-content'in altına inemiyor, taşan kısmı da
              ebeveyn button'daki overflow-hidden kırpıyordu — düzen bozulmuyor
              ama metnin sonu görünmez oluyordu. min-w-0 kabı küçülebilir yapıyor
              (o olmadan truncate hiç çalışmaz, çünkü ebeveyn daralamaz), truncate
              kesme yerine üç nokta gösteriyor, title ile tam metin fare üstünde
              okunabilir kalıyor. flex-wrap dar kartta ders adını ve sınıf
              bilgisini alt alta alıyor; shrink-0 ise "9. sınıf" ibaresinin
              kelime kelime kırılmasını engelliyor.
            -->
            <div class="flex flex-wrap items-baseline gap-x-2 gap-y-0.5 text-[11px] leading-tight">
              {#if q.meta?.subject}
                <span
                  class="min-w-0 truncate text-gray-700 dark:text-gray-300"
                  title={q.meta.subject}
                >
                  {q.meta.subject}
                </span>
              {/if}
              {#if q.meta?.grade}
                <span class="shrink-0 text-gray-500 dark:text-gray-400">{q.meta.grade}. sınıf</span>
              {/if}
            </div>

            <!--
              Kazanım kodları ("MAT.9.1.2.3") boşluksuz tek parça belirteçler;
              varsayılan overflow-wrap: normal onları hiçbir yerden kıramaz.
              Ölçüm (minWidth=1024): 1024 − 225 (çekmece + border) − 48
              (PageShell p-6) = 751px, auto-fill minmax(260px, 1fr) burada 2
              sütun veriyor ⇒ (751 − 16 gap) / 2 ≈ 367px, px-3 sonrası içeriğe
              343px; ızgaranın ilan ettiği taban ise 260px ⇒ 236px. 10px
              font-mono ≈ 6px/karakter olduğundan bölünemez tek bir kod 39–57
              karakteri aşınca ebeveyn button'ın overflow-hidden'ı onu SESSİZCE
              kırpıyordu — hemen yukarıdaki yorumun mahkûm ettiği başarısızlığın
              aynısı.
              wrap-anywhere seçildi, QuestionInspector'daki break-all değil:
              orada her li tek bir kod, burada kodlar " · " ile birleşmiş tek
              bir metin ve break-all sığan kodları da ortadan bölerdi.
              overflow-wrap: anywhere önce ayıraçtan kırar, yalnızca tek başına
              satıra sığmayan kodu böler. Metin kesilmediği için `title`
              gerekmiyor: kodun tamamı ekranda kalıyor.
            -->
            {#if q.outcomes.length > 0}
              <span class="font-mono text-[10px] wrap-anywhere text-gray-500 dark:text-gray-400">
                {q.outcomes.join(" · ")}
              </span>
            {/if}

            <div class="mt-1 flex items-center gap-2">
              <Badge color={BADGE_COLOR[badge]}>{BADGE_LABEL[badge]}</Badge>
              {#if q.stats.times_used > 0}
                <span class="ml-auto tnum text-[11px] text-gray-500 dark:text-gray-400">
                  {q.stats.discrimination_index.toFixed(2)}
                </span>
              {/if}
            </div>
          </div>
        </button>
      {/each}
    </div>
  {/if}
</PageShell>

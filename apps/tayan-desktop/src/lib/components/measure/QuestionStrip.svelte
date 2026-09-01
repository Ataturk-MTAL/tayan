<script lang="ts">
  import { questionPoints, scoreBadge, type Question, type ScoreBadge } from "$lib/types";

  /**
   * Sınavın kesintisiz soru şeridi.
   *
   * Sarılmaz, ölçeklenir: sayfanın tek kesintisiz çizgisi olarak durur ve
   * sınavın tamamı tek bakışta okunur. Her tuşun durumu tuşun KENDİSİNDE
   * yazılıdır — ayrı bir açıklama sütununa bakmak gerekmez.
   *
   * Kırmızı burada da yalnızca değerlendirme demektir: kırmızı bir tuş, o
   * sorunun ölçülmüş ve zayıf çıkmış olduğunu söyler.
   */
  type Props = {
    questions: Question[];
    activeId?: string | null;
    onselect?: (id: string) => void;
  };

  let { questions, activeId = null, onselect }: Props = $props();

  const BADGE_COLOR: Record<ScoreBadge, string> = {
    excellent: "var(--color-mark-excellent)",
    good: "var(--color-mark-good)",
    fair: "var(--color-mark-fair)",
    poor: "var(--color-mark-poor)",
    untested: "var(--color-rule-strong)",
  };

  const BADGE_LABEL: Record<ScoreBadge, string> = {
    excellent: "çok iyi",
    good: "iyi",
    fair: "orta",
    poor: "zayıf",
    untested: "denenmemiş",
  };
</script>

{#if questions.length > 0}
  <div class="ruled-bottom flex shrink-0 items-stretch bg-paper paper-plain">
    <span class="stamp flex items-center px-rule">Sınav</span>

    <!-- Sarma yok: şerit bir liste değil, tek bir çizgi. -->
    <div class="flex min-w-0 flex-1 items-stretch">
      {#each questions as q, i (q.id)}
        {@const badge = scoreBadge(q.stats)}
        <button
          type="button"
          class="group relative min-w-0 flex-1 border-l border-rule px-quarter py-half
                 text-center transition-colors hover:bg-paper-lift"
          class:bg-paper-lift={q.id === activeId}
          title="{i + 1}. soru · {questionPoints(q)} puan · {BADGE_LABEL[badge]}"
          onclick={() => onselect?.(q.id)}
        >
          <span class="block text-[13px] leading-rule tnum" class:font-bold={q.id === activeId}>
            {i + 1}
          </span>
          <!-- Durum tuşun kendisinde: altındaki çizgi ölçümün rengidir. -->
          <span
            class="mt-quarter block h-[3px] w-full"
            style="background: {BADGE_COLOR[badge]}"
            aria-hidden="true"
          ></span>
        </button>
      {/each}
    </div>

    <span class="stamp flex items-center border-l border-rule px-rule tnum">
      {questions.length} soru
    </span>
  </div>
{/if}

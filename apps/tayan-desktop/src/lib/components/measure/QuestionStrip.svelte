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

  /**
   * Beş düzeyli rozet Tailwind sınıfı olarak.
   *
   * Eskiden CSS değişkeniydi (`--color-mark-*`); Flowbite temasında bu
   * değişkenler yok. Beş düzey birbirinden ayrı renk taşımalı — hepsini
   * gri/kırmızıya indirmek "çok iyi" ile "orta"yı ayırt edilemez yapardı.
   * Kırmızı yalnız "zayıf" düzeyinde: kural burada da geçerli, kırmızı
   * yalnızca değerlendirmenin en olumsuz ucunu işaretliyor.
   */
  const BADGE_CLASS: Record<ScoreBadge, string> = {
    excellent: "bg-green-700 dark:bg-green-400",
    good: "bg-primary-600 dark:bg-primary-400",
    fair: "bg-amber-500 dark:bg-amber-400",
    poor: "bg-red-600 dark:bg-red-400",
    untested: "bg-gray-300 dark:bg-gray-600",
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
  <div class="flex shrink-0 items-stretch border-b border-gray-300 bg-white dark:border-gray-600 dark:bg-gray-800">
    <span
      class="flex items-center px-5 text-[11px] font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400"
    >
      Sınav
    </span>

    <!-- Sarma yok: şerit bir liste değil, tek bir çizgi. -->
    <div class="flex min-w-0 flex-1 items-stretch">
      {#each questions as q, i (q.id)}
        {@const badge = scoreBadge(q.stats)}
        <button
          type="button"
          class="group relative min-w-0 flex-1 border-l border-gray-200 px-[5px] py-2.5
                 text-center transition-colors hover:bg-gray-50 dark:border-gray-700 dark:hover:bg-gray-700
                 {q.id === activeId ? 'bg-gray-50 dark:bg-gray-700' : ''}"
          title="{i + 1}. soru · {questionPoints(q)} puan · {BADGE_LABEL[badge]}"
          onclick={() => onselect?.(q.id)}
        >
          <span class="tnum block text-[13px] leading-5" class:font-bold={q.id === activeId}>
            {i + 1}
          </span>
          <!-- Durum tuşun kendisinde: altındaki çizgi ölçümün rengidir. -->
          <span
            class="mt-[5px] block h-[3px] w-full {BADGE_CLASS[badge]}"
            aria-hidden="true"
          ></span>
        </button>
      {/each}
    </div>

    <span
      class="tnum flex items-center border-l border-gray-200 px-5 text-[11px] font-semibold uppercase
             tracking-wider text-gray-500 dark:border-gray-700 dark:text-gray-400"
    >
      {questions.length} soru
    </span>
  </div>
{/if}

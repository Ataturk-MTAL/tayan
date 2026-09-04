<script lang="ts">
  import type { ExamResult, Student } from "$lib/types";

  /**
   * Soru × öğrenci ızgarası. Hücre boyutu defterin karesiyle aynıdır (20px):
   * bu bir benzetme değil, aynı ızgara.
   *
   * Kodlama önce ŞEKİLDİR, sonra renk: ✓ / ✗ / ⁄ işaretleri anlamı tek başına
   * taşır. Renk körlüğü, siyah-beyaz fotokopi ve zorunlu renk kipinde de okunur.
   * Yanlış hücrelerin ardındaki soluk kırmızı yıkama yalnızca tarama kolaylığı
   * içindir, bilgi taşımaz.
   */
  type Props = {
    results: ExamResult[];
    students: Student[];
    questionIds: string[];
  };

  let { results, students, questionIds }: Props = $props();

  type Cell = { state: "correct" | "wrong" | "partial" | "blank"; title: string };

  function cellFor(result: ExamResult | undefined, questionId: string, index: number): Cell {
    const answer = result?.answers.find((a) => a.question_id === questionId);
    const label = `${index + 1}. soru`;

    if (!answer) return { state: "blank", title: `${label}: cevaplanmadı` };
    if (answer.is_correct === true) return { state: "correct", title: `${label}: doğru` };
    if (answer.is_correct === false) return { state: "wrong", title: `${label}: yanlış` };
    return {
      state: answer.points_earned > 0 ? "partial" : "blank",
      title: `${label}: ${answer.points_earned} puan`,
    };
  }

  const GLYPH = { correct: "✓", wrong: "✗", partial: "⁄", blank: "" } as const;

  let rows = $derived(
    students.map((student) => {
      const result = results.find((r) => r.student_id === student.id);
      return {
        student,
        cells: questionIds.map((qid, i) => cellFor(result, qid, i)),
        percent:
          result && result.total_points_max > 0
            ? Math.round((result.total_points_earned / result.total_points_max) * 100)
            : null,
      };
    }),
  );

  /** Sütun başına yanlış sayısı: hangi sorunun sınıfı devirdiği kenarda okunur. */
  let wrongPerQuestion = $derived(
    questionIds.map(
      (_, col) => rows.filter((row) => row.cells[col].state === "wrong").length,
    ),
  );
</script>

<figure
  class="m-0 rounded-lg border border-gray-200 bg-white p-4 shadow-sm dark:border-gray-700 dark:bg-gray-800"
>
  <figcaption class="text-[11px] font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">
    Soru × öğrenci
  </figcaption>

  <div class="mt-2.5 flex flex-wrap items-center gap-5 text-[12px] leading-5 text-gray-500 dark:text-gray-400">
    <span><span class="text-gray-800 dark:text-gray-200">✓</span> doğru</span>
    <span><span class="text-red-600 dark:text-red-400">✗</span> yanlış</span>
    <span><span class="text-red-600 dark:text-red-400">⁄</span> kısmi</span>
    <span>boş: cevaplanmadı</span>
  </div>

  {#if rows.length === 0}
    <p class="mt-2.5 text-[12px] leading-5 text-gray-500 dark:text-gray-400">Bu sınıfta öğrenci yok.</p>
  {:else}
    <div class="mt-2.5 overflow-auto">
      <table class="border-collapse">
        <thead>
          <tr>
            <th
              class="sticky left-0 bg-white px-2.5 text-left text-[11px] font-semibold uppercase
                     tracking-wider text-gray-500 dark:bg-gray-800 dark:text-gray-400"
            >
              Öğrenci
            </th>
            {#each questionIds as _, i}
              <th
                class="tnum w-[20px] text-center text-[11px] font-normal uppercase tracking-wider
                       text-gray-500 dark:text-gray-400"
              >
                {i + 1}
              </th>
            {/each}
            <th
              class="px-2.5 text-right text-[11px] font-semibold uppercase tracking-wider
                     text-gray-500 dark:text-gray-400"
            >
              %
            </th>
          </tr>
        </thead>
        <tbody>
          {#each rows as row (row.student.id)}
            <tr>
              <th
                class="sticky left-0 whitespace-nowrap bg-white px-2.5 text-left text-[12px]
                       font-normal leading-[20px] text-gray-900 dark:bg-gray-800 dark:text-white"
              >
                <span class="tnum text-gray-500 dark:text-gray-400">{row.student.number}</span>
                {row.student.first_name}
                {row.student.last_name}
              </th>
              {#each row.cells as cell}
                <td
                  class="h-[20px] w-[20px] border border-gray-200 text-center text-[12px]
                         leading-[18px] dark:border-gray-700"
                  class:bg-red-50={cell.state === "wrong"}
                  class:dark:bg-red-950={cell.state === "wrong"}
                  class:text-red-600={cell.state === "wrong" || cell.state === "partial"}
                  class:dark:text-red-400={cell.state === "wrong" || cell.state === "partial"}
                  class:text-gray-800={cell.state === "correct"}
                  class:dark:text-gray-200={cell.state === "correct"}
                  title={cell.title}
                >
                  {GLYPH[cell.state]}
                </td>
              {/each}
              <td class="tnum px-2.5 text-right text-[12px] leading-[20px]">
                {row.percent ?? "—"}
              </td>
            </tr>
          {/each}

          <tr>
            <th
              class="sticky left-0 bg-white px-2.5 text-left text-[11px] font-semibold uppercase
                     tracking-wider text-gray-500 dark:bg-gray-800 dark:text-gray-400"
            >
              Yanlış
            </th>
            {#each wrongPerQuestion as count}
              <td
                class="tnum border-t border-gray-300 text-center text-[11px] leading-[20px] dark:border-gray-600"
                class:text-red-600={count > rows.length / 2}
                class:dark:text-red-400={count > rows.length / 2}
                class:font-semibold={count > rows.length / 2}
              >
                {count}
              </td>
            {/each}
            <td></td>
          </tr>
        </tbody>
      </table>
    </div>
  {/if}
</figure>

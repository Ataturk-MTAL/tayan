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

<figure class="m-0">
  <figcaption class="stamp">Soru × öğrenci</figcaption>

  <div class="mt-half flex flex-wrap items-center gap-rule">
    <span class="pencil"><span class="text-ink">✓</span> doğru</span>
    <span class="pencil"><span class="text-red">✗</span> yanlış</span>
    <span class="pencil"><span class="text-red">⁄</span> kısmi</span>
    <span class="pencil">boş: cevaplanmadı</span>
  </div>

  {#if rows.length === 0}
    <p class="pencil mt-half">Bu sınıfta öğrenci yok.</p>
  {:else}
    <div class="mt-half overflow-auto">
      <table class="border-collapse">
        <thead>
          <tr>
            <th class="stamp sticky left-0 bg-paper px-half text-left">Öğrenci</th>
            {#each questionIds as _, i}
              <th class="stamp w-[20px] text-center font-normal tnum">{i + 1}</th>
            {/each}
            <th class="stamp px-half text-right">%</th>
          </tr>
        </thead>
        <tbody>
          {#each rows as row (row.student.id)}
            <tr>
              <th
                class="sticky left-0 whitespace-nowrap bg-paper px-half text-left text-[12px]
                       font-normal leading-[20px]"
              >
                <span class="text-pencil tnum">{row.student.number}</span>
                {row.student.first_name}
                {row.student.last_name}
              </th>
              {#each row.cells as cell}
                <td
                  class="h-[20px] w-[20px] border border-rule text-center text-[12px] leading-[18px]"
                  class:bg-red-wash={cell.state === "wrong"}
                  class:text-red={cell.state === "wrong" || cell.state === "partial"}
                  class:text-ink={cell.state === "correct"}
                  title={cell.title}
                >
                  {GLYPH[cell.state]}
                </td>
              {/each}
              <td class="px-half text-right text-[12px] leading-[20px] tnum">
                {row.percent ?? "—"}
              </td>
            </tr>
          {/each}

          <tr>
            <th class="stamp sticky left-0 bg-paper px-half text-left">Yanlış</th>
            {#each wrongPerQuestion as count}
              <td
                class="border-t border-rule-strong text-center text-[11px] leading-[20px] tnum"
                class:text-red-deep={count > rows.length / 2}
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

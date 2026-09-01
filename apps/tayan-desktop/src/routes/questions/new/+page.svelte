<script lang="ts">
  import QuestionForm from "$lib/components/question/QuestionForm.svelte";

  /**
   * Boş bir ekran yerine çalışan bir örnekle başlar. Öğretmen kalıbın nasıl
   * göründüğünü önce görür, sonra üstüne yazar — Typst'i böyle öğrenir.
   */
  const STARTER = `Aşağıdaki denklemin köklerini bulunuz.

$ x^2 - 5x + 6 = 0 $

#secenekler(dogru: "C",
  [$x = 1$],
  [$x = 2$],
  [$x = 2$ ve $x = 3$],
  [$x = 6$],
  [Hiçbiri],
)
`;
</script>

<QuestionForm initialBody={STARTER} />

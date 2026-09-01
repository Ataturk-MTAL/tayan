<script lang="ts">
  import QuestionForm from "$lib/components/question/QuestionForm.svelte";

  const STARTER = `Aşağıdaki denklemin köklerini bulunuz.

$ x^2 - 5x + 6 = 0 $
`;
</script>

<QuestionForm initialBody={STARTER} />

<script lang="ts">
  import PageShell from "$lib/components/shell/PageShell.svelte";
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

<!--
  scroll={false}: editör kendi yüksekliğini ve kendi kaydırıcılarını
  yönetiyor (kaynak, önizleme, panel — üçü de kendi içinde kayar). Kabuk
  kaydırmayı üstlenirse iç içe iki kaydırıcı fare tekerleğinin hangisini
  süreceğini belirsizleştirirdi.
-->
<PageShell title="Yeni soru" scroll={false}>
  <QuestionForm initialBody={STARTER} />
</PageShell>

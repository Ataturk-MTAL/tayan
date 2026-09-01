<script lang="ts">
  import { blocksFor, type QuestionType } from "$lib/question/templates";

  /**
   * Blok şeridi, pinlenmiş kısıtın taşıyıcısıdır: öğretmen ham Typst yazmaya
   * zorlanmaz, ama tıkladığı her blok kaynağa GÖRÜNÜR biçimde düşer. Neden-sonuç
   * açıkta kaldığı için Typst gizlenmeden öğretilir.
   *
   * Soru kalıpları (şıklar, doğru/yanlış, cevap alanı) solda ve vurgulu durur:
   * o soru tipinin gövdesi onlarsız tamamlanmaz.
   */
  type Props = {
    questionType: QuestionType;
    oninsert: (snippet: string) => void;
  };

  let { questionType, oninsert }: Props = $props();

  let blocks = $derived(blocksFor(questionType));
</script>

<div class="ruled-bottom flex flex-wrap items-center gap-quarter bg-paper px-rule py-half paper-plain">
  <span class="stamp mr-half">Kalıp</span>
  {#each blocks.templates as block}
    <button
      type="button"
      class="border border-red bg-red-wash px-half py-quarter text-[12px] leading-rule
             text-red-deep transition-colors hover:bg-paper-lift"
      title={block.hint}
      onclick={() => oninsert(block.snippet)}
    >
      {block.label}
    </button>
  {/each}

  <span class="stamp mx-half border-l border-rule pl-half">Ekle</span>
  {#each blocks.common as block}
    <button
      type="button"
      class="border border-rule-strong bg-paper-lift px-half py-quarter text-[12px] leading-rule
             text-ink transition-colors hover:border-red hover:text-red-deep"
      title={block.hint}
      onclick={() => oninsert(block.snippet)}
    >
      {block.label}
    </button>
  {/each}
</div>

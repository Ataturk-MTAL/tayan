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
  import { saveImageAsTypst } from "$lib/question/image";
  import { errorText } from "$lib/editor/diagnostics";

  type Props = {
    questionType: QuestionType;
    oninsert: (snippet: string) => void;
  };

  let { questionType, oninsert }: Props = $props();

  let blocks = $derived(blocksFor(questionType));

  let fileInput = $state<HTMLInputElement | undefined>(undefined);
  let imageError = $state<string | null>(null);
  let saving = $state(false);

  /**
   * Görsel elle yol yazılarak eklenmez. Dosya seçilir, uygulama veri klasörüne
   * kopyalanır ve gövdeye GÖRELİ yolla eklenir — böylece veri taşındığında
   * kırılmaz.
   */
  async function handleFile(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    input.value = ""; // aynı dosya art arda seçilebilsin
    if (!file) return;

    saving = true;
    imageError = null;
    try {
      oninsert(await saveImageAsTypst(file));
    } catch (err: unknown) {
      imageError = errorText(err);
    } finally {
      saving = false;
    }
  }
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
  <button
    type="button"
    class="border border-rule-strong bg-paper-lift px-half py-quarter text-[12px] leading-rule
           text-ink transition-colors hover:border-red hover:text-red-deep disabled:opacity-40"
    title="Dosyadan görsel seç — kopyalanır ve göreli yolla eklenir"
    disabled={saving}
    onclick={() => fileInput?.click()}
  >
    {saving ? "Ekleniyor…" : "Görsel"}
  </button>

  <input
    type="file"
    accept="image/png,image/jpeg,image/gif,image/webp"
    class="hidden"
    bind:this={fileInput}
    onchange={handleFile}
  />

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

  {#if imageError}
    <span class="annot w-full">{imageError}</span>
  {/if}
</div>

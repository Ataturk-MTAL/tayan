<script lang="ts">
  /**
   * Kalıp paleti — yüzen, gruplu.
   *
   * Önceki hâli (`BlockPalette`) tam genişlik bir şeritti ve 17 düğmeyi tek
   * sırada sarmalıyordu: bir soru tipi kalıbı + 15 genel blok + Görsel. Hiçbir
   * gruplama yoktu, `Kesir` ile `Şekil + başlık` aynı ağırlıkta duruyordu ve
   * şerit editörün üstünden kalıcı olarak yer yiyordu.
   *
   * Şimdi kapalı başlar (tek pill), açılınca gruplu bir kâğıt tabakası olarak
   * kaynağın üzerinde yüzer. Ekranda aynı anda 4-10 seçenek durur, 17 değil.
   *
   * Pinlenmiş kısıt burada da geçerli: tıklanan her blok kaynağa GÖRÜNÜR Typst
   * olarak düşer. Öğretmen Typst yazmaya zorlanmaz ama her seferinde görür.
   */
  import { groupsFor, type QuestionType } from "$lib/question/templates";
  import { saveImageAsTypst } from "$lib/question/image";
  import { errorText } from "$lib/editor/diagnostics";

  type Props = {
    questionType: QuestionType;
    oninsert: (snippet: string) => void;
  };

  let { questionType, oninsert }: Props = $props();

  let groups = $derived(groupsFor(questionType));

  let open = $state(false);
  let activeId = $state("kalip");
  let root = $state<HTMLDivElement | undefined>(undefined);

  /**
   * Soru tipi değişince kalıp grubunun içeriği değişir ama sekme kimliği aynı
   * kalır — öğretmen matematik eklerken tip değiştirdiğinde sekmenin altından
   * kayması gereksiz bir sürpriz olur.
   */
  let activeGroup = $derived(groups.find((g) => g.id === activeId) ?? groups[0]);

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

  /**
   * Esc ve dışarı tık paneli kapatır.
   *
   * UYARI: Esc şu an burada, çünkü kapatılacak tek katman bu. Sınav çekmecesi,
   * yardım paneli ve boş-belge tip seçici geldiğinde bu dinleyici KALDIRILMALI
   * ve tek sıralı bir Esc merdivenine taşınmalıdır. Katman başına ayrı Esc
   * dinleyicisi, ekranda olmayan bir yığına karşı çözülen Esc demektir
   * (docs/UI-IA.md §9.5).
   */
  function onKeydown(event: KeyboardEvent) {
    if (event.key === "Escape" && open) {
      open = false;
      event.stopPropagation();
    }
  }

  function onWindowPointerDown(event: PointerEvent) {
    if (!open || !root) return;
    if (!root.contains(event.target as Node)) open = false;
  }
</script>

<svelte:window onkeydown={onKeydown} onpointerdown={onWindowPointerDown} />

<!--
  Kâğıt tabakası, kart değil: köşesiz, cetvelli kenarlı, hafif gölgeli.
  Kırmızı yalnız kalıp düğmelerinde — o vurgu şeritten devralındı, çünkü
  soru gövdesi o kalıp olmadan tamamlanmaz.
-->
<div bind:this={root} class="pointer-events-none absolute inset-0 z-10">
  {#if open}
    <div
      class="shadow-float pointer-events-auto absolute right-rule bottom-rule w-[300px]
             border border-rule-strong bg-paper-lift"
    >
      <div class="ruled-bottom flex items-stretch">
        {#each groups as group (group.id)}
          <button
            type="button"
            class="stamp flex-1 border-r border-rule px-quarter py-quarter
                   transition-colors hover:text-red-deep"
            class:bg-paper-sunk={group.id === activeId}
            class:text-ink={group.id === activeId}
            aria-pressed={group.id === activeId}
            onclick={() => (activeId = group.id)}
          >
            {group.label}
          </button>
        {/each}
        <button
          type="button"
          class="stamp px-half py-quarter transition-colors hover:text-red-deep"
          title="Paneli kapat — Esc"
          aria-label="Paleti kapat"
          onclick={() => (open = false)}
        >
          ✕
        </button>
      </div>

      <div class="grid grid-cols-2 gap-quarter p-half">
        {#each activeGroup.blocks as block (block.label)}
          <button
            type="button"
            class="border px-half py-quarter text-left text-[12px] leading-rule
                   transition-colors hover:border-red hover:text-red-deep"
            class:border-red={activeGroup.id === "kalip"}
            class:bg-red-wash={activeGroup.id === "kalip"}
            class:text-red-deep={activeGroup.id === "kalip"}
            class:border-rule-strong={activeGroup.id !== "kalip"}
            class:bg-paper={activeGroup.id !== "kalip"}
            class:text-ink={activeGroup.id !== "kalip"}
            title={block.hint}
            onclick={() => oninsert(block.snippet)}
          >
            {block.label}
          </button>
        {/each}
      </div>

      <div class="ruled-top flex items-center gap-half px-half py-quarter">
        <span class="stamp">Ekle</span>
        <button
          type="button"
          class="border border-rule-strong bg-paper px-half py-quarter text-[12px]
                 leading-rule text-ink transition-colors hover:border-red
                 hover:text-red-deep disabled:opacity-40"
          title="Dosyadan görsel seç — kopyalanır ve göreli yolla eklenir"
          disabled={saving}
          onclick={() => fileInput?.click()}
        >
          {saving ? "Ekleniyor…" : "Görsel"}
        </button>
      </div>

      {#if imageError}
        <p class="annot ruled-top px-half py-quarter">{imageError}</p>
      {/if}
    </div>
  {:else}
    <button
      type="button"
      class="shadow-float pointer-events-auto absolute right-rule bottom-rule flex
             items-center gap-quarter border border-rule-strong bg-paper-lift
             px-half py-quarter text-[12px] leading-rule text-ink transition-colors
             hover:border-red hover:text-red-deep"
      aria-expanded="false"
      title="Kalıp ve matematik parçaları"
      onclick={() => (open = true)}
    >
      <span aria-hidden="true">⊞</span> Ekle
    </button>
  {/if}
</div>

<input
  type="file"
  accept="image/png,image/jpeg,image/gif,image/webp,image/svg+xml"
  class="hidden"
  bind:this={fileInput}
  onchange={handleFile}
/>

<style>
  /*
    Basılacak sayfanın gölgesinden (.sheet) daha hafif: masadan kalkan EN
    yüksek nesne kâğıttır, palet onun altında durur.
  */
  .shadow-float {
    box-shadow:
      0 1px 2px rgba(22, 35, 63, 0.08),
      0 4px 12px rgba(22, 35, 63, 0.1);
  }
</style>

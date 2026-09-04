<script lang="ts">
  /**
   * Kalıp paleti — yüzen, gruplu.
   *
   * Önceki hâli (`BlockPalette`) tam genişlik bir şeritti ve 17 düğmeyi tek
   * sırada sarmalıyordu: bir soru tipi kalıbı + 15 genel blok + Görsel. Hiçbir
   * gruplama yoktu, `Kesir` ile `Şekil + başlık` aynı ağırlıkta duruyordu ve
   * şerit editörün üstünden kalıcı olarak yer yiyordu.
   *
   * Şimdi kapalı başlar (tek pill), açılınca gruplu bir panel olarak
   * kaynağın üzerinde yüzer. Ekranda aynı anda 4-10 seçenek durur, 17 değil.
   *
   * Pinlenmiş kısıt burada da geçerli: tıklanan her blok kaynağa GÖRÜNÜR Typst
   * olarak düşer. Öğretmen Typst yazmaya zorlanmaz ama her seferinde görür.
   */
  import { Button } from "flowbite-svelte";
  import { CloseOutline, GridPlusOutline, ImageOutline } from "flowbite-svelte-icons";
  import { groupsFor, type QuestionType } from "$lib/question/templates";
  import { saveImageAsTypst } from "$lib/question/image";
  import { errorText } from "$lib/editor/diagnostics";
  import { pushEscapeLayer } from "$lib/ui/escape-stack";

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
   * Esc, tek sıralı merdivenden gelir — kendi pencere dinleyicimiz YOK.
   *
   * Önceden kendi dinleyicisi vardı ve şu hatayı üretiyordu: palet açıkken
   * editörde Esc'e basmak CodeMirror'ın tamamlama kutusunu kapatıyor, olay
   * pencereye çıkıyor ve paleti DE kapatıyordu. Merdiven `defaultPrevented`
   * kontrolüyle bunu engelliyor (lib/ui/escape-stack.ts).
   */
  $effect(() => {
    if (!open) return;
    return pushEscapeLayer(() => (open = false));
  });


  function onWindowPointerDown(event: PointerEvent) {
    if (!open || !root) return;
    if (!root.contains(event.target as Node)) open = false;
  }
</script>

<svelte:window onpointerdown={onWindowPointerDown} />

<!--
  Panel kaynağın ÜZERİNDE yüzer, kâğıdın üstüne taşmaz. Kırmızı burada YOK:
  bu bir değerlendirme değil, ekleme aracı — vurgu birincil (mavi) renkte.
-->
<div bind:this={root} class="pointer-events-none absolute inset-0 z-10">
  {#if open}
    <div
      class="pointer-events-auto absolute right-4 bottom-4 w-[300px] rounded-lg border
             border-gray-200 bg-white shadow-lg dark:border-gray-700 dark:bg-gray-800"
    >
      <div class="flex items-stretch border-b border-gray-200 dark:border-gray-700">
        {#each groups as group (group.id)}
          <button
            type="button"
            class="flex-1 border-r border-gray-200 px-2 py-1.5 text-[11px] font-semibold
                   tracking-wide uppercase transition-colors last:border-r-0
                   dark:border-gray-700"
            class:bg-gray-100={group.id === activeId}
            class:text-gray-900={group.id === activeId}
            class:dark:bg-gray-700={group.id === activeId}
            class:dark:text-white={group.id === activeId}
            class:text-gray-500={group.id !== activeId}
            class:hover:text-gray-900={group.id !== activeId}
            class:dark:text-gray-400={group.id !== activeId}
            class:dark:hover:text-white={group.id !== activeId}
            aria-pressed={group.id === activeId}
            onclick={() => (activeId = group.id)}
          >
            {group.label}
          </button>
        {/each}
        <button
          type="button"
          class="flex items-center px-2 text-gray-500 transition-colors hover:text-gray-900
                 dark:text-gray-400 dark:hover:text-white"
          title="Paneli kapat — Esc"
          aria-label="Paleti kapat"
          onclick={() => (open = false)}
        >
          <CloseOutline class="h-4 w-4" />
        </button>
      </div>

      <div class="grid grid-cols-2 gap-1 p-2">
        {#each activeGroup.blocks as block (block.label)}
          <button
            type="button"
            class="rounded border px-2 py-1.5 text-left text-xs transition-colors"
            class:border-primary-300={activeGroup.id === "kalip"}
            class:bg-primary-50={activeGroup.id === "kalip"}
            class:text-primary-700={activeGroup.id === "kalip"}
            class:dark:border-primary-700={activeGroup.id === "kalip"}
            class:dark:bg-primary-900={activeGroup.id === "kalip"}
            class:dark:text-primary-300={activeGroup.id === "kalip"}
            class:border-gray-200={activeGroup.id !== "kalip"}
            class:bg-white={activeGroup.id !== "kalip"}
            class:text-gray-900={activeGroup.id !== "kalip"}
            class:hover:border-primary-400={activeGroup.id !== "kalip"}
            class:hover:text-primary-700={activeGroup.id !== "kalip"}
            class:dark:border-gray-700={activeGroup.id !== "kalip"}
            class:dark:bg-gray-800={activeGroup.id !== "kalip"}
            class:dark:text-white={activeGroup.id !== "kalip"}
            title={block.hint}
            onclick={() => oninsert(block.snippet)}
          >
            {block.label}
          </button>
        {/each}
      </div>

      <div class="flex items-center gap-2 border-t border-gray-200 px-3 py-2 dark:border-gray-700">
        <span class="text-[11px] font-semibold tracking-wide text-gray-500 uppercase dark:text-gray-400">
          Ekle
        </span>
        <Button
          size="xs"
          color="light"
          disabled={saving}
          title="Dosyadan görsel seç — kopyalanır ve göreli yolla eklenir"
          onclick={() => fileInput?.click()}
        >
          <ImageOutline class="me-1 h-3.5 w-3.5" />
          {saving ? "Ekleniyor…" : "Görsel"}
        </Button>
      </div>

      {#if imageError}
        <p class="border-t border-gray-200 px-3 py-2 text-xs text-red-600 dark:border-gray-700 dark:text-red-400">
          {imageError}
        </p>
      {/if}
    </div>
  {:else}
    <button
      type="button"
      class="pointer-events-auto absolute right-4 bottom-4 flex items-center gap-1
             rounded-lg border border-gray-200 bg-white px-3 py-1.5 text-xs font-medium
             text-gray-900 shadow-lg transition-colors hover:bg-gray-50
             dark:border-gray-700 dark:bg-gray-800 dark:text-white dark:hover:bg-gray-700"
      aria-expanded="false"
      title="Kalıp ve matematik parçaları"
      onclick={() => (open = true)}
    >
      <GridPlusOutline class="h-4 w-4" /> Ekle
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

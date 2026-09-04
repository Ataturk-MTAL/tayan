<script lang="ts">
  /**
   * Açık uçlu sorunun puanlama ölçütleri.
   *
   * NEDEN AYRI DOSYA: yalnız klasik soruda görünüyor ve kendi doğrulaması var;
   * QuestionInspector zaten kazanım, künye ve ölçüm taşıyor.
   *
   * KURAL (Rust tarafındaki ClassicQuestion::validate ile aynı): rubrik boş
   * değilse ölçüt puanlarının toplamı soru puanına EŞİT olmak zorunda. Eşit
   * olmadığında kaydetme Rust'ta reddediliyordu ama öğretmen sebebi görmüyordu;
   * fark burada, girerken, canlı gösteriliyor.
   */
  import { Badge, Button } from "flowbite-svelte";
  import { PlusOutline, TrashBinOutline } from "flowbite-svelte-icons";
  import type { RubricItem } from "$lib/types";
  import TypstField from "./TypstField.svelte";

  type Props = {
    rubric: RubricItem[];
    /** Sorunun toplam puanı; ölçüt toplamı buna eşitlenmeli. */
    points: number;
    onchange: (next: RubricItem[]) => void;
  };

  let { rubric, points, onchange }: Props = $props();

  let toplam = $derived(rubric.reduce((sum, r) => sum + r.points, 0));
  let fark = $derived(points - toplam);

  /** Boş rubrik geçerlidir: öğretmen ölçüt yazmak zorunda değil. */
  let hata = $derived.by(() => {
    if (rubric.length === 0) return null;
    if (rubric.some((r) => r.criterion.trim() === "")) {
      return "Ölçüt metni boş olamaz.";
    }
    if (toplam !== points) {
      return fark > 0
        ? `Ölçüt toplamı ${toplam}; soru puanı ${points}. ${fark} puan dağıtılmadı.`
        : `Ölçüt toplamı ${toplam}; soru puanı ${points}. ${-fark} puan fazla.`;
    }
    return null;
  });

  // Değişmezlik: her düzenleme yeni dizi üretir, mevcut olan değiştirilmez.
  function ekle() {
    // Kalan puanı yeni ölçüte öner: en sık istenen dağılım budur ve
    // öğretmenin toplamı elde tutturmasına gerek kalmaz.
    const onerilen = rubric.length === 0 ? points : Math.max(0, fark);
    onchange([...rubric, { criterion: "", points: onerilen }]);
  }

  function sil(index: number) {
    onchange(rubric.filter((_, i) => i !== index));
  }

  function guncelle(index: number, alan: Partial<RubricItem>) {
    onchange(rubric.map((r, i) => (i === index ? { ...r, ...alan } : r)));
  }
</script>

<div class="border-t border-gray-200 pt-3 dark:border-gray-700">
  <div class="flex items-baseline justify-between">
    <h3 class="text-[11px] font-semibold tracking-wide text-gray-500 uppercase dark:text-gray-400">
      Puanlama ölçütleri
    </h3>
    <Badge color={hata !== null ? "red" : "gray"} class="tnum">{toplam} / {points}</Badge>
  </div>

  {#if rubric.length === 0}
    <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
      Ölçüt yok. Cevap anahtarına yalnız örnek çözüm basılır, puanlama tablosu
      basılmaz.
    </p>
  {/if}

  <ol class="mt-1">
    {#each rubric as item, i (i)}
      <!--
        TEK ÇİZGİ, ÜÇ ALAN. Ölçüt, puan ve sil düğmesi aynı çizginin üstünde
        durur; her alan kendi çizgisini çizince satır kırık görünüyordu.
        Çizgi satırın kendisinde, odak da satırın tamamını vurguluyor —
        kırmızı değil, bu yalnızca "aktif satır" göstergesi, bir hata değil.
      -->
      <li
        class="grid grid-cols-[1fr_2.75rem_1.5rem] items-end gap-x-2 border-b
               border-gray-200 py-1 focus-within:border-primary-500
               dark:border-gray-700 dark:focus-within:border-primary-400"
      >
        <TypstField
          value={item.criterion}
          placeholder="Değerlendirme ölçütü"
          ariaLabel="Değerlendirme ölçütü"
          bordered={false}
          onchange={(v) => guncelle(i, { criterion: v })}
        />
        <input
          class="olcut-puan tnum w-full border-0 bg-transparent text-right text-sm
                 text-gray-900 focus:outline-none dark:text-white"
          type="number"
          min="0"
          max={points}
          aria-label="Ölçüt puanı"
          value={item.points}
          oninput={(e) => guncelle(i, { points: Number(e.currentTarget.value) })}
        />
        <button
          type="button"
          class="flex items-center justify-center text-gray-400 transition-colors
                 hover:text-red-600 dark:text-gray-500 dark:hover:text-red-400"
          aria-label="Ölçütü sil"
          title="Ölçütü sil"
          onclick={() => sil(i)}
        >
          <TrashBinOutline class="h-3.5 w-3.5" />
        </button>
      </li>
    {/each}
  </ol>

  <Button size="xs" color="light" class="mt-2" onclick={ekle}>
    <PlusOutline class="me-1 h-3.5 w-3.5" /> Ölçüt ekle
  </Button>

  {#if hata}
    <p class="mt-1 text-xs text-red-600 dark:text-red-400">{hata}</p>
  {/if}
</div>

<style>
  /*
    Yerel artırma okları rakamı eziyordu: dar sütunda "5" ile ok yığını
    üst üste biniyor ve puan okunmaz hâle geliyordu. Okları kaldırıp
    rakamı sağa yaslamak, tabloya benzeyen bir puan sütunu veriyor.
    Değer klavyeden ve yukarı/aşağı tuşlarıyla hâlâ değiştirilebilir.
  */
  .olcut-puan::-webkit-outer-spin-button,
  .olcut-puan::-webkit-inner-spin-button {
    appearance: none;
    margin: 0;
  }
  .olcut-puan {
    appearance: textfield;
  }
</style>

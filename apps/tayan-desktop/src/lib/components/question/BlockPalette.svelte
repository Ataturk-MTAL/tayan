<script lang="ts">
  /**
   * Blok şeridi, pinlenmiş kısıtın taşıyıcısıdır: öğretmen ham Typst yazmaya
   * zorlanmaz, ama tıkladığı her blok kaynağa GÖRÜNÜR biçimde düşer. Neden-sonuç
   * açıkta kaldığı için Typst gizlenmeden öğretilir.
   *
   * Buradaki her parça derlenen gerçek Typst'tir; #blank ve #cb şablonun
   * PREAMBLE'ında tanımlıdır (typst_gen.rs).
   */
  type Block = { label: string; hint: string; snippet: string };

  type Props = { oninsert: (snippet: string) => void };
  let { oninsert }: Props = $props();

  const BLOCKS: Block[] = [
    { label: "Matematik", hint: "$x^2 + y^2$", snippet: "$  $" },
    { label: "Blok matematik", hint: "ortalanmış, kendi satırında", snippet: "\n$ \n$\n" },
    { label: "Kesir", hint: "a bölü b", snippet: "$ a/b $" },
    { label: "Kök", hint: "karekök", snippet: "$ sqrt(x) $" },
    { label: "Boşluk", hint: "doldurulacak çizgi", snippet: "#blank(width: 4cm)" },
    { label: "Kutucuk", hint: "☐ işaretlenebilir", snippet: "#cb()" },
    { label: "Görsel", hint: "dosyadan resim", snippet: '#image("", width: 60%)' },
    { label: "Tablo", hint: "2 sütunlu", snippet: "#table(columns: 2, [], [])" },
    { label: "Kalın", hint: "*kalın*", snippet: "**" },
    { label: "Boşluk bırak", hint: "dikey aralık", snippet: "#v(0.5cm)" },
  ];
</script>

<div class="ruled-bottom flex flex-wrap items-center gap-quarter bg-paper px-rule py-half paper-plain">
  <span class="stamp mr-half">Ekle</span>
  {#each BLOCKS as block}
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

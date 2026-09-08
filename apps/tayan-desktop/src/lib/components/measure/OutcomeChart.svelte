<script lang="ts">
  /**
   * Kazanım başarısı — sınıf hangi konuda zayıf?
   *
   * ÖĞRETMENİN SORUSU: "Neyi tekrar işlemeliyim?" Ortalama puan bu soruya
   * cevap vermiyor; soru soru tablo da vermiyor, çünkü bir kazanımı birden çok
   * soru ölçüyor ve öğretmen onları kafasında toplamak zorunda kalıyor.
   *
   * EN KÖTÜ ÜSTTE. Sıralama alfabetik ya da kod sırası olsaydı grafik bir
   * liste olurdu; başarı sırasına dizilince grafiğin ÜST KISMI doğrudan telafi
   * planı oluyor — öğretmen yukarıdan aşağı okuyup istediği yerde durabiliyor.
   *
   * SORU SAYISI HER ÇUBUKTA YAZILI. Tek soruyla ölçülmüş bir kazanımın %0'ı ile
   * dört soruyla ölçülmüş bir kazanımın %0'ı aynı şey değil: ilki o sorunun
   * kendi kusuru da olabilir, ikincisi gerçek bir öğrenme boşluğu. Sayı olmadan
   * grafik ikisini aynı gösterirdi.
   *
   * YATAY ÇUBUK, DİKEY DEĞİL: kazanım kodları uzun ("MAT.10.4.2"); dikey
   * çubukta etiketler eğik yazılmak zorunda kalır ve okunmaz.
   */
  import { outcomeStats, type ItemStat } from "$lib/analysis/item-stats";
  import { selectQuestions, selection } from "$lib/ui/analysis-selection.svelte";
  import type { Question } from "$lib/types";

  type Props = {
    items: ItemStat[];
    bank: Question[];
    studentCount: number;
    /** Geçme eşiği — çubuğun kırmızıya döndüğü sınır. */
    threshold?: number;
  };

  let { items, bank, studentCount, threshold = 50 }: Props = $props();

  let sonuc = $derived(outcomeStats(items, bank, studentCount));

  /**
   * Uzun listede kaydırma.
   *
   * KESMİYORUZ. 20 soruluk bir sınavda 15+ kazanım çıkabiliyor; "ilk 8'i
   * göster" demek, öğretmenin tam da aradığı zayıf kazanımı gizleyebilirdi.
   * Kesmek yerine kaydırılıyor — en kötüler zaten üstte.
   */
  const ROW_H = 26;
  const MAX_VISIBLE = 12;

  const KAYNAK = "kazanım";

  /**
   * Bir kazanıma tıklamak o kazanımın SORULARINI seçiyor.
   *
   * Seçim birimi her yerde SORU: harita da, tablo da, cevap ızgarası da soru
   * biliyor. Kazanımı ayrı bir seçim türü yapsaydık her kartın iki farklı
   * süzgeci anlaması gerekirdi. Kazanım burada soruya ÇEVRİLİYOR ve diğer
   * kartlar hiçbir şey öğrenmek zorunda kalmıyor.
   *
   * Aynı kazanıma tekrar tıklamak seçimi kaldırıyor — açma/kapama.
   */
  function kazanimSec(outcome: string) {
    const sorular = items
      .filter((i) => bank.find((b) => b.id === i.questionId)?.outcomes.includes(outcome))
      .map((i) => i.questionId);

    const ayniSecim =
      selection.questionIds.length === sorular.length &&
      sorular.every((id) => selection.questionSet.has(id));

    selectQuestions(ayniSecim ? [] : sorular, KAYNAK);
  }
</script>

<figure
  class="m-0 flex h-full flex-col rounded-lg border border-default-medium
         bg-neutral-primary-medium p-4 shadow-sm"
>
  <figcaption class="text-xs font-semibold uppercase tracking-wider text-body-subtle">
    Kazanım başarısı
  </figcaption>

  {#if sonuc.outcomes.length === 0}
    <p class="mt-1 text-xs text-body-subtle">
      {#if sonuc.uncodedQuestions > 0}
        Sınavdaki {sonuc.uncodedQuestions} sorunun hiçbirinde kazanım kodu yok;
        kazanım bazlı okuma yapılamıyor.
      {:else}
        Sonuç girilmemiş.
      {/if}
    </p>
  {:else}
    <!--
      HTML/CSS ile çizildi, SVG ile DEĞİL. Yatay çubuk grafiği aslında bir
      liste; metin uzunluğu, kırpma ve kaydırma tarayıcının işi. SVG'de her
      etiketi elle konumlandırmak, üstelik viewBox ölçeklenmesiyle yazı
      boyutunu da bozmak gerekirdi.
    -->
    <div
      class="mt-2.5 min-h-0 overflow-y-auto"
      style={sonuc.outcomes.length > MAX_VISIBLE
        ? `max-height: ${MAX_VISIBLE * ROW_H}px`
        : undefined}
    >
      <ul class="space-y-1">
        {#each sonuc.outcomes as o (o.outcome)}
          {@const altinda = o.scorePct < threshold}
          <li>
            <!--
              SATIRIN TAMAMI DÜĞME. Yalnız kodu tıklanabilir yapmak, 5.5rem'lik
              dar bir hedef bırakırdı; çubuk da tıklanabilir olunca hedef satır
              boyu oluyor.
            -->
            <button
              type="button"
              class="flex w-full items-center gap-2.5 rounded-sm px-1 py-0.5 text-left text-xs
                     transition-colors hover:bg-neutral-tertiary-medium"
              onclick={() => kazanimSec(o.outcome)}
            >
              <span class="tnum w-[5.5rem] shrink-0 truncate text-body" title={o.outcome}>
                {o.outcome}
              </span>

              <!--
                Çubuk kabı: %100'ün nerede olduğu görünsün diye zemin var.
                Zeminsiz çubukta göz uzunlukları birbirine göre okur, ölçeğe
                göre değil.
              -->
              <span class="relative h-2.5 min-w-0 flex-1 rounded-sm bg-neutral-tertiary-medium">
                <span
                  class="absolute inset-y-0 left-0 rounded-sm {altinda
                    ? 'bg-red-600 dark:bg-red-400'
                    : 'bg-gray-700 dark:bg-gray-300'}"
                  style="width: {Math.max(o.scorePct, 0).toFixed(1)}%"
                ></span>
                <!--
                  Eşik çizgisi HER ÇUBUKTA. Tek bir dikey çizgi çizmek yerine
                  çubuk çubuk işaretlemek, liste kaydırılırken de eşiğin
                  görünmesini sağlıyor.
                -->
                <span
                  class="absolute inset-y-0 w-px bg-default-medium"
                  style="left: {threshold}%"
                ></span>
              </span>

              <span
                class="tnum w-9 shrink-0 text-right {altinda
                  ? 'text-red-600 dark:text-red-400'
                  : 'text-body'}"
              >
                %{o.scorePct.toFixed(0)}
              </span>

              <!-- SORU SAYISI: yüzdenin ne kadar sağlam olduğunu söyleyen tek şey. -->
              <span class="tnum w-12 shrink-0 text-right text-body-subtle">
                {o.questionCount} soru
              </span>
            </button>
          </li>
        {/each}
      </ul>
    </div>

    <p class="mt-1 text-xs text-body-subtle">
      Sınıfın her kazanımdan aldığı puanın alınabilir puana oranı. İnce dikey
      çizgi %{threshold} geçme eşiği; en zayıf kazanım en üstte.
    </p>

    <!--
      KAPSAM UYARISI ŞART. Kazanım kodu girilmemiş sorular hiçbir çubuğa
      girmiyor. Sayı söylenmezse öğretmen grafiği "sınavın tamamı" sanır —
      oysa yarısını kapsıyor olabilir ve eksik kalan, tam da zayıf olan konu
      olabilir.
    -->
    {#if sonuc.uncodedQuestions > 0}
      <p class="mt-1 text-xs text-amber-600 dark:text-amber-400">
        {sonuc.uncodedQuestions} soruda kazanım kodu yok; bu grafiğe girmiyorlar.
      </p>
    {/if}
  {/if}
</figure>

<script lang="ts">
  /**
   * Soru soru madde analizi.
   *
   * Öğretmenin sorduğu iki soruya cevap verir: hangi soru zordu, ve hangi soru
   * işe yaramadı. İkisi ayrı şeyler — çok kolay bir soru da çok zor bir soru
   * da sınıfı ayırt etmez.
   *
   * TEK EKSEN. Güçlük ve ayırt edicilik ayrı ölçüler; ikisini tek çubuğa
   * bindirmek ya da iki ölçekli tek grafik yapmak en sık yapılan grafik
   * hatasıdır. Burada her ölçü kendi sütununda duruyor.
   */
  import { MIN_DISCRIMINATION_N, needsReview, type ItemStat } from "$lib/analysis/item-stats";
  import { bodyPreview } from "$lib/types";
  import type { Question } from "$lib/types";

  type Props = {
    items: ItemStat[];
    bank: Question[];
    /** Sonuç girilen öğrenci sayısı — ayırt ediciliğin anlamlı olup olmadığı. */
    studentCount: number;
  };

  let { items, bank, studentCount }: Props = $props();

  function baslik(item: ItemStat): string {
    const q = bank.find((b) => b.id === item.questionId);
    if (!q) return "Bankada yok";
    const t = q.meta.title.trim();
    return t !== "" ? t : bodyPreview(q.body, 46);
  }

  /** Yığılmış çubuğun parçaları. Sıra bilinçli: doğru → kısmi → yanlış → boş. */
  function parcalar(item: ItemStat) {
    const n = item.correct + item.partial + item.wrong + item.blank;
    if (n === 0) return [];
    return [
      { ad: "doğru", say: item.correct, sinif: "bg-ink" },
      { ad: "kısmi", say: item.partial, sinif: "bg-ink/45" },
      { ad: "yanlış", say: item.wrong, sinif: "bg-red" },
      { ad: "boş", say: item.blank, sinif: "bg-rule-strong" },
    ]
      .filter((p) => p.say > 0)
      .map((p) => ({ ...p, yuzde: (p.say / n) * 100 }));
  }
</script>

<figure class="m-0">
  <figcaption class="stamp">Soru soru</figcaption>

  {#if items.length === 0}
    <p class="pencil mt-quarter">Sonuç girilmemiş.</p>
  {:else}
    <div class="mt-quarter flex flex-wrap gap-half">
      <span class="pencil"><span class="mr-[3px] inline-block h-[8px] w-[8px] bg-ink align-middle"></span>doğru</span>
      <span class="pencil"><span class="mr-[3px] inline-block h-[8px] w-[8px] bg-ink/45 align-middle"></span>kısmi</span>
      <span class="pencil"><span class="mr-[3px] inline-block h-[8px] w-[8px] bg-red align-middle"></span>yanlış</span>
      <span class="pencil"><span class="mr-[3px] inline-block h-[8px] w-[8px] bg-rule-strong align-middle"></span>boş</span>
    </div>

    <table class="mt-half w-full border-collapse">
      <thead>
        <tr class="ruled-bottom">
          <th class="stamp w-[1.5rem] text-left font-normal">#</th>
          <th class="stamp text-left font-normal">Soru</th>
          <th class="stamp w-[34%] text-left font-normal">Dağılım</th>
          <th class="stamp w-[3.5rem] text-right font-normal">Güçlük</th>
          <th class="stamp w-[4.5rem] text-right font-normal">Ayırt</th>
        </tr>
      </thead>
      <tbody>
        {#each items as item (item.questionId)}
          {@const uyari = needsReview(item)}
          <tr class="border-b border-rule align-top">
            <td class="tnum pencil py-quarter">{item.order}</td>
            <td class="py-quarter pr-half">
              <span class="annot">{baslik(item)}</span>
              {#if uyari}
                <span class="pencil block text-red-deep">{uyari}</span>
              {/if}
            </td>
            <td class="py-quarter pr-half">
              <!--
                Yığılmış çubuk: sınıfın bu soruda nasıl dağıldığı. Parçalar
                arasında 1px kâğıt boşluğu var, yoksa iki koyu parça tek parça
                gibi okunuyor.
              -->
              <div class="flex h-[14px] w-full gap-[1px]" title="{item.correct} doğru, {item.partial} kısmi, {item.wrong} yanlış, {item.blank} boş">
                {#each parcalar(item) as p (p.ad)}
                  <div class="{p.sinif}" style="width: {p.yuzde}%"></div>
                {/each}
              </div>
            </td>
            <td class="tnum annot py-quarter text-right">
              {(item.difficulty * 100).toFixed(0)}%
            </td>
            <td class="tnum annot py-quarter text-right">
              {#if item.discrimination === null}
                <span class="pencil">—</span>
              {:else}
                <span class:text-red-deep={item.discrimination < 0.2}>
                  {item.discrimination.toFixed(2)}
                </span>
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>

    <p class="annot mt-quarter">
      <b>Güçlük</b>, sınıfın o sorudan aldığı puanın alınabilecek puana oranı:
      yüksek değer soru kolay demek. <b>Ayırt edicilik</b>, üst %27 ile alt %27
      arasındaki fark; 0.20'nin altı sorunun iyi ve zayıf öğrenciyi
      ayırmadığını gösterir.
    </p>

    {#if studentCount < MIN_DISCRIMINATION_N}
      <p class="pencil mt-quarter">
        Ayırt edicilik hesaplanmadı: {studentCount} öğrenci var,
        {MIN_DISCRIMINATION_N} gerekiyor. Daha az kişide üst dilim bir iki
        öğrenciye iner ve çıkan sayı sorunun niteliğini değil, o öğrencilerin
        o günkü hâlini ölçer. Uydurma bir sayı vermektense boş bırakılıyor.
      </p>
    {/if}
  {/if}
</figure>

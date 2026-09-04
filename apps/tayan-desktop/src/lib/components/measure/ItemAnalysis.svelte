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

  /**
   * Satırda gösterilecek ad: sorunun başlığı, başlık yoksa gövdesinin önizlemesi.
   *
   * maxLen PARAMETRE, çünkü aynı ad iki yerde gerekiyor: hücrede 46 karakterlik
   * kısa hâli, `title` niteliğinde Infinity ile KIRPILMAMIŞ hâli. bodyPreview
   * (types.ts) metni JS'te `slice(0, maxLen) + "…"` ile GERÇEKTEN kesiyor — CSS
   * kırpması değil, kalıcı kayıp; kırpık metni title'a koymak erişim sağlamaz,
   * yalnız sahte güven verir. Infinity'de `text.length > maxLen` hiç sağlanmadığı
   * için tam düz metin dönüyor.
   */
  function baslik(item: ItemStat, maxLen = 46): string {
    const q = bank.find((b) => b.id === item.questionId);
    if (!q) return "Bankada yok";
    const t = q.meta.title.trim();
    return t !== "" ? t : bodyPreview(q.body, maxLen);
  }

  /**
   * Yığılmış çubuğun parçaları. Sıra bilinçli: doğru → kısmi → yanlış → boş.
   *
   * Renkler: doğru/kısmi ANA VERİ (koyu/orta gri), yanlış TEK KIRMIZI —
   * kırmızı burada da yalnız değerlendirme demek. Boş, ızgara tonunda nötr;
   * "cevaplanmadı" bir hata değil, ölçülememiş bir hücre.
   */
  function parcalar(item: ItemStat) {
    const n = item.correct + item.partial + item.wrong + item.blank;
    if (n === 0) return [];
    return [
      { ad: "doğru", say: item.correct, sinif: "bg-gray-800 dark:bg-gray-200" },
      { ad: "kısmi", say: item.partial, sinif: "bg-gray-400 dark:bg-gray-500" },
      { ad: "yanlış", say: item.wrong, sinif: "bg-red-600 dark:bg-red-400" },
      { ad: "boş", say: item.blank, sinif: "bg-gray-300 dark:bg-gray-600" },
    ]
      .filter((p) => p.say > 0)
      .map((p) => ({ ...p, yuzde: (p.say / n) * 100 }));
  }
</script>

<figure
  class="m-0 rounded-lg border border-gray-200 bg-white p-4 shadow-sm dark:border-gray-700 dark:bg-gray-800"
>
  <figcaption class="text-[11px] font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">
    Soru soru
  </figcaption>

  {#if items.length === 0}
    <p class="mt-[5px] text-[12px] leading-5 text-gray-500 dark:text-gray-400">Sonuç girilmemiş.</p>
  {:else}
    <div class="mt-[5px] flex flex-wrap gap-2.5 text-[12px] leading-5 text-gray-500 dark:text-gray-400">
      <span><span class="mr-[3px] inline-block h-[8px] w-[8px] bg-gray-800 align-middle dark:bg-gray-200"></span>doğru</span>
      <span><span class="mr-[3px] inline-block h-[8px] w-[8px] bg-gray-400 align-middle dark:bg-gray-500"></span>kısmi</span>
      <span><span class="mr-[3px] inline-block h-[8px] w-[8px] bg-red-600 align-middle dark:bg-red-400"></span>yanlış</span>
      <span><span class="mr-[3px] inline-block h-[8px] w-[8px] bg-gray-300 align-middle dark:bg-gray-600"></span>boş</span>
    </div>

    <table class="mt-2.5 w-full border-collapse">
      <thead>
        <tr class="border-b border-gray-300 dark:border-gray-600">
          <th class="w-[1.5rem] text-left text-[11px] font-normal uppercase tracking-wider text-gray-500 dark:text-gray-400">#</th>
          <th class="text-left text-[11px] font-normal uppercase tracking-wider text-gray-500 dark:text-gray-400">Soru</th>
          <th class="w-[34%] text-left text-[11px] font-normal uppercase tracking-wider text-gray-500 dark:text-gray-400">Dağılım</th>
          <th class="w-[3.5rem] text-right text-[11px] font-normal uppercase tracking-wider text-gray-500 dark:text-gray-400">Güçlük</th>
          <th class="w-[4.5rem] text-right text-[11px] font-normal uppercase tracking-wider text-gray-500 dark:text-gray-400">Ayırt</th>
        </tr>
      </thead>
      <tbody>
        {#each items as item (item.questionId)}
          {@const uyari = needsReview(item)}
          <tr class="border-b border-gray-200 align-top dark:border-gray-700">
            <td class="tnum py-[5px] text-[12px] leading-5 text-gray-500 dark:text-gray-400">{item.order}</td>
            <!--
              wrap-anywhere: başlık ham kullanıcı metni (q.meta.title, uzunluk
              sınırı yok) ya da bodyPreview çıktısı — boşluksuz, bölünemez bir
              dizge olabilir (matematik önizlemesi, kod/ID biçimli ad).
              `table w-full` otomatik yerleşimde bir sütunun min-content'i,
              ilan edilen genişliklerden (w-[34%], w-[3.5rem], w-[4.5rem])
              ÜSTÜN gelir ve tablo min-content'inin altına inemez; w-full bunu
              engelleyemez. Tablonun kaydırma kabı olmadığı için taşma doğrudan
              kartın dışına akıyordu: 380px kart + 47 karakterlik bölünemez
              başlık → tablo 502px, kartın sağ kenarlığını 139px aşıyor. Taşma
              olmayan genişliklerde bile zarar vardı; 560px kartta aynı başlık
              "Dağılım" sütununu 179px'ten 72px'e eziyor, yani grafiğin asıl
              ürünü olan yığılmış çubuk yarıdan fazla daralıyordu.
              `break-words` YETMİYOR (ölçüldü, tablo yine 502px kaldı): min-content
              hesabına göre `overflow-wrap: break-word` kelimeyi bölünmemiş sayar.
              `break-all` de KULLANILMADI: Türkçe başlıkları da ortadan bölerdi;
              `wrap-anywhere` yalnızca satıra sığmayan uzun dizgeyi kırar.
              overflow-wrap kalıtsal olduğu için alttaki uyarı metni de kapsanır.
            -->
            <td class="py-[5px] pr-2.5 wrap-anywhere">
              <!-- title ŞART: başlıksız sorularda hücredeki ad gövdenin ilk 46 karakteri
                   ve sonuna "…" konuyor — metin JS'te kesiliyor, ekranda kırpılmıyor.
                   46 karakter çoğu zaman hangi soru olduğunu ayırt etmeye yetmez;
                   öğretmen yanlış soruyu gözden geçirilecek diye işaretler. Infinity ile
                   çağrılan baslik() kırpılmamış tam metni verdiği için erişim korunuyor. -->
              <span
                class="text-[12px] leading-5 text-gray-900 dark:text-white"
                title={baslik(item, Infinity)}>{baslik(item)}</span>
              {#if uyari}
                <!-- Gözden geçirme uyarısı GERÇEK bir değerlendirme sonucu: kırmızı burada doğru yerinde. -->
                <span class="block text-[12px] leading-5 text-red-600 dark:text-red-400">{uyari}</span>
              {/if}
            </td>
            <td class="py-[5px] pr-2.5">
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
            <td class="tnum py-[5px] text-right text-[12px] leading-5 text-gray-700 dark:text-gray-300">
              {(item.difficulty * 100).toFixed(0)}%
            </td>
            <td class="tnum py-[5px] text-right text-[12px] leading-5 text-gray-700 dark:text-gray-300">
              {#if item.discrimination === null}
                <span class="text-gray-500 dark:text-gray-400">—</span>
              {:else}
                <span
                  class:text-red-600={item.discrimination < 0.2}
                  class:dark:text-red-400={item.discrimination < 0.2}
                >
                  {item.discrimination.toFixed(2)}
                </span>
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>

    <p class="mt-2.5 text-[12px] leading-5 text-gray-700 dark:text-gray-300">
      <b>Güçlük</b>, sınıfın o sorudan aldığı puanın alınabilecek puana oranı:
      yüksek değer soru kolay demek. <b>Ayırt edicilik</b>, üst %27 ile alt %27
      arasındaki fark; 0.20'nin altı sorunun iyi ve zayıf öğrenciyi
      ayırmadığını gösterir.
    </p>

    {#if studentCount < MIN_DISCRIMINATION_N}
      <p class="mt-2.5 text-[12px] leading-5 text-gray-500 dark:text-gray-400">
        Ayırt edicilik hesaplanmadı: {studentCount} öğrenci var,
        {MIN_DISCRIMINATION_N} gerekiyor. Daha az kişide üst dilim bir iki
        öğrenciye iner ve çıkan sayı sorunun niteliğini değil, o öğrencilerin
        o günkü hâlini ölçer. Uydurma bir sayı vermektense boş bırakılıyor.
      </p>
    {/if}
  {/if}
</figure>

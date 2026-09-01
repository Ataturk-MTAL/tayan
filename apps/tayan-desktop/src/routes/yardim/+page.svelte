<script lang="ts">
  import PageHead from "$lib/components/shell/PageHead.svelte";

  /**
   * Yardım ekranı, ürünün pinlenmiş kısıtını taşır: öğretmen ham Typst yazmaya
   * zorlanmaz, ama Typst'in NE OLDUĞUNU buradan anlayabilir.
   *
   * Bu yüzden her örnek çalışan gerçek koddur — kısaltılmış, "şuna benzer bir
   * şey" değil. Kopyalayıp editöre yapıştırınca derlenir.
   */
  type Row = { kod: string; ne: string };

  const TEMEL: Row[] = [
    { kod: "$x^2$", ne: "Satır içi matematik — cümlenin içinde kalır" },
    { kod: "$ x^2 $", ne: "Blok matematik — kendi satırına düşer, ortalanır" },
    { kod: "$a/b$", ne: "Kesir" },
    { kod: "$(a+b)/c$", ne: "Paylı kesir — birden çok terimi parantezle" },
    { kod: "$sqrt(x)$", ne: "Karekök" },
    { kod: "$x_1$", ne: "Alt indis" },
    { kod: "$alpha$ $beta$ $pi$", ne: "Yunan harfleri: adını yaz" },
    { kod: "$sum_(i=1)^n i$", ne: "Toplam sembolü, alt ve üst sınırlı" },
    { kod: "$integral_0^1 f(x) dif x$", ne: "İntegral" },
    { kod: "*kalın*", ne: "Kalın yazı" },
    { kod: "_eğik_", ne: "Eğik yazı" },
    { kod: "#underline[altı çizili]", ne: "Altı çizili" },
    { kod: "- madde", ne: "Madde işaretli liste" },
    { kod: "+ madde", ne: "Numaralı liste" },
    { kod: "#image(\"resim.png\", width: 60%)", ne: "Görsel ekle" },
    { kod: "#v(0.5cm)", ne: "Dikey boşluk" },
  ];

  const KALIPLAR = [
    {
      ad: "Çoktan seçmeli",
      kod: `#secenekler(dogru: "C",
  [$x = 1$],
  [$x = 2$],
  [$x = 2$ ve $x = 3$],
  [$x = 6$],
  [Hiçbiri],
)`,
      not: "dogru: kâğıda BASILMAZ. Uygulama cevap anahtarını ve madde analizini oradan kurar. Şık harfleri (A, B, C…) sıraya göre kendiliğinden verilir.",
    },
    {
      ad: "Doğru / yanlış",
      kod: "#dogru-yanlis(dogru: true)",
      not: "true ya da false. Kâğıda iki kutucuk basılır, cevap basılmaz.",
    },
    {
      ad: "Boşluk doldurma",
      kod: '#bosluk(cevap: "180|180 derece", width: 2cm)',
      not: "Kabul edilen cevapları | ile ayır. Gövdede kaç tane #bosluk varsa o kadar boşluk oluşur; puan her boşluk için ayrı sayılır.",
    },
    {
      ad: "Klasik",
      kod: "#cevap-alani(satir: 6)",
      not: "Öğrencinin yazacağı çizgiler. satir sayısını soruya göre ayarla.",
    },
  ];

  /** Yunan harfleri. Typst'te adıyla yazılır, büyük harf için baş harf büyük. */
  const YUNAN: Array<[string, string, string, string]> = [
    ["alpha", "α", "Alpha", "Α"],
    ["beta", "β", "Beta", "Β"],
    ["gamma", "γ", "Gamma", "Γ"],
    ["delta", "δ", "Delta", "Δ"],
    ["epsilon", "ε", "Epsilon", "Ε"],
    ["zeta", "ζ", "Zeta", "Ζ"],
    ["eta", "η", "Eta", "Η"],
    ["theta", "θ", "Theta", "Θ"],
    ["iota", "ι", "Iota", "Ι"],
    ["kappa", "κ", "Kappa", "Κ"],
    ["lambda", "λ", "Lambda", "Λ"],
    ["mu", "μ", "Mu", "Μ"],
    ["nu", "ν", "Nu", "Ν"],
    ["xi", "ξ", "Xi", "Ξ"],
    ["pi", "π", "Pi", "Π"],
    ["rho", "ρ", "Rho", "Ρ"],
    ["sigma", "σ", "Sigma", "Σ"],
    ["tau", "τ", "Tau", "Τ"],
    ["upsilon", "υ", "Upsilon", "Υ"],
    ["phi", "φ", "Phi", "Φ"],
    ["chi", "χ", "Chi", "Χ"],
    ["psi", "ψ", "Psi", "Ψ"],
    ["omega", "ω", "Omega", "Ω"],
  ];

  /** Analiz ve türev gösterimi. Hepsi derlenip gözle doğrulandı. */
  const TUREV: Row[] = [
    { kod: "$(dif y)/(dif x)$", ne: "Birinci türev — dy/dx" },
    { kod: "$(dif^2 y)/(dif x^2)$", ne: "İkinci türev" },
    { kod: "$f'(x)$", ne: "Üs notasyonu — f prim" },
    { kod: "$f''(x)$", ne: "İkinci türev, üs notasyonu" },
    { kod: "$(partial f)/(partial x)$", ne: "Kısmi türev — ∂f/∂x" },
    { kod: "$(partial^2 f)/(partial x partial y)$", ne: "Karma ikinci kısmi türev" },
    { kod: "$integral f(x) dif x$", ne: "Belirsiz integral" },
    { kod: "$integral_0^1 x^2 dif x$", ne: "Belirli integral" },
    { kod: "$lim_(x -> 0) (sin x)/x$", ne: "Limit" },
    { kod: "$nabla f$", ne: "Gradyan" },
    { kod: "$nabla dot bold(F)$", ne: "Diverjans" },
    { kod: "$dif f = (partial f)/(partial x) dif x$", ne: "Toplam diferansiyel" },
  ];

  const KISAYOLLAR: Row[] = [
    { kod: "⌘ +", ne: "Önizlemeyi yakınlaştır" },
    { kod: "⌘ −", ne: "Önizlemeyi uzaklaştır" },
    { kod: "⌘ 0", ne: "Önizlemeyi gerçek boyuta getir" },
    { kod: "⌘ + tekerlek", ne: "Sürekli yakınlaştır / uzaklaştır" },
    { kod: "⌘ Z", ne: "Editörde geri al" },
    { kod: "⇧ ⌘ Z", ne: "İleri al" },
    { kod: "Tab", ne: "Girinti ekle" },
    { kod: "⌃ Boşluk", ne: "Otomatik tamamlama listesi" },
  ];

  const HATALAR = [
    {
      mesaj: "unknown variable",
      ne: "Var olmayan bir komut yazdın. Genelde yazım hatası: #secenekler yerine #secenek gibi. Yukarıdaki Kalıp düğmeleri doğrusunu ekler.",
    },
    {
      mesaj: "expected closing bracket",
      ne: "Bir köşeli parantez veya parantez kapanmamış. Editörde kırmızı işaret hangi satırda olduğunu gösterir.",
    },
    {
      mesaj: "Gövdede #secenekler(...) yok",
      ne: "Soru tipi çoktan seçmeli ama gövdede şık kalıbı yok. Kaydet düğmesi bu yüzden kapalı.",
    },
    {
      mesaj: "Doğru cevap \"X\" şıklarla eşleşmiyor",
      ne: "dogru: parametresine yazdığın harf, yazdığın şık sayısından fazla. Beş şık varsa en fazla E olabilir.",
    },
  ];

  const BOLUMLER = [
    { id: "kalip", ad: "Soru kalıpları" },
    { id: "temel", ad: "Typst temelleri" },
    { id: "gorsel", ad: "Görsel ekleme" },
    { id: "yunan", ad: "Yunan harfleri" },
    { id: "turev", ad: "Türev ve integral" },
    { id: "kisayol", ad: "Kısayollar" },
    { id: "kazanim", ad: "Kazanım kodu" },
    { id: "hata", ad: "Hata mesajları" },
    { id: "veri", ad: "Verilerim nerede" },
  ];
</script>

<div class="flex h-full min-h-0 flex-col">
  <PageHead title="Yardım" />

  <div class="grid min-h-0 flex-1 grid-cols-[200px_1fr]">
    <nav class="min-h-0 overflow-auto border-r border-rule-strong px-rule py-half">
      <ul>
        {#each BOLUMLER as bolum}
          <li>
            <a href="#{bolum.id}" class="block py-quarter text-[13px] leading-rule no-underline">
              {bolum.ad}
            </a>
          </li>
        {/each}
      </ul>
    </nav>

    <article class="min-h-0 overflow-auto px-rule py-rule">
      <div class="max-w-[68ch]">
        <p class="leading-rule">
          TAYAN soruları <strong>Typst</strong> ile dizer. Typst, matematiği ve sayfa
          düzenini metinle anlatan bir dizgi dilidir; kelime işlemcinin aksine
          yazdığın şey ile basılan şey birebir aynıdır.
        </p>
        <p class="pencil mt-half">
          Typst öğrenmek zorunda değilsin — yukarıdaki düğmeler her şeyi hazır ekler.
          Ama eklenen şey kaynakta görünür kalır, böylece zamanla ne olduğunu
          kendiliğinden öğrenirsin.
        </p>

        <h2 id="kalip" class="mt-rule border-t border-rule-strong pt-half">Soru kalıpları</h2>
        <p class="pencil mt-quarter">
          Sorunun yapısı ayrı bir formda değil, kaynağın içinde durur. Böylece cevap
          anahtarı ile kâğıtta görünen asla birbirinden ayrı düşmez.
        </p>

        {#each KALIPLAR as kalip}
          <h3 class="mt-rule">{kalip.ad}</h3>
          <pre class="ruled mt-quarter overflow-x-auto p-half font-mono text-[12px] leading-[20px]">{kalip.kod}</pre>
          <p class="pencil mt-quarter">{kalip.not}</p>
        {/each}

        <h2 id="temel" class="mt-rule border-t border-rule-strong pt-half">Typst temelleri</h2>

        <p class="mt-quarter leading-rule">
          <strong>Matematikte tek kural boşluktur.</strong> Dolar işaretinin hemen
          yanına yazarsan matematik cümlenin içinde kalır; boşluk bırakırsan kendi
          satırına düşer ve ortalanır.
        </p>
        <pre class="ruled mt-quarter overflow-x-auto p-half font-mono text-[12px] leading-[20px]">Kök $x = 2$ olarak bulunur.     → cümlenin içinde
Kök $ x = 2 $ olarak bulunur.   → kendi satırında, ortalı</pre>
        <p class="pencil mt-quarter">
          Yukarıdaki Matematik düğmesi satır içi, Blok matematik düğmesi bloklu
          olanı ekler; imleci de yazılacak yere koyar.
        </p>
        <table class="mt-half w-full border-collapse">
          <tbody>
            {#each TEMEL as satir}
              <tr class="border-b border-rule">
                <td class="w-[46%] py-quarter pr-half font-mono text-[12px] leading-rule">{satir.kod}</td>
                <td class="py-quarter text-[13px] leading-rule">{satir.ne}</td>
              </tr>
            {/each}
          </tbody>
        </table>

        <h2 id="gorsel" class="mt-rule border-t border-rule-strong pt-half">Görsel ekleme</h2>
        <p class="mt-quarter leading-rule">İki yol var, ikisi de aynı sonucu verir:</p>
        <ul class="mt-quarter">
          <li class="leading-rule">
            <strong>Yapıştır.</strong> Ekran görüntüsü al, editöre
            <span class="font-mono">⌘V</span> ile yapıştır.
          </li>
          <li class="leading-rule">
            <strong>Görsel düğmesi.</strong> Yukarıdaki şeritten dosya seç.
          </li>
        </ul>
        <p class="mt-half leading-rule">
          Her iki durumda da görsel uygulamanın veri klasörüne kopyalanır ve
          gövdeye şu şekilde eklenir:
        </p>
        <pre class="ruled mt-quarter overflow-x-auto p-half font-mono text-[12px] leading-[20px]">#image("images/soru_20260901_143022_a3f7b2c1.png", width: 60%)</pre>
        <p class="pencil mt-quarter">
          <span class="font-mono">width</span> değerini değiştirerek boyutlandır:
          <span class="font-mono">40%</span>, <span class="font-mono">8cm</span> gibi.
        </p>

        <h3 class="mt-rule">Hizalama ve yerleşim</h3>
        <pre class="ruled mt-quarter overflow-x-auto p-half font-mono text-[12px] leading-[20px]">#align(center)[#image("images/a.png", width: 60%)]   ortalı (varsayılan)
#align(right)[#image("images/a.png", width: 60%)]    sağa dayalı
#image("images/a.png", width: 60%)                   sola dayalı

#figure(image("images/a.png", width: 60%), caption: [Bir çember])
  numaralı şekil: "Şekil 1: Bir çember" altına yazılır

#grid(columns: (1fr, auto), gutter: 0.5cm,
  [Yandaki çemberin alanını bulunuz.],
  image("images/a.png", width: 4cm),
)
  metin solda, şekil sağda</pre>
        <p class="pencil mt-quarter">
          Yapıştırdığın görsel ortalı eklenir; sınav kâğıdında şekil neredeyse
          her zaman ortalanır. Değiştirmek istersen <span class="font-mono">#align</span>
          satırını düzenle.
        </p>

        <h3 class="mt-rule">Vazgeçtiğin görseller</h3>
        <p class="mt-quarter leading-rule">
          Görsel yapıştırıldığı anda diske yazılır; sorunun kaydedilmesi
          beklenmez. Beklenseydi önizleme onu gösteremezdi, çünkü Typst dosyadan
          okur.
        </p>
        <p class="mt-quarter leading-rule">
          Sorudan vazgeçersen o dosya kullanılmadan kalır. Uygulama açılışta,
          hiçbir soruda kullanılmayan ve <strong>24 saatten eski</strong>
          görselleri siler. Yaş sınırı bilinçli: az önce yapıştırdığın ve henüz
          kaydetmediğin görsel hiçbir atıfta görünmez, sınır olmasa tam
          yazarken silinirdi.
        </p>
        <p class="mt-half leading-rule">
          Yol <strong>göreli</strong>dir, mutlak değil. Bu bilinçli: mutlak yol
          kullanıcı adını içerir ve veri başka bir bilgisayara taşındığında
          kırılır — sınav görselsiz basılır, üstelik bunu fark etmek zordur.
        </p>
        <p class="pencil mt-quarter">
          En fazla 8 MB. PNG, JPEG, GIF ve WebP desteklenir. Görseller
          veritabanıyla aynı klasörde durur, yani klasörü kopyalamak görselleri de
          yedekler.
        </p>

        <h2 id="yunan" class="mt-rule border-t border-rule-strong pt-half">Yunan harfleri</h2>
        <p class="mt-quarter leading-rule">
          Adıyla yazılır. Büyük harf için baş harfi büyüt:
          <span class="font-mono">$alpha$</span> → α,
          <span class="font-mono">$Delta$</span> → Δ.
          Editörde <span class="font-mono">$</span> yazınca hepsi listelenir.
        </p>
        <table class="mt-half w-full border-collapse text-[13px]">
          <tbody>
            {#each YUNAN as [kucukAd, kucuk, buyukAd, buyuk]}
              <tr class="border-b border-rule">
                <td class="py-quarter font-mono text-[12px] leading-rule">${kucukAd}$</td>
                <td class="py-quarter pr-half text-[15px]">{kucuk}</td>
                <td class="py-quarter font-mono text-[12px] leading-rule">${buyukAd}$</td>
                <td class="py-quarter text-[15px]">{buyuk}</td>
              </tr>
            {/each}
          </tbody>
        </table>
        <p class="pencil mt-quarter">
          Artı-eksi için <span class="font-mono">$plus.minus$</span> yaz.
          <span class="font-mono">+-</span> işe yaramaz; Typst onu ayrı iki işaret
          olarak dizer.
        </p>

        <h2 id="turev" class="mt-rule border-t border-rule-strong pt-half">Türev ve integral</h2>
        <p class="mt-quarter leading-rule">
          İki ayrı <span class="font-mono">d</span> vardır ve karıştırılmamalıdır:
          <span class="font-mono">dif</span> düz diferansiyel <span class="font-mono">d</span>'yi,
          <span class="font-mono">partial</span> kısmi türev <span class="font-mono">∂</span>'yi verir.
          Düz harf <span class="font-mono">d</span> yazarsan değişken gibi eğik dizilir,
          matematiksel olarak yanlış olur.
        </p>
        <table class="mt-half w-full border-collapse">
          <tbody>
            {#each TUREV as satir}
              <tr class="border-b border-rule">
                <td class="w-[52%] py-quarter pr-half font-mono text-[12px] leading-rule">{satir.kod}</td>
                <td class="py-quarter text-[13px] leading-rule">{satir.ne}</td>
              </tr>
            {/each}
          </tbody>
        </table>
        <p class="pencil mt-quarter">
          Blok olarak istersen dolarların yanına boşluk koy:
          <span class="font-mono">$ (dif y)/(dif x) = 2x + 3 $</span>
        </p>

        <h2 id="kisayol" class="mt-rule border-t border-rule-strong pt-half">Kısayollar</h2>
        <table class="mt-half w-full border-collapse">
          <tbody>
            {#each KISAYOLLAR as satir}
              <tr class="border-b border-rule">
                <td class="w-[46%] py-quarter pr-half font-mono text-[12px] leading-rule">{satir.kod}</td>
                <td class="py-quarter text-[13px] leading-rule">{satir.ne}</td>
              </tr>
            {/each}
          </tbody>
        </table>

        <h2 id="kazanim" class="mt-rule border-t border-rule-strong pt-half">Kazanım kodu</h2>
        <p class="mt-quarter leading-rule">
          MEB biçiminde yazılır: <span class="font-mono">MAT.9.1.2</span> — ders,
          sınıf, ünite, kazanım. Bir soruya birden çok kazanım yazabilirsin; boşluk
          veya virgülle ayır.
        </p>
        <p class="pencil mt-quarter">
          Analiz ekranı kazanım başına başarıyı buradan hesaplar. Kazanım girilmemiş
          soru analizde görünmez.
        </p>

        <h2 id="hata" class="mt-rule border-t border-rule-strong pt-half">Hata mesajları</h2>
        <p class="pencil mt-quarter">
          Editörde <span class="text-red-deep">kırmızı</span> gördüğün her şey hatadır.
          Renklendirmede kırmızı kullanılmaz; kenardaki
          <span class="text-red-deep font-bold">✗</span> ve satır altındaki dalgalı
          çizgi yalnızca derleme hatasında çıkar.
        </p>
        {#each HATALAR as hata}
          <div class="mt-half border-t border-rule pt-half">
            <p class="font-mono text-[12px] leading-rule text-red-deep">{hata.mesaj}</p>
            <p class="mt-quarter text-[13px] leading-rule">{hata.ne}</p>
          </div>
        {/each}

        <h2 id="veri" class="mt-rule border-t border-rule-strong pt-half">Verilerim nerede</h2>
        <p class="mt-quarter leading-rule">
          Her şey kendi bilgisayarında. İnternet gerekmez, hesap yoktur, hiçbir veri
          dışarı çıkmaz.
        </p>
        <pre class="ruled mt-quarter overflow-x-auto p-half font-mono text-[12px] leading-[20px]">~/Library/Application Support/tayan/</pre>
        <p class="pencil mt-quarter">
          Soru bankası, sınavlar, sınıflar ve sonuçlar bu klasördeki veritabanı
          dosyasında durur. Yedek almak için klasörü kopyalaman yeterlidir.
        </p>
      </div>
    </article>
  </div>
</div>

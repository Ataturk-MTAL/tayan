<script lang="ts">
  import { onMount } from "svelte";
  import PageShell from "$lib/components/shell/PageShell.svelte";
  import { Button } from "flowbite-svelte";
  import { api } from "$lib/api";
  import { errorText } from "$lib/editor/diagnostics";

  /**
   * Dil sunucusu kurulumu.
   *
   * İkili uygulamayla paketlenmiyor: platform başına 60 MB ve dondurulmuş bir
   * sürüm demekti. Kurulum AÇIK bir eylem — "tamamen çevrimdışı" bir üründe
   * kullanıcı sormadan ağa çıkmak kabul edilemez.
   */
  let lspStatus = $state<{ installed: boolean; path: string | null; version: string; supported: boolean } | null>(null);
  let lspBusy = $state(false);
  let lspError = $state<string | null>(null);
  let lspDone = $state<string | null>(null);

  onMount(async () => {
    try {
      lspStatus = await api.lsp.status();
    } catch (err: unknown) {
      lspError = errorText(err);
    }
  });

  async function installLsp() {
    lspBusy = true;
    lspError = null;
    lspDone = null;
    try {
      const path = await api.lsp.install();
      lspDone = path;
      lspStatus = await api.lsp.status();
    } catch (err: unknown) {
      lspError = errorText(err);
    } finally {
      lspBusy = false;
    }
  }

  async function removeLsp() {
    lspBusy = true;
    lspError = null;
    lspDone = null;
    try {
      await api.lsp.uninstall();
      lspStatus = await api.lsp.status();
    } catch (err: unknown) {
      lspError = errorText(err);
    } finally {
      lspBusy = false;
    }
  }

  /**
   * Yardım ekranı, ürünün pinlenmiş kısıtını taşır: öğretmen ham Typst yazmaya
   * zorlanmaz, ama Typst'in NE OLDUĞUNU buradan anlayabilir.
   *
   * Bu yüzden her örnek çalışan gerçek koddur — kısaltılmış, "şuna benzer bir
   * şey" değil. Kopyalayıp editöre yapıştırınca derlenir.
   */
  type Row = { code: string; description: string };

  const BASICS: Row[] = [
    { code: "$x^2$", description: "Satır içi matematik — cümlenin içinde kalır" },
    { code: "$ x^2 $", description: "Blok matematik — kendi satırına düşer, ortalanır" },
    { code: "$a/b$", description: "Kesir" },
    { code: "$(a+b)/c$", description: "Paylı kesir — birden çok terimi parantezle" },
    { code: "$sqrt(x)$", description: "Karekök" },
    { code: "$x_1$", description: "Alt indis" },
    { code: "$alpha$ $beta$ $pi$", description: "Yunan harfleri: adını yaz" },
    { code: "$sum_(i=1)^n i$", description: "Toplam sembolü, alt ve üst sınırlı" },
    { code: "$integral_0^1 f(x) dif x$", description: "İntegral" },
    { code: "*kalın*", description: "Kalın yazı" },
    { code: "_eğik_", description: "Eğik yazı" },
    { code: "#underline[altı çizili]", description: "Altı çizili" },
    { code: "- madde", description: "Madde işaretli liste" },
    { code: "+ madde", description: "Numaralı liste" },
    { code: "#image(\"resim.png\", width: 60%)", description: "Görsel ekle" },
    { code: "#v(0.5cm)", description: "Dikey boşluk" },
  ];

  const TEMPLATES = [
    {
      name: "Çoktan seçmeli",
      code: `#secenekler(dogru: "C",
  [$x = 1$],
  [$x = 2$],
  [$x = 2$ ve $x = 3$],
  [$x = 6$],
  [Hiçbiri],
)`,
      note: "dogru: kâğıda BASILMAZ. Uygulama cevap anahtarını ve madde analizini oradan kurar. Şık harfleri (A, B, C…) sıraya göre kendiliğinden verilir.",
    },
    {
      name: "Doğru / yanlış",
      code: "#dogru-yanlis(dogru: true)",
      note: "true ya da false. Kâğıda iki kutucuk basılır, cevap basılmaz.",
    },
    {
      name: "Boşluk doldurma",
      code: '#bosluk(cevap: "180|180 derece", width: 2cm)',
      note: "Kabul edilen cevapları | ile ayır. Gövdede kaç tane #bosluk varsa o kadar boşluk oluşur; puan her boşluk için ayrı sayılır.",
    },
    {
      name: "Klasik",
      code: `#cevap-alani(satir: 6, bicim: "cizgili")
#cevap-alani(satir: 10, bicim: "kareli")
#cevap-alani(satir: 8, bicim: "bos")`,
      note: 'Üç biçim: "cizgili" yazı çizgileri, "kareli" 5×5 mm kareli alan (grafik ve şema için), "bos" çerçeveli boş kutu. satir yüksekliği verir — kareli alanda bir satır bir 5 mm karedir. Genişlik verilmez: alan bulunduğu sütunun tamamını kaplar, çift sütunlu kâğıtta kendiliğinden daralır.',
    },
  ];

  /** Yunan harfleri. Typst'te adıyla yazılır, büyük harf için baş harf büyük. */
  const GREEK_LETTERS: Array<[string, string, string, string]> = [
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
  const CALCULUS: Row[] = [
    { code: "$(dif y)/(dif x)$", description: "Birinci türev — dy/dx" },
    { code: "$(dif^2 y)/(dif x^2)$", description: "İkinci türev" },
    { code: "$f'(x)$", description: "Üs notasyonu — f prim" },
    { code: "$f''(x)$", description: "İkinci türev, üs notasyonu" },
    { code: "$(partial f)/(partial x)$", description: "Kısmi türev — ∂f/∂x" },
    { code: "$(partial^2 f)/(partial x partial y)$", description: "Karma ikinci kısmi türev" },
    { code: "$integral f(x) dif x$", description: "Belirsiz integral" },
    { code: "$integral_0^1 x^2 dif x$", description: "Belirli integral" },
    { code: "$lim_(x -> 0) (sin x)/x$", description: "Limit" },
    { code: "$nabla f$", description: "Gradyan" },
    { code: "$nabla dot bold(F)$", description: "Diverjans" },
    { code: "$dif f = (partial f)/(partial x) dif x$", description: "Toplam diferansiyel" },
  ];

  const SHORTCUTS: Row[] = [
    { code: "⌘ +", description: "Önizlemeyi yakınlaştır" },
    { code: "⌘ −", description: "Önizlemeyi uzaklaştır" },
    { code: "⌘ 0", description: "Önizlemeyi gerçek boyuta getir" },
    { code: "⌘ + tekerlek", description: "Sürekli yakınlaştır / uzaklaştır" },
    { code: "⌘ Z", description: "Editörde geri al" },
    { code: "⇧ ⌘ Z", description: "İleri al" },
    { code: "Tab", description: "Girinti ekle" },
    { code: "⌃ Boşluk", description: "Otomatik tamamlama listesi" },
  ];

  const COMMON_ERRORS = [
    {
      message: "unknown variable",
      description: "Var olmayan bir komut yazdın. Genelde yazım hatası: #secenekler yerine #secenek gibi. Yukarıdaki Kalıp düğmeleri doğrusunu ekler.",
    },
    {
      message: "expected closing bracket",
      description: "Bir köşeli parantez veya parantez kapanmamış. Editörde kırmızı işaret hangi satırda olduğunu gösterir.",
    },
    {
      message: "Gövdede #secenekler(...) yok",
      description: "Soru tipi çoktan seçmeli ama gövdede şık kalıbı yok. Kaydet düğmesi bu yüzden kapalı.",
    },
    {
      message: "Doğru cevap \"X\" şıklarla eşleşmiyor",
      description: "dogru: parametresine yazdığın harf, yazdığın şık sayısından fazla. Beş şık varsa en fazla E olabilir.",
    },
  ];

  const SECTIONS = [
    { id: "kalip", name: "Soru kalıpları" },
    { id: "temel", name: "Typst temelleri" },
    { id: "gorsel", name: "Görsel ekleme" },
    { id: "lsp", name: "Dil sunucusu" },
    { id: "yunan", name: "Yunan harfleri" },
    { id: "turev", name: "Türev ve integral" },
    { id: "kisayol", name: "Kısayollar" },
    { id: "kazanim", name: "Kazanım kodu" },
    { id: "rubrik", name: "Rubrik ve cevap anahtarı" },
    { id: "hata", name: "Hata mesajları" },
    { id: "yayin", name: "Yayınla ve PDF kaydet" },
    { id: "veri", name: "Verilerim nerede" },
  ];
</script>

<PageShell title="Yardım" scroll={false}>
  <!--
    Yan gezinme kendi kaydırıcısını istiyor: makale uzunken çekmece de onunla
    kayarsa BÖLÜMLER listesi gözden kaybolur. Bu yüzden PageShell'in kaydırmasını
    kapatıp burada iki bağımsız kaydırıcı kuruyoruz.
  -->
  <div class="grid h-full min-h-0 grid-cols-[200px_1fr]">
    <nav class="min-h-0 overflow-auto border-r border-gray-200 p-3 dark:border-gray-700">
      <ul class="space-y-1">
        {#each SECTIONS as bolum}
          <li>
            <a
              href="#{bolum.id}"
              class="block rounded-lg px-3 py-1.5 text-sm text-gray-600 no-underline
                     transition-colors hover:bg-gray-100 dark:text-gray-300
                     dark:hover:bg-gray-700/60"
            >
              {bolum.name}
            </a>
          </li>
        {/each}
      </ul>
    </nav>

    <article class="min-h-0 overflow-auto px-6 py-6">
      <div class="max-w-[68ch]">
        <p class="text-gray-700 dark:text-gray-300">
          TAYAN soruları <strong class="text-gray-900 dark:text-white">Typst</strong> ile dizer.
          Typst, matematiği ve sayfa düzenini metinle anlatan bir dizgi dilidir; kelime
          işlemcinin aksine yazdığın şey ile basılan şey birebir aynıdır.
        </p>
        <p class="mt-2.5 text-sm text-gray-500 dark:text-gray-400">
          Typst öğrenmek zorunda değilsin — yukarıdaki düğmeler her şeyi hazır ekler. Ama eklenen
          şey kaynakta görünür kalır, böylece zamanla ne olduğunu kendiliğinden öğrenirsin.
        </p>

        <h2
          id="kalip"
          class="mt-8 border-t border-gray-200 pt-3 text-lg font-semibold text-gray-900
                 dark:border-gray-700 dark:text-white"
        >
          Soru kalıpları
        </h2>
        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
          Sorunun yapısı ayrı bir formda değil, kaynağın içinde durur. Böylece cevap anahtarı ile
          kâğıtta görünen asla birbirinden ayrı düşmez.
        </p>

        {#each TEMPLATES as kalip}
          <h3 class="mt-5 text-sm font-semibold text-gray-900 dark:text-white">{kalip.name}</h3>
          <pre
            class="mt-1 overflow-x-auto rounded-lg border border-gray-200 bg-gray-50 p-2.5
                   font-mono text-xs leading-5 text-gray-700 dark:border-gray-700
                   dark:bg-gray-800 dark:text-gray-300">{kalip.code}</pre>
          <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">{kalip.note}</p>
        {/each}

        <h2
          id="temel"
          class="mt-8 border-t border-gray-200 pt-3 text-lg font-semibold text-gray-900
                 dark:border-gray-700 dark:text-white"
        >
          Typst temelleri
        </h2>

        <p class="mt-1 text-gray-700 dark:text-gray-300">
          <strong class="text-gray-900 dark:text-white">Matematikte tek kural boşluktur.</strong>
          Dolar işaretinin hemen yanına yazarsan matematik cümlenin içinde kalır; boşluk bırakırsan
          kendi satırına düşer ve ortalanır.
        </p>
        <pre
          class="mt-1 overflow-x-auto rounded-lg border border-gray-200 bg-gray-50 p-2.5 font-mono
                 text-xs leading-5 text-gray-700 dark:border-gray-700 dark:bg-gray-800
                 dark:text-gray-300">Kök $x = 2$ olarak bulunur.     → cümlenin içinde
Kök $ x = 2 $ olarak bulunur.   → kendi satırında, ortalı</pre>
        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
          Yukarıdaki Matematik düğmesi satır içi, Blok matematik düğmesi bloklu olanı ekler; imleci
          de yazılacak yere koyar.
        </p>
        <table class="mt-2.5 w-full border-collapse text-sm">
          <tbody>
            {#each BASICS as satir}
              <tr class="border-b border-gray-200 dark:border-gray-700">
                <td class="w-[46%] py-1.5 pr-2.5 align-top font-mono text-xs text-gray-700 dark:text-gray-300">
                  {satir.code}
                </td>
                <td class="py-1.5 align-top text-gray-700 dark:text-gray-300">{satir.description}</td>
              </tr>
            {/each}
          </tbody>
        </table>

        <h2
          id="gorsel"
          class="mt-8 border-t border-gray-200 pt-3 text-lg font-semibold text-gray-900
                 dark:border-gray-700 dark:text-white"
        >
          Görsel ekleme
        </h2>
        <p class="mt-1 text-gray-700 dark:text-gray-300">İki yol var, ikisi de aynı sonucu verir:</p>
        <ul class="mt-1 list-inside list-disc text-gray-700 dark:text-gray-300">
          <li>
            <strong class="text-gray-900 dark:text-white">Yapıştır.</strong> Ekran görüntüsü al,
            editöre <span class="font-mono">⌘V</span> ile yapıştır.
          </li>
          <li>
            <strong class="text-gray-900 dark:text-white">Görsel düğmesi.</strong> Yukarıdaki
            şeritten dosya seç.
          </li>
        </ul>
        <p class="mt-2.5 text-gray-700 dark:text-gray-300">
          Her iki durumda da görsel uygulamanın veri klasörüne kopyalanır ve gövdeye şu şekilde
          eklenir:
        </p>
        <pre
          class="mt-1 overflow-x-auto rounded-lg border border-gray-200 bg-gray-50 p-2.5 font-mono
                 text-xs leading-5 text-gray-700 dark:border-gray-700 dark:bg-gray-800
                 dark:text-gray-300">#image("images/soru_20260901_143022_a3f7b2c1.png", width: 60%)</pre>
        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
          <span class="font-mono">width</span> değerini değiştirerek boyutlandır:
          <span class="font-mono">40%</span>, <span class="font-mono">8cm</span> gibi.
        </p>

        <h3 class="mt-5 text-sm font-semibold text-gray-900 dark:text-white">
          Hizalama ve yerleşim
        </h3>
        <pre
          class="mt-1 overflow-x-auto rounded-lg border border-gray-200 bg-gray-50 p-2.5 font-mono
                 text-xs leading-5 text-gray-700 dark:border-gray-700 dark:bg-gray-800
                 dark:text-gray-300">#align(center)[#image("images/a.png", width: 60%)]   ortalı (varsayılan)
#align(right)[#image("images/a.png", width: 60%)]    sağa dayalı
#image("images/a.png", width: 60%)                   sola dayalı

#figure(image("images/a.png", width: 60%), caption: [Bir çember])
  numaralı şekil: "Şekil 1: Bir çember" altına yazılır

#grid(columns: (1fr, auto), gutter: 0.5cm,
  [Yandaki çemberin alanını bulunuz.],
  image("images/a.png", width: 4cm),
)
  metin solda, şekil sağda</pre>
        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
          Yapıştırdığın görsel ortalı eklenir; sınav kâğıdında şekil neredeyse her zaman
          ortalanır. Değiştirmek istersen <span class="font-mono">#align</span> satırını
          düzenle.
        </p>

        <h3 class="mt-5 text-sm font-semibold text-gray-900 dark:text-white">
          Vazgeçtiğin görseller
        </h3>
        <p class="mt-1 text-gray-700 dark:text-gray-300">
          Görsel yapıştırıldığı anda diske yazılır; sorunun kaydedilmesi beklenmez. Beklenseydi
          önizleme onu gösteremezdi, çünkü Typst dosyadan okur.
        </p>
        <p class="mt-1 text-gray-700 dark:text-gray-300">
          Sorudan vazgeçersen o dosya kullanılmadan kalır. Uygulama açılışta, hiçbir soruda
          kullanılmayan ve <strong class="text-gray-900 dark:text-white">24 saatten eski</strong>
          görselleri siler. Yaş sınırı bilinçli: az önce yapıştırdığın ve henüz kaydetmediğin
          görsel hiçbir atıfta görünmez, sınır olmasa tam yazarken silinirdi.
        </p>
        <p class="mt-2.5 text-gray-700 dark:text-gray-300">
          Yol <strong class="text-gray-900 dark:text-white">göreli</strong>dir, mutlak değil. Bu
          bilinçli: mutlak yol kullanıcı adını içerir ve veri başka bir bilgisayara taşındığında
          kırılır — sınav görselsiz basılır, üstelik bunu fark etmek zordur.
        </p>
        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
          En fazla 8 MB. PNG, JPEG, GIF ve WebP desteklenir. Görseller veritabanıyla aynı klasörde
          durur, yani klasörü kopyalamak görselleri de yedekler.
        </p>

        <h2
          id="yunan"
          class="mt-8 border-t border-gray-200 pt-3 text-lg font-semibold text-gray-900
                 dark:border-gray-700 dark:text-white"
        >
          Yunan harfleri
        </h2>
        <p class="mt-1 text-gray-700 dark:text-gray-300">
          Adıyla yazılır. Büyük harf için baş harfi büyüt:
          <span class="font-mono">$alpha$</span> → α, <span class="font-mono">$Delta$</span> → Δ.
          Editörde <span class="font-mono">$</span> yazınca hepsi listelenir.
        </p>
        <table class="mt-2.5 w-full border-collapse text-sm">
          <tbody>
            {#each GREEK_LETTERS as [kucukAd, kucuk, buyukAd, buyuk]}
              <tr class="border-b border-gray-200 dark:border-gray-700">
                <td class="py-1.5 font-mono text-xs text-gray-700 dark:text-gray-300">${kucukAd}$</td>
                <td class="py-1.5 pr-2.5 text-base text-gray-900 dark:text-white">{kucuk}</td>
                <td class="py-1.5 font-mono text-xs text-gray-700 dark:text-gray-300">${buyukAd}$</td>
                <td class="py-1.5 text-base text-gray-900 dark:text-white">{buyuk}</td>
              </tr>
            {/each}
          </tbody>
        </table>
        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
          Artı-eksi için <span class="font-mono">$plus.minus$</span> yaz.
          <span class="font-mono">+-</span> işe yaramaz; Typst onu ayrı iki işaret olarak dizer.
        </p>

        <h2
          id="turev"
          class="mt-8 border-t border-gray-200 pt-3 text-lg font-semibold text-gray-900
                 dark:border-gray-700 dark:text-white"
        >
          Türev ve integral
        </h2>
        <p class="mt-1 text-gray-700 dark:text-gray-300">
          İki ayrı <span class="font-mono">d</span> vardır ve karıştırılmamalıdır:
          <span class="font-mono">dif</span> düz diferansiyel
          <span class="font-mono">d</span>'yi, <span class="font-mono">partial</span> kısmi türev
          <span class="font-mono">∂</span>'yi verir. Düz harf <span class="font-mono">d</span>
          yazarsan değişken gibi eğik dizilir, matematiksel olarak yanlış olur.
        </p>
        <table class="mt-2.5 w-full border-collapse text-sm">
          <tbody>
            {#each CALCULUS as satir}
              <tr class="border-b border-gray-200 dark:border-gray-700">
                <td class="w-[52%] py-1.5 pr-2.5 align-top font-mono text-xs text-gray-700 dark:text-gray-300">
                  {satir.code}
                </td>
                <td class="py-1.5 align-top text-gray-700 dark:text-gray-300">{satir.description}</td>
              </tr>
            {/each}
          </tbody>
        </table>
        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
          Blok olarak istersen dolarların yanına boşluk koy:
          <span class="font-mono">$ (dif y)/(dif x) = 2x + 3 $</span>
        </p>

        <h2
          id="kisayol"
          class="mt-8 border-t border-gray-200 pt-3 text-lg font-semibold text-gray-900
                 dark:border-gray-700 dark:text-white"
        >
          Kısayollar
        </h2>
        <table class="mt-2.5 w-full border-collapse text-sm">
          <tbody>
            {#each SHORTCUTS as satir}
              <tr class="border-b border-gray-200 dark:border-gray-700">
                <td class="w-[46%] py-1.5 pr-2.5 align-top font-mono text-xs text-gray-700 dark:text-gray-300">
                  {satir.code}
                </td>
                <td class="py-1.5 align-top text-gray-700 dark:text-gray-300">{satir.description}</td>
              </tr>
            {/each}
          </tbody>
        </table>

        <h2
          id="kazanim"
          class="mt-8 border-t border-gray-200 pt-3 text-lg font-semibold text-gray-900
                 dark:border-gray-700 dark:text-white"
        >
          Kazanım kodu
        </h2>
        <p class="mt-1 text-gray-700 dark:text-gray-300">
          MEB biçiminde yazılır: <span class="font-mono">MAT.9.1.2</span> — ders kodu 1-5 harf
          (Türkçe harf geçerli: <span class="font-mono">FİZ</span>,
          <span class="font-mono">COĞ</span>), sonra sınıf, sınıf, ünite, kazanım. Bir soruya
          birden çok kazanım yazabilirsin; boşluk veya virgülle ayır.
        </p>
        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
          Analiz ekranı kazanım başına başarıyı buradan hesaplar. Kazanım girilmemiş soru analizde
          görünmez.
        </p>

        <h2
          id="rubrik"
          class="mt-8 border-t border-gray-200 pt-3 text-lg font-semibold text-gray-900
                 dark:border-gray-700 dark:text-white"
        >
          Rubrik ve cevap anahtarı
        </h2>
        <p class="mt-1 text-gray-700 dark:text-gray-300">
          Açık uçlu (klasik) sorularda puanı neye göre verdiğini yazabilirsin. Soruyu açtığında
          paneldeki <b>Puanlama ölçütleri</b> bölümünden <b>+ Ölçüt ekle</b> ile satır eklersin:
          solda ölçüt metni, sağda puanı.
        </p>
        <p class="mt-1 text-gray-700 dark:text-gray-300">
          Ölçüt puanlarının toplamı <b>soru puanına eşit olmak zorunda</b>. Eşit değilken sağ
          üstteki sayaç kırmızıya döner ve kaydetme kilitlenir; altında kaç puanın dağıtılmadığı
          yazar. Ölçüt yazmak zorunlu değil — boş bırakırsan cevap anahtarına puanlama tablosu
          basılmaz.
        </p>
        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
          Ölçüt alanı Typst kabul eder: <span class="font-mono">$</span> yazınca sembol listesi
          açılır. &quot;Formül
          <span class="font-mono">$R = (V_(&quot;pin&quot;) - V_F)/I$</span>
          yazılmış&quot; gibi ölçütler böyle yazılır.
        </p>

        <h3
          class="mt-4 text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400"
        >
          Cevap anahtarını basmak
        </h3>
        <p class="mt-1 text-gray-700 dark:text-gray-300">
          Ayrı bir &quot;cevap anahtarı oluştur&quot; adımı yok. Sınavı aç,
          <b>Cevap anahtarı</b> kutucuğunu işaretle — önizleme anında değişir — sonra
          <b>PDF kaydet</b>. Dosya adı kendiliğinden <span class="font-mono">_cevap</span> ile
          biter.
        </p>
        <p class="mt-1 text-gray-700 dark:text-gray-300">
          Kutucuk işaretliyken kâğıda <b>ek olarak</b> şunlar girer: doğru şık işareti,
          doğru/yanlış cevabı, soru başlığı, puanlama ölçütleri tablosu (TOPLAM satırıyla) ve
          örnek cevap. İşaretsizken hiçbiri basılmaz.
        </p>
        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
          İki kâğıdın soru sırası birebir aynıdır: karıştırma sırası sınav kimliğinden türer.
          <b>Kitapçık türünü değiştirdiysen anahtarı da aynı türle bas</b> — B kitapçığının
          anahtarını A ile basarsan sıralar tutmaz.
        </p>
        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
          Soru başlığı da yalnız cevap anahtarında görünür. Öğrenci nüshasında basılsaydı konuyu
          ele verirdi: &quot;Dijital Çıkış — LED Sürme&quot; başlığı, sorunun neyi sorduğunu
          okumadan söyler.
        </p>
        <h3
          class="mt-4 text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400"
        >
          Eski dosyandan rubrik yapıştırmak
        </h3>
        <p class="mt-1 text-gray-700 dark:text-gray-300">
          Elindeki cevap anahtarı dosyalarında ölçütler
          <span class="font-mono">#rubrik((([ölçüt], puan), ...))</span> biçiminde yazılıysa bunu
          doğrudan soru gövdesine yapıştırabilirsin. Üstte bir şerit çıkar ve
          <b>Panele taşı ve gövdeden kaldır</b> düğmesiyle ölçütler panele geçer, blok gövdeden
          silinir.
        </p>
        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
          Yalnız düz <span class="font-mono">([ölçüt], puan)</span> demetleri taşınabiliyor.
          Değişkenle yazılmış (<span class="font-mono">#rubrik(olcutler)</span>), hesaplanmış
          puanlı (<span class="font-mono">3 + 2</span>) veya döngüyle üretilmiş rubrikler
          okunmaz — şerit sebebini yazar, ölçütleri panele elle girersin. Bu sınır bilinçli: yanlış
          okunmuş bir ölçüt yanlış not demek olurdu.
        </p>
        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
          <b>Gövdede kalan rubrik kaydettirmez.</b> Panele taşımazsan kaydetme reddedilir, çünkü
          gövdedeki blok hiçbir yerde işlemez: cevap anahtarına basılmaz, sonuç girişinde kutucuk
          çıkmaz, toplam doğrulaması çalışmaz. Ölçütleri yazdığını sanıp hiçbirinin işlememesi, en
          baştan hata almaktan kötüdür.
        </p>
        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
          Ayrıca güvenlik ağı var: önsözdeki <span class="font-mono">#rubrik</span> öğrenci
          nüshasında hiçbir şey basmaz. Gözden kaçan bir blok bile öğrencinin önüne düşemez.
        </p>

        <h3
          class="mt-4 text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400"
        >
          Ölçütlere göre puanlamak
        </h3>
        <p class="mt-1 text-gray-700 dark:text-gray-300">
          <b>Analiz &rarr; Sonuç girişi</b> ekranında açık uçlu sorunun ölçütleri kutucuk olarak
          çıkar. İşaretledikçe puan kendiliğinden toplanır. Puan kutusunu elle de
          değiştirebilirsin: ölçüte tam uymayan ama karşılığı olan bir cevabı takdir
          edebilmelisin — rubrik yardımcıdır, kelepçe değil.
        </p>
        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
          Rubriği sonradan düzenlersen <b>daha önce girilmiş notlar değişmez.</b> Puan giriş anında
          hesaplanıp kaydedilir. Yalnız kaydedilen ölçüt kırılımı eski rubriğe işaret ediyor
          olabilir.
        </p>

        <h2
          id="hata"
          class="mt-8 border-t border-gray-200 pt-3 text-lg font-semibold text-gray-900
                 dark:border-gray-700 dark:text-white"
        >
          Hata mesajları
        </h2>
        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
          Editörde <span class="text-red-600 dark:text-red-500">kırmızı</span> gördüğün her şey
          hatadır. Renklendirmede kırmızı kullanılmaz; kenardaki
          <span class="font-bold text-red-600 dark:text-red-500">✗</span> ve satır altındaki
          dalgalı çizgi yalnızca derleme hatasında çıkar.
        </p>
        {#each COMMON_ERRORS as hata}
          <div class="mt-3 border-t border-gray-200 pt-3 dark:border-gray-700">
            <p class="font-mono text-xs text-red-600 dark:text-red-500">{hata.message}</p>
            <p class="mt-1 text-sm text-gray-700 dark:text-gray-300">{hata.description}</p>
          </div>
        {/each}

        <h2
          id="lsp"
          class="mt-8 border-t border-gray-200 pt-3 text-lg font-semibold text-gray-900
                 dark:border-gray-700 dark:text-white"
        >
          Dil sunucusu
        </h2>
        <p class="mt-1 text-gray-700 dark:text-gray-300">
          Editör kutudan çıktığı hâliyle Typst'in <strong class="text-gray-900 dark:text-white"
            >560 sembolünü</strong
          > tanır: işlevler, parametreleri, matematik sembolleri, senin yazdığın
          <span class="font-mono">#let</span> tanımları. İnternet gerekmez.
        </p>
        <p class="mt-2.5 text-gray-700 dark:text-gray-300">
          İstersen <strong class="text-gray-900 dark:text-white">tinymist</strong> dil sunucusunu
          kurabilirsin. Üstüne şunları ekler: içe aktarılan paketlerin sembolleri, belge üzerinden
          hover açıklamaları ve daha isabetli öneri sıralaması.
        </p>
        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
          Uygulamayla birlikte gelmiyor — platform başına 60 MB ve dondurulmuş bir sürüm demek
          olurdu. Kurulum senin açık isteğinle yapılır; indirilen dosya sha256 ile doğrulanır,
          tutmazsa kurulmaz.
        </p>

        <div
          class="mt-2.5 rounded-lg border border-gray-200 bg-gray-50 p-4 dark:border-gray-700
                 dark:bg-gray-800"
        >
          {#if lspStatus === null}
            <p class="text-sm text-gray-500 dark:text-gray-400">Durum okunuyor…</p>
          {:else if !lspStatus.supported}
            <p class="text-sm text-red-600 dark:text-red-500">Bu platform için hazır yapı yok.</p>
          {:else if lspStatus.installed}
            <p class="text-gray-700 dark:text-gray-300">
              Kurulu — <span class="font-mono text-xs">{lspStatus.version}</span>
            </p>
            <p class="font-mono text-xs text-gray-500 dark:text-gray-400">{lspStatus.path}</p>
            <div class="mt-2.5">
              <Button size="sm" color="alternative" disabled={lspBusy} onclick={removeLsp}>
                {lspBusy ? "Kaldırılıyor…" : "Kaldır"}
              </Button>
            </div>
          {:else}
            <p class="text-gray-700 dark:text-gray-300">Kurulu değil.</p>
            <div class="mt-2.5">
              <Button size="sm" disabled={lspBusy} onclick={installLsp}>
                {lspBusy ? "İndiriliyor… (60 MB)" : `Kur (${lspStatus.version}, 60 MB)`}
              </Button>
            </div>
          {/if}

          {#if lspDone}
            <p class="mt-2.5 text-sm text-gray-500 dark:text-gray-400">
              Kuruldu: <span class="font-mono text-xs">{lspDone}</span>
            </p>
          {/if}
          {#if lspError}
            <p class="mt-2.5 text-sm text-red-600 dark:text-red-500">{lspError}</p>
          {/if}
        </div>

        <h2
          id="yayin"
          class="mt-8 border-t border-gray-200 pt-3 text-lg font-semibold text-gray-900
                 dark:border-gray-700 dark:text-white"
        >
          Yayınla ve PDF kaydet
        </h2>
        <p class="mt-1 text-gray-700 dark:text-gray-300">
          İkisi farklı işler yapar ve birbirinin yerine geçmez.
        </p>

        <dl
          class="mt-2.5 rounded-lg border border-gray-200 bg-gray-50 p-4 dark:border-gray-700
                 dark:bg-gray-800"
        >
          <dt
            class="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400"
          >
            Yayınla
          </dt>
          <dd class="mt-1 text-gray-700 dark:text-gray-300">
            Sınavın durumunu <span class="font-mono">Taslak</span> yerine
            <span class="font-mono">Yayında</span> yapar. Kâğıt basmaz, dosya üretmez. Anlamı
            şudur: <strong class="text-gray-900 dark:text-white">bu sınav artık hazır</strong> —
            soru listesi ve puanlar dondurulmuş sayılır, sonuç girilebilir. Analiz ekranı yalnız
            yayımlanmış sınavları hazır seçenek olarak alır.
          </dd>

          <dt
            class="mt-2.5 text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400"
          >
            PDF kaydet
          </dt>
          <dd class="mt-1 text-gray-700 dark:text-gray-300">
            Kâğıdı derler ve <strong class="text-gray-900 dark:text-white"
              >senin seçtiğin yere</strong
            > bir PDF dosyası yazar. Sınavın durumuna dokunmaz; taslak bir sınavın da PDF'i alınır.
            Yazıcıya gidecek olan budur.
          </dd>
        </dl>

        <p class="mt-2.5 text-sm text-gray-500 dark:text-gray-400">
          Sıralama şöyle olur: soruları ekle, puanları ayarla, kâğıdı önizle,
          <strong class="text-gray-900 dark:text-white">PDF kaydet</strong> ile bas, ve sınav
          uygulanmaya hazır olduğunda <strong class="text-gray-900 dark:text-white"
            >Yayınla</strong
          > de. Cevap anahtarı ayrı bir PDF'tir: <span class="font-mono">Cevap anahtarı</span>
          kutusunu işaretleyip tekrar kaydet.
        </p>

        <h2
          id="veri"
          class="mt-8 border-t border-gray-200 pt-3 text-lg font-semibold text-gray-900
                 dark:border-gray-700 dark:text-white"
        >
          Verilerim nerede
        </h2>
        <p class="mt-1 text-gray-700 dark:text-gray-300">
          Her şey kendi bilgisayarında. İnternet gerekmez, hesap yoktur, hiçbir veri dışarı
          çıkmaz.
        </p>
        <pre
          class="mt-1 overflow-x-auto rounded-lg border border-gray-200 bg-gray-50 p-2.5 font-mono
                 text-xs leading-5 text-gray-700 dark:border-gray-700 dark:bg-gray-800
                 dark:text-gray-300">~/Library/Application Support/tayan/</pre>
        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
          Soru bankası, sınavlar, sınıflar ve sonuçlar bu klasördeki veritabanı dosyasında durur.
          Yedek almak için klasörü kopyalaman yeterlidir.
        </p>
      </div>
    </article>
  </div>
</PageShell>

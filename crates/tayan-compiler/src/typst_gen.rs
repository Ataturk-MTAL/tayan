use tayan_core::domain::exam_management::aggregates::Exam;
use tayan_core::domain::exam_management::entities::question::Question;
use tayan_core::domain::shared::to_typst::{ToTypst, TypstContext};

#[derive(Debug, thiserror::Error)]
pub enum CompilerError {
    #[error("Question not found: {0}")]
    QuestionNotFound(String),
    #[error("Render error: {0}")]
    Render(#[from] anyhow::Error),
}

/// Converts an `Exam` + resolved questions into a complete Typst source document.
pub struct TypstGenerator;

impl TypstGenerator {
    pub fn generate_exam(
        exam:      &Exam,
        questions: &[Question],
        ctx:       TypstContext,
    ) -> Result<String, CompilerError> {
        let mut out = String::new();

        out.push_str(PREAMBLE);
        out.push_str(&exam_header(exam, ctx.booklet.as_deref()));

        // Karıştırma tohumu sınav kimliğinden gelir: aynı sınav her basıldığında
        // aynı sırayı üretir, farklı sınavlar farklı sıra alır.
        let mut ctx = ctx;
        // Tohuma kitapçık türü katılır: aynı sınavın A ve B kitapçığı farklı
        // sıra alır, ama HER İKİSİ de yeniden basıldığında kendi sırasını
        // birebir tekrarlar. Cevap anahtarı da aynı tohumdan üretildiği için
        // türle eşleşmemesi imkânsızdır.
        let seed_base = tayan_core::domain::shared::to_typst::seed_from(&exam.id.0.to_string());
        ctx.shuffle_seed = match ctx.booklet.as_deref() {
            Some(b) => seed_base ^ tayan_core::domain::shared::to_typst::seed_from(b),
            None    => seed_base,
        };

        // Çift sütun, soruları saran tek bir #columns bloğudur; başlık ve imza
        // bloğu dışarıda kalır çünkü ikisi de kâğıdın TAM genişliğine aittir.
        //
        // Soruların sütun sonunda ikiye bölünmemesi ayrı bir ayar gerektirmez:
        // her soru zaten `breakable: false` bir blokta üretiliyor.
        let two_columns = exam.meta.columns >= 2;
        if two_columns {
            // Sütun genişliği yarıya inince 11pt satır başına ~35 karakter
            // bırakıyor. 10pt kırılmayı azaltır ve kâğıtta hâlâ rahat okunur.
            out.push_str("#set text(size: 10pt)\n");
            out.push_str("#columns(2, gutter: 0.85cm)[\n");
        }

        for (i, q) in questions.iter().enumerate() {
            let q_ctx = ctx.clone().with_number((i + 1) as u32);
            out.push_str(&q.to_typst(&q_ctx));
            out.push('\n');
        }

        if two_columns {
            out.push_str("]\n");
        }

        out.push_str(&signature_block(&exam.meta.signers));

        Ok(out)
    }

    /// Tek bir soru gövdesini, sınavın GERÇEK önsözüyle sarmalayıp önizlenebilir
    /// bir Typst belgesi üretir.
    ///
    /// Önsözü ön yüze kopyalamak yerine burada tutmanın sebebi sürüklenmedir:
    /// kopyalanan bir önsöz er ya da geç asıl şablondan ayrışır ve öğretmen
    /// önizlemede gördüğünden başka bir kâğıt basar.
    /// preview_document'in gövdeden ÖNCE eklediği satır sayısı.
    ///
    /// Typst tanılamaları birleşik belgeye göre satır verir. Öğretmenin
    /// editöründe o satır yoktur: 5. satırdaki hata "satır 98" diye raporlanır,
    /// kenardaki işaret hiç çıkmaz ve mesaj kimseye bir şey söylemez.
    pub fn preview_line_offset() -> usize {
        PREAMBLE.lines().count()
    }

    /// Şablonun önsözü. Sembol dökümü buradan kendi yardımcılarını okur.
    pub fn preamble() -> &'static str {
        PREAMBLE
    }

    pub fn preview_document(body: &str) -> String {
        format!("{PREAMBLE}{body}\n")
    }

    /// Banka kartı için küçük resim belgesi.
    ///
    /// preview_document A4 üretir çünkü önsöz `page(paper: "a4")` diyor ve
    /// editörde doğrusu budur — öğretmen basılacak kâğıdı görmeli. Ama 300 px
    /// genişliğinde bir kartta A4'ün %95'i boş kalır ve soru okunmaz.
    ///
    /// Burada sayfa YENİDEN tanımlanıyor: genişlik sabit, yükseklik içeriğe
    /// göre. Böylece kart tam sorunun kapladığı kadar yer tutar.
    ///
    /// Genişlik 10cm sabit, `auto` değil: değişken genişlik satır kırılmasını
    /// karta göre değiştirir ve aynı soru iki farklı kartta farklı görünür.
    /// Yazı 9pt — A4'teki 11pt bu genişlikte kart için fazla iri.
    pub fn thumbnail_document(body: &str) -> String {
        format!(
            "{PREAMBLE}#set page(width: 10cm, height: auto, margin: 6mm)\n\
             #set text(size: 9pt)\n{body}\n"
        )
    }
}

fn escape_typst(s: &str) -> String {
    s.replace('"', "\\\"").replace('#', "\\#")
}

/// Kâğıdın alt imza bloğu. İmzacı yoksa hiç basılmaz.
///
/// `float: true` ile sayfanın altına oturur ve SON sayfada çıkar. Sütun
/// bloğunun DIŞINDA üretilir; kâğıdın tam genişliğine aittir, bir sütuna değil.
fn signature_block(signers: &[tayan_core::domain::exam_management::aggregates::ExamSigner]) -> String {
    if signers.is_empty() {
        return String::new();
    }

    let cols = vec!["1fr"; signers.len()].join(", ");
    let cells: Vec<String> = signers
        .iter()
        .map(|s| {
            format!(
                "    imzasatir(\"{}\", \"{}\"),",
                escape_typst(&s.name),
                escape_typst(&s.title)
            )
        })
        .collect();

    format!(
        "
#place(bottom + center, float: true, clearance: 12pt, block(width: 100%)[
  #line(length: 100%, stroke: 0.6pt)
  #v(3mm)
  #grid(columns: ({cols}), gutter: 0.6cm, row-gutter: 0.3cm,
{cells}
  )
  #v(2mm)
  #line(length: 100%, stroke: 0.6pt)
])
",
        cells = cells.join("\n")
    )
}

fn exam_header(exam: &Exam, booklet: Option<&str>) -> String {
    let title   = escape_typst(&exam.meta.title);
    let subject = escape_typst(&exam.meta.subject);
    let class   = escape_typst(&exam.meta.classroom);
    let teacher = escape_typst(&exam.meta.teacher);
    let dur     = exam.meta.duration_min;
    let date    = exam.meta.date.format("%d.%m.%Y").to_string();
    let count   = exam.questions.len();

    // Kurum satırları isteğe bağlı: okul adı girilmemişse boş bir satır
    // basmak yerine hiç basmıyoruz.
    let school_line = match exam.meta.school.as_deref() {
        Some(s) if !s.trim().is_empty() => format!(
            "  #text(12pt, weight: \"bold\")[{}]\n  #linebreak()\n",
            escape_typst(s)
        ),
        _ => String::new(),
    };
    let dept_line = match exam.meta.department.as_deref() {
        Some(d) if !d.trim().is_empty() => format!(
            "  #text(9.5pt, weight: \"bold\")[{}]\n  #linebreak()\n",
            escape_typst(d)
        ),
        _ => String::new(),
    };

    // Tek kitapçıkta etiket basılmaz — "Kitapçık A" yazmak, B yokken gürültüdür.
    let booklet_line = match booklet {
        Some(b) => format!(
            "  #linebreak()\n  #text(11pt, weight: \"bold\")[KİTAPÇIK {}]\n",
            escape_typst(b)
        ),
        None => String::new(),
    };

    let instructions = exam
        .meta
        .instructions
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .map(escape_typst)
        .unwrap_or_else(|| {
            format!(
                "Sınav {count} sorudan oluşur. İstediğiniz sorudan başlayabilirsiniz; \
                 işlem gerektiren sorularda işlemleri gösteriniz. Başarılar dileriz."
            )
        });

    format!(
        "#set page(footer: context [
  #set text(8pt, fill: luma(45%))
  #line(length: 100%, stroke: 0.4pt + luma(75%))
  #grid(columns: (1fr, 1fr, 1fr),
    align(left)[{subject}],
    align(center)[Sayfa #counter(page).display() / #context counter(page).final().first()],
    align(right)[{title}],
  )
])

#align(center, block(spacing: 0pt)[
  #set par(leading: 0.4em)
{school_line}{dept_line}  #text(9.5pt, weight: \"bold\")[{class} #sym.dot.c {subject} #sym.dot.c {teacher}]
  #linebreak()
  #text(10pt, weight: \"bold\")[{title}]
{booklet_line}])
#v(1.4mm)

#table(
  columns: (1fr, 0.7fr, 0.7fr, 0.7fr),
  stroke: 0.6pt + luma(40%),
  inset: (x: 6pt, y: 4pt),
  [*Adı Soyadı:*], [*No:*], [*Sınıf/Şube:*], [*Aldığı Puan:*],
)
#v(1mm)

#block(fill: luma(96%), stroke: 0.5pt + luma(70%), radius: 3pt,
       inset: (x: 7pt, y: 5pt), width: 100%,
  text(8.5pt)[
    #text(weight: \"bold\")[Açıklamalar: ] {instructions}
    #h(0.6em) Süre: *{dur} dk* #h(0.6em) Tarih: {date}
  ])
#v(1.4mm)
#line(length: 100%, stroke: 0.7pt)
#v(1.4mm)

"
    )
}

const PREAMBLE: &str = r##"#set page(paper: "a4", margin: (x: 2cm, y: 2.5cm))
#set text(lang: "tr", size: 11pt, font: "Libertinus Serif")
#set par(leading: 0.75em, justify: false)
#set list(marker: ([--], [•]))

#let blank(width: 4cm) = box(width: width, baseline: 20%, stroke: (bottom: 0.5pt + black), height: 1.1em)
// İşaretler ÇİZİLİR, Unicode'dan ödünç ALINMAZ.
//
// ☐ (U+2610), ☑ (U+2611) ve ✓ (U+2713) Libertinus Serif'te yoktur. Yedek font
// bulunamadığında Typst "tofu" basar: kutu içinde soru işareti. Kâğıtta bunun
// karşılığı, öğrencinin işaretleyeceği kutucuğun yerinde bir hata simgesi
// görünmesidir.
//
// Çizilen işaret hiçbir fonta bağlı değildir; her kurulumda aynı çıkar.
#let tik(size: 0.72em) = box(width: size, height: size, baseline: 0.08em, {
  place(dx: 0.12 * size, dy: 0.44 * size,
        line(end: (0.22 * size, 0.24 * size), stroke: 1.1pt + black))
  place(dx: 0.34 * size, dy: 0.68 * size,
        line(end: (0.44 * size, -0.52 * size), stroke: 1.1pt + black))
})

#let cb(checked: false) = box(
  width: 0.85em,
  height: 0.85em,
  baseline: 0.16em,
  stroke: 0.55pt + black,
  inset: 0pt,
  if checked { place(center + horizon, tik(size: 0.66em)) },
)

// ── Soru kalıpları ────────────────────────────────────────────────────────────
// Bunlar hem kâğıdı dizer hem de sorunun YAPISINI kaynakta taşır. Uygulama
// doğru cevabı bu çağrılardan geri okur, böylece ayrı bir form paneline gerek
// kalmaz ve tek doğru kaynak Typst metni olur.

// Çoktan seçmeli şıklar.
// `dogru` öğrenci nüshasında BASILMAZ; yalnızca uygulamanın cevap anahtarını
// ve madde analizini kurabilmesi için kaynakta durur.
// karistir: uygulamaya "bu sorunun şıkları sınavda karıştırılsın" der.
// Dizgide kullanılmaz; sıra uygulamadan sira: ile gelir, çünkü karıştırma
// soruya değil BASKIYA aittir: aynı soru A ve B kitapçığında farklı sırada
// çıkmalıdır ve kaynak tek bir sıraya hapsedilmemelidir.
#let secenekler(dogru: none, karistir: false, sira: none, anahtar: false, ..items) = {
  let harfler = ("A", "B", "C", "D", "E", "F")
  let secilenler = items.pos()
  let n = secilenler.len()

  // sira verilmezse şıklar YAZILDIĞI sırada dizilir. Verilirse o permütasyona
  // göre. Karıştırma kaynağı değiştirmez; yalnızca dizgi anında uygulanır,
  // çünkü aynı soru A ve B kitapçığında farklı sırada çıkmalıdır.
  let duzen = if sira == none { range(n) } else { sira }

  // dogru harfi YAZILDIĞI sıradaki şıkkı gösterir. Karıştırıldıktan sonraki
  // konumu burada hesaplanır; böylece cevap anahtarı sıradan bağımsız kalır ve
  // kâğıtla anahtarın ayrı düşmesi imkânsız olur.
  let dogru-index = if dogru == none { -1 } else {
    let bulunan = harfler.position(h => h == upper(dogru))
    if bulunan == none { -1 } else { bulunan }
  }

  let satirlar = ()
  for (yeni, eski) in duzen.enumerate() {
    let etiket = harfler.at(yeni) + ")"
    if anahtar and eski == dogru-index {
      satirlar.push([#etiket #tik()])
    } else {
      satirlar.push([#etiket])
    }
    satirlar.push(secilenler.at(eski))
  }
  v(0.3cm)
  grid(columns: (auto, 1fr), row-gutter: 0.45em, column-gutter: 0.5em, ..satirlar)
}

// Doğru / yanlış. `dogru` yine basılmaz, kaynakta taşınır.
#let dogru-yanlis(dogru: true) = {
  v(0.3cm)
  [#cb() Doğru #h(2em) #cb() Yanlış]
}

// Boşluk doldurma. `cevap` basılmaz; kabul edilen cevaplar | ile ayrılır.
// blank() ile aynı çizgiyi çizer, farkı cevabı kaynakta taşımasıdır.
#let bosluk(cevap: none, width: 4cm) = blank(width: width)

// Kâğıdın altındaki tek imza sütunu: çizgi, ad, unvan.
// signature_block bunu imzacı sayısı kadar bir grid içinde çağırır.
#let imzasatir(ad, unvan) = align(center)[
  #line(length: 3.2cm, stroke: 0.5pt)
  #linebreak()
  #text(weight: "bold")[#ad]
  #linebreak()
  #text(8.5pt)[#unvan]
]

// Klasik soru için cevap alanı: öğrencinin yazacağı çizgiler.
// Klasik soru için cevap alanı.
//
// bicim: "cizgili" (varsayılan) — düz yazı çizgileri
//        "kareli"              — 5x5 mm kareli alan, grafik ve şema için
//        "bos"                 — çerçeveli boş alan
//
// GENİŞLİK VERİLMEZ: alan her zaman içinde bulunduğu sütunun tamamını kaplar.
// Çift sütunlu kâğıtta sütun yarıya iner ve alan da kendiliğinden daralır;
// elle genişlik yazmak bu uyumu bozardı.
//
// `satir` her üç biçimde de yüksekliği belirler. Kareli alanda bir "satır"
// bir 5 mm karedir, yani satir: 10 → 50 mm yükseklik.
#let cevap-alani(satir: 6, bicim: "cizgili") = {
  v(0.3cm)

  if bicim == "kareli" {
    // Kare deseni tiling ile çizilir; böylece genişlik ne olursa olsun kareler
    // 5 mm kalır. Grid ile çizmek sütun sayısını önceden bilmeyi gerektirirdi
    // ve sütun genişliği değiştiğinde kareler bozulurdu.
    rect(
      width: 100%,
      height: satir * 5mm,
      stroke: 0.5pt + luma(55%),
      fill: tiling(size: (5mm, 5mm), {
        place(line(start: (0mm, 0mm), end: (5mm, 0mm), stroke: 0.3pt + luma(80%)))
        place(line(start: (0mm, 0mm), end: (0mm, 5mm), stroke: 0.3pt + luma(80%)))
      }),
    )
  } else if bicim == "bos" {
    rect(width: 100%, height: satir * 0.9em, stroke: 0.5pt + luma(55%))
  } else {
    for _ in range(satir) {
      line(length: 100%, stroke: 0.4pt + luma(65%))
      v(0.9em)
    }
  }
}

"##;

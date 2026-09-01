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

        for (i, q) in questions.iter().enumerate() {
            let q_ctx = ctx.clone().with_number((i + 1) as u32);
            out.push_str(&q.to_typst(&q_ctx));
            out.push('\n');
        }

        Ok(out)
    }

    /// Tek bir soru gövdesini, sınavın GERÇEK önsözüyle sarmalayıp önizlenebilir
    /// bir Typst belgesi üretir.
    ///
    /// Önsözü ön yüze kopyalamak yerine burada tutmanın sebebi sürüklenmedir:
    /// kopyalanan bir önsöz er ya da geç asıl şablondan ayrışır ve öğretmen
    /// önizlemede gördüğünden başka bir kâğıt basar.
    pub fn preview_document(body: &str) -> String {
        format!("{PREAMBLE}{body}\n")
    }
}

fn escape_typst(s: &str) -> String {
    s.replace('"', "\\\"").replace('#', "\\#")
}

fn exam_header(exam: &Exam, booklet: Option<&str>) -> String {
    let title   = escape_typst(&exam.meta.title);
    let subject = escape_typst(&exam.meta.subject);
    let class   = escape_typst(&exam.meta.classroom);
    let teacher = escape_typst(&exam.meta.teacher);
    let dur     = exam.meta.duration_min;

    // Tek kitapçıkta etiket basılmaz — "Kitapçık A" yazmak, B yokken gürültüdür.
    let booklet_line = match booklet {
        Some(b) => format!(
            "\n  #linebreak()\n  #text(size: 12pt, weight: \"bold\")[KİTAPÇIK {}]",
            escape_typst(b)
        ),
        None => String::new(),
    };

    format!(
        "#align(center)[
  #text(weight: \"bold\", size: 14pt)[{title}]
  #linebreak()
  #text(size: 10pt)[{subject} #h(1em) | #h(1em) {class} #h(1em) | #h(1em) {teacher}]
  #linebreak()
  #text(size: 9pt, fill: gray)[Süre: {dur} dk]{booklet_line}
]
#line(length: 100%)
#v(0.4cm)
#grid(columns: (1fr, 1fr, 1fr),
  [Ad Soyad: #underline(offset: 2pt)[#h(5cm)]],
  [No: #underline(offset: 2pt)[#h(2cm)]],
  [Puan: #underline(offset: 2pt)[#h(2cm)]],
)
#v(0.6cm)

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

// Klasik soru için cevap alanı: öğrencinin yazacağı çizgiler.
#let cevap-alani(satir: 6) = {
  v(0.3cm)
  for _ in range(satir) {
    line(length: 100%, stroke: 0.4pt + luma(65%))
    v(0.9em)
  }
}

"##;

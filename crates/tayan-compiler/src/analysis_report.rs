//! Sınav analizi raporu — Typst kaynağı üretir.
//!
//! GRAFİKLER PAKETSİZ ÇİZİLİR. cetz gibi bir çizim paketi kullanmak paket
//! indirmek demek; TAYAN'ın sarsılmaz kısıtı tamamen çevrimdışı çalışmak.
//! Çubuk grafik zaten `rect(width: %)` demek — çekirdek ilkeller yetiyor.
//!
//! SAYILAR BURADA HESAPLANMAZ. Ekranla kâğıdın ayrışmaması için bütün ölçüler
//! dışarıdan, hesaplanmış olarak geliyor. Rapor yalnız dizer.

use serde::{Deserialize, Serialize};

/// Bir sorunun rapordaki satırı. Alanlar ön yüzdeki ItemStat ile birebir.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportItem {
    pub order: u32,
    pub title: String,
    pub correct: u32,
    pub partial: u32,
    pub wrong: u32,
    pub blank: u32,
    /// Güçlük ∈ [0, 1].
    pub difficulty: f32,
    /// Ayırt edicilik; None = güvenilir hesaplanamıyor.
    pub discrimination: Option<f32>,
    pub review_note: Option<String>,
}

/// Bir öğrencinin rapordaki satırı.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportStudent {
    pub number: String,
    pub name: String,
    pub percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub exam_title: String,
    pub subject: String,
    pub classroom: String,
    pub teacher: String,
    pub date: String,
    pub school: Option<String>,
    pub department: Option<String>,
    pub mean: f32,
    pub median: f32,
    pub min: f32,
    pub max: f32,
    pub q1: f32,
    pub q3: f32,
    pub threshold: f32,
    pub below_threshold: u32,
    pub students: Vec<ReportStudent>,
    pub items: Vec<ReportItem>,
}

fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    for ch in s.chars() {
        if matches!(ch, '#' | '@' | '$' | '*' | '_' | '`' | '~' | '<' | '>' | '\\' | '[' | ']') {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}

/// Yüzdeyi 0-100 aralığına sıkıştırır.
///
/// Elle puanlanan klasik sorularda öğretmen soru puanından fazla verebiliyor;
/// sıkıştırılmazsa çubuk kutusundan taşar ve sayfa düzeni bozulur.
fn clamp_pct(v: f32) -> f32 {
    v.clamp(0.0, 100.0)
}

/// Yığılmış dağılım çubuğu: doğru / kısmi / yanlış / boş.
///
/// Parçalar arasında ince kâğıt boşluğu var; olmayınca iki koyu parça tek
/// parça gibi okunuyor ve öğretmen yanlış sayıyı görüyor.
fn dagilim_cubugu(item: &ReportItem) -> String {
    let n = (item.correct + item.partial + item.wrong + item.blank).max(1) as f32;
    let parca = |say: u32, renk: &str| -> String {
        if say == 0 {
            return String::new();
        }
        let w = (say as f32 / n) * 100.0;
        format!("rect(width: {w:.2}%, height: 7pt, stroke: none, fill: {renk}), ")
    };

    format!(
        "stack(dir: ltr, spacing: 0.6pt, {}{}{}{})",
        parca(item.correct, "rgb(\"#16233f\")"),
        parca(item.partial, "rgb(\"#16233f\").lighten(55%)"),
        parca(item.wrong, "rgb(\"#c8102e\")"),
        parca(item.blank, "luma(80%)"),
    )
}

/// Puan yayılımı: her öğrenci bir nokta.
///
/// HİSTOGRAM DEĞİL. Bir sınıfta 6-30 öğrenci var; binleme bu boyutta yayılımı
/// gizler ve kutu sınırı kaydıkça şekil değişir. Nokta grafiğinde her öğrenci
/// kendi yerinde durur. Aynı puandakiler üst üste binmesin diye istifleniyor.
fn yayilim_grafigi(r: &AnalysisReport) -> String {
    const GENISLIK_CM: f32 = 16.0;
    const YUKSEKLIK_CM: f32 = 2.4;
    const NOKTA_PT: f32 = 4.0;
    const KATMAN_CM: f32 = 0.28;
    const YIGIN_ARALIGI: f32 = 2.5;

    let mut sirali: Vec<f32> = r.students.iter().map(|s| clamp_pct(s.percentage)).collect();
    sirali.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mut yerlesim: Vec<f32> = Vec::new();
    let mut noktalar = String::new();
    for p in &sirali {
        let katman = yerlesim.iter().filter(|o| (*o - p).abs() < YIGIN_ARALIGI).count();
        yerlesim.push(*p);

        let x = GENISLIK_CM * (p / 100.0);
        let y = YUKSEKLIK_CM - 0.25 - (katman as f32) * KATMAN_CM;
        let renk = if *p < r.threshold { "rgb(\"#c8102e\")" } else { "rgb(\"#16233f\")" };
        // BAŞTAKİ `#` ŞART. İçerik bloğunda `#` olmadan yazılan şey düz
        // metindir ve metindeki `#16233f` Typst'te kod ifadesi başlatır:
        // "invalid number suffix: `f`". Renk kodu yüzünden bütün rapor
        // derlenmiyordu.
        noktalar.push_str(&format!(
            "  #place(dx: {x:.2}cm - {half}pt, dy: {y:.2}cm, circle(radius: {half}pt, fill: {renk}, stroke: none))\n",
            half = NOKTA_PT / 2.0,
        ));
    }

    let q1x = GENISLIK_CM * (clamp_pct(r.q1) / 100.0);
    let q3x = GENISLIK_CM * (clamp_pct(r.q3) / 100.0);
    let ortx = GENISLIK_CM * (clamp_pct(r.mean) / 100.0);
    let esikx = GENISLIK_CM * (clamp_pct(r.threshold) / 100.0);

    format!(
        "#block(width: {GENISLIK_CM}cm, height: {YUKSEKLIK_CM}cm, breakable: false)[\n\
         #place(dx: {q1x:.2}cm, dy: 0cm, rect(width: {kutu:.2}cm, height: {YUKSEKLIK_CM}cm, stroke: none, fill: luma(93%)))\n\
         #place(dx: {esikx:.2}cm, dy: 0cm, line(length: {YUKSEKLIK_CM}cm, angle: 90deg, stroke: (paint: rgb(\"#c8102e\"), dash: \"dashed\", thickness: 0.6pt)))\n\
         #place(dx: {ortx:.2}cm, dy: 0cm, line(length: {YUKSEKLIK_CM}cm, angle: 90deg, stroke: 0.8pt + rgb(\"#c8102e\")))\n\
         {noktalar}\
         ]\n\
         #block(width: {GENISLIK_CM}cm)[\n\
         #grid(columns: (1fr,) * 6, ..([0], [20], [40], [60], [80], [100]).map(t => text(7pt, fill: luma(40%))[#t]))\n\
         ]\n",
        kutu = (q3x - q1x).max(0.06),
    )
}

pub fn generate_report(r: &AnalysisReport) -> String {
    let mut out = String::new();

    out.push_str(
        "#set page(paper: \"a4\", margin: (x: 1.6cm, top: 1.2cm, bottom: 1.4cm), \
         footer: context [\n  #set text(8pt, fill: luma(45%))\n  \
         #line(length: 100%, stroke: 0.4pt + luma(75%))\n  \
         #grid(columns: (1fr, 1fr), align(left)[Sınav analizi], \
         align(right)[Sayfa #counter(page).display() / #context counter(page).final().first()])\n])\n",
    );
    out.push_str("#set text(font: \"Libertinus Serif\", size: 10pt, lang: \"tr\")\n");
    out.push_str("#set par(justify: false, leading: 0.65em)\n\n");

    // ── Başlık ────────────────────────────────────────────────────────────
    out.push_str("#align(center, block(spacing: 0pt)[\n");
    if let Some(okul) = &r.school {
        out.push_str(&format!("  #text(12pt, weight: \"bold\")[{}] \\\n", esc(okul)));
    }
    if let Some(alan) = &r.department {
        out.push_str(&format!("  #text(9.5pt)[{}] \\\n", esc(alan)));
    }
    out.push_str(&format!(
        "  #text(11pt, weight: \"bold\")[{} — {} — {}] \\\n  #text(9pt)[{} · {}]\n])\n",
        esc(&r.exam_title),
        esc(&r.subject),
        esc(&r.classroom),
        esc(&r.teacher),
        esc(&r.date),
    ));
    out.push_str("#v(3mm)\n#line(length: 100%, stroke: 0.7pt)\n#v(3mm)\n\n");

    // ── Özet ──────────────────────────────────────────────────────────────
    out.push_str(&format!(
        "#grid(columns: (1fr,) * 5, gutter: 6pt,\n\
         {}{}{}{}{})\n#v(4mm)\n\n",
        ozet_kutu("Öğrenci", &format!("{}", r.students.len())),
        ozet_kutu("Ortalama", &format!("%{:.0}", r.mean)),
        ozet_kutu("Ortanca", &format!("%{:.0}", r.median)),
        ozet_kutu("En düşük / yüksek", &format!("{:.0} / {:.0}", r.min, r.max)),
        ozet_kutu("Eşiğin altında", &format!("{} / {}", r.below_threshold, r.students.len())),
    ));

    // ── Yayılım ───────────────────────────────────────────────────────────
    out.push_str("#text(9pt, weight: \"bold\")[PUAN YAYILIMI]\n#v(1.5mm)\n");
    out.push_str(&yayilim_grafigi(r));
    out.push_str(&format!(
        "#v(1mm)\n#text(8pt, fill: luma(40%))[Her nokta bir öğrenci. Gri kutu ortadaki yarı \
         (%{:.0}–%{:.0}); kesiksiz çizgi ortalama (%{:.0}), kesikli çizgi geçme eşiği (%{:.0}). \
         Eşiğin altındaki noktalar kırmızı.]\n#v(5mm)\n\n",
        r.q1, r.q3, r.mean, r.threshold,
    ));

    // ── Soru soru ─────────────────────────────────────────────────────────
    out.push_str("#text(9pt, weight: \"bold\")[SORU SORU]\n#v(1.5mm)\n");
    out.push_str(
        "#table(\n  columns: (1.2cm, 1fr, 4.2cm, 1.8cm, 1.8cm),\n  \
         stroke: (x, y) => (bottom: 0.4pt + luma(75%)),\n  \
         inset: (x: 4pt, y: 3.5pt),\n  \
         align: (col, row) => if col >= 3 { right + horizon } else { left + horizon },\n  \
         table.header(\n    text(8pt, weight: \"bold\")[Soru], text(8pt, weight: \"bold\")[Başlık],\n    \
         text(8pt, weight: \"bold\")[Dağılım], text(8pt, weight: \"bold\")[Güçlük],\n    \
         text(8pt, weight: \"bold\")[Ayırt]\n  ),\n",
    );
    for item in &r.items {
        let ayirt = match item.discrimination {
            Some(d) => format!("{d:.2}"),
            None => "—".into(),
        };
        let not = match &item.review_note {
            Some(n) => format!(
                " \\\n    #text(7.5pt, fill: rgb(\"#c8102e\"))[{}]",
                esc(n)
            ),
            None => String::new(),
        };
        out.push_str(&format!(
            "  text(9pt)[{}], [#text(9pt)[{}]{}],\n  {},\n  text(9pt)[%{:.0}], text(9pt)[{}],\n",
            item.order,
            esc(&item.title),
            not,
            dagilim_cubugu(item),
            item.difficulty * 100.0,
            ayirt,
        ));
    }
    out.push_str(")\n");
    out.push_str(
        "#v(1.5mm)\n#text(8pt, fill: luma(40%))[Dağılım çubuğu soldan sağa: doğru, kısmi, \
         yanlış, boş. Güçlük sınıfın o sorudan aldığı puanın alınabilecek puana oranıdır; \
         yüksek değer soru kolay demektir. Ayırt edicilik üst %27 ile alt %27 arasındaki \
         farktır, 0.20'nin altı sorunun iyi ve zayıf öğrenciyi ayırmadığını gösterir. \
         \"—\" hesaplanamadı demektir.]\n#v(5mm)\n\n",
    );

    // ── Öğrenci listesi ───────────────────────────────────────────────────
    out.push_str("#text(9pt, weight: \"bold\")[ÖĞRENCİ SONUÇLARI]\n#v(1.5mm)\n");
    out.push_str(
        "#table(\n  columns: (1.6cm, 1fr, 1.6cm, 5cm),\n  \
         stroke: (x, y) => (bottom: 0.4pt + luma(75%)),\n  \
         inset: (x: 4pt, y: 3pt),\n  \
         align: (col, row) => if col == 2 { right + horizon } else { left + horizon },\n  \
         table.header(\n    text(8pt, weight: \"bold\")[No], text(8pt, weight: \"bold\")[Ad Soyad],\n    \
         text(8pt, weight: \"bold\")[Yüzde], text(8pt, weight: \"bold\")[]\n  ),\n",
    );
    for s in &r.students {
        let pct = clamp_pct(s.percentage);
        let renk = if pct < r.threshold { "rgb(\"#c8102e\")" } else { "rgb(\"#16233f\")" };
        out.push_str(&format!(
            "  text(9pt)[{}], text(9pt)[{}], text(9pt)[%{:.0}],\n  \
             rect(width: {:.2}%, height: 6pt, stroke: none, fill: {renk}),\n",
            esc(&s.number),
            esc(&s.name),
            pct,
            pct.max(0.5),
        ));
    }
    out.push_str(")\n");

    out
}

fn ozet_kutu(baslik: &str, deger: &str) -> String {
    format!(
        "  block(width: 100%, inset: (x: 5pt, y: 4pt), fill: luma(96%))[\
         #text(7.5pt, fill: luma(35%))[{}] \\ #text(13pt, weight: \"bold\")[{}]],\n",
        esc(baslik),
        esc(deger),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::TayanWorld;

    /// Kaynak ÜRETMEK yetmez; Typst'in onu gerçekten dizebildiği görülmeli.
    ///
    /// Hata mesajına kaynağın numaralı satırları ekleniyor: Typst "satır 36"
    /// diyor ama o satır üretilen belgede, elimizdeki kodda değil. Satırı
    /// göstermeden hata ayıklamak kör aramaya dönüşüyor.
    fn derlenir(r: &AnalysisReport) -> Result<usize, String> {
        let kaynak = generate_report(r);
        TayanWorld::compile_svg(kaynak.clone())
            .map(|s| s.len())
            .map_err(|e| {
                let numarali: String = kaynak
                    .lines()
                    .enumerate()
                    .map(|(i, l)| format!("{:>3} | {l}\n", i + 1))
                    .collect();
                format!("{e}\n\n--- üretilen kaynak ---\n{numarali}")
            })
    }

    fn ornek() -> AnalysisReport {
        AnalysisReport {
            exam_title: "1. Dönem 1. Yazılı".into(),
            subject: "Matematik".into(),
            classroom: "9-A".into(),
            teacher: "Hakan GÜLEN".into(),
            date: "2026-11-12".into(),
            school: Some("Atatürk Mesleki ve Teknik Anadolu Lisesi".into()),
            department: Some("Elektrik-Elektronik Teknolojisi Alanı".into()),
            mean: 53.0,
            median: 65.0,
            min: 15.0,
            max: 90.0,
            q1: 30.0,
            q3: 70.0,
            threshold: 50.0,
            below_threshold: 3,
            students: vec![
                ReportStudent { number: "101".into(), name: "Ayşe YILMAZ".into(), percentage: 70.0 },
                ReportStudent { number: "102".into(), name: "Berk DEMİR".into(), percentage: 45.0 },
                ReportStudent { number: "105".into(), name: "Emre ÇELİK".into(), percentage: 15.0 },
            ],
            items: vec![
                ReportItem {
                    order: 1,
                    title: "Sayı Sistemleri".into(),
                    correct: 4,
                    partial: 0,
                    wrong: 2,
                    blank: 0,
                    difficulty: 0.67,
                    discrimination: Some(0.45),
                    review_note: None,
                },
                ReportItem {
                    order: 2,
                    title: "Doğrusal Fonksiyon Grafiği".into(),
                    correct: 0,
                    partial: 6,
                    wrong: 0,
                    blank: 0,
                    difficulty: 0.15,
                    discrimination: None,
                    review_note: Some("Çok zor — kimse yapamamış".into()),
                },
            ],
        }
    }

    #[test]
    fn rapor_derlenir() {
        assert!(derlenir(&ornek()).expect("rapor derlenmeli") >= 1);
    }

    #[test]
    fn sonucsuz_rapor_cokmez() {
        // Sonuç girilmemiş sınavda da rapor alınabilmeli; boş tablo çökmemeli.
        let mut r = ornek();
        r.students = vec![];
        r.items = vec![];
        derlenir(&r).expect("boş rapor derlenmeli");
    }

    #[test]
    fn yuzde_yuzun_ustu_sayfayi_tasirmaz() {
        // Klasik soruda öğretmen soru puanından fazla verebiliyor. Sıkıştırma
        // olmasaydı çubuk kutusundan taşar ve sayfa düzeni bozulurdu.
        let mut r = ornek();
        r.students[0].percentage = 140.0;
        derlenir(&r).expect("taşan yüzde derlenmeli");
        assert_eq!(clamp_pct(140.0), 100.0);
        assert_eq!(clamp_pct(-5.0), 0.0);
    }

    #[test]
    fn ozel_karakterli_ad_kaynagi_bozmaz() {
        // Öğrenci adı ya da soru başlığı Typst'te anlamlı karakter içerebilir.
        let mut r = ornek();
        r.students[0].name = "Ali #[deneme] $x$ *kalın*".into();
        r.items[0].title = "Dizi [a] ve #hesap()".into();
        derlenir(&r).expect("kaçırılmış metin derlenmeli");
    }

    #[test]
    fn tek_ogrencide_kutu_gorunur_kalir() {
        // q1 == q3 olduğunda kutu genişliği sıfıra iner ve grafik kaybolur;
        // en az bir tutam genişlik kalmalı.
        let mut r = ornek();
        r.students = vec![r.students[0].clone()];
        r.q1 = 70.0;
        r.q3 = 70.0;
        derlenir(&r).expect("tek öğrencili rapor derlenmeli");
    }
}

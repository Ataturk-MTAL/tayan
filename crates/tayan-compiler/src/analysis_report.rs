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
    /// Mod: en kalabalık puan aralığının orta noktası.
    pub mode: f32,
    /// Standart sapma (örneklem, n-1).
    pub sd: f32,
    /// Çarpıklık katsayısı; None = güvenilir hesaplanamıyor.
    pub skewness: Option<f32>,
    /// Çarpıklığın sözle karşılığı. EKRANDA ÜRETİLİR: eşikleri iki yerde
    /// tutmak, kâğıtla ekranın farklı yorum yazması demekti.
    pub skew_label: String,
    /// Frekans dağılımı: her aralıktaki öğrenci sayısı, 0'dan 100'e.
    pub bins: Vec<u32>,
    pub bin_width: u32,
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

/// Puan dağılımı: FREKANS DİKEY, PUAN YATAY.
///
/// Ölçme-değerlendirmenin standart görünümü. Öğretmen mod, medyan ve
/// ortalamanın birbirine göre yerinden sınıfın durumunu okuyor: mod > medyan >
/// ortalama sola çarpıktır ve sınıf başarılıdır, ters sıra sınıfın
/// zorlandığını gösterir.
///
/// ÇUBUKLARIN ALTINDA HAM NOKTALAR VAR. Binleme küçük sınıfta yanıltır:
/// altı öğrencide her aralığa bir kişi düşer ve grafik veriyi değil aralık
/// genişliğini gösterir. Nokta şeridi her öğrenciyi kendi puanında gösterir.
fn yayilim_grafigi(r: &AnalysisReport) -> String {
    const GENISLIK_CM: f32 = 15.0;
    const YUKSEKLIK_CM: f32 = 3.2;
    const SERIT_CM: f32 = 0.45;
    const NOKTA_PT: f32 = 3.0;

    let x = |p: f32| GENISLIK_CM * (clamp_pct(p) / 100.0);
    let en_yuksek = r.bins.iter().copied().max().unwrap_or(1).max(1) as f32;
    let aralik_sayisi = r.bins.len().max(1) as f32;
    let cubuk_w = GENISLIK_CM / aralik_sayisi;

    let mut govde = String::new();

    // Çubuklar. Aralarında ince kâğıt boşluğu var; bitişik çubuklar tek parça
    // gibi okunur ve öğretmen frekansı yanlış sayar.
    for (i, say) in r.bins.iter().enumerate() {
        if *say == 0 {
            continue;
        }
        let h = YUKSEKLIK_CM * (*say as f32 / en_yuksek);
        govde.push_str(&format!(
            "  #place(dx: {dx:.3}cm, dy: {dy:.3}cm, rect(width: {w:.3}cm, height: {h:.3}cm, stroke: none, fill: rgb(\"#16233f\")))\n",
            dx = i as f32 * cubuk_w + 0.03,
            dy = YUKSEKLIK_CM - h,
            w = cubuk_w - 0.06,
        ));
    }

    // Geçme eşiği: kesikli. Yatay eksenin anlam ortası.
    govde.push_str(&format!(
        "  #place(dx: {:.2}cm, dy: 0cm, line(length: {YUKSEKLIK_CM}cm, angle: 90deg, stroke: (paint: rgb(\"#c8102e\"), dash: \"dashed\", thickness: 0.6pt)))\n",
        x(r.threshold),
    ));

    // Mod, medyan, ortalama. Etiketleri üstte; üçü çakışırsa bile hangisinin
    // nerede olduğu okunabilsin diye farklı yüksekliklere yazılıyor.
    for (i, (ad, deger)) in [("Mod", r.mode), ("Medyan", r.median), ("Ort", r.mean)]
        .iter()
        .enumerate()
    {
        let dx = x(*deger);
        govde.push_str(&format!(
            "  #place(dx: {dx:.2}cm, dy: 0cm, line(length: {YUKSEKLIK_CM}cm, angle: 90deg, stroke: 0.7pt + rgb(\"#c8102e\")))\n\
             #place(dx: {dx:.2}cm + 1.5pt, dy: {dy:.2}cm, text(6.5pt, fill: rgb(\"#c8102e\"))[{ad}])\n",
            dy = 0.02 + i as f32 * 0.32,
        ));
    }

    // Ham puanlar.
    let mut noktalar = String::new();
    for s in &r.students {
        let p = clamp_pct(s.percentage);
        let renk = if p < r.threshold { "rgb(\"#c8102e\")" } else { "rgb(\"#16233f\")" };
        noktalar.push_str(&format!(
            "  #place(dx: {dx:.2}cm - {half}pt, dy: 0.12cm, circle(radius: {half}pt, fill: {renk}, stroke: none))\n",
            dx = x(p),
            half = NOKTA_PT / 2.0,
        ));
    }

    format!(
        "#block(width: {GENISLIK_CM}cm, height: {YUKSEKLIK_CM}cm, breakable: false, \
         stroke: (bottom: 0.5pt + luma(60%), left: 0.5pt + luma(60%)))[\n{govde}]\n\
         #block(width: {GENISLIK_CM}cm, height: {SERIT_CM}cm, breakable: false)[\n{noktalar}]\n\
         #block(width: {GENISLIK_CM}cm)[\n\
         #grid(columns: (1fr,) * 6, ..([0], [20], [40], [60], [80], [100]).map(t => text(7pt, fill: luma(40%))[#t]))\n\
         ]\n",
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
    let carpiklik = match r.skewness {
        Some(s) => format!("{s:.2}"),
        None => "—".into(),
    };

    out.push_str(&format!(
        "#v(1mm)\n#text(8pt, fill: luma(40%))[Yatay eksen puan, dikey eksen frekans \
         ({} puanlık aralıklar). Kesikli çizgi geçme eşiği (%{:.0}); işaretli dikey \
         çizgiler mod, medyan ve ortalama. Çubukların altındaki noktalar tek tek \
         öğrencileri gösterir; eşiğin altındakiler kırmızı.]\n#v(3mm)\n\n",
        r.bin_width, r.threshold,
    ));

    // Merkezî eğilim ölçüleri ve çarpıklık — öğretmenin alanının dili.
    out.push_str(&format!(
        "#grid(columns: (1fr,) * 5, gutter: 6pt,\n{}{}{}{}{})\n\
         #v(1.5mm)\n#text(8pt)[*Yorum:* {}]\n#v(5mm)\n\n",
        ozet_kutu("Mod", &format!("{:.0}", r.mode)),
        ozet_kutu("Medyan", &format!("{:.1}", r.median)),
        ozet_kutu("Ortalama", &format!("{:.1}", r.mean)),
        ozet_kutu("Std. sapma", &format!("{:.1}", r.sd)),
        ozet_kutu("Çarpıklık", &carpiklik),
        esc(&r.skew_label),
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
            mode: 65.0,
            sd: 28.4,
            skewness: Some(-0.42),
            skew_label: "Simetrik — puanlar ortada toplanmış".into(),
            bins: vec![0, 1, 0, 1, 0, 1, 1, 1, 0, 1],
            bin_width: 10,
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

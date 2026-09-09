import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "scripts"))

from tymm_pdf_sections import parse_units
from tymm_outcomes import SKILL_PREFIXES, detect_scheme, parse_outcomes

SAMPLE = """9. SINIF
1. ÜNİTE: FİZİK BİLİMİ VE KARİYER KEŞFİ
Bu ünitede öğrencilerin fizik bilimini tanımlamaları amaçlanmaktadır.
DERS SAATİ 8
ALAN
BECERİLERİ            FBAB2. Sınıflandırma, FBAB10. Tümevarımsal Akıl Yürütme
KAVRAMSAL
BECERİLER             KB2.8. Sorgulama, KB2.15. Yansıtma
EĞİLİMLER             E1.4. Kendine İnanma, E3.8. Soru Sorma
Değerler              D3. Çalışkanlık, D19. Vatanseverlik
Okuryazarlık Becerileri OB1. Bilgi Okuryazarlığı
İLİŞKİLER             KB2.7. Karşılaştırma
İÇERİK ÇERÇEVESİ      Fizik Bilimi
Anahtar Kavramlar     fizik bilimi, bilimsel araştırma merkezi
2. ÜNİTE: KUVVET VE HAREKET
Bu ünitede kuvvet ele alınmaktadır.
DERS SAATİ 24
"""


class ParseUnitsTest(unittest.TestCase):
    def test_iki_unite_bulunur(self):
        units = parse_units(SAMPLE, "FİZ")
        self.assertEqual(len(units), 2)

    def test_baslik_ve_sinif_okunur(self):
        unit = parse_units(SAMPLE, "FİZ")[0]
        self.assertEqual(unit["grade"], 9)
        self.assertEqual(unit["unit"], 1)
        self.assertEqual(unit["title"], "FİZİK BİLİMİ VE KARİYER KEŞFİ")

    def test_ders_saati_sayiya_cevrilir(self):
        self.assertEqual(parse_units(SAMPLE, "FİZ")[0]["lesson_hours"], 8)
        self.assertEqual(parse_units(SAMPLE, "FİZ")[1]["lesson_hours"], 24)

    def test_cok_satirli_etiket_birlestirilir(self):
        # "ALAN" ve "BECERİLERİ" ayrı satırlarda; tek bölüm olmalı.
        sections = parse_units(SAMPLE, "FİZ")[0]["sections"]
        self.assertIn("ALAN BECERİLERİ", sections)
        self.assertIn("FBAB2. Sınıflandırma", sections["ALAN BECERİLERİ"])

    def test_aciklama_baslik_ile_ders_saati_arasindadir(self):
        unit = parse_units(SAMPLE, "FİZ")[0]
        self.assertTrue(unit["description"].startswith("Bu ünitede"))
        self.assertNotIn("DERS SAATİ", unit["description"])

    def test_anahtar_kavramlar_okunur(self):
        sections = parse_units(SAMPLE, "FİZ")[0]["sections"]
        self.assertEqual(
            sections["Anahtar Kavramlar"], "fizik bilimi, bilimsel araştırma merkezi"
        )


# Belgelerin başında "PROGRAMIN YAPISI" bölümünde ÖRNEK bir ünite gösteriliyor.
# Gerçek üniteyle aynı (sınıf, ünite) anahtarını taşıyor ama yalnız birkaç
# bölümü var. Fizik'te bu, 15 yerine 16 ünite üretiyordu.
DUPLICATE_SAMPLE = """12. SINIF
2. ÜNİTE: KUVVET VE HAREKET
Örnek gösterim.
DERS SAATİ 24
İÇERİK ÇERÇEVESİ      Örnek içerik
12. SINIF
2. ÜNİTE: KUVVET VE HAREKET
Gerçek ünite açıklaması.
DERS SAATİ 24
ALAN
BECERİLERİ            FBAB2. Sınıflandırma
KAVRAMSAL
BECERİLER             KB2.8. Sorgulama
EĞİLİMLER             E1.4. Kendine İnanma
İÇERİK ÇERÇEVESİ      Gerçek içerik
Anahtar Kavramlar     kuvvet, hareket
"""


class DuplicateUnitTest(unittest.TestCase):
    def test_ayni_unite_bir_kez_doner(self):
        units = parse_units(DUPLICATE_SAMPLE, "FİZ")
        self.assertEqual(len(units), 1)

    def test_en_dolgun_blok_kazanir(self):
        unit = parse_units(DUPLICATE_SAMPLE, "FİZ")[0]
        self.assertIn("ALAN BECERİLERİ", unit["sections"])
        self.assertEqual(unit["sections"]["İÇERİK ÇERÇEVESİ"], "Gerçek içerik")
        self.assertTrue(unit["description"].startswith("Gerçek"))


# Ünite başlığı her derste "ÜNİTE" demiyor: Matematik "N. TEMA", Hayat Bilgisi
# "N. ÖĞRENME ALANI" kullanıyor. Yalnız ÜNİTE aranınca 111 dersin çoğunda
# çıktılar üniteye bağlanamıyor ve sahipsiz kalıyordu (3069/3877).
TEMA_SAMPLE = """1. SINIF
1. TEMA: SAYILAR VE NİCELİKLER
Bu temada sayılar ele alınmaktadır.
DERS SAATİ 40
İÇERİK ÇERÇEVESİ      Doğal sayılar
"""

OGRENME_ALANI_SAMPLE = """1. SINIF
2. ÖĞRENME ALANI: SAĞLIĞIM VE GÜVENLİĞİM
Bu öğrenme alanında güvenlik ele alınmaktadır.
DERS SAATİ 20
İÇERİK ÇERÇEVESİ      Güvenli davranış
"""


class UnitHeaderVariantTest(unittest.TestCase):
    def test_tema_basligi_taninir(self):
        units = parse_units(TEMA_SAMPLE, "MAT")
        self.assertEqual(len(units), 1)
        self.assertEqual(units[0]["unit"], 1)
        self.assertEqual(units[0]["title"], "SAYILAR VE NİCELİKLER")
        self.assertEqual(units[0]["kind"], "TEMA")

    def test_ogrenme_alani_basligi_taninir(self):
        units = parse_units(OGRENME_ALANI_SAMPLE, "HB")
        self.assertEqual(len(units), 1)
        self.assertEqual(units[0]["unit"], 2)
        self.assertEqual(units[0]["kind"], "ÖĞRENME ALANI")
        self.assertEqual(units[0]["lesson_hours"], 20)

    def test_unite_basliginda_kind_alani_dolu(self):
        self.assertEqual(parse_units(SAMPLE, "FİZ")[0]["kind"], "ÜNİTE")


# Türkçe programları NOKTALI ön ek ve ÜÇ sayılı kod kullanıyor: "T.O.5.2." =
# ön ek T.O (okuma), sınıf 5, sıra 2. Ünite kademesi yok. Dört sayılı kod
# arayan bir tarayıcı bu derslerde hiçbir çıktı bulamıyordu — 31 dersin 27'si
# bu yüzden boş kalmıştı.
THREE_SEGMENT = """T.O.5.2. Akıcı okuyabilme
   a) Metni uygun hızda okur.
   b) Noktalama işaretlerine dikkat eder.
T.O.5.3. Söz varlığını geliştirebilme
   a) Bilmediği kelimeleri belirler.
"""

FOUR_SEGMENT = """FİZ.9.1.2. Fizik biliminin alt dallarını sınıflandırabilme
   a) Alt dalların niteliklerini belirler.
"""


class OutcomeSchemeTest(unittest.TestCase):
    def test_dort_sayili_sema_bulunur(self):
        prefixes, segments = detect_scheme(FOUR_SEGMENT)
        self.assertEqual(prefixes, ["FİZ"])
        self.assertEqual(segments, 4)

    def test_uc_sayili_sema_bulunur(self):
        prefixes, segments = detect_scheme(THREE_SEGMENT)
        self.assertEqual(prefixes, ["T.O"])
        self.assertEqual(segments, 3)

    def test_uc_sayili_kod_ayristirilir(self):
        outcomes = parse_outcomes(THREE_SEGMENT, "T.O", 3)
        self.assertEqual(len(outcomes), 2)
        self.assertEqual(outcomes[0]["code"], "T.O.5.2")
        self.assertEqual(outcomes[0]["grade"], 5)
        self.assertIsNone(outcomes[0]["unit"])
        self.assertEqual(outcomes[0]["order"], 2)
        self.assertEqual(outcomes[0]["text"], "Akıcı okuyabilme")
        self.assertEqual(len(outcomes[0]["components"]), 2)

    def test_dort_sayili_kod_ayristirilir(self):
        outcomes = parse_outcomes(FOUR_SEGMENT, "FİZ", 4)
        self.assertEqual(outcomes[0]["code"], "FİZ.9.1.2")
        self.assertEqual(outcomes[0]["grade"], 9)
        self.assertEqual(outcomes[0]["unit"], 1)
        self.assertEqual(outcomes[0]["order"], 2)


# Bir ders BİRDEN FAZLA ön ek kullanabilir: Türkçe'de T.D (dinleme), T.O
# (okuma), T.Y (yazma), T.K (konuşma) ayrı ayrı geçiyor. Yalnız baskın ön eki
# almak Ortaokul Türkçe'de 444 koddan 93'ünü getiriyordu.
MULTI_PREFIX = """T.O.5.2. Akıcı okuyabilme
   a) Metni uygun hızda okur.
T.D.5.1. Dinlediğini anlayabilme
   a) Dinlediği metnin konusunu belirler.
T.Y.5.3. Yazabilme
   a) Yazma amacını belirler.
"""

# Beceri çerçevesinin kodları (D1.1, KB2.8, OB4.3) ÇIKTI DEĞİLDİR. Okul
# Öncesi belgesinde bunlar çıktı sanılıp ön ek olarak seçilmişti.
SKILL_CODE_NOISE = """D.1.1. Adalet değeri
MYB.5.1. Motor beceriyi sergileyebilme
   a) Hareketi uygular.
MYB.5.2. Denge kurabilme
   a) Dengeyi korur.
"""


class MultiPrefixTest(unittest.TestCase):
    def test_tum_on_ekler_toplanir(self):
        prefixes, segments = detect_scheme(MULTI_PREFIX)
        self.assertEqual(segments, 3)
        self.assertEqual(set(prefixes), {"T.O", "T.D", "T.Y"})

    def test_tum_on_eklerin_ciktilari_ayristirilir(self):
        prefixes, segments = detect_scheme(MULTI_PREFIX)
        outcomes = parse_outcomes(MULTI_PREFIX, prefixes, segments)
        self.assertEqual(len(outcomes), 3)
        self.assertEqual({o["code"] for o in outcomes}, {"T.O.5.2", "T.D.5.1", "T.Y.5.3"})

    def test_beceri_on_ekleri_cikti_sayilmaz(self):
        prefixes, segments = detect_scheme(SKILL_CODE_NOISE)
        self.assertNotIn("D", prefixes)
        self.assertIn("MYB", prefixes)

    def test_bilinen_beceri_on_ekleri_listelenmis(self):
        for prefix in ("KB", "SDB", "OB", "D", "E"):
            self.assertIn(prefix, SKILL_PREFIXES)


# Dört sayılı belgelerde özet tablolarında bir sürü ÇIPLAK üç sayılı atıf var
# ("FB.3.1" gibi). Toplam geçiş sayısına bakan bir seçim bunları görüp belgeyi
# üç sayılı sanıyordu: FB.3.1.2. kodu FB.3.1 + metin "2 ..." diye kesiliyor,
# süreç bileşenleri kayboluyordu (7608 -> 4106).
FOUR_WITH_NOISE = """FB.3.1.1. Bilimsel gözlem yapabilme
   a) Gözlem amacını belirler.
FB.3.1.2. Sınıflandırabilme
   a) Ölçüt belirler.
FB.3.1.3. Çıkarım yapabilme
   a) Veriye dayalı sonuç üretir.
Özet tablo: FB.3.1 FB.3.1 FB.3.1 FB.3.1 FB.3.1 FB.3.1 FB.3.1 FB.3.1
Devam: FB.3.2 FB.3.2 FB.3.2 FB.3.2 FB.3.2 FB.3.2 FB.3.2 FB.3.2
"""


class SchemeTieBreakTest(unittest.TestCase):
    def test_ciplak_atiflar_semayi_dusurmez(self):
        prefixes, segments = detect_scheme(FOUR_WITH_NOISE)
        self.assertEqual(segments, 4)
        self.assertEqual(prefixes, ["FB"])

    def test_dort_sayili_ciktilar_butun_kalir(self):
        prefixes, segments = detect_scheme(FOUR_WITH_NOISE)
        outcomes = parse_outcomes(FOUR_WITH_NOISE, prefixes, segments)
        self.assertEqual(len(outcomes), 3)
        self.assertEqual(outcomes[0]["code"], "FB.3.1.1")
        self.assertEqual(outcomes[0]["text"], "Bilimsel gözlem yapabilme")
        self.assertEqual(sum(len(o["components"]) for o in outcomes), 3)


# Sonunda nokta OLMAYAN dört sayılı atıflar ("FB.5.1.1" gibi, tablolarda bol)
# üç sayılı desene de uyuyor: FB.5.1 + metin "1". Fen Bilimleri'nde bu sahte
# eşleşmeler 182'ye 181 ile şemayı düşürüyor ve tüm süreç bileşenleri
# kayboluyordu. İmza: üç sayılı eşleşmenin metni RAKAMLA başlar.
TRUNCATED_FOUR = """FB.3.1.1. Bilimsel gözlem yapabilme
   a) Gözlem amacını belirler.
FB.3.1.2. Sınıflandırabilme
   a) Ölçüt belirler.
Tablo satırı FB.5.1.1
Tablo satırı FB.5.1.2
Tablo satırı FB.5.2.1
Tablo satırı FB.5.2.2
Tablo satırı FB.5.3.1
"""


class TruncatedReferenceTest(unittest.TestCase):
    def test_noktasiz_dort_sayili_atif_semayi_dusurmez(self):
        prefixes, segments = detect_scheme(TRUNCATED_FOUR)
        self.assertEqual(segments, 4)

    def test_bilesenler_korunur(self):
        prefixes, segments = detect_scheme(TRUNCATED_FOUR)
        outcomes = parse_outcomes(TRUNCATED_FOUR, prefixes, segments)
        self.assertEqual(len(outcomes), 2)
        self.assertEqual(sum(len(o["components"]) for o in outcomes), 2)


if __name__ == "__main__":
    unittest.main()

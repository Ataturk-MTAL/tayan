import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "scripts"))

from tymm_pdf_sections import parse_units

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


if __name__ == "__main__":
    unittest.main()

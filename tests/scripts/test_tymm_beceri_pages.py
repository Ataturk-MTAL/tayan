"""Beceri sayfası ayrıştırıcısının sessiz kayıp noktaları.

Buradaki her sınıf, çıktıda ÖLÇÜLEN bir kayba karşılık gelir; hiçbiri
varsayımsal değildir. Kayıpların ortak biçimi aynı: ayrıştırıcı satırı
tanıyor ama kaydı bir yere yazmadan düşürüyor.
"""

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "scripts"))

from tymm_beceri_lines import normalize_ws
from fetch_tymm_beceriler import merge_sets, page_title_of, parse_page


def lines(text: str) -> list[str]:
    return [line.strip() for line in text.strip().splitlines() if line.strip()]


class NormalizeWhitespaceTest(unittest.TestCase):
    def test_bolunmez_bosluk_sadelesir(self):
        self.assertEqual(normalize_ws("Öğretim programı"), "Öğretim programı")

    def test_yinelenen_bosluk_tek_bosluga_iner(self):
        self.assertEqual(normalize_ws("Bilimsel  Gözlem   Becerisi"), "Bilimsel Gözlem Becerisi")

    def test_bas_ve_son_bosluk_atilir(self):
        self.assertEqual(normalize_ws("  Sorgulama \n"), "Sorgulama")


class SetNameTest(unittest.TestCase):
    """Sayfa başlığı iki seti birden anıyor; kısaltmalı satır tek seti anıyor."""

    PAGE = """
    Yabancı Dil Alan Becerileri ve Yabancı Dil Destekleyici Beceriler
    Daha fazla oku
    Yabancı Dil Alan Becerileri (YDAB)
    YDAB1.Dinleme Becerisi
    YDDB.Yabancı Dil Destekleyici Becerileri
    YDDB1.Dil Bilgisi Becerisi
    """

    def test_kisaltmali_satir_seti_adlandirir(self):
        sets = {s["code"]: s for s in parse_page("/x", lines(self.PAGE))}
        self.assertEqual(sets["YDAB"]["name"], "Yabancı Dil Alan Becerileri")

    def test_sayfa_basligindan_kod_soneki_atilir(self):
        self.assertEqual(
            page_title_of(["Türkçe Alan Becerileri (TAB)"]), "Türkçe Alan Becerileri"
        )


class CanonicalNameTest(unittest.TestCase):
    """Kod satırı adı kısaltıyor, başlık tam adı taşıyor: 'Bilimsel Gözlem' vs
    'Bilimsel Gözlem Becerisi'. Ders programları TAM adla atıf yapıyor."""

    PAGE = """
    Fen Bilimleri Alan Becerileri (FBAB)
    Daha fazla oku
    Bilimsel Gözlem Becerisi (FBAB1)
    Doğadaki olayları duyu organlarıyla incelemektir.
    FBAB1.Bilimsel Gözlem
    """

    def test_baslik_adi_uzatiyorsa_kanonik_ad_olur(self):
        node = parse_page("/x", lines(self.PAGE))[0]["skills"][0]
        self.assertEqual(node["name"], "Bilimsel Gözlem Becerisi")

    def test_baslik_adi_uzatmiyorsa_dokunulmaz(self):
        page = lines("""
        Sosyal-Duygusal Öğrenme Becerileri
        Daha fazla oku
        SDB2.Sosyal Yaşam Becerileri (SDB2)
        SDB2.Sosyal Yaşam Becerileri
        """)
        node = parse_page("/x", page)[0]["skills"][0]
        self.assertEqual(node["name"], "Sosyal Yaşam Becerileri")


class MultiLineDescriptionTest(unittest.TestCase):
    """Açıklama ilk parçada kesiliyordu; ikinci satır sessizce düşüyordu."""

    def test_dugum_aciklamasi_birlestirilir(self):
        page = lines("""
        Erdem-Değer-Eylem Çerçevesi
        Daha fazla oku
        D18.Temizlik
        Kişisel temizlik ve bakıma dikkat ederek kendini korumayı
        kapsar.
        """)
        node = parse_page("/x", page)[0]["skills"][0]
        self.assertEqual(
            node["description"],
            "Kişisel temizlik ve bakıma dikkat ederek kendini korumayı kapsar.",
        )

    def test_baslik_aciklamasi_birlestirilir(self):
        page = lines("""
        Yabancı Dil Alan Becerileri (YDAB)
        Daha fazla oku
        1.1.Dinleme Becerisi (YDAB1)
        Alımlayıcı becerilerde dinleme becerisi birinci sıradadır.
        Yabancı dil dinleme becerisi üç gruba ayrılmıştır.
        YDAB1.Dinleme Becerisi (Bütüncül Yaklaşım)
        """)
        node = parse_page("/x", page)[0]["skills"][0]
        self.assertIn("üç gruba ayrılmıştır", node["description"])
        self.assertIn("birinci sıradadır", node["description"])


class GroupLabelTest(unittest.TestCase):
    """'Anlama Becerileri' gibi kodsuz gruplama kademesi hiç yakalanmıyordu."""

    def test_grup_etiketi_dugume_islenir(self):
        page = lines("""
        Türkçe Alan Becerileri (TAB)
        Daha fazla oku
        Anlama Becerileri
        TAB1.Dinleme Becerisi
        Anlatma Becerileri
        TAB5.Konuşma Becerisi
        """)
        skills = {k["code"]: k for k in parse_page("/x", page)[0]["skills"]}
        self.assertEqual(skills["TAB1"]["group"], "Anlama Becerileri")
        self.assertEqual(skills["TAB5"]["group"], "Anlatma Becerileri")

    def test_sayfa_basligi_grup_sayilmaz(self):
        page = lines("""
        Kavramsal Beceriler
        Daha fazla oku
        Kavramsal Beceriler
        KB2.Bütünleşik Beceriler
        """)
        node = parse_page("/x", page)[0]["skills"][0]
        self.assertNotIn("group", node)


class BorrowedNodeTest(unittest.TestCase):
    """Alan sayfaları KB kodlarını KENDİ disiplinlerine göre yeniden ifade
    ediyor. Birleştirme bunları düşürüyordu: 34 düğüm, 99 süreç bileşeni."""

    def two_pages(self):
        canonical = parse_page("/beceriler/kavramsal-beceriler", lines("""
        Kavramsal Beceriler
        Daha fazla oku
        KB2.8.Sorgulama Becerisi
        Süreç bileşenleri
        KB2.8.SB1. Sorgulanacak durumu belirlemek
        """))
        borrowed = parse_page("/beceriler/sosyal-bilimler-alan-becerileri", lines("""
        Sosyal Bilimler Alan Becerileri (SBAB)
        Daha fazla oku
        KB2.8.Sorgulama Becerisi
        Süreç bileşenleri
        KB2.8.SB1. Tarihsel kanıtı sorgulanacak duruma bağlamak
        """))
        return canonical + borrowed

    def test_farkli_metinli_bilesenler_varyant_olarak_saklanir(self):
        merged, _ = merge_sets(self.two_pages())
        node = [k for k in merged["KB"]["skills"] if k["code"] == "KB2.8"][0]
        self.assertEqual(len(node["variants"]), 1)
        self.assertEqual(
            node["variants"][0]["components"][0]["text"],
            "Tarihsel kanıtı sorgulanacak duruma bağlamak",
        )

    def test_kanonik_bilesen_korunur(self):
        merged, _ = merge_sets(self.two_pages())
        node = [k for k in merged["KB"]["skills"] if k["code"] == "KB2.8"][0]
        self.assertEqual(
            node["components"][0]["text"], "Sorgulanacak durumu belirlemek"
        )

    def test_ayni_metinli_tekrar_varyant_uretmez(self):
        same = parse_page("/beceriler/kavramsal-beceriler", lines("""
        Kavramsal Beceriler
        Daha fazla oku
        KB2.8.Sorgulama Becerisi
        Süreç bileşenleri
        KB2.8.SB1. Sorgulanacak durumu belirlemek
        """))
        merged, report = merge_sets(same + parse_page(
            "/beceriler/matematik-alan-becerileri", lines("""
            Matematik Alan Becerileri (MAB)
            Daha fazla oku
            KB2.8.Sorgulama Becerisi
            Süreç bileşenleri
            KB2.8.SB1. Sorgulanacak durumu belirlemek
            """)))
        node = [k for k in merged["KB"]["skills"] if k["code"] == "KB2.8"][0]
        self.assertNotIn("variants", node)
        self.assertEqual(report["repeated_nodes"], 1)


class AnomalyPassthroughTest(unittest.TestCase):
    """Anomali üretiliyordu ama birleştirme kopyalamadığı için JSON'a hiç
    ulaşmıyordu; yanlış kodlu bileşen uyarısız doğru görünüyordu."""

    def test_anomali_birlestirmeden_gecer(self):
        page = parse_page("/x", lines("""
        Din Eğitimi ve Öğretimi Alan Becerileri (DAB)
        Daha fazla oku
        DAB4.4.Yorumlama Becerisi
        Süreç bileşenleri
        DAB4.3.SB4. Yorumu gerekçelendirmek
        """))
        merged, _ = merge_sets(page)
        anomalies = merged["DAB"]["anomalies"]
        self.assertEqual(len(anomalies), 1)
        self.assertEqual(anomalies[0]["kind"], "component_code_mismatch")
        self.assertEqual(anomalies[0]["code"], "DAB4.3.SB4")


if __name__ == "__main__":
    unittest.main()

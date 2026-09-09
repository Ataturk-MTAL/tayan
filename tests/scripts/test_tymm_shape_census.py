"""Kod şekli sayımı ve manifest denetimi.

Bu modül `detect_scheme` sezgisinin yerini aldı. Sezgi tek aileyi hacme göre
seçiyor ve azınlıkta kalanı SESSİZCE düşürüyordu; buradaki her sınıf o
sessizliğin bir biçimine karşılık gelir.
"""

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "scripts"))

from tymm_outcomes import SKILL_PREFIXES
from tymm_shape_census import (
    bootstrap_roles,
    census,
    check_manifest,
    declared_families,
    outcome_families,
    shape_key,
)


def roles_of(text):
    return bootstrap_roles(census(text), SKILL_PREFIXES, free_min=1)


def by_key(text):
    return {shape_key(s): s for s in census(text)}


class CensusDispositionTest(unittest.TestCase):
    """Bir eşleşme dört sebepten "serbest" olmayabilir; ayırt edilmezse hepsi
    gerçek geçiş sayılır ve gölge aileler çıktı ailesiyle yarışır."""

    def test_ayni_konumdaki_uzun_kod_kisayi_golgeler(self):
        # "GS.9.1.1." hem noktalı-4 hem noktalı-3 eşler; kısa olan gölgedir.
        shapes = by_key("GS.9.1.1. Sanat dallarını tanıyabilme\n")
        self.assertEqual(shapes["GS/noktalı-4"]["free"], 1)
        self.assertEqual(shapes["GS/noktalı-3"]["free"], 0)
        self.assertEqual(shapes["GS/noktalı-3"]["shadow_of"], "GS/noktalı-4")

    def test_son_noktasi_dusmus_kod_kirpik_sayilir(self):
        # "FİZ.9.2.1" (son nokta yok) noktalı-3 eşler ama kuyruk rakamla başlar.
        shapes = by_key("Tablo: FİZ.9.2.1\n")
        self.assertEqual(shapes["FİZ/noktalı-3"]["truncated"], 1)
        self.assertEqual(shapes["FİZ/noktalı-3"]["free"], 0)

    def test_uzun_kodun_ortasindan_kopan_esitleme_ic_sayilir(self):
        # "DE.5.1.SP3.1." içindeki "SP3.1." yapışık-2 eşler; SP ön ek DEĞİL.
        shapes = by_key("DE.5.1.SP3.1. Vorbereitung auf den Sprechprozess\n")
        self.assertEqual(shapes["SP/yapışık-2"]["inner"], 1)
        self.assertEqual(shapes["SP/yapışık-2"]["free"], 0)

    def test_kardes_kodlar_ic_sayilmaz(self):
        # "KB2.4. Çözümleme, KB2.14. Yorumlama" — ikincisinden önce BOŞLUK var.
        shapes = by_key("KB2.4. Çözümleme, KB2.14. Yorumlama\n")
        self.assertEqual(shapes["KB/yapışık-2"]["inner"], 0)

    def test_a_etiketli_satir_isaretlenir(self):
        shapes = by_key("   a) TDE1.1.1. Seçim yapar.\n")
        self.assertEqual(shapes["TDE/yapışık-3"]["labelled"], 1)


class BootstrapRoleTest(unittest.TestCase):
    def test_beceri_on_eki_atif_sayilir(self):
        self.assertEqual(roles_of("KB2.8. Sorgulama\n")["KB/yapışık-2"], "skill_reference")

    def test_ingilizce_ceviri_beceri_adlari_da_atiftir(self):
        # Sezgisel çoklu aile bu yüzden bozuluyordu: CS x542 hacimde
        # ENG'i geçiyor ve gerçek çıktı ailesini gölgeliyordu.
        roles = roles_of("CS2.1. Conflict Resolution Skill\nSELS1.1. Self-Awareness\n")
        self.assertEqual(roles["CS/yapışık-2"], "skill_reference")
        self.assertEqual(roles["SELS/yapışık-2"], "skill_reference")

    def test_etiketli_kademe_bilesen_sayilir(self):
        text = (
            "TDE1.1. Dinlemeyi yönetebilme\n"
            "   a) TDE1.1.1. Seçim yapar.\n"
            "   b) TDE1.1.2. İlişkiyi sürdürür.\n"
        )
        roles = roles_of(text)
        self.assertEqual(roles["TDE/yapışık-3"], "component")
        self.assertEqual(roles["TDE/yapışık-2"], "outcome")

    def test_bilesen_kademesi_derinlik_yarisina_girmez(self):
        # "En derin şema çıktıdır" kuralı bileşeni de sayarsa çıktı kademesi
        # "başlık" sayılır ve Türk Dili'nin 239 çıktısı sıfırlanır.
        text = (
            "TDE1.1. Dinlemeyi yönetebilme\n"
            "   a) TDE1.1.1. Seçim yapar.\n"
        )
        self.assertNotEqual(roles_of(text)["TDE/yapışık-2"], "heading")

    def test_sig_kademe_baslik_sayilir(self):
        text = (
            "MAT.1.1. Sayılar ve İşlemler\n"
            "MAT.1.1.1. Rakamları tanıyabilme\n"
            "MAT.1.1.2. Sayıları karşılaştırabilme\n"
        )
        roles = roles_of(text)
        self.assertEqual(roles["MAT/noktalı-4"], "outcome")
        self.assertEqual(roles["MAT/noktalı-3"], "heading")

    def test_noktali_alt_aile_hayalet_sayilmaz(self):
        # "MAT.H" (hazırlık) MAT'in kırpılmışı DEĞİLDİR; seyrek diye elenirse
        # dersin hazırlık çıktıları tümden kaybolur.
        text = "".join(f"MAT.9.1.{i}. Çıktı {i}\n" for i in range(1, 13))
        text += "MAT.H.1.1. Hazırlık çıktısı\n"
        self.assertEqual(roles_of(text)["MAT.H/noktalı-3"], "outcome")

    def test_kirpilmis_on_ek_hayalet_sayilir(self):
        text = "".join(f"TAB{i}.1. Beceri {i}\n" for i in range(1, 12))
        text += "TA3.2. Kırpılmış satır\n"
        self.assertEqual(roles_of(text)["TA/yapışık-2"], "noise")

    def test_bosluklu_desen_satir_sonunu_asmaz(self):
        # "SÜRE\n 1.3." — kendi satırındaki kelime, sonraki satırın bölüm
        # numarasıyla birleşip ön ek sanılıyordu; altı sahte aile.
        self.assertNotIn("SÜRE/boşluklu-2", by_key("SÜRE\n     1.3. TABLOLARI\n"))

    def test_bosluklu_gercek_kod_taninir(self):
        self.assertIn("TKMT/boşluklu-2", by_key("TKMT 3.1. Eğitimdeki değişimi sorgulayabilme\n"))


class ManifestCheckTest(unittest.TestCase):
    """Değişmez: sayımdaki her şekil manifestte rol almalı."""

    TEXT = "FİZ.9.1.1. Fizik biliminin tanımı\nKB2.8. Sorgulama\n"

    def test_beyan_edilmeyen_sekil_hata_uretir(self):
        errors = check_manifest(census(self.TEXT), {"outcome": ["FİZ/noktalı-4"]})
        kinds = {(e["kind"], e["shape"]) for e in errors}
        self.assertIn(("undeclared_shape", "KB/yapışık-2"), kinds)

    def test_tam_beyan_hatasiz(self):
        declared = {
            role: [shape_key(s) for s in census(self.TEXT)
                   if bootstrap_roles(census(self.TEXT), SKILL_PREFIXES, 1)[shape_key(s)] == role]
            for role in ("outcome", "skill_reference", "shadow", "noise", "heading", "component")
        }
        self.assertEqual(check_manifest(census(self.TEXT), declared), [])

    def test_kaynaktan_kalkan_beyan_bildirilir(self):
        declared = {"outcome": ["YOK/noktalı-4"]}
        kinds = {e["kind"] for e in check_manifest(census("ABC.1.1.1. x\n"), declared)}
        self.assertIn("stale_declaration", kinds)


class DeclaredFamiliesTest(unittest.TestCase):
    def test_beyan_ayristiriciya_cevrilir(self):
        families = declared_families({"outcome": ["GS/noktalı-4", "GS.H/noktalı-3"]})
        self.assertEqual(sorted(families), [(["GS"], 4), (["GS.H"], 3)])

    def test_ayni_semadaki_on_ekler_birlesir(self):
        families = declared_families({"outcome": ["T.O/noktalı-3", "T.D/noktalı-3"]})
        self.assertEqual(families, [(["T.D", "T.O"], 3)])

    def test_yalniz_outcome_rolu_ayristirilir(self):
        families = declared_families({
            "outcome": ["GS/noktalı-4"],
            "skill_reference": ["KB/yapışık-2"],
            "heading": ["GS/noktalı-3"],
        })
        self.assertEqual(families, [(["GS"], 4)])

    def test_cok_aileli_belge_ikisini_de_verir(self):
        text = (
            "".join(f"GS.9.1.{i}. Çıktı {i}\n" for i in range(1, 6))
            + "".join(f"GS.H.1.{i}. Hazırlık çıktısı {i}\n" for i in range(1, 4))
        )
        families = outcome_families(text, SKILL_PREFIXES, free_min=1)
        self.assertEqual(sorted(families), [(["GS"], 4), (["GS.H"], 3)])


if __name__ == "__main__":
    unittest.main()

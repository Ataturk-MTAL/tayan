import type { Question } from "$lib/types";

/**
 * Başlangıç ders listesi.
 *
 * Kapsayıcı değil, OLAMAZ da: meslek liselerinde her alanın kendi dersleri var
 * ve okul bazında değişir. Bu yüzden ders alanı serbest girişli bir combobox —
 * liste yalnızca öneri, kilit değil.
 *
 * Amaç yazım birliği: "Matematik" ile "matematik"in iki ayrı ders sayılmasını
 * önlemek. Kazanım kodu (MAT.9.1.2) derse bağlı olduğu için bu birlik, ileride
 * kazanım eşleşmesinin doğru çalışmasının ön şartı.
 */
export const STARTER_SUBJECTS: string[] = [
  // Ortak dersler
  "Türk Dili ve Edebiyatı",
  "Matematik",
  "Fizik",
  "Kimya",
  "Biyoloji",
  "Tarih",
  "T.C. İnkılap Tarihi ve Atatürkçülük",
  "Coğrafya",
  "İngilizce",
  "Din Kültürü ve Ahlak Bilgisi",
  "Felsefe",
  "Beden Eğitimi ve Spor",
  "Görsel Sanatlar",
  "Müzik",
  "Sağlık Bilgisi ve Trafik Kültürü",
  "Bilgisayar Bilimi",

  // Elektrik-Elektronik Teknolojisi alanı
  "Mesleki Gelişim Atölyesi",
  "Elektrik-Elektronik Esasları",
  "Doğru Akım Devreleri",
  "Alternatif Akım Devreleri",
  "Elektronik Devreler",
  "Sayısal Elektronik",
  "Mikrodenetleyici ve Güvenlik Atölyesi",
  "Güvenlik Sistemleri",
  "Elektrik Motorları ve Sürücüleri",
  "Kumanda Teknikleri",
  "Endüstriyel Kontrol ve Arıza Analizi",
  "Görüntü ve Ses Sistemleri",
];

/**
 * Öneri listesi: bankada GERÇEKTEN kullanılan dersler önce, sonra başlangıç
 * listesinin kalanı.
 *
 * Öğretmenin kendi sözlüğü zamanla birikir ve en üste çıkar; benim yazdığım
 * liste yalnızca boş bir bankada işe yarar. Sıralama bilerek böyle: en sık
 * yazılan ders ilk açılışta parmağın altında olmalı.
 */
export function subjectSuggestions(questions: Question[]): string[] {
  const used: string[] = [];
  for (const q of questions) {
    const s = q.meta?.subject?.trim();
    if (s && !used.includes(s)) used.push(s);
  }

  const rest = STARTER_SUBJECTS.filter((s) => !used.includes(s));
  return [...used.sort((a, b) => a.localeCompare(b, "tr")), ...rest];
}

/** Sınıf seviyesi seçenekleri: 1-12. Kapalı liste, serbest giriş yok. */
export const GRADE_OPTIONS = Array.from({ length: 12 }, (_, i) => ({
  value: String(i + 1),
  label: `${i + 1}. sınıf`,
}));

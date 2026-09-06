/**
 * Analiz ekranının paylaşılan seçimi — kartları birbirine bağlayan şey.
 *
 * NEDEN ORTAK DURUM, PROP DEĞİL. Beş kart var ve her biri hem seçim
 * ÜRETEBİLİYOR hem seçime TEPKİ verebiliyor: haritada bölge fırçalamak soru
 * seçer, kazanım çubuğuna tıklamak da soru seçer, dağılımda aralık fırçalamak
 * öğrenci seçer. Prop'la bağlasaydık her kart diğer dördünü tanımak zorunda
 * kalırdı — beş bileşen, yirmi bağ. Ortak durumda her kart yalnız BURAYI
 * tanıyor.
 *
 * BOŞ SEÇİM = SÜZGEÇ YOK, "hiçbiri" DEĞİL. Bu ayrım kritik: liste boşken
 * bütün satırlar görünür. Tersi olsaydı ekran açılışta bomboş gelirdi.
 *
 * SEÇİM SÜZMÜYOR, VURGULUYOR. Seçim dışı satırlar gizlenmiyor, soluklaşıyor.
 * Gizlemek iki şeyi bozar: toplam sayılar (yüzdeler neye göre hesaplandı?) ve
 * öğretmenin bağlamı — "seçtiğim beş soru diğer on beşe göre nerede duruyor"
 * sorusu ancak diğerleri de görünürken cevaplanır.
 */

class AnalysisSelection {
  /** Seçili soru kimlikleri. Boş = süzgeç yok. */
  questionIds = $state<string[]>([]);
  /** Seçili öğrenci kimlikleri. Boş = süzgeç yok. */
  studentIds = $state<string[]>([]);

  /**
   * Seçimin nereden geldiği — yalnız arayüzdeki "temizle" satırını
   * anlamlandırmak için. Mantığı etkilemiyor.
   */
  source = $state<string | null>(null);

  /**
   * İmlecin üstünde durduğu soru/öğrenci — SEÇİMDEN AYRI bir kavram.
   *
   * Seçim kalıcı ve niyetlidir (fırçaladın, tıkladın); imleç geçicidir ve
   * niyet taşımaz. İkisini tek alanda tutsaydık fareyi gezdirmek seçimi
   * silerdi. Ayrı tutulunca haritada bir soruya gelmek tablodaki satırı
   * aydınlatıyor ama seçime dokunmuyor.
   */
  hoveredQuestionId = $state<string | null>(null);
  hoveredStudentId = $state<string | null>(null);

  get isActive(): boolean {
    return this.questionIds.length > 0 || this.studentIds.length > 0;
  }

  /*
    Set'ler TÜRETİLMİŞ: her satır için `includes()` çağırmak 220 öğrenci ×
    20 soruluk bir ızgarada 4400 doğrusal arama demek. Set ile sabit zaman.
  */
  questionSet = $derived(new Set(this.questionIds));
  studentSet = $derived(new Set(this.studentIds));

  hasQuestion(id: string): boolean {
    return this.questionSet.size === 0 || this.questionSet.has(id);
  }

  hasStudent(id: string): boolean {
    return this.studentSet.size === 0 || this.studentSet.has(id);
  }
}

export const selection = new AnalysisSelection();

export function selectQuestions(ids: string[], source: string): void {
  selection.questionIds = ids;
  selection.source = ids.length > 0 ? source : null;
}

export function selectStudents(ids: string[], source: string): void {
  selection.studentIds = ids;
  selection.source = ids.length > 0 ? source : null;
}

/**
 * Bir soruyu seçime ekler/çıkarır — haritada tek noktaya tıklamak için.
 *
 * Tıklama fırçayı EZMİYOR, üstüne ekliyor: öğretmen bir bölge fırçalayıp
 * sonra tek bir soruyu daha katabilsin.
 */
export function toggleQuestion(id: string, source: string): void {
  const has = selection.questionIds.includes(id);
  selectQuestions(
    has ? selection.questionIds.filter((x) => x !== id) : [...selection.questionIds, id],
    source,
  );
}

export function hoverQuestion(id: string | null): void {
  selection.hoveredQuestionId = id;
}

export function hoverStudent(id: string | null): void {
  selection.hoveredStudentId = id;
}

export function clearSelection(): void {
  selection.hoveredQuestionId = null;
  selection.hoveredStudentId = null;
  selection.questionIds = [];
  selection.studentIds = [];
  selection.source = null;
}

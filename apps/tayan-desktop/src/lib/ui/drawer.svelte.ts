/**
 * Sol navigasyon çekmecesinin açık/kapalı durumu.
 *
 * DURUM BURADA, İKİ BİLEŞENDE DEĞİL. Çekmecenin kendisi `+layout.svelte`'de,
 * onu açıp kapatan düğme ise `PageShell.svelte`'in başlık şeridinde duruyor.
 * Aralarında `{@render children()}` sınırı var; prop geçemezler. Tema seçimi
 * de aynı sorunu yaşıyordu ve çözümü `theme.svelte.ts`: modül düzeyinde
 * `$state` tutan bir sınıf, tek örnek olarak dışa verilir. Aynı desen.
 *
 * ÇEKMECE HER ZAMAN İÇERİĞİN ÜSTÜNE BİNER. Önceki sürüm geniş pencerede
 * içeriğe yer AÇIYOR, yalnız dar pencerede üste biniyordu; iki kip, iki ayrı
 * yerleşim ve "geniş penceredeki kalıcı tercih" diye üçüncü bir değişken
 * gerektiriyordu. Tek kip bunların hepsini siliyor.
 *
 * KALICILIK YOK, BİLEREK. Üste binen bir çekmecenin "açık" hâlini hatırlamak,
 * uygulamayı her açılışta içeriğin üstü örtülü başlatırdı. Overlay geçici bir
 * katmandır; kalıcı bir tercih değil.
 *
 * `layout.svelte.ts` İLE KARIŞTIRILMAMALI: o, soru EDİTÖRÜNÜN panel yerleşimi
 * (taraf, genişlik, görünüm modu). Bu dosya uygulama KABUĞUNUN çekmecesi.
 */

class DrawerState {
  /**
   * Çekmece görünür mü.
   *
   * KAPALI BAŞLAR. Açık başlasaydı uygulama, kullanıcı istemeden içeriğin
   * üstünü örten bir katmanla açılırdı.
   */
  open = $state(false);
}

export const drawer = new DrawerState();

export function openDrawer() {
  drawer.open = true;
}

export function closeDrawer() {
  drawer.open = false;
}

export function toggleDrawer() {
  drawer.open = !drawer.open;
}

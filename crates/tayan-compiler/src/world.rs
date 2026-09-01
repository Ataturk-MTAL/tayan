use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

use anyhow::{bail, Context};
use chrono::Datelike;
use typst::{
    LibraryExt, WorldExt,
    diag::{FileError, FileResult, PackageError, SourceDiagnostic, Warned},
    foundations::{Bytes, Datetime, Duration},
    syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot},
    text::{Font, FontBook, FontInfo},
    utils::LazyHash,
    Library, World,
};
use typst_layout::PagedDocument;

/// Typst'in memoization önbelleğini (comemo) budar.
///
/// comemo süreç genelinde ve SINIRSIZ büyür. Tek atışlık bir CLI'da bu sorun
/// değildir; uzun ömürlü bir masaüstü uygulamasında, üstelik her tuş
/// duraklamasında yeniden derleyen bir canlı önizlemede, doğrudan bellek
/// sızıntısıdır. typst CLI de her derlemeden sonra aynı çağrıyı yapar.
///
/// Eşik 5: son 5 tahliye turunda kullanılmayan girdiler atılır. Ardışık
/// derlemeler arasındaki paylaşımı korurken sınırsız büyümeyi keser.
const COMEMO_RETAIN_ROUNDS: usize = 5;

fn evict_memo_cache() {
    comemo::evict(COMEMO_RETAIN_ROUNDS);
}

/// Tahliyeyi Drop'a bağlamak, hata yollarında da çalışmasını garanti eder.
/// Derleme hatası da önbelleğe girdi yazar; erken dönüşte atlanırsa sızıntı
/// tam olarak hata ayıklarken, yani en sık derlenen anda birikir.
struct EvictOnDrop;

impl Drop for EvictOnDrop {
    fn drop(&mut self) {
        evict_memo_cache();
    }
}

// ── World ─────────────────────────────────────────────────────────────────────

pub struct TayanWorld {
    source:       Source,
    library:      &'static LazyHash<Library>,
    book:         &'static LazyHash<FontBook>,
    fonts:        &'static [FontSlot],
    package_root: PathBuf,
    file_cache:   Mutex<HashMap<FileId, Bytes>>,
}

impl TayanWorld {
    pub fn new(source_text: String) -> anyhow::Result<Self> {
        let (book, fonts) = font_registry();
        let package_root = typst_package_cache_dir()?;

        // 0.15'te FileId bir RootedPath'ten üretiliyor; kök açıkça belirtiliyor.
        let vpath = VirtualPath::new("main.typ")
            .map_err(|e| anyhow::anyhow!("Sanal yol kurulamadı: {e:?}"))?;
        let file_id = FileId::new(RootedPath::new(VirtualRoot::Project, vpath));
        let source = Source::new(file_id, source_text);

        Ok(Self {
            source,
            library: standard_library(),
            book,
            fonts,
            package_root,
            file_cache: Mutex::new(HashMap::new()),
        })
    }

    pub fn compile_pdf(source_text: String) -> anyhow::Result<Vec<u8>> {
        let world = Self::new(source_text)?;
        let _evict = EvictOnDrop;
        let Warned { output, warnings: _ } = typst::compile(&world);

        let doc = output.map_err(|errors| {
            let msgs: Vec<String> = errors
                .iter()
                .map(|e: &SourceDiagnostic| format_diagnostic(&world, e))
                .collect();
            anyhow::anyhow!("Typst derleme hatası:\n{}", msgs.join("\n"))
        })?;

        typst_pdf::pdf(&doc, &typst_pdf::PdfOptions::default())
            .map_err(|errors| {
                let msgs: Vec<String> = errors
                    .iter()
                    .map(|e: &SourceDiagnostic| format_diagnostic(&world, e))
                    .collect();
                anyhow::anyhow!("PDF oluşturma hatası:\n{}", msgs.join("\n"))
            })
    }

    /// Canlı önizleme yolu: sayfa başına bir SVG dizesi döndürür.
    ///
    /// PDF yolundan ayrıdır çünkü önizleme her tuş vuruşunda yeniden derlenir.
    /// Sayfaları ayrı ayrı döndürmek, ön yüzün yalnızca değişen sayfayı
    /// değiştirmesine ve kaydırma konumunu korumasına izin verir; tek parça
    /// PDF'te bu mümkün değildir.
    pub fn compile_svg(source_text: String) -> anyhow::Result<Vec<String>> {
        let world = Self::new(source_text)?;
        let _evict = EvictOnDrop;
        let Warned { output, warnings: _ } = typst::compile::<PagedDocument>(&world);

        let doc = output.map_err(|errors| {
            let msgs: Vec<String> = errors
                .iter()
                .map(|e: &SourceDiagnostic| format_diagnostic(&world, e))
                .collect();
            anyhow::anyhow!("Typst derleme hatası:\n{}", msgs.join("\n"))
        })?;

        // 0.15'te svg() seçenek alıyor ve pages bir metot.
        let opts = typst_svg::SvgOptions::default();
        Ok(doc.pages().iter().map(|p| typst_svg::svg(p, &opts)).collect())
    }
}

fn format_diagnostic(world: &TayanWorld, diag: &SourceDiagnostic) -> String {
    let mut msg = diag.message.to_string();

    // 0.15'te span çözümü WorldExt::range üzerinden yapılıyor; Source::range
    // artık ham SpanNumber istiyor ve doğrudan kullanılması amaçlanmamış.
    let loc = world.range(diag.span).and_then(|range| {
        let id = diag.span.id()?;
        let src = world.source(id).ok()?;
        let (line, col) = src.lines().byte_to_line_column(range.start)?;
        Some((line + 1, col + 1))
    });

    if let Some((line, col)) = loc {
        msg.push_str(&format!(" (satır {line}, sütun {col})"));
    }

    if !diag.hints.is_empty() {
        let hints = diag.hints
            .iter()
            // 0.15'te ipuçları Spanned<EcoString>; metin .v alanında.
            .map(|h| h.v.to_string())
            .collect::<Vec<_>>()
            .join(" | ");
        if !hints.is_empty() {
            msg.push_str(&format!("\nİpucu: {hints}"));
        }
    }

    msg
}

impl World for TayanWorld {
    fn library(&self) -> &LazyHash<Library> {
        self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        self.book
    }

    fn main(&self) -> FileId {
        self.source.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.source.id() {
            return Ok(self.source.clone());
        }
        let bytes = self.file(id)?;
        let text = std::str::from_utf8(&bytes)
            .map_err(|_| FileError::InvalidUtf8)?
            .to_owned();
        Ok(Source::new(id, text))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if let Some(cached) = self.file_cache.lock().unwrap().get(&id) {
            return Ok(cached.clone());
        }

        let bytes = self.resolve_file(id).map_err(|e| {
            if matches!(id.root(), VirtualRoot::Package(_)) {
                FileError::Package(PackageError::Other(Some(
                    format!("{e:#}").into()
                )))
            } else {
                FileError::NotFound(PathBuf::from(id.vpath().get_without_slash()))
            }
        })?;

        self.file_cache.lock().unwrap().insert(id, bytes.clone());
        Ok(bytes)
    }

    fn font(&self, index: usize) -> Option<Font> {
        // Baytlar burada, yani yalnızca bu yüz gerçekten dizilecekse yüklenir.
        self.fonts.get(index)?.get()
    }

    fn today(&self, offset: Option<Duration>) -> Option<Datetime> {
        let local = chrono::Local::now();
        // 0.15'te offset saat sayısı değil, bir Duration.
        // decompose() [hafta, gün, saat, dakika, saniye] veriyor.
        let offset_secs = offset
            .map(|d| {
                let [w, dd, h, m, sec] = d.decompose();
                ((w * 7 + dd) * 24 + h) * 3600 + m * 60 + sec
            })
            .unwrap_or(0);
        let dt = local + chrono::Duration::seconds(offset_secs);
        Datetime::from_ymd(
            dt.year(),
            dt.month() as u8,
            dt.day() as u8,
        )
    }
}

impl TayanWorld {
    fn resolve_file(&self, id: FileId) -> anyhow::Result<Bytes> {
        if let VirtualRoot::Package(spec) = id.root() {
            let pkg_dir = self.package_root
                .join(spec.namespace.as_str())
                .join(spec.name.as_str())
                .join(spec.version.to_string());

            if !pkg_dir.exists() {
                download_package(spec, &pkg_dir)?;
            }

            let file_path = pkg_dir.join(id.vpath().get_without_slash());
            let data = std::fs::read(&file_path)
                .with_context(|| format!("Dosya okunamadı: {}", file_path.display()))?;
            return Ok(Bytes::new(data));
        }

        let rootless = id.vpath().get_without_slash();

        // Göreli yol: uygulama veri klasöründen çözülür.
        //
        // Görseller kaynakta "images/xxx.png" olarak durur, mutlak yol olarak
        // değil. Mutlak yol kullanıcı adını gömer ve veri başka bir makineye
        // veya başka bir kullanıcıya taşındığında kırılır; sınav kâğıdı
        // görselsiz basılır ve bunu fark etmek zordur.
        let app_relative = app_data_root().join(rootless);
        if app_relative.exists() {
            let data = std::fs::read(&app_relative)
                .with_context(|| format!("Dosya okunamadı: {}", app_relative.display()))?;
            return Ok(Bytes::new(data));
        }

        // Mutlak yol (eski kayıtlar böyle). VirtualPath baştaki '/' işaretini
        // attığı için geri konuyor.
        let abs = PathBuf::from("/").join(rootless);
        if abs.exists() {
            let data = std::fs::read(&abs)
                .with_context(|| format!("Dosya okunamadı: {}", abs.display()))?;
            return Ok(Bytes::new(data));
        }

        bail!("Çözülemeyen dosya: {:?}", id.vpath().get_without_slash())
    }
}

// ── Font loading ──────────────────────────────────────────────────────────────

/// Bir font yüzü: nerede olduğu bilinir, baytları İSTENENE KADAR açılmaz.
///
/// Ayrım kritik. Typst'in `World` arayüzü zaten iki aşamalıdır:
///   - `book()` yalnızca KÜNYE ister (aile adı, ağırlık, genişlik, italik).
///   - `font(index)` yalnızca belgenin GERÇEKTEN kullandığı yüz için çağrılır.
///
/// Bir sınav kâğıdı tipik olarak bir ya da iki aile kullanır. Bütün sistem
/// fontlarını belleğe almak, kullanılmayacak gigabaytları taşımaktır: ölçülen
/// bir kurulumda ~/Library/Fonts 2.2 GB, /System/Library/Fonts 566 MB.
struct FontSlot {
    source: FontSource,
    font:   OnceLock<Option<Font>>,
}

enum FontSource {
    /// typst-assets içinden gelir; veri zaten 'static, disk erişimi yok.
    Embedded(&'static [u8], u32),
    /// Diskteki dosya; yalnızca ilk istendiğinde açılır.
    File(PathBuf, u32),
}

impl FontSlot {
    fn get(&self) -> Option<Font> {
        self.font
            .get_or_init(|| match &self.source {
                FontSource::Embedded(data, index) => {
                    Font::new(Bytes::new(*data), *index)
                }
                FontSource::File(path, index) => {
                    let data = map_font_file(path)?;
                    Font::new(data, *index)
                }
            })
            .clone()
    }
}

/// Font dosyasını belleğe eşler.
///
/// GÜVENLİK: `Mmap::map` unsafe'tir çünkü eşlenen dosya başka bir süreç
/// tarafından değiştirilirse davranış tanımsızdır. Sistem fontları çalışma
/// sırasında değişmez; typst CLI de aynı ödünü verir. Kazanç, künye taraması
/// sırasında dosyaların tamamının belleğe kopyalanmamasıdır — yalnızca
/// okunan sayfalar sayfalanır.
fn map_font_file(path: &std::path::Path) -> Option<Bytes> {
    let file = std::fs::File::open(path).ok()?;
    let mmap = unsafe { memmap2::Mmap::map(&file).ok()? };
    Some(Bytes::new(mmap))
}

/// Künye kaydı — süreç başına BİR kez kurulur.
///
/// Kurulum sırasında da baytlar kopyalanmaz: her dosya bellek-eşlenir, künyesi
/// (`FontInfo`) çıkarılır ve eşleme bırakılır. Baytlar ancak `FontSlot::get`
/// çağrıldığında, yani o yüz gerçekten dizilecekse tutulur.
static FONT_REGISTRY: OnceLock<(LazyHash<FontBook>, Vec<FontSlot>)> = OnceLock::new();

/// Typst standart kütüphanesi de derleme başına yeniden kurulmaz.
static STANDARD_LIBRARY: OnceLock<LazyHash<Library>> = OnceLock::new();

fn font_registry() -> (&'static LazyHash<FontBook>, &'static [FontSlot]) {
    let (book, slots) = FONT_REGISTRY.get_or_init(build_font_registry);
    (book, slots.as_slice())
}

fn build_font_registry() -> (LazyHash<FontBook>, Vec<FontSlot>) {
    let mut book  = FontBook::new();
    let mut slots = Vec::new();

    // Gömülü fontlar önbelleğe girmez: disk erişimi yok, ayrıştırmaları ucuz,
    // ve typst-assets sürümü değiştiğinde önbellek geçersizleştirme derdi
    // doğurmazlar.
    index_embedded_fonts(&mut book, &mut slots);

    let files       = collect_font_files();
    let fingerprint = fingerprint_of(&files);

    match load_font_index(fingerprint) {
        Some(faces) => {
            for face in faces {
                book.push(face.info);
                slots.push(FontSlot {
                    source: FontSource::File(face.path, face.index),
                    font:   OnceLock::new(),
                });
            }
        }
        None => {
            let faces = parse_font_faces(&files);
            for face in &faces {
                book.push(face.info.clone());
                slots.push(FontSlot {
                    source: FontSource::File(face.path.clone(), face.index),
                    font:   OnceLock::new(),
                });
            }
            store_font_index(fingerprint, &faces);
        }
    }

    (LazyHash::new(book), slots)
}

// ── Künye önbelleği ───────────────────────────────────────────────────────────

/// Önbellek biçimi sürümü. FontInfo'nun kodlaması typst sürümüyle değişebilir;
/// bu sayı artırılınca eski önbellek sessizce atılır.
///
/// 2: typst 0.14.2 -> 0.15.1 yükseltmesi ve postcard -> CBOR geçişi.
///
/// Neden CBOR: typst 0.15'te FontInfo şu alanı taşıyor —
///     #[serde(default, skip_serializing_if = "Vec::is_empty")]
///     pub axes: Vec<FontAxis>,
/// skip_serializing_if KENDİNİ TANIMLAYAN biçimler için bir özelliktir. Postcard
/// alan adı yazmaz, sırayla bayt yazar; boş axes atlanınca akış kayar ve okuma
/// "Serde Deserialization Error" ile düşer. Postcard bu yapıyı prensip olarak
/// taşıyamaz. CBOR alan adlarını yazdığı için atlanan alanı sorunsuz karşılar.
const FONT_INDEX_VERSION: u32 = 2;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct CachedFace {
    path:  PathBuf,
    index: u32,
    info:  FontInfo,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct FontIndex {
    version:     u32,
    fingerprint: u64,
    faces:       Vec<CachedFace>,
}

fn font_index_path() -> Option<PathBuf> {
    dirs_next::data_local_dir().map(|d| d.join("tayan").join("font-index.cbor"))
}

/// Font dosyalarının listesi: yol, değişim zamanı, boyut.
///
/// Yalnızca dizin gezme ve `metadata` — dosya İÇERİĞİ okunmaz. Bu adım
/// milisaniyeler sürer; pahalı olan, sonraki adımdaki künye ayrıştırmasıdır.
fn collect_font_files() -> Vec<(PathBuf, u64, u64)> {
    let mut dirs = collect_system_font_paths();
    if let Some(data_dir) = dirs_next::data_local_dir() {
        dirs.push(data_dir.join("tayan").join("fonts"));
    }

    let mut files = Vec::new();
    for dir in &dirs {
        if dir.exists() {
            collect_font_files_in_dir(dir, &mut files);
        }
    }

    // Sıralama şart: parmak izi dizin okuma sırasına bağlı olmamalı, yoksa
    // önbellek hiçbir şey değişmese bile rastgele geçersizleşir.
    files.sort_by(|a, b| a.0.cmp(&b.0));
    files
}

fn collect_font_files_in_dir(dir: &std::path::Path, out: &mut Vec<(PathBuf, u64, u64)>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            collect_font_files_in_dir(&path, out);
            continue;
        }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !matches!(ext.to_ascii_lowercase().as_str(), "ttf" | "otf" | "ttc" | "otc") {
            continue;
        }

        let Ok(meta) = entry.metadata() else { continue };
        let mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);

        out.push((path, mtime, meta.len()));
    }
}

fn fingerprint_of(files: &[(PathBuf, u64, u64)]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    FONT_INDEX_VERSION.hash(&mut hasher);
    files.len().hash(&mut hasher);
    for (path, mtime, len) in files {
        path.hash(&mut hasher);
        mtime.hash(&mut hasher);
        len.hash(&mut hasher);
    }
    hasher.finish()
}

/// Önbelleği okur. Her hata sessizce None döner ve yeniden tarama yapılır —
/// bozuk bir önbellek yüzünden uygulama açılmamazlık edemez.
fn load_font_index(fingerprint: u64) -> Option<Vec<CachedFace>> {
    let path  = font_index_path()?;
    let bytes = std::fs::read(&path).ok()?;
    let index: FontIndex = ciborium::from_reader(bytes.as_slice()).ok()?;

    if index.version != FONT_INDEX_VERSION || index.fingerprint != fingerprint {
        return None;
    }

    Some(index.faces)
}

/// Önbelleği yazar. Başarısızlık yutulmaz ama ölümcül de değildir: yazılamazsa
/// uygulama çalışır, yalnızca sonraki açılışta yine tarama yapar.
fn store_font_index(fingerprint: u64, faces: &[CachedFace]) {
    let Some(path) = font_index_path() else { return };

    if let Some(parent) = path.parent()
        && std::fs::create_dir_all(parent).is_err()
    {
        return;
    }

    let index = FontIndex {
        version: FONT_INDEX_VERSION,
        fingerprint,
        faces: faces.to_vec(),
    };

    let mut bytes = Vec::new();
    if ciborium::into_writer(&index, &mut bytes).is_err() {
        return;
    }

    // Önce geçici dosya, sonra rename: iki uygulama örneği aynı anda yazarsa
    // yarım kalmış bir dosya okunmasın.
    let tmp = path.with_extension("cbor.tmp");
    if std::fs::write(&tmp, &bytes).is_ok() {
        let _ = std::fs::rename(&tmp, &path);
    }
}

/// Pahalı adım: her dosyayı bellek-eşleyip künyelerini ayrıştırır.
/// Ölçüm: 510 aile için 2,7-4,4 saniye. Önbellek tam olarak bunu atlatır.
fn parse_font_faces(files: &[(PathBuf, u64, u64)]) -> Vec<CachedFace> {
    let mut faces = Vec::new();

    for (path, _, _) in files {
        let Some(mmap) = mmap_for_indexing(path) else { continue };
        for (index, info) in FontInfo::iter(&mmap).enumerate() {
            faces.push(CachedFace {
                path:  path.clone(),
                index: index as u32,
                info,
            });
        }
    }

    faces
}

/// Font künye kaydını önceden kurar.
///
/// Neden gerekli: kayıt ilk derlemede tembel kurulur ve ölçülen makinede bu
/// 510 aile için 2,7–4,4 saniye sürüyor (sıcak sayfa önbelleğinde 2,73 s;
/// maliyet disk okuma değil, FontInfo ayrıştırma). Tembel bırakılırsa bu süre
/// öğretmenin İLK tuş vuruşuna biner ve canlı önizleme donmuş gibi görünür.
///
/// Açılışta ayrı bir iş parçacığından çağrılır: uygulama penceresi açılırken
/// tarama arka planda biter, editöre gelindiğinde kayıt hazırdır.
pub fn warm_font_registry() {
    let _ = font_registry();
}

/// Kullanılabilir font aileleri, alfabetik.
///
/// Künye kaydından okunur; hiçbir font baytı yüklenmez. Öğretmene font seçtiren
/// bir arayüz bunu kullanır — ve tarama sessizce boş dönerse burada görülür.
pub fn available_font_families() -> Vec<String> {
    let (book, _) = font_registry();
    let mut families: Vec<String> =
        book.families().map(|(name, _)| name.to_string()).collect();
    families.sort_by_key(|f| f.to_lowercase());
    families
}

/// Standart kütüphaneye dışarıdan erişim (sembol dökümü için).
pub fn standard_library_ref() -> &'static Library {
    standard_library()
}

fn standard_library() -> &'static LazyHash<Library> {
    STANDARD_LIBRARY.get_or_init(|| LazyHash::new(Library::default()))
}

/// Gömülü fontlar (Libertinus Serif, New Computer Modern Math, DejaVu Sans Mono).
/// Bunlar her kurulumda vardır; çıktının makineden makineye aynı çıkması buna
/// dayanır.
fn index_embedded_fonts(book: &mut FontBook, slots: &mut Vec<FontSlot>) {
    for data in typst_assets::fonts() {
        for (index, info) in FontInfo::iter(data).enumerate() {
            book.push(info);
            slots.push(FontSlot {
                source: FontSource::Embedded(data, index as u32),
                font:   OnceLock::new(),
            });
        }
    }
}

fn mmap_for_indexing(path: &std::path::Path) -> Option<memmap2::Mmap> {
    let file = std::fs::File::open(path).ok()?;
    // GÜVENLİK: yukarıdaki map_font_file ile aynı ödün.
    unsafe { memmap2::Mmap::map(&file).ok() }
}

/// Uygulama verisinin kök klasörü. Veritabanı, font indeksi ve görseller
/// hepsi burada durur — tek klasörü kopyalamak tam yedek demek.
pub fn app_data_root() -> PathBuf {
    dirs_next::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("tayan")
}

fn collect_system_font_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    #[cfg(target_os = "macos")]
    {
        paths.push(PathBuf::from("/Library/Fonts"));
        paths.push(PathBuf::from("/System/Library/Fonts"));
        if let Some(home) = dirs_next::home_dir() {
            paths.push(home.join("Library/Fonts"));
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(windir) = std::env::var("WINDIR") {
            paths.push(PathBuf::from(windir).join("Fonts"));
        }
    }

    #[cfg(target_os = "linux")]
    {
        paths.push(PathBuf::from("/usr/share/fonts"));
        paths.push(PathBuf::from("/usr/local/share/fonts"));
        if let Some(home) = dirs_next::home_dir() {
            paths.push(home.join(".fonts"));
            paths.push(home.join(".local/share/fonts"));
        }
    }

    paths
}

// ── Package cache ─────────────────────────────────────────────────────────────

fn typst_package_cache_dir() -> anyhow::Result<PathBuf> {
    if let Ok(p) = std::env::var("TYPST_PACKAGE_CACHE_PATH") {
        return Ok(PathBuf::from(p));
    }
    let cache = dirs_next::cache_dir()
        .context("Önbellek dizini bulunamadı")?;
    Ok(cache.join("typst").join("packages"))
}

fn download_package(
    spec: &typst::syntax::package::PackageSpec,
    dest: &std::path::Path,
) -> anyhow::Result<()> {
    let url = format!(
        "https://packages.typst.org/{}/{}-{}.tar.gz",
        spec.namespace,
        spec.name,
        spec.version,
    );

    let response = ureq::get(&url)
        .call()
        .with_context(|| format!("Paket indirilemedi: {url}"))?;

    let gz_decoder = flate2::read::GzDecoder::new(response.into_reader());
    let mut archive = tar::Archive::new(gz_decoder);

    std::fs::create_dir_all(dest)
        .with_context(|| format!("Dizin oluşturulamadı: {}", dest.display()))?;

    archive.unpack(dest)
        .with_context(|| format!("Paket açılamadı: {}", spec.name))?;

    Ok(())
}

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

use anyhow::{bail, Context};
use chrono::Datelike;
use typst::{
    LibraryExt,
    diag::{FileError, FileResult, PackageError, SourceDiagnostic, Warned},
    foundations::{Bytes, Datetime},
    layout::PagedDocument,
    syntax::{FileId, Source, VirtualPath},
    text::{Font, FontBook, FontInfo},
    utils::LazyHash,
    Library, World,
};

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

        let source = Source::new(
            FileId::new(None, VirtualPath::new("/main.typ")),
            source_text,
        );

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

        Ok(doc.pages.iter().map(typst_svg::svg).collect())
    }
}

fn format_diagnostic(world: &TayanWorld, diag: &SourceDiagnostic) -> String {
    let mut msg = diag.message.to_string();

    let loc = diag.span.id()
        .and_then(|id| world.source(id).ok())
        .and_then(|src| {
            let start = src
                .range(diag.span)
                .or_else(|| diag.span.range())
                .map(|r| r.start)?;
            let (line, col) = src.lines().byte_to_line_column(start)?;
            Some((line + 1, col + 1))
        });

    if let Some((line, col)) = loc {
        msg.push_str(&format!(" (satır {line}, sütun {col})"));
    }

    if !diag.hints.is_empty() {
        let hints = diag.hints
            .iter()
            .map(|h| h.to_string())
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
            if id.package().is_some() {
                FileError::Package(PackageError::Other(Some(
                    format!("{e:#}").into()
                )))
            } else {
                FileError::NotFound(id.vpath().as_rootless_path().to_owned())
            }
        })?;

        self.file_cache.lock().unwrap().insert(id, bytes.clone());
        Ok(bytes)
    }

    fn font(&self, index: usize) -> Option<Font> {
        // Baytlar burada, yani yalnızca bu yüz gerçekten dizilecekse yüklenir.
        self.fonts.get(index)?.get()
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let local = chrono::Local::now();
        let offset_secs = offset.unwrap_or(0) * 3600;
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
        if let Some(spec) = id.package() {
            let pkg_dir = self.package_root
                .join(spec.namespace.as_str())
                .join(spec.name.as_str())
                .join(spec.version.to_string());

            if !pkg_dir.exists() {
                download_package(spec, &pkg_dir)?;
            }

            let file_path = pkg_dir.join(id.vpath().as_rootless_path());
            let data = std::fs::read(&file_path)
                .with_context(|| format!("Dosya okunamadı: {}", file_path.display()))?;
            return Ok(Bytes::new(data));
        }

        // Try as an absolute filesystem path (e.g. image files)
        // VirtualPath strips the leading '/', so we restore it.
        let abs = PathBuf::from("/").join(id.vpath().as_rootless_path());
        if abs.exists() {
            let data = std::fs::read(&abs)
                .with_context(|| format!("Dosya okunamadı: {}", abs.display()))?;
            return Ok(Bytes::new(data));
        }

        bail!("Çözülemeyen dosya: {:?}", id.vpath().as_rootless_path())
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
    let (book, slots) = FONT_REGISTRY.get_or_init(|| {
        let mut book  = FontBook::new();
        let mut slots = Vec::new();

        index_embedded_fonts(&mut book, &mut slots);

        let mut font_paths = collect_system_font_paths();
        if let Some(data_dir) = dirs_next::data_local_dir() {
            font_paths.push(data_dir.join("tayan").join("fonts"));
        }
        for path in &font_paths {
            if path.exists() {
                index_fonts_in_dir(path, &mut book, &mut slots);
            }
        }

        (LazyHash::new(book), slots)
    });
    (book, slots.as_slice())
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

fn index_fonts_in_dir(dir: &std::path::Path, book: &mut FontBook, slots: &mut Vec<FontSlot>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            index_fonts_in_dir(&path, book, slots);
            continue;
        }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !matches!(ext.to_ascii_lowercase().as_str(), "ttf" | "otf" | "ttc" | "otc") {
            continue;
        }

        // Eşleme yalnızca künye çıkarmak için açılır ve bu kapsamın sonunda
        // bırakılır. Dosyanın tamamı belleğe alınmaz.
        let Some(mmap) = mmap_for_indexing(&path) else { continue };
        for (index, info) in FontInfo::iter(&mmap).enumerate() {
            book.push(info);
            slots.push(FontSlot {
                source: FontSource::File(path.clone(), index as u32),
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

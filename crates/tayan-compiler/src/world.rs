use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Mutex,
};

use anyhow::{bail, Context};
use chrono::Datelike;
use typst::{
    LibraryExt,
    diag::{FileError, FileResult, SourceDiagnostic, Warned},
    foundations::{Bytes, Datetime},
    syntax::{FileId, Source, VirtualPath},
    text::{Font, FontBook},
    utils::LazyHash,
    Library, World,
};

// ── World ─────────────────────────────────────────────────────────────────────

pub struct TayanWorld {
    source:       Source,
    library:      LazyHash<Library>,
    book:         LazyHash<FontBook>,
    fonts:        Vec<Font>,
    package_root: PathBuf,
    file_cache:   Mutex<HashMap<FileId, Bytes>>,
}

impl TayanWorld {
    pub fn new(source_text: String) -> anyhow::Result<Self> {
        let mut font_paths = collect_system_font_paths();

        if let Some(data_dir) = dirs_next::data_local_dir() {
            font_paths.push(data_dir.join("tayan").join("fonts"));
        }

        let (book, fonts) = load_fonts(&font_paths);
        let package_root = typst_package_cache_dir()?;

        let source = Source::new(
            FileId::new(None, VirtualPath::new("/main.typ")),
            source_text,
        );

        Ok(Self {
            source,
            library: Library::default().into(),
            book:    book.into(),
            fonts,
            package_root,
            file_cache: Mutex::new(HashMap::new()),
        })
    }

    pub fn compile_pdf(source_text: String) -> anyhow::Result<Vec<u8>> {
        let world = Self::new(source_text)?;
        let Warned { output, warnings: _ } = typst::compile(&world);

        let doc = output.map_err(|errors| {
            let msgs: Vec<String> = errors
                .iter()
                .map(|e: &SourceDiagnostic| e.message.to_string())
                .collect();
            anyhow::anyhow!("Typst derleme hatası:\n{}", msgs.join("\n"))
        })?;

        typst_pdf::pdf(&doc, &typst_pdf::PdfOptions::default())
            .map_err(|errors| {
                let msgs: Vec<String> = errors
                    .iter()
                    .map(|e: &SourceDiagnostic| e.message.to_string())
                    .collect();
                anyhow::anyhow!("PDF oluşturma hatası:\n{}", msgs.join("\n"))
            })
    }
}

impl World for TayanWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
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

        let bytes = self.resolve_file(id).map_err(|_| FileError::NotFound(
            id.vpath().as_rootless_path().to_owned(),
        ))?;

        self.file_cache.lock().unwrap().insert(id, bytes.clone());
        Ok(bytes)
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
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

fn load_fonts(search_paths: &[PathBuf]) -> (FontBook, Vec<Font>) {
    let mut book  = FontBook::new();
    let mut fonts = Vec::new();

    for path in search_paths {
        if !path.exists() { continue; }
        load_fonts_from_dir(path, &mut book, &mut fonts);
    }

    (book, fonts)
}

fn load_fonts_from_dir(dir: &std::path::Path, book: &mut FontBook, fonts: &mut Vec<Font>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            load_fonts_from_dir(&path, book, fonts);
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !matches!(ext.to_ascii_lowercase().as_str(), "ttf" | "otf" | "ttc") {
            continue;
        }
        let Ok(data) = std::fs::read(&path) else { continue };
        let bytes = Bytes::new(data);
        for i in 0.. {
            match Font::new(bytes.clone(), i) {
                Some(font) => {
                    book.push(font.info().clone());
                    fonts.push(font);
                }
                None => break,
            }
        }
    }
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

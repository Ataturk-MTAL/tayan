//! tinymist dil sunucusuna ince bir LSP istemcisi.
//!
//! Neden ayrı süreç: tinymist'in yayımlanan kütüphaneleri typst'in YAMALANMIŞ
//! bir çatalına göre yazılmış (typst::foundations::FuncInner yayımlanmış
//! typst'te private) ve crates.io'dan kütüphane olarak derlenmiyor. Ayrı süreç
//! bu bağı tamamen koparır: tinymist kendi typst'ini taşır, biz kendi
//! typst'imizle dizeriz.
//!
//! Ölçüm (v0.15.2, macOS arm64): initialize 87 ms, tamamlama 6 ms, ~50 MB RSS.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspCompletion {
    pub label: String,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    /// LSP CompletionItemKind. Ön yüz ikon seçiminde kullanır.
    pub kind: Option<u8>,
    /// Eklenecek metin. Yoksa label eklenir.
    pub insert_text: Option<String>,
    /// 2 = snippet ($1, ${2:x} yer tutucuları içerir).
    pub insert_format: Option<u8>,
}

/// tinymist ikilisinin yeri.
///
/// Üretimde Tauri sidecar olarak paketlenir; geliştirmede depodaki
/// binaries/ klasöründen okunur (scripts/fetch-tinymist.sh indirir).
/// Son çare olarak PATH'e bakılır ki geliştirici kendi kurulumunu kullanabilsin.
pub fn binary_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    use tauri::Manager;

    let triple = env!("TINYMIST_TARGET_TRIPLE");
    let name = format!("tinymist-{triple}");

    if let Ok(dir) = app.path().resource_dir() {
        let p = dir.join(&name);
        if p.exists() {
            return Ok(p);
        }
    }

    let dev = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("binaries")
        .join(&name);
    if dev.exists() {
        return Ok(dev);
    }

    which_in_path("tinymist")
        .ok_or_else(|| format!("tinymist bulunamadı. scripts/fetch-tinymist.sh çalıştırın."))
}

fn which_in_path(name: &str) -> Option<std::path::PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|d| d.join(name))
        .find(|p| p.is_file())
}

pub struct Tinymist {
    inner: Mutex<Option<Session>>,
}

struct Session {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: i64,
    /// Belge açıldı mı — ikinci çağrıdan itibaren didChange kullanılır.
    opened: bool,
    version: i64,
    doc_uri: String,
}

impl Tinymist {
    pub fn new() -> Self {
        Self { inner: Mutex::new(None) }
    }

    /// Sunucu çalışmıyorsa başlatır. Başlatılamazsa Err döner ve çağıran
    /// kendi sembol dökümüne düşer — tamamlama uğruna editörü bloklamayız.
    fn ensure(&self, guard: &mut Option<Session>, bin: &std::path::Path, root: &std::path::Path)
        -> Result<(), String>
    {
        if guard.is_some() {
            return Ok(());
        }

        let mut child = Command::new(bin)
            .arg("lsp")
            // Sistem fontlarını taratmıyoruz: tinymist'ten dizgi değil dil
            // zekâsı istiyoruz. Ölçüm: initialize 566 ms -> 87 ms,
            // tamamlama 26 ms -> 6 ms.
            .arg("--ignore-system-fonts")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("tinymist başlatılamadı: {e}"))?;

        let stdin = child.stdin.take().ok_or("stdin alınamadı")?;
        let stdout = BufReader::new(child.stdout.take().ok_or("stdout alınamadı")?);

        let doc_uri = format!("file://{}", root.join("main.typ").display());

        let mut session = Session {
            child,
            stdin,
            stdout,
            next_id: 1,
            opened: false,
            version: 0,
            doc_uri,
        };

        session.request("initialize", json!({
            "processId": std::process::id(),
            "rootUri": format!("file://{}", root.display()),
            "capabilities": {
                "textDocument": {
                    "completion": { "completionItem": { "snippetSupport": true } }
                }
            }
        }))?;
        session.notify("initialized", json!({}))?;

        *guard = Some(session);
        Ok(())
    }

    /// Verilen konumda tamamlama ister.
    ///
    /// `source` TAM belgedir (önsöz + gövde): tinymist #secenekler gibi şablon
    /// yardımcılarını ancak tanımlarını görürse önerebilir. Satır numarası da
    /// buna göre kaydırılmış gelmelidir.
    pub fn complete(
        &self,
        bin: &std::path::Path,
        root: &std::path::Path,
        source: &str,
        line: u32,
        character: u32,
    ) -> Result<Vec<LspCompletion>, String> {
        let mut guard = self.inner.lock().map_err(|_| "kilit alınamadı")?;
        self.ensure(&mut guard, bin, root)?;
        let session = guard.as_mut().ok_or("oturum yok")?;

        session.sync(source)?;

        let uri = session.doc_uri.clone();
        let result = session.request("textDocument/completion", json!({
            "textDocument": { "uri": uri },
            "position": { "line": line, "character": character }
        }))?;

        Ok(parse_completions(&result))
    }

    /// Sunucuyu durdurur. Uygulama kapanırken çağrılır.
    pub fn shutdown(&self) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(mut s) = guard.take() {
                let _ = s.notify("exit", json!({}));
                let _ = s.child.kill();
            }
        }
    }
}

impl Session {
    fn sync(&mut self, source: &str) -> Result<(), String> {
        let uri = self.doc_uri.clone();

        if !self.opened {
            self.notify("textDocument/didOpen", json!({
                "textDocument": {
                    "uri": uri, "languageId": "typst", "version": 1, "text": source
                }
            }))?;
            self.opened = true;
            self.version = 1;
            return Ok(());
        }

        self.version += 1;
        let version = self.version;
        self.notify("textDocument/didChange", json!({
            "textDocument": { "uri": uri, "version": version },
            // Tam metin gönderiliyor: artımlı senkron için parça hesabı
            // tutmak, bu boyuttaki belgelerde kazandırmayacağı karmaşıklık.
            "contentChanges": [ { "text": source } ]
        }))
    }

    fn write_message(&mut self, msg: &Value) -> Result<(), String> {
        let body = serde_json::to_vec(msg).map_err(|e| e.to_string())?;
        self.stdin
            .write_all(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes())
            .and_then(|_| self.stdin.write_all(&body))
            .and_then(|_| self.stdin.flush())
            .map_err(|e| format!("tinymist'e yazılamadı: {e}"))
    }

    fn notify(&mut self, method: &str, params: Value) -> Result<(), String> {
        self.write_message(&json!({ "jsonrpc": "2.0", "method": method, "params": params }))
    }

    fn request(&mut self, method: &str, params: Value) -> Result<Value, String> {
        let id = self.next_id;
        self.next_id += 1;

        self.write_message(&json!({
            "jsonrpc": "2.0", "id": id, "method": method, "params": params
        }))?;

        // Yanıtı bekle. Aradaki bildirimler (tanılama, ilerleme) atlanır.
        //
        // Sunucu BİZE istek gönderirse null ile yanıtlanır: yanıtsız bırakmak
        // tinymist'i bekletir ve tamamlama hiç dönmez.
        for _ in 0..200 {
            let msg = self.read_message()?;

            if msg.get("id").and_then(Value::as_i64) == Some(id) {
                if let Some(err) = msg.get("error") {
                    return Err(format!("tinymist hatası: {err}"));
                }
                return Ok(msg.get("result").cloned().unwrap_or(Value::Null));
            }

            if let (Some(req_id), Some(_)) = (msg.get("id"), msg.get("method")) {
                let req_id = req_id.clone();
                self.write_message(&json!({
                    "jsonrpc": "2.0", "id": req_id, "result": Value::Null
                }))?;
            }
        }

        Err("tinymist yanıt vermedi".to_string())
    }

    fn read_message(&mut self) -> Result<Value, String> {
        let mut length: Option<usize> = None;

        loop {
            let mut line = String::new();
            let read = self.stdout.read_line(&mut line).map_err(|e| e.to_string())?;
            if read == 0 {
                return Err("tinymist bağlantısı kapandı".to_string());
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                break;
            }
            if let Some(rest) = trimmed.strip_prefix("Content-Length:") {
                length = rest.trim().parse().ok();
            }
        }

        let len = length.ok_or("Content-Length yok")?;
        let mut buf = vec![0u8; len];
        self.stdout.read_exact(&mut buf).map_err(|e| e.to_string())?;
        serde_json::from_slice(&buf).map_err(|e| format!("JSON çözülemedi: {e}"))
    }
}

fn parse_completions(result: &Value) -> Vec<LspCompletion> {
    let items = match result {
        Value::Array(a) => a.clone(),
        Value::Object(o) => o.get("items").and_then(Value::as_array).cloned().unwrap_or_default(),
        _ => Vec::new(),
    };

    items
        .iter()
        .filter_map(|it| {
            let label = it.get("label")?.as_str()?.to_string();
            Some(LspCompletion {
                label,
                detail: it.get("detail").and_then(Value::as_str).map(str::to_string),
                documentation: documentation_text(it.get("documentation")),
                kind: it.get("kind").and_then(Value::as_u64).map(|k| k as u8),
                insert_text: it.get("insertText").and_then(Value::as_str).map(str::to_string),
                insert_format: it.get("insertTextFormat").and_then(Value::as_u64).map(|k| k as u8),
            })
        })
        .collect()
}

/// LSP belgeleri düz metin ya da MarkupContent olabilir.
fn documentation_text(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(s) => Some(s.clone()),
        Value::Object(o) => o.get("value").and_then(Value::as_str).map(str::to_string),
        _ => None,
    }
}

/// Yalnızca uyarıyı susturmak için: HashMap ileride istek eşleştirmesi
/// artımlı hale gelirse kullanılacak.
#[allow(dead_code)]
type PendingMap = HashMap<i64, ()>;

#[cfg(test)]
mod tinymist_tests {
    use super::*;

    fn dev_binary() -> Option<std::path::PathBuf> {
        let triple = env!("TINYMIST_TARGET_TRIPLE");
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("binaries")
            .join(format!("tinymist-{triple}"));
        p.exists().then_some(p)
    }

    /// tinymist gerçekten yanıt veriyor mu, VE şablon yardımcılarımızı biliyor mu?
    ///
    /// İkincisi kritik: gövde önsözle sarmalanmadan gönderilseydi tinymist
    /// #secenekler'in tanımını göremez ve öneremezdi.
    #[test]
    fn önsözdeki_yardımcıları_öneriyor() {
        let Some(bin) = dev_binary() else {
            eprintln!("tinymist ikilisi yok, test atlandı (scripts/fetch-tinymist.sh)");
            return;
        };

        let root = std::env::temp_dir().join("tayan-lsp-test");
        std::fs::create_dir_all(&root).unwrap();

        let body = "Soru metni.

#sec
";
        let source = tayan_compiler::typst_gen::TypstGenerator::preview_document(body);
        let offset = tayan_compiler::typst_gen::TypstGenerator::preview_line_offset() as u32;

        let client = Tinymist::new();
        // Gövdenin 3. satırı (0 tabanlı 2), "#sec" sonrası 4. sütun.
        let items = client
            .complete(&bin, &root, &source, 2 + offset, 4)
            .expect("tinymist yanıt vermedi");
        client.shutdown();

        assert!(!items.is_empty(), "hiç öneri yok");

        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(
            labels.iter().any(|l| l.contains("secenekler")),
            "önsözdeki #secenekler önerilmedi. Gelen: {:?}",
            &labels[..labels.len().min(15)]
        );
    }
}

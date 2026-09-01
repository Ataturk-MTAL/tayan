/// Context passed during Typst code generation — controls output mode.
#[derive(Debug, Clone)]
pub struct TypstContext {
    /// Generate answer key version (shows correct answers).
    pub answer_key: bool,
    /// Shuffle multiple-choice options.
    pub shuffle: bool,
    /// 1-based question number to prefix in output.
    pub question_number: Option<u32>,
    /// Kitapçık türü ("A", "B", ...). None ise tek kitapçık: kâğıda etiket
    /// basılmaz ve sıra yalnızca sınav kimliğinden türetilir.
    pub booklet: Option<String>,
    /// Karıştırma tohumu. Sınav kimliğinden (ve ileride kitapçık türünden)
    /// türetilir. Aynı tohum her zaman aynı sırayı verir — yeniden basılan
    /// kâğıt öncekiyle birebir aynı çıkar, cevap anahtarı da öyle.
    pub shuffle_seed: u64,
}

impl Default for TypstContext {
    fn default() -> Self {
        Self { answer_key: false, shuffle: false, question_number: None, booklet: None, shuffle_seed: 0 }
    }
}

impl TypstContext {
    pub fn answer_key() -> Self {
        Self { answer_key: true, shuffle: false, question_number: None, booklet: None, shuffle_seed: 0 }
    }

    pub fn with_number(mut self, n: u32) -> Self {
        self.question_number = Some(n);
        self
    }
}

/// Every domain type that can emit Typst source code implements this.
pub trait ToTypst {
    fn to_typst(&self, ctx: &TypstContext) -> String;
}

/// Bir metinden deterministik tohum üretir (FNV-1a).
///
/// Kriptografik değil, olması da gerekmiyor: tek istenen aynı girdinin her
/// zaman aynı sayıya düşmesi. Karıştırma sırasının yeniden basımda değişmemesi
/// buna dayanır.
pub fn seed_from(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

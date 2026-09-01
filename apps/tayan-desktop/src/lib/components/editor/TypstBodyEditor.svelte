<script lang="ts">
  import { api } from '$lib/api';
  import { onDestroy } from 'svelte';

  const PREVIEW_PREFIX = `#set page(paper: "a4", margin: (x: 2cm, y: 2.5cm))
#set text(lang: "tr", size: 11pt, font: ("Linux Libertine", "Times New Roman", "Georgia", "serif"))
#v(0.2cm)

`;
  const PREVIEW_PREFIX_LINES = PREVIEW_PREFIX.split('\n').length - 1;

  let {
    value = '',
    onchange,
    placeholder = '#set text(lang: "tr")\n\nSoru metnini Typst ile yazın...\n',
    rows = 10,
    livePreview = true,
    debounceMs = 500,
  }: {
    value?: string;
    onchange?: (value: string) => void;
    placeholder?: string;
    rows?: number;
    livePreview?: boolean;
    debounceMs?: number;
  } = $props();

  const AUTO_PAIRS: Record<string, string> = { '$': '$', '(': ')', '[': ']', '{': '}' };

  let editorEl: HTMLTextAreaElement | undefined = $state();
  let previewEnabled = $state(true);
  let blobUrl = $state<string | null>(null);
  let compiling = $state(false);
  let error = $state<string | null>(null);
  let errorLine = $state<number | null>(null);
  let lineHighlight = $state<number | null>(null);
  let lineHeightPx = $state(22);
  let scrollTopPx = $state(0);
  let activeSnippetGroup = $state<'sinav' | 'math' | 'greek' | 'layout'>('sinav');

  let highlightTimer: ReturnType<typeof setTimeout> | null = null;
  let lastAutoRevealLine: number | null = null;
  let requestSeq = 0;

  function onInput(e: Event) {
    const next = (e.currentTarget as HTMLTextAreaElement).value;
    onchange?.(next);
  }

  function onScroll(e: Event) {
    scrollTopPx = (e.currentTarget as HTMLTextAreaElement).scrollTop;
  }

  function insertSnippet(snippet: string) {
    if (!editorEl) {
      onchange?.((value ?? '') + snippet);
      return;
    }

    const start = editorEl.selectionStart ?? value.length;
    const end = editorEl.selectionEnd ?? start;
    const next = value.slice(0, start) + snippet + value.slice(end);
    onchange?.(next);

    setTimeout(() => {
      if (!editorEl) return;
      const cursor = start + snippet.length;
      editorEl.focus();
      editorEl.setSelectionRange(cursor, cursor);
    });
  }

  function onKeyDown(e: KeyboardEvent) {
    if (!editorEl) return;

    // Tab → iki boşluk (textarea'dan çıkmak yerine)
    if (e.key === 'Tab') {
      e.preventDefault();
      insertSnippet('  ');
      return;
    }

    // Otomatik kapama: $ ( [ {
    const closing = AUTO_PAIRS[e.key];
    if (!closing) return;

    // Seçili metin varsa çevresine sar
    const start = editorEl.selectionStart ?? 0;
    const end   = editorEl.selectionEnd   ?? start;
    if (start !== end) {
      e.preventDefault();
      const selected = value.slice(start, end);
      const next = value.slice(0, start) + e.key + selected + closing + value.slice(end);
      onchange?.(next);
      setTimeout(() => {
        if (!editorEl) return;
        editorEl.focus();
        editorEl.setSelectionRange(start + 1, end + 1);
      });
      return;
    }

    // Seçim yoksa: sonraki karakter zaten aynı kapama mı? → sadece ilerle (eat)
    const nextChar = value[start];
    if (nextChar === closing && e.key === closing) {
      e.preventDefault();
      setTimeout(() => {
        if (!editorEl) return;
        editorEl.focus();
        editorEl.setSelectionRange(start + 1, start + 1);
      });
      return;
    }

    // Normal auto-pair
    e.preventDefault();
    const next = value.slice(0, start) + e.key + closing + value.slice(start);
    onchange?.(next);
    setTimeout(() => {
      if (!editorEl) return;
      editorEl.focus();
      editorEl.setSelectionRange(start + 1, start + 1);
    });
  }

  function clearPreview() {
    if (blobUrl) URL.revokeObjectURL(blobUrl);
    blobUrl = null;
  }

  function updateLineMetrics() {
    if (!editorEl) return;
    const cs = getComputedStyle(editorEl);
    const lh = Number.parseFloat(cs.lineHeight);
    if (Number.isFinite(lh) && lh > 0) {
      lineHeightPx = lh;
      return;
    }
    const fs = Number.parseFloat(cs.fontSize);
    lineHeightPx = Number.isFinite(fs) && fs > 0 ? fs * 1.45 : 22;
  }

  function flashLine(line: number) {
    lineHighlight = line;
    if (highlightTimer) {
      clearTimeout(highlightTimer);
      highlightTimer = null;
    }
    highlightTimer = setTimeout(() => {
      lineHighlight = null;
      highlightTimer = null;
    }, 1500);
  }

  function lineStartOffset(lines: string[], line: number): number {
    let offset = 0;
    for (let i = 0; i < line - 1; i += 1) {
      offset += lines[i].length + 1;
    }
    return offset;
  }

  function revealLine(line: number, focus: boolean) {
    if (!editorEl) return;
    updateLineMetrics();

    const lines = value.split('\n');
    const target = Math.max(1, Math.min(line, Math.max(lines.length, 1)));
    const start = lineStartOffset(lines, target);
    const end = start + (lines[target - 1]?.length ?? 0);
    const top = Math.max(0, (target - 1) * lineHeightPx);

    editorEl.scrollTop = Math.max(0, top - editorEl.clientHeight * 0.35);
    scrollTopPx = editorEl.scrollTop;

    if (focus) {
      editorEl.focus();
      editorEl.setSelectionRange(start, Math.max(start, end));
    }

    flashLine(target);
  }

  function setPreviewFromBase64(b64: string) {
    clearPreview();
    const bytes = Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
    const blob = new Blob([bytes], { type: 'application/pdf' });
    blobUrl = URL.createObjectURL(blob);
  }

  function buildPreviewSource(body: string): string {
    const safeBody = body.trim() ? body : '[Boş içerik]';
    return `${PREVIEW_PREFIX}${safeBody}`;
  }

  function parseRawLine(message: string): number | null {
    const patterns = [
      /satır\s+(\d+)/i,
      /line\s+(\d+)/i,
      /:(\d+):(\d+)/,
    ];
    for (const re of patterns) {
      const m = message.match(re);
      if (m) return Number(m[1]);
    }
    return null;
  }

  function extractEditorLine(message: string): number | null {
    const raw = parseRawLine(message);
    if (!raw) return null;
    const adjusted = raw - PREVIEW_PREFIX_LINES;
    return adjusted > 0 ? adjusted : 1;
  }

  function jumpToErrorLine() {
    if (!errorLine) return;
    revealLine(errorLine, true);
  }

  function textareaHighlightStyle() {
    if (!lineHighlight) return '';
    const top = (lineHighlight - 1) * lineHeightPx - scrollTopPx;
    if (top < -lineHeightPx) return '';
    const start = Math.max(0, top);
    const end = Math.max(start + lineHeightPx, start + 2);
    return `background-image: linear-gradient(to bottom, transparent ${start}px, rgba(239, 68, 68, 0.14) ${start}px, rgba(239, 68, 68, 0.14) ${end}px, transparent ${end}px); background-repeat: no-repeat; background-attachment: local;`;
  }

  $effect(() => {
    const code = value;
    if (!livePreview || !previewEnabled) return;

    if (!code.trim()) {
      clearPreview();
      error = null;
      errorLine = null;
      lineHighlight = null;
      lastAutoRevealLine = null;
      compiling = false;
      return;
    }

    const seq = ++requestSeq;
    const timer = setTimeout(async () => {
      compiling = true;
      error = null;
      errorLine = null;

      try {
        const source = buildPreviewSource(code);
        const b64 = await api.compiler.previewTypst(source);
        if (seq !== requestSeq) return;
        setPreviewFromBase64(b64);
        lastAutoRevealLine = null;
      } catch (e) {
        if (seq !== requestSeq) return;
        clearPreview();
        error = String(e);
        errorLine = extractEditorLine(error);
        if (errorLine && errorLine !== lastAutoRevealLine) {
          revealLine(errorLine, false);
          lastAutoRevealLine = errorLine;
        }
      } finally {
        if (seq === requestSeq) compiling = false;
      }
    }, debounceMs);

    return () => clearTimeout(timer);
  });

  onDestroy(() => {
    clearPreview();
    if (highlightTimer) clearTimeout(highlightTimer);
  });

  const snippetGroups: Array<{
    id: 'sinav' | 'math' | 'greek' | 'layout';
    label: string;
    snippets: Array<{ label: string; code: string; title: string }>;
  }> = [
    {
      id: 'sinav',
      label: 'Sınav',
      snippets: [
        { label: 'Soru Başlığı', code: '= Soru Başlığı\n\nAçıklama metni buraya.\n', title: 'Başlık ve kısa açıklama' },
        { label: '4 Şık', code: '#grid(columns: (1fr, 1fr), gutter: 0.6em,\n  [A) Seçenek], [B) Seçenek],\n  [C) Seçenek], [D) Seçenek],\n)\n', title: 'Çoktan seçmeli 4 seçenek' },
        { label: '5 Şık', code: '#grid(columns: (1fr, 1fr, 1fr), gutter: 0.6em,\n  [A) Seçenek], [B) Seçenek], [C) Seçenek],\n  [D) Seçenek], [E) Seçenek], [],\n)\n', title: 'Çoktan seçmeli 5 seçenek' },
        { label: 'Doğru/Yanlış', code: '#let cb = box(stroke: 0.5pt, width: 0.85em, height: 0.85em, baseline: 0.15em)[]\n#cb Doğru #h(2em) #cb Yanlış\n', title: 'Doğru/Yanlış satırı (font bağımsız kutucuk)' },
        { label: 'Boşluk', code: '#box(width: 4cm, stroke: (bottom: 0.5pt + black), height: 1.1em)[]\n', title: 'Boşluk doldurma çizgisi' },
        { label: 'Cevap Alanı', code: '#rect(width: 100%, height: 5cm, stroke: 0.5pt)[]\n', title: 'Klasik cevap alanı' },
        { label: 'Numaralı Soru', code: '#set enum(numbering: "1.")\n+ Birinci soru metni\n+ İkinci soru metni\n', title: 'Otomatik numaralı sorular' },
        { label: 'Alt Sorular', code: '#set enum(numbering: "a)")\n+ Alt soru a\n+ Alt soru b\n+ Alt soru c\n', title: 'a) b) c) alt sorular' },
        { label: 'Puan Kutusu', code: '#box(stroke: 0.5pt, inset: (x: 6pt, y: 4pt), baseline: 2pt)[___ / 10 puan]\n', title: 'Puan kutusu' },
        { label: 'Eşleştirme', code: '#grid(columns: (1fr, 0.2fr, 1fr), gutter: 0.5em,\n  [1. Kavram A], [], [a. Tanım A],\n  [2. Kavram B], [], [b. Tanım B],\n  [3. Kavram C], [], [c. Tanım C],\n)\n', title: 'Eşleştirme sorusu' },
        { label: 'Başlık Şablonu', code: '#align(center)[\n  *OKUL ADI*\\\n  Ders Adı Sınavı #h(1fr) Tarih: ............\\\n  Öğrenci: ............................... #h(1fr) Süre: ... dk\n]\n#line(length: 100%)\n', title: 'Sınav başlık şablonu' },
      ],
    },
    {
      id: 'math',
      label: 'Matematik',
      snippets: [
        { label: 'Denklem', code: '$x^2 + y^2 = z^2$\n', title: 'Temel denklem' },
        { label: 'Kesir', code: '$frac(a, b)$\n', title: 'Kesir' },
        { label: 'Karekök', code: '$sqrt(x)$\n', title: 'Karekök' },
        { label: 'n. Kök', code: '$root(n, x)$\n', title: 'n. kök' },
        { label: 'Toplam', code: '$sum_(i=1)^n i$\n', title: 'Sigma toplamı' },
        { label: 'Çarpım', code: '$product_(i=1)^n i$\n', title: 'Pi çarpımı' },
        { label: 'İntegral', code: '$integral_0^1 x^2 dif x$\n', title: 'Belirli integral' },
        { label: 'Limit', code: '$lim_(x -> 0) frac(sin x, x) = 1$\n', title: 'Limit ifadesi' },
        { label: 'Türev', code: '$frac(dif f, dif x)$\n', title: 'Birinci türev Leibniz' },
        { label: 'Kısmi Türev', code: '$frac(partial f, partial x)$\n', title: 'Kısmi türev' },
        { label: 'Mutlak Değer', code: '$abs(x)$\n', title: 'Mutlak değer' },
        { label: 'Üst Çizgi', code: '$overline(A B)$\n', title: 'Üst çizgi (ortalama)' },
        { label: 'Vektör', code: '$arrow(v)$\n', title: 'Vektör oku' },
        { label: 'Matris', code: '$mat(1, 2; 3, 4)$\n', title: '2×2 matris' },
        { label: 'Koşullu', code: '$f(x) = cases(x quad x > 0, -x quad x <= 0)$\n', title: 'Parçalı fonksiyon' },
        { label: 'Büyük O', code: '$cal(O)(n^2)$\n', title: 'Büyük O notasyonu' },
        { label: 'Veri Seti', code: '$macron(x) = frac(1, n) sum_(i=1)^n x_i$\n', title: 'Aritmetik ortalama' },
      ],
    },
    {
      id: 'greek',
      label: 'Yunan',
      snippets: [
        { label: 'α alfa',    code: '$alpha$\n',   title: 'Alfa (α)' },
        { label: 'β beta',    code: '$beta$\n',    title: 'Beta (β)' },
        { label: 'γ gama',    code: '$gamma$\n',   title: 'Gamma (γ)' },
        { label: 'Γ Gama',    code: '$Gamma$\n',   title: 'Büyük Gamma (Γ)' },
        { label: 'δ delta',   code: '$delta$\n',   title: 'Delta (δ)' },
        { label: 'Δ Delta',   code: '$Delta$\n',   title: 'Büyük Delta (Δ)' },
        { label: 'ε epsilon', code: '$epsilon$\n', title: 'Epsilon (ε)' },
        { label: 'ζ zeta',    code: '$zeta$\n',    title: 'Zeta (ζ)' },
        { label: 'η eta',     code: '$eta$\n',     title: 'Eta (η)' },
        { label: 'θ teta',    code: '$theta$\n',   title: 'Theta (θ)' },
        { label: 'Θ Teta',    code: '$Theta$\n',   title: 'Büyük Theta (Θ)' },
        { label: 'λ lambda',  code: '$lambda$\n',  title: 'Lambda (λ)' },
        { label: 'Λ Lambda',  code: '$Lambda$\n',  title: 'Büyük Lambda (Λ)' },
        { label: 'μ mü',      code: '$mu$\n',      title: 'Mü (μ)' },
        { label: 'π pi',      code: '$pi$\n',      title: 'Pi (π)' },
        { label: 'Π Pi',      code: '$Pi$\n',      title: 'Büyük Pi (Π)' },
        { label: 'ρ ro',      code: '$rho$\n',     title: 'Ro (ρ)' },
        { label: 'σ sigma',   code: '$sigma$\n',   title: 'Sigma (σ)' },
        { label: 'Σ Sigma',   code: '$Sigma$\n',   title: 'Büyük Sigma (Σ)' },
        { label: 'τ tau',     code: '$tau$\n',     title: 'Tau (τ)' },
        { label: 'φ fi',      code: '$phi$\n',     title: 'Phi (φ)' },
        { label: 'Φ Fi',      code: '$Phi$\n',     title: 'Büyük Phi (Φ)' },
        { label: 'χ ki',      code: '$chi$\n',     title: 'Chi (χ)' },
        { label: 'ψ psi',     code: '$psi$\n',     title: 'Psi (ψ)' },
        { label: 'Ψ Psi',     code: '$Psi$\n',     title: 'Büyük Psi (Ψ)' },
        { label: 'ω omega',   code: '$omega$\n',   title: 'Omega (ω)' },
        { label: 'Ω Omega',   code: '$Omega$\n',   title: 'Büyük Omega (Ω)' },
        { label: 'nabla',     code: '$nabla$\n',   title: 'Nabla (∇)' },
        { label: 'inf',       code: '$infinity$\n', title: 'Sonsuz (∞)' },
      ],
    },
    {
      id: 'layout',
      label: 'Düzen',
      snippets: [
        { label: 'Liste', code: '- Madde\n- Madde\n', title: 'Madde listesi' },
        { label: 'Numaralı Liste', code: '+ Birinci\n+ İkinci\n+ Üçüncü\n', title: 'Numaralı liste' },
        { label: 'Tablo', code: '#table(columns: 3,\n  [Başlık 1], [Başlık 2], [Başlık 3],\n  [A], [B], [C],\n)\n', title: 'Basit tablo' },
        { label: '2 Sütun', code: '#columns(2)[\n  İçerik buraya gelir. Typst otomatik olarak iki sütuna böler.\n]\n', title: 'İki sütunlu düzen' },
        { label: 'Resim', code: '#figure(image("/path/to/image.png", width: 60%), caption: [Şekil açıklaması])\n', title: 'Şekil ve başlık' },
        { label: 'Not Kutusu', code: '#block(fill: luma(240), inset: 8pt, radius: 4pt)[Not metni]\n', title: 'Vurgulu kutu' },
        { label: 'Uyarı Kutusu', code: '#block(fill: rgb("#fff3cd"), stroke: rgb("#ffc107"), inset: 8pt, radius: 4pt)[⚠ Uyarı metni]\n', title: 'Uyarı vurgu kutusu' },
        { label: 'Ayraç', code: '#line(length: 100%)\n', title: 'Yatay ayraç' },
        { label: 'Ortalı Metin', code: '#align(center)[Ortalanmış metin]\n', title: 'Yatay ortala' },
        { label: 'Sayfa Sonu', code: '#pagebreak()\n', title: 'Yeni sayfa' },
      ],
    },
  ];

  let activeSnippets = $derived.by(
    () => snippetGroups.find((g) => g.id === activeSnippetGroup)?.snippets ?? [],
  );
</script>

<div class="space-y-2">
  <div class="space-y-1 rounded-md border border-input bg-muted/40 px-2 py-1.5">
    <div class="flex flex-wrap items-center gap-1">
      {#each snippetGroups as g}
        <button
          type="button"
          onclick={() => (activeSnippetGroup = g.id)}
          class="rounded px-2 py-1 text-xs transition-colors {activeSnippetGroup === g.id ? 'bg-primary text-primary-foreground' : 'border border-border bg-background hover:bg-muted'}"
        >
          {g.label}
        </button>
      {/each}

      {#if livePreview}
        <button
          type="button"
          onclick={() => (previewEnabled = !previewEnabled)}
          class="ml-auto rounded border border-border bg-background px-2 py-1 text-xs hover:bg-muted transition-colors"
        >
          {previewEnabled ? 'Önizleme Açık' : 'Önizleme Kapalı'}
        </button>
      {/if}
    </div>

    <div class="flex flex-wrap items-center gap-1">
      {#each activeSnippets as s}
        <button
          type="button"
          onclick={() => insertSnippet(s.code)}
          title={s.title}
          class="rounded border border-border bg-background px-2 py-1 text-xs hover:bg-muted transition-colors"
        >
          {s.label}
        </button>
      {/each}
    </div>
  </div>

  <textarea
    bind:this={editorEl}
    value={value}
    oninput={onInput}
    onscroll={onScroll}
    onkeydown={onKeyDown}
    rows={rows}
    spellcheck="false"
    placeholder={placeholder}
    class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm font-mono leading-relaxed
           ring-offset-background placeholder:text-muted-foreground resize-y
           focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
    style={textareaHighlightStyle()}
  ></textarea>

  {#if livePreview && previewEnabled}
    <div class="overflow-hidden rounded-md border border-input bg-card">
      <div class="flex items-center justify-between border-b bg-muted/40 px-3 py-2 text-xs">
        <span class="font-medium">Canlı PDF Önizleme</span>
        <span class="text-muted-foreground">{compiling ? 'Derleniyor…' : 'Hazır'}</span>
      </div>

      <div class="relative h-56 bg-muted/20">
        {#if blobUrl}
          <iframe
            src={blobUrl}
            title="Typst canlı önizleme"
            class="h-full w-full border-0"
          ></iframe>
        {:else if !error}
          <div class="absolute inset-0 flex items-center justify-center text-xs text-muted-foreground">
            Yazdıkça PDF önizleme burada güncellenir.
          </div>
        {/if}

        {#if compiling}
          <div class="absolute inset-0 flex items-center justify-center bg-background/50">
            <span class="inline-block h-6 w-6 animate-spin rounded-full border-2 border-primary/30 border-t-primary"></span>
          </div>
        {/if}

        {#if error}
          <div class="absolute inset-0 overflow-auto bg-destructive/10 p-3 text-xs text-destructive">
            <p class="whitespace-pre-wrap">{error}</p>
            {#if errorLine}
              <button
                type="button"
                onclick={jumpToErrorLine}
                class="mt-2 rounded border border-destructive/40 bg-background px-2 py-1 text-xs hover:bg-destructive/5"
              >
                Satır {errorLine} konumuna git
              </button>
            {/if}
          </div>
        {/if}
      </div>
    </div>
  {/if}

  <p class="text-xs text-muted-foreground">
    Bu modda içerik doğrudan Typst olarak kaydedilir ve PDF üretiminde ham haliyle kullanılır.
  </p>
</div>

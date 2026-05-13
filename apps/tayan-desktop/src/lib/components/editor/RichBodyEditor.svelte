<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Editor } from '@tiptap/core';
  import StarterKit from '@tiptap/starter-kit';
  import Underline from '@tiptap/extension-underline';
  import { Image as TiptapImage } from '@tiptap/extension-image';
  import FileHandler from '@tiptap/extension-file-handler';
  import { Mathematics } from '@tiptap/extension-mathematics';
  import katex from 'katex';
  import 'katex/contrib/mhchem';
  import { invoke } from '@tauri-apps/api/core';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import type { ContentNode, TextStyle } from '$lib/types';

  // ── Props ──────────────────────────────────────────────────────────────────
  let {
    nodes = [],
    onchange,
    placeholder = 'Metni buraya yazın…',
    context = 'img',
  }: {
    nodes?: ContentNode[];
    onchange?: (nodes: ContentNode[]) => void;
    placeholder?: string;
    /** Naming context for saved images, e.g. 'soru' or 'sinav' */
    context?: string;
  } = $props();

  // ── Types ──────────────────────────────────────────────────────────────────
  type ImgEntry = {
    id: string;
    src: string;
    alt: string;
    align: 'left' | 'center' | 'right';
    width: string;
    wrap: boolean;
  };

  // ── State ──────────────────────────────────────────────────────────────────
  let _idc = 0;
  let imgList = $state<ImgEntry[]>(
    nodes
      .filter((n): n is Extract<ContentNode, { type: 'image' }> => n.type === 'image')
      .map((n, k) => ({
        id: `i${k}`,
        src: n.src,
        alt: n.alt,
        align: n.align ?? 'center',
        width: n.width ?? '50%',
        wrap: n.wrap ?? false,
      }))
  );
  if (imgList.length) _idc = imgList.length;

  let editorEl:  HTMLDivElement     | undefined = $state();
  let fileRef:   HTMLInputElement   | undefined = $state();
  let mathPopupInputEl: HTMLTextAreaElement | undefined = $state();

  let showLatexRef    = $state(false);
  let mathPopupVisible = $state(false);
  let mathPopupMode    = $state<'inline' | 'block'>('inline');
  let mathPopupVal     = $state('');
  let mathPopupEqCat   = $state(0);

  let editor: Editor | undefined;
  // Reactive editor state — reassigned on each transaction to trigger Svelte re-renders
  // (required for toolbar isActive checks per TipTap Svelte docs)
  let editorState = $state<{ editor: Editor | null }>({ editor: null });

  // ── Escape helpers ─────────────────────────────────────────────────────────
  function escAttr(s: string): string {
    return s.replace(/&/g, '&amp;').replace(/"/g, '&quot;');
  }

  // ── Image style ────────────────────────────────────────────────────────────
  function imgStyle(align: string, wrap: boolean, width: string): string {
    if (wrap && align !== 'center') {
      const margin = align === 'left'
        ? 'margin-right:0.75em;margin-bottom:0.5em;'
        : 'margin-left:0.75em;margin-bottom:0.5em;';
      return `float:${align};${margin}width:${width};max-height:250px;object-fit:contain;`;
    }
    const blk =
      align === 'center' ? 'display:block;margin-left:auto;margin-right:auto;'
      : align === 'right' ? 'display:block;margin-left:auto;'
      : 'display:block;';
    return `${blk}width:${width};max-height:250px;object-fit:contain;margin-bottom:0.5em;`;
  }

  // ── ContentNode[] → TipTap HTML ────────────────────────────────────────────
  function nodesToHtml(ns: ContentNode[]): string {
    const imgMap = new Map(imgList.map(img => [img.src, img]));
    const parts: string[] = [];
    for (const n of ns) {
      if (n.type === 'text') {
        let html = n.text
          .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
        if (n.style.strikethrough) html = `<s>${html}</s>`;
        if (n.style.underline)     html = `<u>${html}</u>`;
        if (n.style.italic)        html = `<em>${html}</em>`;
        if (n.style.bold)          html = `<strong>${html}</strong>`;
        parts.push(html);
      } else if (n.type === 'math') {
        if (n.display === 'block') {
          parts.push(`$$${n.raw}$$`);
        } else {
          parts.push(`$${n.raw}$`);
        }
      } else if (n.type === 'image') {
        const e = imgMap.get(n.src);
        if (e) {
          parts.push(`<img src="${escAttr(e.src)}" alt="${escAttr(e.alt)}" data-id="${escAttr(e.id)}" data-align="${escAttr(e.align)}" data-wrap="${e.wrap}" data-width="${escAttr(e.width)}">`);
        }
      } else if (n.type === 'newline') {
        parts.push('<br>');
      }
    }
    return `<p>${parts.join('')}</p>`;
  }

  // ── TipTap JSON → ContentNode[] ────────────────────────────────────────────
  function jsonToNodes(json: Record<string, unknown>): ContentNode[] {
    const result: ContentNode[] = [];

    function processNode(node: Record<string, unknown>, style: TextStyle) {
      const type = node.type as string;
      const marks = (node.marks as Record<string, unknown>[] | undefined) ?? [];
      const childStyle: TextStyle = {
        bold:          style.bold          || marks.some(m => m.type === 'bold'),
        italic:        style.italic        || marks.some(m => m.type === 'italic'),
        underline:     style.underline     || marks.some(m => m.type === 'underline'),
        strikethrough: style.strikethrough || marks.some(m => m.type === 'strike'),
      };

      if (type === 'text') {
        const text = (node.text as string) ?? '';
        if (text) result.push({ type: 'text', text, style: childStyle });
        return;
      }
      if (type === 'inlineMath') {
        const latex = ((node.attrs as Record<string, unknown>)?.latex as string) ?? '';
        if (latex) result.push({ type: 'math', raw: latex, display: 'inline' });
        return;
      }
      if (type === 'blockMath') {
        const latex = ((node.attrs as Record<string, unknown>)?.latex as string) ?? '';
        if (latex) result.push({ type: 'math', raw: latex, display: 'block' });
        return;
      }
      if (type === 'image') {
        const attrs = (node.attrs as Record<string, unknown>) ?? {};
        const src = (attrs.src as string) ?? '';
        if (src) {
          result.push({
            type: 'image',
            src,
            alt: (attrs.alt as string) ?? '',
            align: ((attrs.align as string) ?? 'center') as 'left' | 'center' | 'right',
            width: (attrs.width as string) ?? '50%',
            wrap: Boolean(attrs.wrap),
          });
        }
        return;
      }
      if (type === 'hardBreak') {
        result.push({ type: 'newline' });
        return;
      }

      const isBlock = ['paragraph', 'heading', 'listItem', 'bulletList', 'orderedList', 'blockquote'].includes(type);
      const children = (node.content as Record<string, unknown>[] | undefined) ?? [];
      if (isBlock && result.length > 0) result.push({ type: 'newline' });
      for (const child of children) processNode(child, childStyle);
    }

    const root = (json.content as Record<string, unknown>[] | undefined) ?? [];
    for (const node of root) processNode(node, { bold: false, italic: false, underline: false, strikethrough: false });
    return result;
  }

  // ── Emit ──────────────────────────────────────────────────────────────────
  function emit() {
    if (!editor) return;
    const json = editor.getJSON() as Record<string, unknown>;
    onchange?.(jsonToNodes(json));
  }

  // ── Custom Image extension — stores align/wrap/width as TipTap node attrs ────────
  const CustomImage = TiptapImage.extend({
    addAttributes() {
      return {
        ...this.parent?.(),
        'data-id': {
          default: null,
          parseHTML: el => el.getAttribute('data-id'),
          renderHTML: attrs => (attrs['data-id'] ? { 'data-id': attrs['data-id'] } : {}),
        },
        align: {
          default: 'center',
          parseHTML: el => el.getAttribute('data-align') ?? 'center',
          renderHTML: () => ({}),
        },
        wrap: {
          default: false,
          parseHTML: el => el.getAttribute('data-wrap') === 'true',
          renderHTML: () => ({}),
        },
        width: {
          default: '50%',
          parseHTML: el => el.getAttribute('data-width') ?? '50%',
          renderHTML: attrs => ({
            'data-align': attrs.align,
            'data-wrap': String(attrs.wrap),
            'data-width': attrs.width,
            style: imgStyle(
              String(attrs.align ?? 'center'),
              Boolean(attrs.wrap),
              String(attrs.width ?? '50%'),
            ),
          }),
        },
      };
    },
  }).configure({ inline: true, allowBase64: false, HTMLAttributes: { class: 'rounded border editor-img' } });

  // ── Mount ─────────────────────────────────────────────────────────────────
  onMount(() => {
    editor = new Editor({
      element: editorEl,
      extensions: [
        StarterKit.configure({ strike: {}, bold: {}, italic: {}, hardBreak: {} }),
        Underline,
        CustomImage.configure({ inline: true, allowBase64: false }),
        Mathematics.configure({ katexOptions: { throwOnError: false } }),
      ],
      content: nodesToHtml(nodes),
      onUpdate: () => emit(),
      onTransaction: ({ editor: e }) => {
        // Reassign to trigger Svelte reactivity for isActive checks
        editorState = { editor: e };
      },
      editorProps: {
        attributes: {
          class: 'tiptap-editor',
        },
      },
    });
    editorState = { editor };
  });

  onDestroy(() => editor?.destroy());

  // ── Toolbar actions ────────────────────────────────────────────────────────
  function fmt(cmd: 'bold' | 'italic' | 'underline' | 'strike') {
    editor?.chain().focus().toggleMark(cmd).run();
  }

  function openMathPopup(mode: 'inline' | 'block') {
    mathPopupMode    = mode;
    mathPopupVal     = '';
    mathPopupVisible = true;
    showLatexRef     = false;
    setTimeout(() => mathPopupInputEl?.focus());
  }

  function closeMathPopup() {
    mathPopupVisible = false;
    mathPopupVal     = '';
    editor?.commands.focus();
  }

  function commitMathPopup() {
    const raw = mathPopupVal.trim();
    if (!raw || !editor) { closeMathPopup(); return; }
    mathPopupVisible = false;
    mathPopupVal     = '';
    editor.chain().focus().run();
    if (mathPopupMode === 'block') {
      editor.commands.insertBlockMath({ latex: raw });
    } else {
      editor.commands.insertInlineMath({ latex: raw });
    }
  }

  // Inserts a latex snippet into the popup textarea at current cursor position
  function insertSymToPopup(latex: string) {
    if (!mathPopupInputEl) { mathPopupVal += latex; return; }
    const start = mathPopupInputEl.selectionStart ?? mathPopupVal.length;
    const end   = mathPopupInputEl.selectionEnd   ?? start;
    mathPopupVal = mathPopupVal.slice(0, start) + latex + mathPopupVal.slice(end);
    setTimeout(() => {
      mathPopupInputEl!.setSelectionRange(start + latex.length, start + latex.length);
      mathPopupInputEl!.focus();
    });
  }

  // ── Image upload helper ─────────────────────────────────────────────────────────────
  function handleImageFile(file: File, insertFn: (src: string, alt: string, id: string) => void) {
    const ext = file.name.split('.').pop() ?? 'png';
    const reader = new FileReader();
    reader.onload = async ev => {
      const b64data = ev.target!.result as string;
      let src: string;
      try {
        const absPath = await invoke<string>('save_image', { data: b64data, ext, context });
        src = convertFileSrc(absPath);
      } catch {
        src = b64data;
      }
      const id = `i${_idc++}`;
      insertFn(src, file.name, id);
    };
    reader.readAsDataURL(file);
  }

  function onFile(e: Event) {
    const file = (e.target as HTMLInputElement).files?.[0];
    if (!file) return;
    handleImageFile(file, (src, alt, id) => {
      editor?.chain().focus().insertContent({
        type: 'image',
        attrs: { src, alt, 'data-id': id, align: 'center', wrap: false, width: '50%' },
      }).run();
    });
    (e.target as HTMLInputElement).value = '';
  }

  function delImg(i: number) {
    const entry = imgList[i];
    editor?.chain().focus().command(({ tr, state }) => {
      let found = false;
      state.doc.descendants((node, pos) => {
        if (node.type.name === 'image' && node.attrs['data-id'] === entry.id) {
          tr.delete(pos, pos + node.nodeSize);
          found = true;
          return false;
        }
      });
      return found;
    }).run();
    emit();
  }

  // ── Sync imgList from editor JSON (runs on every transaction) ──────────────────
  function syncImgListFromEditor(e: Editor) {
    const json = e.getJSON() as { content?: unknown[] };
    const imgs: ImgEntry[] = [];
    function walk(node: Record<string, unknown>) {
      if (node.type === 'image') {
        const attrs = (node.attrs as Record<string, unknown>) ?? {};
        const id = (attrs['data-id'] as string) ?? '';
        const src = (attrs.src as string) ?? '';
        if (id && src) {
          imgs.push({
            id,
            src,
            alt: (attrs.alt as string) ?? '',
            align: ((attrs.align as string) ?? 'center') as 'left' | 'center' | 'right',
            width: (attrs.width as string) ?? '50%',
            wrap: Boolean(attrs.wrap),
          });
        }
      }
      const children = (node.content as Record<string, unknown>[] | undefined) ?? [];
      for (const child of children) walk(child);
    }
    const root = (json.content as Record<string, unknown>[] | undefined) ?? [];
    for (const node of root) walk(node as Record<string, unknown>);
    imgList = imgs;
  }

  // ── Update a TipTap image node's attrs by data-id ─────────────────────────────
  function updateNodeAttrs(dataId: string, newAttrs: Partial<{ align: string; wrap: boolean; width: string }>) {
    if (!editor) return;
    editor.chain().command(({ tr, state }) => {
      let found = false;
      state.doc.descendants((node, pos) => {
        if (node.type.name === 'image' && node.attrs['data-id'] === dataId) {
          tr.setNodeMarkup(pos, undefined, { ...node.attrs, ...newAttrs });
          found = true;
          return false;
        }
      });
      return found;
    }).run();
    emit();
  }

  function toggleWrap(i: number) {
    updateNodeAttrs(imgList[i].id, { wrap: !imgList[i].wrap });
  }
  function setImgAlign(i: number, align: ImgEntry['align']) {
    updateNodeAttrs(imgList[i].id, { align });
  }
  function setImgWidth(i: number, pct: number) {
    updateNodeAttrs(imgList[i].id, { width: `${pct}%` });
  }

  // ── Equation categories ────────────────────────────────────────────────────
  type EqItem = { sym: string; latex: string; title: string };
  type EqCat  = { label: string; items: EqItem[] };

  const EQ_CATS: EqCat[] = [
    { label: 'Yunan', items: [
      { sym: 'α', latex: '\\alpha',   title: 'alfa' },
      { sym: 'β', latex: '\\beta',    title: 'beta' },
      { sym: 'γ', latex: '\\gamma',   title: 'gama' },
      { sym: 'δ', latex: '\\delta',   title: 'delta' },
      { sym: 'ε', latex: '\\epsilon', title: 'epsilon' },
      { sym: 'θ', latex: '\\theta',   title: 'teta' },
      { sym: 'λ', latex: '\\lambda',  title: 'lambda' },
      { sym: 'μ', latex: '\\mu',      title: 'mü' },
      { sym: 'π', latex: '\\pi',      title: 'pi' },
      { sym: 'σ', latex: '\\sigma',   title: 'sigma' },
      { sym: 'φ', latex: '\\phi',     title: 'fi' },
      { sym: 'Δ', latex: '\\Delta',   title: 'büyük delta' },
      { sym: 'Σ', latex: '\\Sigma',   title: 'büyük sigma' },
      { sym: 'Ω', latex: '\\Omega',   title: 'büyük omega' },
    ]},
    { label: 'Aritmetik', items: [
      { sym: '×',   latex: '\\times',      title: 'çarpı' },
      { sym: '÷',   latex: '\\div',        title: 'bölü' },
      { sym: '±',   latex: '\\pm',         title: 'artı eksi' },
      { sym: '·',   latex: '\\cdot',       title: 'nokta çarpım' },
      { sym: 'a/b', latex: '\\frac{a}{b}', title: 'kesir' },
      { sym: '√x',  latex: '\\sqrt{x}',    title: 'karekök' },
      { sym: 'ⁿ√x', latex: '\\sqrt[n]{x}', title: 'n. kök' },
      { sym: 'xⁿ',  latex: 'x^{n}',        title: 'üs' },
      { sym: 'xₙ',  latex: 'x_{n}',        title: 'alt indis' },
      { sym: '|x|', latex: '|x|',           title: 'mutlak değer' },
    ]},
    { label: 'İlişki', items: [
      { sym: '≤', latex: '\\leq',    title: 'küçük eşit' },
      { sym: '≥', latex: '\\geq',    title: 'büyük eşit' },
      { sym: '≠', latex: '\\neq',    title: 'eşit değil' },
      { sym: '≈', latex: '\\approx', title: 'yaklaşık eşit' },
      { sym: '∞', latex: '\\infty',  title: 'sonsuz' },
    ]},
    { label: 'Analiz', items: [
      { sym: '∫',    latex: '\\int',            title: 'integral' },
      { sym: '∫ₐᵇ', latex: '\\int_a^b',         title: 'belirli integral' },
      { sym: '∑',    latex: '\\sum_{i=1}^{n}',   title: 'toplam' },
      { sym: 'lim',  latex: '\\lim_{x \\to 0}',  title: 'limit' },
      { sym: 'd/dx', latex: '\\frac{d}{dx}',      title: 'türev' },
      { sym: '∂',    latex: '\\partial',           title: 'kısmi türev' },
    ]},
    { label: 'Küme', items: [
      { sym: '∈', latex: '\\in',        title: 'elemanı' },
      { sym: '⊂', latex: '\\subset',    title: 'alt küme' },
      { sym: '∪', latex: '\\cup',       title: 'birleşim' },
      { sym: '∩', latex: '\\cap',       title: 'kesişim' },
      { sym: '∅', latex: '\\emptyset',  title: 'boş küme' },
      { sym: 'ℝ', latex: '\\mathbb{R}', title: 'gerçel sayılar' },
    ]},
    { label: 'Mantık', items: [
      { sym: '∀', latex: '\\forall',          title: 'her için' },
      { sym: '∃', latex: '\\exists',          title: 'var olan' },
      { sym: '¬', latex: '\\neg',             title: 'değil' },
      { sym: '∧', latex: '\\land',            title: 've' },
      { sym: '∨', latex: '\\lor',             title: 'veya' },
      { sym: '⇒', latex: '\\Rightarrow',      title: 'öndeyişe' },
      { sym: '⟺', latex: '\\Leftrightarrow',  title: 'ancak ve ancak' },
    ]},
    { label: 'Geometri', items: [
      { sym: '∠',  latex: '\\angle',    title: 'açı' },
      { sym: '∥',  latex: '\\parallel', title: 'paralel' },
      { sym: '⊥',  latex: '\\perp',     title: 'dik' },
      { sym: '△',  latex: '\\triangle', title: 'üçgen' },
      { sym: '→v', latex: '\\vec{v}',   title: 'vektör' },
      { sym: '°',  latex: '^{\\circ}',  title: 'derece' },
    ]},
  ];

  // ── LaTeX reference ────────────────────────────────────────────────────────
  const LATEX_REF = [
    { cat: 'Kesir',      code: '\\frac{a}{b}',            ex: '\\frac{3}{4}' },
    { cat: 'Karekök',    code: '\\sqrt{x}',               ex: '\\sqrt{16}' },
    { cat: 'n. Kök',     code: '\\sqrt[n]{x}',            ex: '\\sqrt[3]{8}' },
    { cat: 'Üs',         code: 'x^{n}',                    ex: '2^{10}' },
    { cat: 'Alt İndis',  code: 'x_{n}',                    ex: 'a_{1} + a_{2}' },
    { cat: 'İntegral',   code: '\\int_a^b f(x)\\,dx',     ex: '\\int_0^1 x^2\\,dx' },
    { cat: 'Toplam',     code: '\\sum_{i=1}^{n} a_i',      ex: '\\sum_{i=1}^{4} i^2' },
    { cat: 'Limit',      code: '\\lim_{x \\to 0} f(x)',    ex: '\\lim_{x\\to 0}\\frac{\\sin x}{x}' },
    { cat: 'Türev',      code: '\\frac{d}{dx} f(x)',        ex: '\\frac{d}{dx} x^3 = 3x^2' },
    { cat: 'Vektör',     code: '\\vec{v}',                 ex: '\\vec{F} = m\\vec{a}' },
    { cat: 'Matris',     code: '\\begin{pmatrix}a&b\\\\c&d\\end{pmatrix}', ex: '\\begin{pmatrix}1&2\\\\3&4\\end{pmatrix}' },
    { cat: 'Pi',         code: '\\pi',                     ex: '\\pi r^2' },
    { cat: 'Sonsuz',     code: '\\infty',                  ex: 'n \\to \\infty' },
    { cat: 'Yunan',      code: '\\alpha, \\beta, \\gamma', ex: '\\alpha^2 + \\beta^2 = \\gamma^2' },
  ];

  const LATEX_TIPS = [
    'Süslü parantez { } birden fazla karakteri gruplar: x^{10}, \\frac{a+b}{c+d}',
    'Boşluklar formül içinde genellikle yoksayılır',
    'Satır içi formül için $formül$ yazın — otomatik dönüşür',
    'Blok formül için $$formül$$ yazın — ortalanmış büyük formül',
  ];

  const CHEM_REF = [
    { cat: 'Su',              code: '\\ce{H2O}',                  ex: '\\ce{H2O}' },
    { cat: 'Karbon dioksit',  code: '\\ce{CO2}',                  ex: '\\ce{CO2}' },
    { cat: 'Sülfürik asit',   code: '\\ce{H2SO4}',                ex: '\\ce{H2SO4}' },
    { cat: 'Tuz',             code: '\\ce{NaCl}',                 ex: '\\ce{NaCl}' },
    { cat: 'Pozitif iyon',    code: '\\ce{Ca^{2+}}',              ex: '\\ce{Ca^{2+}}' },
    { cat: 'Negatif iyon',    code: '\\ce{Cl^{-}}',               ex: '\\ce{Cl^{-}}' },
    { cat: 'Reaksiyon oku',   code: '\\ce{A -> B}',               ex: '\\ce{2H2 + O2 -> 2H2O}' },
    { cat: 'Denge oku',       code: '\\ce{A <=> B}',              ex: '\\ce{N2 + 3H2 <=> 2NH3}' },
    { cat: 'Oksidasyon',      code: '\\ce{Zn -> Zn^{2+} + 2e^-}', ex: '\\ce{Zn -> Zn^{2+} + 2e^-}' },
    { cat: 'Çökelti',         code: '\\ce{AgCl v}',               ex: '\\ce{Ag+ + Cl- -> AgCl v}' },
    { cat: 'Gaz',             code: '\\ce{CO2 ^}',                ex: '\\ce{CaCO3 -> CaO + CO2 ^}' },
    { cat: 'İzotop',          code: '\\ce{^{14}_{6}C}',           ex: '\\ce{^{14}_{6}C}' },
  ];
</script>

<div class="space-y-2">

  <!-- Toolbar -->
  <div class="flex flex-wrap items-center gap-1 rounded-md border border-input bg-muted/40 px-2 py-1.5">
    <button type="button" onmousedown={(e) => { e.preventDefault(); fmt('bold'); }}
      title="Kalın (Ctrl+B)"
      class="w-7 rounded py-1 text-sm font-bold hover:bg-background transition-colors select-none {editorState.editor?.isActive('bold') ? 'bg-muted ring-1 ring-ring' : ''}">B</button>
    <button type="button" onmousedown={(e) => { e.preventDefault(); fmt('italic'); }}
      title="İtalik (Ctrl+I)"
      class="w-7 rounded py-1 text-sm italic hover:bg-background transition-colors select-none {editorState.editor?.isActive('italic') ? 'bg-muted ring-1 ring-ring' : ''}">I</button>
    <button type="button" onmousedown={(e) => { e.preventDefault(); fmt('underline'); }}
      title="Altı çizili (Ctrl+U)"
      class="w-7 rounded py-1 text-sm underline hover:bg-background transition-colors select-none {editorState.editor?.isActive('underline') ? 'bg-muted ring-1 ring-ring' : ''}">U</button>
    <button type="button" onmousedown={(e) => { e.preventDefault(); fmt('strike'); }}
      title="Üstü çizili"
      class="w-7 rounded py-1 text-sm line-through hover:bg-background transition-colors select-none {editorState.editor?.isActive('strike') ? 'bg-muted ring-1 ring-ring' : ''}">S</button>

    <span class="mx-1 h-4 w-px shrink-0 bg-border"></span>

    <button type="button" onclick={() => openMathPopup('inline')}
      title="Satır içi formül — ya da $...$ yazın"
      class="rounded px-2 py-1 text-sm font-mono hover:bg-background transition-colors select-none {mathPopupVisible && mathPopupMode === 'inline' ? 'bg-background ring-1 ring-ring' : ''}">
      <i>f</i>(x)
    </button>
    <button type="button" onclick={() => openMathPopup('block')}
      title="Blok formül — ya da $$...$$ yazın"
      class="rounded px-2 py-1 text-sm hover:bg-background transition-colors select-none {mathPopupVisible && mathPopupMode === 'block' ? 'bg-background ring-1 ring-ring' : ''}">
      ∑
    </button>

    <span class="mx-1 h-4 w-px shrink-0 bg-border"></span>

    <button type="button" onclick={() => fileRef?.click()}
      title="Resim ekle"
      class="flex items-center gap-1.5 rounded px-2 py-1 text-sm hover:bg-background transition-colors select-none">
      <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24"
        fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <rect width="18" height="18" x="3" y="3" rx="2" ry="2"/>
        <circle cx="9" cy="9" r="2"/>
        <path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"/>
      </svg>
      Resim
    </button>
    <input bind:this={fileRef} type="file" accept="image/*" class="hidden" onchange={onFile} />

    {#if editorState.editor?.isActive('image')}
      <span class="mx-1 h-4 w-px shrink-0 bg-border"></span>
      <div class="flex gap-0.5" title="Resim hizalama">
        <button type="button"
          onmousedown={(e) => { e.preventDefault(); editorState.editor?.chain().focus().updateAttributes('image', { align: 'left' }).run(); }}
          title="Sola hizala"
          class="rounded border px-2 py-0.5 text-sm transition-colors {editorState.editor?.getAttributes('image').align === 'left' ? 'bg-primary text-primary-foreground' : 'hover:bg-muted'}">←</button>
        <button type="button"
          onmousedown={(e) => { e.preventDefault(); editorState.editor?.chain().focus().updateAttributes('image', { align: 'center' }).run(); }}
          title="Ortala"
          class="rounded border px-2 py-0.5 text-sm transition-colors {(editorState.editor?.getAttributes('image').align ?? 'center') === 'center' ? 'bg-primary text-primary-foreground' : 'hover:bg-muted'}">⊡</button>
        <button type="button"
          onmousedown={(e) => { e.preventDefault(); editorState.editor?.chain().focus().updateAttributes('image', { align: 'right' }).run(); }}
          title="Sağa hizala"
          class="rounded border px-2 py-0.5 text-sm transition-colors {editorState.editor?.getAttributes('image').align === 'right' ? 'bg-primary text-primary-foreground' : 'hover:bg-muted'}">→</button>
      </div>
    {/if}

    <span class="mx-1 h-4 w-px shrink-0 bg-border"></span>

    <button type="button"
      onclick={() => { showLatexRef = !showLatexRef; mathPopupVisible = false; }}
      title="LaTeX kılavuzu"
      class="ml-auto flex h-6 w-6 items-center justify-center rounded-full border text-xs font-bold hover:bg-background transition-colors select-none {showLatexRef ? 'bg-background ring-1 ring-ring' : 'text-muted-foreground'}">
      ?
    </button>
  </div>

  <!-- ── Math popup ──────────────────────────────────────────────────────── -->
  {#if mathPopupVisible}
    <div class="rounded-lg border border-primary bg-card shadow-lg text-sm">
      <!-- Header -->
      <div class="flex items-center justify-between border-b px-3 py-2">
        <div class="flex gap-1">
          <button type="button" onclick={() => mathPopupMode = 'inline'}
            class="rounded px-2.5 py-1 text-xs font-medium transition-colors {mathPopupMode === 'inline' ? 'bg-primary text-primary-foreground' : 'hover:bg-muted text-muted-foreground'}">
            <i>f</i>(x) Satır içi
          </button>
          <button type="button" onclick={() => mathPopupMode = 'block'}
            class="rounded px-2.5 py-1 text-xs font-medium transition-colors {mathPopupMode === 'block' ? 'bg-primary text-primary-foreground' : 'hover:bg-muted text-muted-foreground'}">
            ∑ Blok (ortalı)
          </button>
        </div>
        <button type="button" onclick={closeMathPopup}
          class="text-muted-foreground hover:text-foreground leading-none text-base">✕</button>
      </div>

      <div class="p-3 space-y-2.5">
        <!-- Live preview -->
        <div class="min-h-[3.5rem] flex items-center justify-{mathPopupMode === 'block' ? 'center' : 'start'} rounded border bg-muted/30 px-4 py-2 overflow-x-auto">
          {#if mathPopupVal.trim()}
            <!-- eslint-disable-next-line svelte/no-at-html-tags -->
            {@html katex.renderToString(mathPopupVal, { displayMode: mathPopupMode === 'block', throwOnError: false })}
          {:else}
            <span class="text-xs text-muted-foreground">Önizleme burada görünür…</span>
          {/if}
        </div>

        <!-- LaTeX input -->
        <textarea
          bind:this={mathPopupInputEl}
          bind:value={mathPopupVal}
          rows="3"
          placeholder="LaTeX kodu girin…&#10;örn: \frac&#123;a&#125;&#123;b&#125;  ya da  \int_0^1 x^2\,dx"
          class="w-full rounded border bg-background px-3 py-2 font-mono text-sm resize-none focus:outline-none focus:ring-1 focus:ring-ring"
          onkeydown={(e) => {
            if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); commitMathPopup(); }
            if (e.key === 'Escape') closeMathPopup();
          }}
        ></textarea>

        <!-- Symbol palette -->
        <div class="space-y-1.5">
          <div class="flex flex-wrap gap-1">
            {#each EQ_CATS as cat, i}
              <button type="button" onclick={() => mathPopupEqCat = i}
                class="rounded px-2 py-0.5 text-xs font-medium transition-colors {mathPopupEqCat === i ? 'bg-primary text-primary-foreground' : 'hover:bg-muted text-muted-foreground'}">
                {cat.label}
              </button>
            {/each}
          </div>
          <div class="flex flex-wrap gap-1">
            {#each EQ_CATS[mathPopupEqCat].items as item}
              <button type="button" onclick={() => insertSymToPopup(item.latex)}
                title="{item.title} — {item.latex}"
                class="min-w-[2.5rem] rounded border bg-background px-1.5 py-1 text-center font-mono text-xs hover:bg-muted transition-colors">
                {item.sym}
              </button>
            {/each}
          </div>
        </div>

        <!-- Actions -->
        <div class="flex justify-end gap-2 pt-0.5">
          <button type="button" onclick={closeMathPopup}
            class="rounded border px-3 py-1 text-xs hover:bg-muted transition-colors">İptal</button>
          <button type="button" onclick={commitMathPopup}
            class="rounded bg-primary px-3 py-1 text-xs text-primary-foreground hover:bg-primary/90 transition-colors">Ekle</button>
        </div>
      </div>
    </div>
  {/if}

  <!-- TipTap editor surface -->
  <div
    bind:this={editorEl}
    class="tiptap-wrap min-h-[8rem] w-full rounded-md border border-input bg-background px-3 py-2 text-sm leading-relaxed overflow-auto"
  ></div>

  <!-- Image control cards -->
  {#each imgList as img, i}
    <div class="rounded border bg-card p-2 space-y-1.5 text-xs">
      <div class="flex flex-wrap items-center gap-2">
        <span class="max-w-[10rem] truncate font-medium">{img.alt}</span>
        <button type="button" onclick={() => toggleWrap(i)}
          title="Metin resmin etrafına sarsın"
          class="rounded border px-1.5 py-0.5 font-medium transition-colors {img.wrap ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-muted'}">
          ⇌ Sar
        </button>
        <div class="flex gap-0.5">
          <button type="button" onclick={() => setImgAlign(i, 'left')} title="Sola"
            class="rounded border px-2 py-0.5 transition-colors {img.align === 'left' ? 'bg-primary text-primary-foreground' : 'hover:bg-muted'}">←</button>
          <button type="button" onclick={() => setImgAlign(i, 'center')} title="Ortala"
            class="rounded border px-2 py-0.5 transition-colors {img.align === 'center' ? 'bg-primary text-primary-foreground' : 'hover:bg-muted'}">⊡</button>
          <button type="button" onclick={() => setImgAlign(i, 'right')} title="Sağa"
            class="rounded border px-2 py-0.5 transition-colors {img.align === 'right' ? 'bg-primary text-primary-foreground' : 'hover:bg-muted'}">→</button>
        </div>
        <div class="ml-auto flex items-center gap-1.5">
          <span class="w-8 text-right tabular-nums text-muted-foreground">{img.width}</span>
          <input type="range" min="20" max="100" step="5"
            value={parseInt(img.width)}
            oninput={(e) => setImgWidth(i, parseInt((e.target as HTMLInputElement).value))}
            class="w-24 accent-primary"
          />
        </div>
        <button type="button" onclick={() => delImg(i)}
          class="flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-destructive text-xs text-destructive-foreground">
          ✕
        </button>
      </div>
      <img src={img.src} alt={img.alt} class="h-12 w-auto max-w-full rounded border object-contain" />
    </div>
  {/each}

  <!-- LaTeX reference modal -->
  {#if showLatexRef}
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="fixed inset-0 z-40 bg-black/40" onclick={() => showLatexRef = false}></div>

    <div class="fixed inset-0 z-50 flex items-center justify-center p-4 pointer-events-none">
      <div class="pointer-events-auto w-full max-w-2xl max-h-[80vh] flex flex-col rounded-xl border bg-card shadow-xl text-sm">

        <!-- Header -->
        <div class="flex items-center justify-between border-b px-5 py-3 shrink-0">
          <h3 class="font-semibold text-base">LaTeX Formül Kılavuzu</h3>
          <button type="button" onclick={() => showLatexRef = false}
            class="text-muted-foreground hover:text-foreground text-lg leading-none transition-colors">✕</button>
        </div>

        <!-- Scrollable body -->
        <div class="overflow-y-auto p-5 space-y-5">

          <!-- Inline vs Block intro -->
          <div class="space-y-2">
            <p class="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground">Formül Ekleme</p>
            <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
              <div class="rounded border bg-muted/30 px-3 py-2.5 space-y-1.5">
                <code class="block text-xs text-muted-foreground">$formül$ yazın → otomatik dönüşür</code>
                <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                <div class="py-0.5">{@html katex.renderToString('x^2 + y^2 = z^2', { throwOnError: false })}</div>
                <p class="text-xs text-muted-foreground">Satır içi — metin arasında gösterilir</p>
              </div>
              <div class="rounded border bg-muted/30 px-3 py-2.5 space-y-1.5">
                <code class="block text-xs text-muted-foreground">$$formül$$ yazın → ortalanmış blok</code>
                <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                <div class="py-0.5">{@html katex.renderToString('\\int_0^1 x^2\\,dx', { displayMode: true, throwOnError: false })}</div>
                <p class="text-xs text-muted-foreground">Blok — ortalanmış büyük formül</p>
              </div>
            </div>
          </div>

          <!-- Commands table -->
          <div class="space-y-2">
            <p class="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground">Sık Kullanılan Komutlar</p>
            <div class="overflow-x-auto rounded border">
              <table class="w-full text-xs">
                <thead class="bg-muted/50 text-muted-foreground">
                  <tr>
                    <th class="px-3 py-2 text-left font-medium">Açıklama</th>
                    <th class="px-3 py-2 text-left font-medium">Kod</th>
                    <th class="px-3 py-2 text-left font-medium">Görünüm</th>
                    <th class="px-3 py-2 w-8"></th>
                  </tr>
                </thead>
                <tbody class="divide-y">
                  {#each LATEX_REF as row}
                    <tr class="hover:bg-muted/20">
                      <td class="px-3 py-1.5 text-muted-foreground">{row.cat}</td>
                      <td class="px-3 py-1.5">
                        <code class="rounded bg-muted px-1 py-0.5 text-[11px] font-mono">{row.code}</code>
                      </td>
                      <td class="px-3 py-1.5">
                        <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                        {@html katex.renderToString(row.ex, { throwOnError: false })}
                      </td>
                      <td class="px-2 py-1.5">
                        <button type="button"
                          onclick={() => {
                            mathPopupVal = row.code;
                            mathPopupMode = 'inline';
                            mathPopupVisible = true;
                            showLatexRef = false;
                            setTimeout(() => mathPopupInputEl?.focus());
                          }}
                          title="Düzenleyicide aç"
                          class="rounded border px-1.5 py-0.5 text-[10px] text-muted-foreground hover:bg-muted hover:text-foreground transition-colors">
                          ↗
                        </button>
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>

          <!-- Chemistry table -->
          <div class="space-y-2">
            <p class="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground">Kimyasal Formüller <span class="normal-case font-normal">(\\ce{"{"}...{"}"} söz dizimi)</span></p>
            <div class="overflow-x-auto rounded border">
              <table class="w-full text-xs">
                <thead class="bg-muted/50 text-muted-foreground">
                  <tr>
                    <th class="px-3 py-2 text-left font-medium">Açıklama</th>
                    <th class="px-3 py-2 text-left font-medium">Kod</th>
                    <th class="px-3 py-2 text-left font-medium">Görünüm</th>
                    <th class="px-3 py-2 w-8"></th>
                  </tr>
                </thead>
                <tbody class="divide-y">
                  {#each CHEM_REF as row}
                    <tr class="hover:bg-muted/20">
                      <td class="px-3 py-1.5 text-muted-foreground">{row.cat}</td>
                      <td class="px-3 py-1.5">
                        <code class="rounded bg-muted px-1 py-0.5 text-[11px] font-mono">{row.code}</code>
                      </td>
                      <td class="px-3 py-1.5">
                        <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                        {@html katex.renderToString(row.ex, { throwOnError: false })}
                      </td>
                      <td class="px-2 py-1.5">
                        <button type="button"
                          onclick={() => {
                            mathPopupVal = row.code;
                            mathPopupMode = 'inline';
                            mathPopupVisible = true;
                            showLatexRef = false;
                            setTimeout(() => mathPopupInputEl?.focus());
                          }}
                          title="Düzenleyicide aç"
                          class="rounded border px-1.5 py-0.5 text-[10px] text-muted-foreground hover:bg-muted hover:text-foreground transition-colors">
                          ↗
                        </button>
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>

          <!-- Tips -->
          <div class="rounded border bg-muted/30 px-3 py-2.5 space-y-1">
            <p class="text-xs font-medium">İpuçları</p>
            {#each LATEX_TIPS as tip}
              <p class="text-xs text-muted-foreground">• {tip}</p>
            {/each}
          </div>
        </div>

        <!-- Footer -->
        <div class="border-t px-5 py-2.5 shrink-0 flex justify-end">
          <button type="button" onclick={() => showLatexRef = false}
            class="rounded bg-primary px-4 py-1.5 text-xs text-primary-foreground hover:bg-primary/90 transition-colors">
            Kapat
          </button>
        </div>

      </div>
    </div>
  {/if}

</div>

<style>
  :global(.tiptap-wrap .ProseMirror) {
    outline: none;
    min-height: 6rem;
  }
  :global(.tiptap-wrap .ProseMirror p) {
    margin: 0;
  }
  :global(.editor-img) {
    cursor: default;
    vertical-align: middle;
  }
</style>

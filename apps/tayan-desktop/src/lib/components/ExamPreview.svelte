<script lang="ts">
  import type { Exam, Question } from '$lib/types';
  import { api } from '$lib/api';
  import { onDestroy, untrack } from 'svelte';

  let {
    exam,
    showAnswerKey = $bindable(false),
    onclose,
  }: {
    exam:          Exam;
    showAnswerKey: boolean;
    onclose:       () => void;
  } = $props();

  let blobUrl    = $state<string | null>(null);
  let compiling  = $state(false);
  let error      = $state<string | null>(null);

  async function compile() {
    compiling = true;
    error     = null;

    // revoke previous object URL to avoid memory leak
    if (blobUrl) {
      URL.revokeObjectURL(blobUrl);
      blobUrl = null;
    }

    try {
      const b64   = await api.compiler.exportPdf(exam.id, showAnswerKey);
      const bytes = Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
      const blob  = new Blob([bytes], { type: 'application/pdf' });
      blobUrl = URL.createObjectURL(blob);
    } catch (e) {
      error = String(e);
    } finally {
      compiling = false;
    }
  }

  // Recompile whenever showAnswerKey changes.
  // untrack(compile) prevents blobUrl reads inside compile() from
  // being tracked — otherwise setting blobUrl triggers the effect again.
  $effect(() => {
    showAnswerKey;
    untrack(compile);
  });

  onDestroy(() => {
    if (blobUrl) URL.revokeObjectURL(blobUrl);
  });

  async function downloadPdf() {
    if (!blobUrl) return;
    const a    = document.createElement('a');
    const suffix = showAnswerKey ? '_cevap' : '';
    a.href     = blobUrl;
    a.download = `${exam.meta.title}${suffix}.pdf`;
    a.click();
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="fixed inset-0 z-40 bg-black/50" onclick={onclose}></div>

<div class="fixed inset-0 z-50 flex items-stretch justify-center p-6 pointer-events-none">
  <div class="pointer-events-auto flex flex-col w-full max-w-4xl bg-white dark:bg-zinc-900 rounded-xl shadow-2xl border my-4 overflow-hidden">

    <!-- Toolbar -->
    <div class="flex items-center gap-3 px-5 py-3 border-b bg-muted/30 shrink-0">
      <span class="font-semibold text-sm flex-1 truncate">{exam.meta.title} — PDF Önizleme</span>

      <label class="flex items-center gap-2 text-sm cursor-pointer select-none">
        <input type="checkbox" bind:checked={showAnswerKey} class="rounded" />
        Cevap Anahtarı
      </label>

      <button
        type="button"
        onclick={downloadPdf}
        disabled={!blobUrl || compiling}
        class="inline-flex items-center gap-1.5 rounded-md border px-3 py-1.5 text-xs font-medium hover:bg-accent disabled:opacity-40 transition-colors"
      >
        İndir
      </button>

      <button
        type="button"
        onclick={onclose}
        class="text-muted-foreground hover:text-foreground text-lg leading-none"
      >✕</button>
    </div>

    <!-- Content area -->
    <div class="relative flex-1 min-h-0 bg-muted/20">

      {#if compiling}
        <div class="absolute inset-0 flex flex-col items-center justify-center gap-3 bg-background/80 z-10">
          <span class="inline-block w-8 h-8 border-4 border-primary/30 border-t-primary rounded-full animate-spin"></span>
          <p class="text-sm text-muted-foreground">PDF oluşturuluyor…</p>
        </div>
      {/if}

      {#if error}
        <div class="absolute inset-0 flex items-center justify-center p-8 z-10">
          <div class="rounded-lg border border-destructive/30 bg-destructive/10 px-5 py-4 text-sm text-destructive max-w-lg text-center whitespace-pre-wrap">
            {error}
          </div>
        </div>
      {/if}

      {#if blobUrl}
        <iframe
          src={blobUrl}
          title="PDF Önizleme"
          class="w-full h-full border-0"
        ></iframe>
      {/if}

    </div>

  </div>
</div>

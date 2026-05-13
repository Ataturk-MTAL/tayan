<script lang="ts">
  import { goto } from '$app/navigation';
  import { api } from '$lib/api';
  import type { ExamMeta } from '$lib/types';
  import Button from '$lib/components/ui/Button.svelte';

  let title        = $state('');
  let subject      = $state('');
  let classroom    = $state('');
  let teacher      = $state('');
  let durationMin  = $state(40);
  let date         = $state(new Date().toISOString().slice(0, 10));
  let instructions = $state('');

  let submitting = $state(false);
  let error      = $state<string | null>(null);

  async function submit() {
    error = null;
    if (!title.trim())     { error = 'Sınav adı boş olamaz.'; return; }
    if (!subject.trim())   { error = 'Ders adı boş olamaz.'; return; }
    if (!classroom.trim()) { error = 'Sınıf boş olamaz.'; return; }
    if (!teacher.trim())   { error = 'Öğretmen adı boş olamaz.'; return; }

    submitting = true;
    try {
      const meta: ExamMeta = {
        title:        title.trim(),
        subject:      subject.trim(),
        classroom:    classroom.trim(),
        teacher:      teacher.trim(),
        duration_min: durationMin,
        date,
        instructions: instructions.trim() || null,
      };
      const id = await api.exams.create(meta);
      goto(`/exams/${id}`);
    } catch (e) { error = String(e); }
    finally { submitting = false; }
  }
</script>

<div class="p-6 max-w-2xl mx-auto">
  <div class="flex items-center gap-3 mb-6">
    <a href="/exams" class="text-muted-foreground hover:text-foreground text-sm">← Sınavlar</a>
    <span class="text-muted-foreground">/</span>
    <h1 class="text-xl font-bold">Yeni Sınav</h1>
  </div>

  <form onsubmit={(e) => { e.preventDefault(); submit(); }} class="space-y-4">

    <div class="space-y-1.5">
      <label class="text-sm font-medium" for="title">Sınav Adı</label>
      <input id="title" type="text" bind:value={title} placeholder="örn. 1. Dönem 1. Yazılı"
        class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm
               focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring placeholder:text-muted-foreground" />
    </div>

    <div class="grid grid-cols-2 gap-4">
      <div class="space-y-1.5">
        <label class="text-sm font-medium" for="subject">Ders</label>
        <input id="subject" type="text" bind:value={subject} placeholder="örn. Matematik"
          class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm
                 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring placeholder:text-muted-foreground" />
      </div>
      <div class="space-y-1.5">
        <label class="text-sm font-medium" for="classroom">Sınıf</label>
        <input id="classroom" type="text" bind:value={classroom} placeholder="örn. 7-A"
          class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm
                 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring placeholder:text-muted-foreground" />
      </div>
    </div>

    <div class="grid grid-cols-2 gap-4">
      <div class="space-y-1.5">
        <label class="text-sm font-medium" for="teacher">Öğretmen</label>
        <input id="teacher" type="text" bind:value={teacher} placeholder="Ad Soyad"
          class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm
                 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring placeholder:text-muted-foreground" />
      </div>
      <div class="space-y-1.5">
        <label class="text-sm font-medium" for="duration">Süre (dakika)</label>
        <input id="duration" type="number" min="5" max="300" bind:value={durationMin}
          class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm
                 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" />
      </div>
    </div>

    <div class="space-y-1.5 w-52">
      <label class="text-sm font-medium" for="date">Sınav Tarihi</label>
      <input id="date" type="date" bind:value={date}
        class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm
               focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" />
    </div>

    <div class="space-y-1.5">
      <label class="text-sm font-medium" for="instr">
        Talimatlar <span class="font-normal text-muted-foreground">(isteğe bağlı)</span>
      </label>
      <textarea id="instr" bind:value={instructions} rows="3"
        placeholder="Öğrencilere sınav başında gösterilecek açıklama…"
        class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm resize-none
               focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring placeholder:text-muted-foreground"
      ></textarea>
    </div>

    {#if error}
      <div class="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
        {error}
      </div>
    {/if}

    <div class="flex items-center justify-end gap-3 pt-2">
      <Button variant="ghost" onclick={() => goto('/exams')}>İptal</Button>
      <Button type="submit" disabled={submitting}>
        {submitting ? 'Kaydediliyor…' : 'Oluştur'}
      </Button>
    </div>

  </form>
</div>

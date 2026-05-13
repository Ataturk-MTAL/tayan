<script lang="ts">
  import { api } from '$lib/api';
  import type { Classroom, Student } from '$lib/types';
  import { onMount } from 'svelte';

  let classrooms          = $state<Classroom[]>([]);
  let selectedId          = $state<string | null>(null);
  let students            = $state<Student[]>([]);
  let loading             = $state(true);
  let studentsLoading     = $state(false);
  let error               = $state<string | null>(null);

  // ── Add classroom modal ────────────────────────────────────────────────────
  let showAddClass  = $state(false);
  let newClassName  = $state('');
  let newGrade      = $state(9);
  let newBranch     = $state('A');
  let savingClass   = $state(false);
  let classError    = $state<string | null>(null);

  // ── Add student modal ──────────────────────────────────────────────────────
  let showAddStudent  = $state(false);
  let newNumber       = $state('');
  let newFirst        = $state('');
  let newLast         = $state('');
  let savingStudent   = $state(false);
  let studentError    = $state<string | null>(null);
  let deletingStudent = $state<string | null>(null);
  let deletingClass   = $state<string | null>(null);

  let selectedClassroom = $derived(classrooms.find((c) => c.id === selectedId) ?? null);

  onMount(async () => {
    try {
      classrooms = await api.students.listClassrooms();
      if (classrooms.length > 0) await loadStudents(classrooms[0].id);
    } catch (e) { error = String(e); }
    finally { loading = false; }
  });

  async function loadStudents(classroomId: string) {
    selectedId = classroomId;
    studentsLoading = true;
    try {
      students = await api.students.listByClassroom(classroomId);
    } catch (e) { error = String(e); }
    finally { studentsLoading = false; }
  }

  async function createClassroom() {
    classError = null;
    if (!newClassName.trim()) { classError = 'Sınıf adı boş olamaz.'; return; }
    savingClass = true;
    try {
      const id = await api.students.createClassroom({
        name:   newClassName.trim(),
        grade:  newGrade,
        branch: newBranch.trim() || 'A',
      });
      classrooms = await api.students.listClassrooms();
      showAddClass = false;
      newClassName = ''; newGrade = 9; newBranch = 'A';
      await loadStudents(id);
    } catch (e) { classError = String(e); }
    finally { savingClass = false; }
  }

  async function addStudent() {
    studentError = null;
    if (!selectedId)        { studentError = 'Önce bir sınıf seçin.'; return; }
    if (!newNumber.trim())  { studentError = 'Numara boş olamaz.'; return; }
    if (!newFirst.trim())   { studentError = 'Ad boş olamaz.'; return; }
    if (!newLast.trim())    { studentError = 'Soyad boş olamaz.'; return; }
    savingStudent = true;
    try {
      await api.students.addStudent({
        classroom_id: selectedId,
        number:       newNumber.trim(),
        first_name:   newFirst.trim(),
        last_name:    newLast.trim(),
      });
      students = await api.students.listByClassroom(selectedId);
      showAddStudent = false;
      newNumber = ''; newFirst = ''; newLast = '';
    } catch (e) { studentError = String(e); }
    finally { savingStudent = false; }
  }

  function fmtDate(d: string) {
    return new Date(d).toLocaleDateString('tr-TR', { day: '2-digit', month: 'short', year: 'numeric' });
  }

  async function deleteStudent(s: Student) {
    if (deletingStudent !== s.id) { deletingStudent = s.id; return; }
    try {
      await api.students.deleteStudent(s.id);
      students = students.filter((x) => x.id !== s.id);
      deletingStudent = null;
    } catch (e) { deletingStudent = null; }
  }

  async function deleteClassroom(c: Classroom) {
    if (deletingClass !== c.id) { deletingClass = c.id; return; }
    try {
      await api.students.deleteClassroom(c.id);
      classrooms = classrooms.filter((x) => x.id !== c.id);
      if (selectedId === c.id) { selectedId = null; students = []; }
      deletingClass = null;
    } catch (e) { deletingClass = null; }
  }
</script>

<div class="flex h-full">

  <!-- ── Left panel: Classrooms ────────────────────────────────────────────── -->
  <div class="w-64 shrink-0 border-r flex flex-col">
    <div class="flex items-center justify-between px-4 py-3 border-b">
      <h2 class="font-semibold text-sm">Sınıflar</h2>
      <button
        type="button"
        onclick={() => { showAddClass = true; }}
        title="Yeni sınıf"
        class="flex h-6 w-6 items-center justify-center rounded border text-sm hover:bg-accent transition-colors"
      >+</button>
    </div>

    {#if loading}
      <p class="px-4 py-3 text-sm text-muted-foreground">Yükleniyor…</p>
    {:else if error}
      <p class="px-4 py-3 text-sm text-destructive">{error}</p>
    {:else if classrooms.length === 0}
      <p class="px-4 py-3 text-sm text-muted-foreground">Henüz sınıf yok.</p>
    {:else}
      <ul class="flex-1 overflow-y-auto py-2">
        {#each classrooms as c (c.id)}
          <li>
            <div
              class="group w-full flex items-start transition-colors
                     {selectedId === c.id
                       ? 'bg-primary text-primary-foreground'
                       : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground'}"
            >
              <button
                type="button"
                onclick={() => loadStudents(c.id)}
                class="flex-1 text-left px-4 py-2.5 text-sm"
              >
                <span class="block font-medium">{c.grade}{c.branch} — {c.name}</span>
                <span class="block text-xs opacity-70 mt-0.5">
                  {c.academic_year} · {c.student_ids.length} öğrenci
                </span>
              </button>
              <button
                type="button"
                onclick={() => deleteClassroom(c)}
                title={deletingClass === c.id ? 'Tekrar tıkla — sil' : 'Sınıfı sil'}
                class="px-2 py-2.5 text-xs opacity-0 group-hover:opacity-100 transition-opacity
                       {deletingClass === c.id
                         ? 'opacity-100 text-white bg-destructive rounded'
                         : 'hover:text-destructive'}
                       {selectedId === c.id ? 'opacity-60' : ''}"
              >{deletingClass === c.id ? 'Sil?' : '✕'}</button>
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  </div>

  <!-- ── Right panel: Students ─────────────────────────────────────────────── -->
  <div class="flex-1 flex flex-col min-w-0">
    <div class="flex items-center justify-between px-6 py-3 border-b">
      <h1 class="text-lg font-semibold">
        {#if selectedClassroom}
          {selectedClassroom.grade}{selectedClassroom.branch} — {selectedClassroom.name}
          <span class="ml-2 text-sm font-normal text-muted-foreground">{selectedClassroom.academic_year}</span>
        {:else}
          Öğrenciler
        {/if}
      </h1>
      {#if selectedId}
        <button
          type="button"
          onclick={() => { showAddStudent = true; }}
          class="inline-flex items-center gap-1.5 rounded-md bg-primary px-3 py-1.5 text-sm
                 font-medium text-primary-foreground hover:bg-primary/90 transition-colors"
        >
          + Öğrenci Ekle
        </button>
      {/if}
    </div>

    <div class="flex-1 overflow-y-auto p-6">
      {#if !selectedId}
        <div class="rounded-lg border bg-card p-12 text-center text-muted-foreground text-sm">
          Sol panelden bir sınıf seçin.
        </div>
      {:else if studentsLoading}
        <p class="text-muted-foreground text-sm">Yükleniyor…</p>
      {:else if students.length === 0}
        <div class="rounded-lg border bg-card p-12 text-center text-muted-foreground text-sm">
          Bu sınıfta henüz öğrenci yok.
        </div>
      {:else}
        <div class="rounded-lg border overflow-hidden">
          <table class="w-full text-sm">
            <thead class="bg-muted/50 text-muted-foreground">
              <tr>
                <th class="px-4 py-2.5 text-left font-medium w-20">No</th>
                <th class="px-4 py-2.5 text-left font-medium">Ad Soyad</th>
                <th class="px-4 py-2.5 text-left font-medium">Eklenme</th>
                <th class="px-2 py-2.5 w-10"></th>
              </tr>
            </thead>
            <tbody class="divide-y">
              {#each students.slice().sort((a, b) =>
                a.number.localeCompare(b.number, 'tr', { numeric: true })) as s (s.id)}
                <tr class="hover:bg-muted/30 transition-colors group">
                  <td class="px-4 py-2.5 font-mono text-muted-foreground">{s.number}</td>
                  <td class="px-4 py-2.5 font-medium">{s.first_name} {s.last_name}</td>
                  <td class="px-4 py-2.5 text-muted-foreground">{fmtDate(s.created_at)}</td>
                  <td class="px-2 py-2.5">
                    <button
                      type="button"
                      onclick={() => deleteStudent(s)}
                      title={deletingStudent === s.id ? 'Tekrar tıkla — sil' : 'Öğrenciyi sil'}
                      class="opacity-0 group-hover:opacity-100 transition-opacity px-1.5 py-1 rounded text-xs
                             {deletingStudent === s.id
                               ? 'opacity-100 bg-destructive text-white'
                               : 'text-destructive hover:bg-destructive/10'}"
                    >{deletingStudent === s.id ? 'Sil?' : '✕'}</button>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </div>
  </div>
</div>

<!-- ── Add Classroom Modal ──────────────────────────────────────────────────── -->
{#if showAddClass}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="fixed inset-0 z-40 bg-black/40"
    onclick={() => { showAddClass = false; classError = null; }}></div>
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 pointer-events-none">
    <div class="pointer-events-auto w-full max-w-sm rounded-xl border bg-card shadow-xl">
      <div class="flex items-center justify-between border-b px-5 py-3">
        <h3 class="font-semibold">Yeni Sınıf</h3>
        <button type="button" onclick={() => { showAddClass = false; classError = null; }}
          class="text-muted-foreground hover:text-foreground text-lg leading-none transition-colors">✕</button>
      </div>
      <form onsubmit={(e) => { e.preventDefault(); createClassroom(); }} class="p-5 space-y-4">
        <div class="space-y-1.5">
          <label class="text-sm font-medium" for="cls-name">Sınıf Adı</label>
          <input id="cls-name" type="text" bind:value={newClassName}
            placeholder="örn. Fizik 9A veya 9-A"
            class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm
                   focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring
                   placeholder:text-muted-foreground" />
        </div>
        <div class="grid grid-cols-2 gap-3">
          <div class="space-y-1.5">
            <label class="text-sm font-medium" for="cls-grade">Sınıf Düzeyi</label>
            <input id="cls-grade" type="number" min="1" max="12" bind:value={newGrade}
              class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm
                     focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" />
          </div>
          <div class="space-y-1.5">
            <label class="text-sm font-medium" for="cls-branch">Şube</label>
            <input id="cls-branch" type="text" bind:value={newBranch} maxlength="3" placeholder="A"
              class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm
                     focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring
                     placeholder:text-muted-foreground" />
          </div>
        </div>
        {#if classError}
          <p class="text-sm text-destructive">{classError}</p>
        {/if}
        <div class="flex justify-end gap-2 pt-1">
          <button type="button" onclick={() => { showAddClass = false; classError = null; }}
            class="rounded-md border px-4 py-1.5 text-sm hover:bg-accent transition-colors">
            İptal
          </button>
          <button type="submit" disabled={savingClass}
            class="rounded-md bg-primary px-4 py-1.5 text-sm text-primary-foreground
                   hover:bg-primary/90 transition-colors disabled:opacity-50">
            {savingClass ? 'Kaydediliyor…' : 'Oluştur'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}

<!-- ── Add Student Modal ────────────────────────────────────────────────────── -->
{#if showAddStudent}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="fixed inset-0 z-40 bg-black/40"
    onclick={() => { showAddStudent = false; studentError = null; }}></div>
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 pointer-events-none">
    <div class="pointer-events-auto w-full max-w-sm rounded-xl border bg-card shadow-xl">
      <div class="flex items-center justify-between border-b px-5 py-3">
        <h3 class="font-semibold">
          Öğrenci Ekle
          {#if selectedClassroom}
            <span class="ml-1 text-sm font-normal text-muted-foreground">
              — {selectedClassroom.grade}{selectedClassroom.branch}
            </span>
          {/if}
        </h3>
        <button type="button" onclick={() => { showAddStudent = false; studentError = null; }}
          class="text-muted-foreground hover:text-foreground text-lg leading-none transition-colors">✕</button>
      </div>
      <form onsubmit={(e) => { e.preventDefault(); addStudent(); }} class="p-5 space-y-4">
        <div class="space-y-1.5">
          <label class="text-sm font-medium" for="stu-num">Okul Numarası</label>
          <input id="stu-num" type="text" bind:value={newNumber} placeholder="örn. 1234"
            class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm
                   focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring
                   placeholder:text-muted-foreground" />
        </div>
        <div class="grid grid-cols-2 gap-3">
          <div class="space-y-1.5">
            <label class="text-sm font-medium" for="stu-first">Ad</label>
            <input id="stu-first" type="text" bind:value={newFirst} placeholder="Ahmet"
              class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm
                     focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring
                     placeholder:text-muted-foreground" />
          </div>
          <div class="space-y-1.5">
            <label class="text-sm font-medium" for="stu-last">Soyad</label>
            <input id="stu-last" type="text" bind:value={newLast} placeholder="Yılmaz"
              class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm
                     focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring
                     placeholder:text-muted-foreground" />
          </div>
        </div>
        {#if studentError}
          <p class="text-sm text-destructive">{studentError}</p>
        {/if}
        <div class="flex justify-end gap-2 pt-1">
          <button type="button" onclick={() => { showAddStudent = false; studentError = null; }}
            class="rounded-md border px-4 py-1.5 text-sm hover:bg-accent transition-colors">
            İptal
          </button>
          <button type="submit" disabled={savingStudent}
            class="rounded-md bg-primary px-4 py-1.5 text-sm text-primary-foreground
                   hover:bg-primary/90 transition-colors disabled:opacity-50">
            {savingStudent ? 'Kaydediliyor…' : 'Ekle'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}

<script lang="ts">
  import { onMount } from "svelte";
  import {
    Alert,
    Button,
    Input,
    Label,
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    Textarea,
  } from "flowbite-svelte";
  import PageShell from "$lib/components/shell/PageShell.svelte";
  import { api } from "$lib/api";
  import { errorText } from "$lib/editor/diagnostics";
  import type { Classroom, Student } from "$lib/types";

  let classrooms = $state<Classroom[]>([]);
  let students = $state<Student[]>([]);
  let activeId = $state<string | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let busy = $state(false);

  // Yeni sınıf
  let className = $state("");
  let grade = $state(9);
  let branch = $state("A");

  // Toplu öğrenci girişi — satır başına "numara,ad soyad"
  let bulkText = $state("");

  onMount(loadClassrooms);

  async function loadClassrooms() {
    loading = true;
    try {
      classrooms = await api.students.listClassrooms();
      error = null;
      if (!activeId && classrooms.length > 0) await selectClassroom(classrooms[0].id);
    } catch (err: unknown) {
      error = errorText(err);
    } finally {
      loading = false;
    }
  }

  async function selectClassroom(id: string) {
    activeId = id;
    try {
      students = await api.students.listByClassroom(id);
      error = null;
    } catch (err: unknown) {
      error = errorText(err);
    }
  }

  async function createClassroom() {
    if (className.trim() === "") {
      error = "Sınıf adı boş olamaz.";
      return;
    }
    busy = true;
    try {
      const id = await api.students.createClassroom({
        name: className.trim(),
        grade,
        branch: branch.trim(),
      });
      className = "";
      await loadClassrooms();
      await selectClassroom(id);
      error = null;
    } catch (err: unknown) {
      error = errorText(err);
    } finally {
      busy = false;
    }
  }

  /**
   * Toplu giriş: satır başına "numara,ad soyad". Öğretmen listeyi e-okuldan
   * kopyalayıp yapıştırır; tek tek form doldurmak gerçek bir zaman kaybı.
   */
  type ParsedStudent = { number: string; first: string; last: string };

  function parseBulk(text: string): ParsedStudent[] {
    return text
      .split("\n")
      .map((line) => line.trim())
      .filter(Boolean)
      .map((line) => {
        const [numberPart, ...nameParts] = line.split(/[,;\t]/);
        const fullName = nameParts.join(" ").trim();
        const words = fullName.split(/\s+/).filter(Boolean);
        const last = words.length > 1 ? (words.pop() as string) : "";
        return { number: numberPart.trim(), first: words.join(" "), last };
      })
      .filter((s) => s.number !== "" && s.first !== "");
  }

  let parsed = $derived(parseBulk(bulkText));

  async function addBulk() {
    if (!activeId || parsed.length === 0) return;
    busy = true;
    try {
      for (const s of parsed) {
        await api.students.addStudent({
          classroom_id: activeId,
          number: s.number,
          first_name: s.first,
          last_name: s.last,
        });
      }
      bulkText = "";
      await selectClassroom(activeId);
      error = null;
    } catch (err: unknown) {
      error = errorText(err);
    } finally {
      busy = false;
    }
  }
</script>

<!--
  scroll={false}: üç sütun (sınıflar / öğrenciler / toplu ekle) her biri kendi
  kaydırıcısını taşıyor. Kabuk da kaydırırsa fare tekerleğinin hangi sütunu
  süreceği belirsizleşir — PageShell'in kendi uyarısı da bunu söylüyor.
-->
<PageShell title="Öğrenciler" subtitle={loading ? null : `${classrooms.length} sınıf`} scroll={false}>
  <div class="flex h-full min-h-0 flex-col">
    {#if error}
      <Alert color="red" class="m-4 shrink-0">{error}</Alert>
    {/if}

    <div class="grid min-h-0 flex-1 grid-cols-[240px_1fr_300px]">
      <section class="min-h-0 overflow-auto border-r border-gray-200 dark:border-gray-700">
        <h2
          class="sticky top-0 border-b border-gray-200 bg-gray-50 px-4 py-2 text-xs font-semibold
                 uppercase tracking-wide text-gray-500 dark:border-gray-700 dark:bg-gray-900
                 dark:text-gray-400"
        >
          Sınıflar
        </h2>
        {#if classrooms.length === 0}
          <p class="p-4 text-sm text-gray-500 dark:text-gray-400">Sınıf yok.</p>
        {:else}
          <ul>
            {#each classrooms as c (c.id)}
              <li>
                <button
                  type="button"
                  class="w-full border-b border-gray-200 px-4 py-2 text-left text-sm
                         hover:bg-gray-100 dark:border-gray-700 dark:hover:bg-gray-700/60"
                  class:bg-gray-100={c.id === activeId}
                  class:dark:bg-gray-700={c.id === activeId}
                  class:font-semibold={c.id === activeId}
                  onclick={() => selectClassroom(c.id)}
                >
                  {c.name}
                  <span class="block text-xs text-gray-500 dark:text-gray-400">
                    {c.student_ids.length} öğrenci
                  </span>
                </button>
              </li>
            {/each}
          </ul>
        {/if}

        <div class="border-t border-gray-200 px-4 py-3 dark:border-gray-700">
          <h3 class="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
            Yeni sınıf
          </h3>
          <div class="mt-2">
            <Label for="new-classroom-name" class="mb-1">Ad</Label>
            <Input id="new-classroom-name" type="text" bind:value={className} placeholder="9-A" />
          </div>
          <div class="mt-3 grid grid-cols-2 gap-2">
            <div>
              <Label for="new-classroom-grade" class="mb-1">Seviye</Label>
              <Input id="new-classroom-grade" type="number" min="1" max="12" bind:value={grade} />
            </div>
            <div>
              <Label for="new-classroom-branch" class="mb-1">Şube</Label>
              <Input id="new-classroom-branch" type="text" bind:value={branch} />
            </div>
          </div>
          <div class="mt-3">
            <Button size="sm" color="alternative" disabled={busy} onclick={createClassroom}>
              Oluştur
            </Button>
          </div>
        </div>
      </section>

      <section class="min-h-0 overflow-auto border-r border-gray-200 dark:border-gray-700">
        <h2
          class="sticky top-0 border-b border-gray-200 bg-gray-50 px-4 py-2 text-xs font-semibold
                 uppercase tracking-wide text-gray-500 dark:border-gray-700 dark:bg-gray-900
                 dark:text-gray-400"
        >
          Öğrenci listesi
        </h2>
        {#if !activeId}
          <p class="p-4 text-sm text-gray-500 dark:text-gray-400">Bir sınıf seç.</p>
        {:else if students.length === 0}
          <p class="p-4 text-sm text-gray-500 dark:text-gray-400">Bu sınıfta öğrenci yok.</p>
        {:else}
          <Table>
            <TableBody>
              {#each students as s (s.id)}
                <TableBodyRow>
                  <TableBodyCell class="w-[60px] tnum text-gray-500 dark:text-gray-400">
                    {s.number}
                  </TableBodyCell>
                  <TableBodyCell>{s.first_name} {s.last_name}</TableBodyCell>
                  <TableBodyCell class="text-right">
                    <Button
                      size="xs"
                      color="alternative"
                      disabled={busy}
                      onclick={async () => {
                        await api.students.deleteStudent(s.id);
                        if (activeId) await selectClassroom(activeId);
                      }}
                    >
                      Sil
                    </Button>
                  </TableBodyCell>
                </TableBodyRow>
              {/each}
            </TableBody>
          </Table>
        {/if}
      </section>

      <aside class="min-h-0 overflow-auto px-4 py-3">
        <h2 class="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
          Toplu ekle
        </h2>
        <p class="mt-2 text-sm text-gray-500 dark:text-gray-400">
          Satır başına bir öğrenci: numara, ad soyad. Listeyi olduğu gibi yapıştırabilirsin.
        </p>
        <Textarea
          rows={10}
          class="mt-3 font-mono text-xs"
          bind:value={bulkText}
          placeholder={"101, Ayşe Yılmaz\n102, Mehmet Demir"}
        />
        <div class="mt-3 flex items-center gap-2">
          <Button size="sm" disabled={busy || !activeId || parsed.length === 0} onclick={addBulk}>
            {parsed.length} öğrenci ekle
          </Button>
          {#if bulkText.trim() !== "" && parsed.length === 0}
            <span class="text-xs text-red-600 dark:text-red-500">Hiçbir satır okunamadı.</span>
          {/if}
        </div>
      </aside>
    </div>
  </div>
</PageShell>

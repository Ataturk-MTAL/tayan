<script lang="ts">
  import { onMount } from "svelte";
  import PageHead from "$lib/components/shell/PageHead.svelte";
  import PenButton from "$lib/components/shell/PenButton.svelte";
  import RuledField from "$lib/components/shell/RuledField.svelte";
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

<div class="flex h-full min-h-0 flex-col">
  <PageHead title="Öğrenciler" count={loading ? null : `${classrooms.length} sınıf`} />

  {#if error}
    <p class="ruled-bottom annot shrink-0 bg-red-wash px-rule py-quarter">{error}</p>
  {/if}

  <div class="grid min-h-0 flex-1 grid-cols-[240px_1fr_300px]">
    <section class="min-h-0 overflow-auto border-r border-rule-strong">
      <h2 class="stamp ruled-bottom sticky top-0 bg-paper px-rule py-quarter">Sınıflar</h2>
      {#if classrooms.length === 0}
        <p class="pencil p-rule">Sınıf yok.</p>
      {:else}
        <ul>
          {#each classrooms as c (c.id)}
            <li>
              <button
                type="button"
                class="w-full border-b border-rule px-rule py-half text-left hover:bg-paper-lift"
                class:bg-paper-lift={c.id === activeId}
                class:font-semibold={c.id === activeId}
                onclick={() => selectClassroom(c.id)}
              >
                {c.name}
                <span class="pencil block">{c.student_ids.length} öğrenci</span>
              </button>
            </li>
          {/each}
        </ul>
      {/if}

      <div class="border-t border-rule-strong px-rule py-half">
        <h3 class="stamp">Yeni sınıf</h3>
        <div class="mt-quarter">
          <RuledField label="Ad"><input type="text" bind:value={className} placeholder="9-A" /></RuledField>
        </div>
        <div class="mt-half grid grid-cols-2 gap-half">
          <RuledField label="Seviye"><input type="number" min="1" max="12" bind:value={grade} /></RuledField>
          <RuledField label="Şube"><input type="text" bind:value={branch} /></RuledField>
        </div>
        <div class="mt-half">
          <PenButton kind="quiet" disabled={busy} onclick={createClassroom}>Oluştur</PenButton>
        </div>
      </div>
    </section>

    <section class="min-h-0 overflow-auto border-r border-rule-strong">
      <h2 class="stamp ruled-bottom sticky top-0 bg-paper px-rule py-quarter">Öğrenci listesi</h2>
      {#if !activeId}
        <p class="pencil p-rule">Bir sınıf seç.</p>
      {:else if students.length === 0}
        <p class="pencil p-rule">Bu sınıfta öğrenci yok.</p>
      {:else}
        <table class="w-full border-collapse text-[13px]">
          <tbody>
            {#each students as s (s.id)}
              <tr class="border-b border-rule">
                <td class="w-[60px] px-rule py-quarter tnum text-pencil">{s.number}</td>
                <td class="px-half py-quarter">{s.first_name} {s.last_name}</td>
                <td class="px-rule py-quarter text-right">
                  <PenButton
                    kind="quiet"
                    disabled={busy}
                    onclick={async () => {
                      await api.students.deleteStudent(s.id);
                      if (activeId) await selectClassroom(activeId);
                    }}
                  >
                    Sil
                  </PenButton>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </section>

    <aside class="min-h-0 overflow-auto px-rule py-half">
      <h2 class="stamp">Toplu ekle</h2>
      <p class="pencil mt-quarter">
        Satır başına bir öğrenci: numara, ad soyad. Listeyi olduğu gibi yapıştırabilirsin.
      </p>
      <textarea
        rows="10"
        class="mt-half w-full border border-rule-strong bg-paper-lift p-half font-mono
               text-[12px] leading-rule focus:border-red focus:outline-none"
        bind:value={bulkText}
        placeholder={"101, Ayşe Yılmaz\n102, Mehmet Demir"}
      ></textarea>
      <div class="mt-half flex items-center gap-half">
        <PenButton kind="ink" disabled={busy || !activeId || parsed.length === 0} onclick={addBulk}>
          {parsed.length} öğrenci ekle
        </PenButton>
        {#if bulkText.trim() !== "" && parsed.length === 0}
          <span class="annot">Hiçbir satır okunamadı.</span>
        {/if}
      </div>
    </aside>
  </div>
</div>

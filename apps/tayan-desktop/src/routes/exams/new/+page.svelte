<script lang="ts">
  import PageHead from "$lib/components/shell/PageHead.svelte";
  import PenButton from "$lib/components/shell/PenButton.svelte";
  import RuledField from "$lib/components/shell/RuledField.svelte";
  import { api } from "$lib/api";
  import { errorText } from "$lib/editor/diagnostics";
  import { goto } from "$app/navigation";

  let title = $state("");
  let subject = $state("");
  let classroom = $state("");
  let teacher = $state("");
  let durationMin = $state(40);
  let date = $state(new Date().toISOString().slice(0, 10));
  let instructions = $state("");

  let saving = $state(false);
  let saveError = $state<string | null>(null);

  async function create() {
    if (title.trim() === "") {
      saveError = "Sınav başlığı boş olamaz.";
      return;
    }

    saving = true;
    saveError = null;
    try {
      const id = await api.exams.create({
        title: title.trim(),
        subject: subject.trim(),
        classroom: classroom.trim(),
        teacher: teacher.trim(),
        duration_min: durationMin,
        date,
        instructions: instructions.trim() || null,
      });
      await goto(`/exams/${id}`);
    } catch (err: unknown) {
      saveError = errorText(err);
    } finally {
      saving = false;
    }
  }
</script>

<div class="flex h-full min-h-0 flex-col">
  <PageHead title="Yeni sınav">
    <PenButton kind="ink" disabled={saving} onclick={create}>
      {saving ? "Oluşturuluyor…" : "Oluştur"}
    </PenButton>
  </PageHead>

  <div class="min-h-0 flex-1 overflow-auto">
    <div class="mx-auto max-w-[620px] px-rule py-rule">
      {#if saveError}
        <p class="annot mb-rule bg-red-wash px-half py-quarter">{saveError}</p>
      {/if}

      <div class="grid grid-cols-2 gap-x-rule gap-y-rule">
        <div class="col-span-2">
          <RuledField label="Başlık">
            <input type="text" bind:value={title} placeholder="1. Dönem 2. Yazılı" />
          </RuledField>
        </div>

        <RuledField label="Ders">
          <input type="text" bind:value={subject} placeholder="Matematik" />
        </RuledField>

        <RuledField label="Sınıf">
          <input type="text" bind:value={classroom} placeholder="9-A" />
        </RuledField>

        <RuledField label="Öğretmen">
          <input type="text" bind:value={teacher} />
        </RuledField>

        <RuledField label="Tarih">
          <input type="date" bind:value={date} />
        </RuledField>

        <RuledField label="Süre" hint="dakika">
          <input type="number" min="1" bind:value={durationMin} />
        </RuledField>

        <div class="col-span-2">
          <RuledField label="Talimatlar" hint="Kâğıdın başında öğrenciye görünür">
            <textarea rows="3" bind:value={instructions}></textarea>
          </RuledField>
        </div>
      </div>
    </div>
  </div>
</div>

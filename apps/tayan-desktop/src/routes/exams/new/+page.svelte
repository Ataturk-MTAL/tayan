<script lang="ts">
  import PageShell from "$lib/components/shell/PageShell.svelte";
  import { Alert, Button, Heading, Input, Label, Select, Textarea } from "flowbite-svelte";
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

  /**
   * Kâğıdın sütun sayısı. Çift sütun kısa sorularda kâğıt tasarrufu sağlar;
   * uzun gövdeli veya geniş görselli soruda okunaksızdır. Karar öğretmenin.
   */
  let columns = $state(1);
  const COLUMN_OPTIONS = [
    { value: "1", name: "Tek sütun" },
    { value: "2", name: "Çift sütun" },
  ];
  let school = $state("");
  let department = $state("");

  /**
   * İmza bloğu. Boş bırakılırsa kâğıda hiç basılmaz.
   *
   * Değişiklikler YERİNDE yapılmaz: her ekleme/silme/düzenleme yeni bir dizi
   * üretir. Yerinde değiştirmek, aynı diziyi tutan başka bir yerin sessizce
   * etkilenmesi demektir.
   */
  let signers = $state<Array<{ name: string; title: string }>>([]);

  function addSigner() {
    signers = [...signers, { name: "", title: "" }];
  }

  function removeSigner(index: number) {
    signers = signers.filter((_, i) => i !== index);
  }

  function updateSigner(index: number, field: "name" | "title", value: string) {
    signers = signers.map((s, i) => (i === index ? { ...s, [field]: value } : s));
  }

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
        columns,
        school: school.trim() || null,
        department: department.trim() || null,
        // Adı da unvanı da boş olan satırlar gönderilmez: yarım doldurulmuş bir
        // imza satırı kâğıtta boş bir çizgi olarak basılırdı.
        signers: signers
          .map((s) => ({ name: s.name.trim(), title: s.title.trim() }))
          .filter((s) => s.name !== "" || s.title !== ""),
      });
      await goto(`/exams/${id}`);
    } catch (err: unknown) {
      saveError = errorText(err);
    } finally {
      saving = false;
    }
  }
</script>

<PageShell title="Yeni sınav">
  {#snippet actions()}
    <Button size="sm" disabled={saving} onclick={create}>
      {saving ? "Oluşturuluyor…" : "Oluştur"}
    </Button>
  {/snippet}

  <div class="mx-auto max-w-[620px]">
    {#if saveError}
      <Alert color="red" class="mb-5">{saveError}</Alert>
    {/if}

    <div class="grid grid-cols-2 gap-x-5 gap-y-5">
      <div class="col-span-2">
        <Label for="title" class="mb-1">Başlık</Label>
        <Input id="title" type="text" bind:value={title} placeholder="1. Dönem 2. Yazılı" />
      </div>

      <div>
        <Label for="subject" class="mb-1">Ders</Label>
        <Input id="subject" type="text" bind:value={subject} placeholder="Matematik" />
      </div>

      <div>
        <Label for="classroom" class="mb-1">Sınıf</Label>
        <Input id="classroom" type="text" bind:value={classroom} placeholder="9-A" />
      </div>

      <div>
        <Label for="teacher" class="mb-1">Öğretmen</Label>
        <Input id="teacher" type="text" bind:value={teacher} />
      </div>

      <div>
        <Label for="date" class="mb-1">Tarih</Label>
        <Input id="date" type="date" bind:value={date} />
      </div>

      <div>
        <Label for="duration" class="mb-1">
          Süre <span class="font-normal text-gray-500 dark:text-gray-400">(dakika)</span>
        </Label>
        <Input id="duration" type="number" min="1" bind:value={durationMin} />
      </div>

      <div class="col-span-2">
        <Label for="instructions" class="mb-1">
          Talimatlar
          <span class="font-normal text-gray-500 dark:text-gray-400"
            >— Kâğıdın başında öğrenciye görünür</span
          >
        </Label>
        <Textarea id="instructions" rows={3} bind:value={instructions} />
      </div>

      <!--
        Kâğıdın biçimi. Bunlar sorunun değil BASKININ özellikleri: aynı soru
        bir sınavda tek sütun, başkasında çift sütun basılabilir.
      -->
      <div class="col-span-2 border-t border-gray-200 pt-2.5 dark:border-gray-700">
        <Heading
          tag="h2"
          class="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400"
        >
          Kâğıt biçimi
        </Heading>
      </div>

      <div>
        <Label class="mb-1">
          Sütun
          <span class="font-normal text-gray-500 dark:text-gray-400"
            >— Çift sütun kısa sorularda kâğıt kazandırır</span
          >
        </Label>
        <Select
          items={COLUMN_OPTIONS}
          value={String(columns)}
          placeholder=""
          onchange={(e) => (columns = Number((e.currentTarget as HTMLSelectElement).value))}
        />
      </div>

      <div></div>

      <div>
        <Label for="school" class="mb-1">
          Okul <span class="font-normal text-gray-500 dark:text-gray-400">— Boşsa kâğıda basılmaz</span>
        </Label>
        <Input
          id="school"
          type="text"
          bind:value={school}
          placeholder="Atatürk Mesleki ve Teknik Anadolu Lisesi"
        />
      </div>

      <div>
        <Label for="department" class="mb-1">
          Alan / Bölüm
          <span class="font-normal text-gray-500 dark:text-gray-400">— Boşsa kâğıda basılmaz</span>
        </Label>
        <Input
          id="department"
          type="text"
          bind:value={department}
          placeholder="Elektrik-Elektronik Teknolojisi Alanı"
        />
      </div>

      <div class="col-span-2">
        <div class="flex items-center gap-2.5">
          <span class="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
            İmzalar
          </span>
          <span class="text-sm text-gray-500 dark:text-gray-400">
            Boşsa kâğıdın altına imza bloğu basılmaz
          </span>
          <Button size="xs" color="alternative" class="ml-auto" onclick={addSigner}>
            + İmza ekle
          </Button>
        </div>

        {#each signers as signer, i (i)}
          <div class="mt-2.5 flex items-end gap-2.5">
            <div class="flex-1">
              <Label class="mb-1">Ad Soyad</Label>
              <Input
                type="text"
                value={signer.name}
                placeholder="Hakan GÜLEN"
                oninput={(e) => updateSigner(i, "name", (e.currentTarget as HTMLInputElement).value)}
              />
            </div>
            <div class="flex-1">
              <Label class="mb-1">Unvan</Label>
              <Input
                type="text"
                value={signer.title}
                placeholder="Ders Öğretmeni"
                oninput={(e) => updateSigner(i, "title", (e.currentTarget as HTMLInputElement).value)}
              />
            </div>
            <Button
              size="sm"
              color="alternative"
              aria-label="{i + 1}. imzayı sil"
              onclick={() => removeSigner(i)}
            >
              Sil
            </Button>
          </div>
        {/each}
      </div>
    </div>
  </div>
</PageShell>

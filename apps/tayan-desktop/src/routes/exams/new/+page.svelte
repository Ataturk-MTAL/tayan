<script lang="ts">
  import PageHead from "$lib/components/shell/PageHead.svelte";
  import PenButton from "$lib/components/shell/PenButton.svelte";
  import RuledField from "$lib/components/shell/RuledField.svelte";
  import SelectBox from "$lib/components/shell/SelectBox.svelte";
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

        <!--
          Kâğıdın biçimi. Bunlar sorunun değil BASKININ özellikleri: aynı soru
          bir sınavda tek sütun, başkasında çift sütun basılabilir.
        -->
        <div class="col-span-2 border-t border-rule pt-half">
          <h2 class="stamp">Kâğıt biçimi</h2>
        </div>

        <RuledField label="Sütun" hint="Çift sütun kısa sorularda kâğıt kazandırır">
          <SelectBox
            value={String(columns)}
            options={[
              { value: "1", label: "Tek sütun" },
              { value: "2", label: "Çift sütun" },
            ]}
            onchange={(v) => (columns = Number(v))}
          />
        </RuledField>

        <div></div>

        <RuledField label="Okul" hint="Boşsa kâğıda basılmaz">
          <input
            type="text"
            bind:value={school}
            placeholder="Atatürk Mesleki ve Teknik Anadolu Lisesi"
          />
        </RuledField>

        <RuledField label="Alan / Bölüm" hint="Boşsa kâğıda basılmaz">
          <input
            type="text"
            bind:value={department}
            placeholder="Elektrik-Elektronik Teknolojisi Alanı"
          />
        </RuledField>

        <div class="col-span-2">
          <div class="flex items-center gap-half">
            <span class="stamp">İmzalar</span>
            <span class="pencil">Boşsa kâğıdın altına imza bloğu basılmaz</span>
            <button
              type="button"
              class="ml-auto border border-rule-strong bg-paper-lift px-half py-quarter
                     text-[12px] leading-rule text-ink transition-colors
                     hover:border-red hover:text-red-deep"
              onclick={addSigner}
            >
              + İmza ekle
            </button>
          </div>

          {#each signers as signer, i (i)}
            <div class="mt-half flex items-end gap-half">
              <div class="flex-1">
                <RuledField label="Ad Soyad">
                  <input
                    type="text"
                    value={signer.name}
                    placeholder="Hakan GÜLEN"
                    oninput={(e) => updateSigner(i, "name", e.currentTarget.value)}
                  />
                </RuledField>
              </div>
              <div class="flex-1">
                <RuledField label="Unvan">
                  <input
                    type="text"
                    value={signer.title}
                    placeholder="Ders Öğretmeni"
                    oninput={(e) => updateSigner(i, "title", e.currentTarget.value)}
                  />
                </RuledField>
              </div>
              <button
                type="button"
                class="border border-rule-strong bg-paper-lift px-half py-quarter
                       text-[12px] leading-rule text-pencil transition-colors
                       hover:border-red hover:text-red-deep"
                aria-label="{i + 1}. imzayı sil"
                onclick={() => removeSigner(i)}
              >
                Sil
              </button>
            </div>
          {/each}
        </div>
      </div>
    </div>
  </div>
</div>

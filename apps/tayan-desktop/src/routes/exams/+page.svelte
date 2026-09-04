<script lang="ts">
  import { onMount } from "svelte";
  import {
    Alert,
    Badge,
    Button,
    Spinner,
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    TableHead,
    TableHeadCell,
  } from "flowbite-svelte";
  import PageShell from "$lib/components/shell/PageShell.svelte";
  import { api } from "$lib/api";
  import { errorText } from "$lib/editor/diagnostics";
  import { EXAM_STATUS_LABELS, type Exam } from "$lib/types";
  import { goto } from "$app/navigation";

  let exams = $state<Exam[]>([]);
  let loading = $state(true);
  let loadError = $state<string | null>(null);

  onMount(load);

  async function load() {
    loading = true;
    try {
      exams = await api.exams.list();
      loadError = null;
    } catch (err: unknown) {
      loadError = errorText(err);
    } finally {
      loading = false;
    }
  }
</script>

<PageShell title="Sınavlar" subtitle={loading ? null : `${exams.length} sınav`}>
  {#snippet actions()}
    <Button size="sm" onclick={() => goto("/exams/new")}>Yeni sınav</Button>
  {/snippet}

  {#if loading}
    <div class="flex items-center gap-2 text-gray-500 dark:text-gray-400">
      <Spinner size="5" />
      Sınavlar okunuyor…
    </div>
  {:else if loadError}
    <Alert color="red">{loadError}</Alert>
  {:else if exams.length === 0}
    <p class="text-gray-500 dark:text-gray-400">Henüz sınav yok.</p>
  {:else}
    <!--
      Bu üç metin hücresine sarma izni verildi. flowbite'ın tableBodyCell
      base'i her hücreye `px-6 py-4 whitespace-nowrap` koyuyor; 6 sütun ×
      48px = 288px salt dolgu, minWidth=1024'te kullanılabilir genişlik
      ise 1024 − 225 (çekmece) − 48 (PageShell p-6) = 751px. Gerçekçi bir
      başlık — "2024–2025 Güz Dönemi 1. Ortak Yazılı Sınavı" ≈ 315px —
      tek başına kalanın üçte ikisini yiyor; nowrap yüzünden satır
      kabından geniş kalıyor, overflow-x-auto yatay kaydırıcı açıyor ve
      en sağdaki "Durum" sütunu ekranın dışında kalıyordu. Oysa rozet,
      sınavın hâlâ düzenlenebilir olup olmadığını söyleyen tek işaret.
      whitespace-normal ile boşluklu bir başlığın min-content'i "2024–2025"
      ≈ 70px'e düşüyor, tablo sığıyor, hiçbir Türkçe metin kısaltılmıyor.
      (tailwind-variants twMerge ile birleştirdiği için whitespace-normal
      base'deki whitespace-nowrap'i eziyor.) Sayısal hücreler ve rozet
      hücresi nowrap kalıyor: "40 dk" bölünmemeli.
      Üç metin hücresinde de ayrıca `wrap-anywhere` var, `break-words` DEĞİL:
      sınav adı, ders adı ve sınıf üçü de ham öğretmen metni, üçü de boşluksuz
      tek parça olabilir (yapıştırılmış bir ad, tire içermeyen uzun bir dizge,
      "Mikrodenetleyici" gibi tek kelimelik bir ders). min-content hesabında
      overflow-wrap: break-word kelimeyi bölünmemiş sayar — ItemAnalysis.svelte'te
      ölçüldü, o tabloda break-words ile genişlik 502px'te kalmıştı — yani böyle
      bir metinde whitespace-normal tek başına min-content'i düşürmüyor ve
      nowrap'teki başarısızlık aynen geri geliyordu: yatay kaydırıcı, "Durum"
      sütunu ekran dışında. overflow-wrap: anywhere min-content'i tek karaktere
      indirir, boşluklu metinleri ise yine kelime sınırından kırar.
    -->
    <Table hoverable>
      <TableHead>
        <TableHeadCell>Sınav</TableHeadCell>
        <TableHeadCell>Ders</TableHeadCell>
        <TableHeadCell>Sınıf</TableHeadCell>
        <TableHeadCell class="text-right">Soru</TableHeadCell>
        <TableHeadCell class="text-right">Süre</TableHeadCell>
        <TableHeadCell class="text-right">Durum</TableHeadCell>
      </TableHead>
      <TableBody>
        {#each exams as exam (exam.id)}
          <TableBodyRow class="cursor-pointer" onclick={() => goto(`/exams/${exam.id}`)}>
            <TableBodyCell class="whitespace-normal wrap-anywhere font-semibold">
              {exam.meta.title}
            </TableBodyCell>
            <TableBodyCell class="whitespace-normal wrap-anywhere">{exam.meta.subject}</TableBodyCell>
            <TableBodyCell class="whitespace-normal wrap-anywhere">{exam.meta.classroom}</TableBodyCell>
            <TableBodyCell class="text-right tnum">{exam.questions.length}</TableBodyCell>
            <TableBodyCell class="text-right tnum">{exam.meta.duration_min} dk</TableBodyCell>
            <TableBodyCell class="text-right">
              <!-- Yayına çıkmış sınav artık düzenlenemez; kırmızı rozet bu tekliği hatırlatıyor. -->
              <Badge color={exam.status === "Published" ? "red" : "gray"}>
                {EXAM_STATUS_LABELS[exam.status]}
              </Badge>
            </TableBodyCell>
          </TableBodyRow>
        {/each}
      </TableBody>
    </Table>
  {/if}
</PageShell>

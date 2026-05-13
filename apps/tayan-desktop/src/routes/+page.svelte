<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';

  let examCount     = $state<number | null>(null);
  let questionCount = $state<number | null>(null);
  let studentCount  = $state<number | null>(null);

  onMount(async () => {
    const [exams, questions, classrooms] = await Promise.all([
      api.exams.list().catch(() => []),
      api.questions.list().catch(() => []),
      api.students.listClassrooms().catch(() => []),
    ]);
    examCount     = exams.length;
    questionCount = questions.length;
    studentCount  = classrooms.reduce((s, c) => s + (c.student_ids?.length ?? 0), 0);
  });
</script>

<div class="p-8">
  <h1 class="text-3xl font-bold tracking-tight mb-2">TAYAN</h1>
  <p class="text-muted-foreground">Sınav Analiz ve Oluşturma Platformu</p>

  <!-- Stats row -->
  <div class="mt-6 grid grid-cols-3 gap-4">
    {#each [
      { label: 'Sınav',    value: examCount     },
      { label: 'Soru',     value: questionCount },
      { label: 'Öğrenci',  value: studentCount  },
    ] as stat}
      <div class="rounded-lg border bg-card px-5 py-4">
        <div class="text-2xl font-bold">
          {stat.value === null ? '—' : stat.value}
        </div>
        <div class="text-sm text-muted-foreground mt-0.5">{stat.label}</div>
      </div>
    {/each}
  </div>

  <div class="mt-6 grid grid-cols-1 md:grid-cols-3 gap-4">
    <a href="/exams/new" class="block rounded-lg border bg-card p-6 hover:shadow-md transition-shadow">
      <h2 class="font-semibold text-lg mb-1">Yeni Sınav</h2>
      <p class="text-sm text-muted-foreground">Soru bankasından sorular seçerek sınav oluştur</p>
    </a>
    <a href="/questions" class="block rounded-lg border bg-card p-6 hover:shadow-md transition-shadow">
      <h2 class="font-semibold text-lg mb-1">Soru Bankası</h2>
      <p class="text-sm text-muted-foreground">Soru ekle, düzenle ve kazanımlarla ilişkilendir</p>
    </a>
    <a href="/analysis" class="block rounded-lg border bg-card p-6 hover:shadow-md transition-shadow">
      <h2 class="font-semibold text-lg mb-1">Sınav Analizi</h2>
      <p class="text-sm text-muted-foreground">Sınıf ve bireysel başarı raporları</p>
    </a>
  </div>
</div>

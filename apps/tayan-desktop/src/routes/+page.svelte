<script lang="ts">
  // Dashboard — entry point
</script>

<div class="p-8">
  <h1 class="text-3xl font-bold tracking-tight mb-2">TAYAN</h1>
  <p class="text-muted-foreground">Sınav Analiz ve Oluşturma Platformu</p>

  <div class="mt-8 grid grid-cols-1 md:grid-cols-3 gap-4">
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

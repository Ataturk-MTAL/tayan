<script lang="ts">
  /**
   * Tema seçici: açık · koyu · sistem.
   *
   * ÜÇ SEÇENEK, İKİ DEĞİL. Yalnız aç/kapa bir anahtar olsaydı "sistemim ne
   * diyorsa" diyen öğretmen için yer kalmazdı; sistem gündüz açığa dönünce
   * uygulamanın da dönmesi ayrı bir istektir ve o anda koyu olmasıyla aynı
   * şey değildir.
   */
  import { setTheme, theme, type ThemeChoice } from "$lib/ui/theme.svelte";
  import { DesktopPcOutline, MoonOutline, SunOutline } from "flowbite-svelte-icons";

  const SECENEKLER: { id: ThemeChoice; ad: string; icon: typeof SunOutline }[] = [
    { id: "light", ad: "Açık tema", icon: SunOutline },
    { id: "dark", ad: "Koyu tema", icon: MoonOutline },
    { id: "system", ad: "Sistem teması", icon: DesktopPcOutline },
  ];
</script>

<div
  class="flex items-center gap-1 rounded-lg bg-gray-100 p-1 dark:bg-gray-700/60"
  role="group"
  aria-label="Tema"
>
  {#each SECENEKLER as s (s.id)}
    <button
      type="button"
      class="flex flex-1 items-center justify-center rounded-md py-1.5 transition-colors"
      class:bg-white={theme.choice === s.id}
      class:shadow-sm={theme.choice === s.id}
      class:dark:bg-gray-800={theme.choice === s.id}
      class:text-primary-700={theme.choice === s.id}
      class:dark:text-primary-300={theme.choice === s.id}
      class:text-gray-500={theme.choice !== s.id}
      class:dark:text-gray-400={theme.choice !== s.id}
      class:hover:text-gray-900={theme.choice !== s.id}
      class:dark:hover:text-white={theme.choice !== s.id}
      title={s.ad}
      aria-label={s.ad}
      aria-pressed={theme.choice === s.id}
      onclick={() => setTheme(s.id)}
    >
      <s.icon class="h-4 w-4" />
    </button>
  {/each}
</div>

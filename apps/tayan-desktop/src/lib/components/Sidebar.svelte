<script lang="ts">
  import { page } from '$app/stores';

  const nav = [
    { href: '/',          label: 'Panel',        icon: '⊞' },
    { href: '/exams',     label: 'Sınavlar',     icon: '📋' },
    { href: '/questions', label: 'Soru Bankası', icon: '❓' },
    { href: '/students',  label: 'Öğrenciler',   icon: '👥' },
    { href: '/analysis',  label: 'Analiz',       icon: '📊' },
  ];
</script>

<nav class="w-56 shrink-0 border-r bg-card flex flex-col h-full">
  <div class="px-4 py-5 border-b">
    <span class="font-bold text-xl tracking-wide">TAYAN</span>
  </div>
  <ul class="flex-1 py-3 space-y-0.5 px-2">
    {#each nav as item}
      {@const active = $page.url.pathname === item.href ||
                       ($page.url.pathname.startsWith(item.href) && item.href !== '/')}
      <li>
        <a
          href={item.href}
          class="flex items-center gap-2.5 rounded-md px-3 py-2 text-sm transition-colors
                 {active ? 'bg-primary text-primary-foreground font-medium'
                         : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground'}"
        >
          <span>{item.icon}</span>
          {item.label}
        </a>
      </li>
    {/each}
  </ul>
</nav>

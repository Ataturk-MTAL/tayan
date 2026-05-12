<script lang="ts">
  import '../app.css';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import { page } from '$app/stores';
</script>

<div class="flex h-screen overflow-hidden bg-background">
  <Sidebar />
  <main class="flex-1 overflow-y-auto">
    <slot />
  </main>
</div>

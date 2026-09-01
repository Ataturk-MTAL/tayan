<script lang="ts">
  type Props = {
    title: string;
    count?: string | null;
    children?: import("svelte").Snippet;
  };
  let { title, count = null, children }: Props = $props();
</script>

<div class="ruled-bottom flex shrink-0 items-baseline gap-rule bg-paper-lift px-rule py-half paper-plain">
  <h1>{title}</h1>
  {#if count}
    <span class="pencil tnum">{count}</span>
  {/if}
  <div class="ml-auto flex items-center gap-half">
    {@render children?.()}
  </div>
</div>

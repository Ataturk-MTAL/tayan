<script lang="ts">
  type Props = {
    label: string;
    hint?: string | null;
    children: import("svelte").Snippet;
  };
  let { label, hint = null, children }: Props = $props();
</script>

<label class="block">
  <span class="stamp block">{label}</span>
  {@render children()}
  {#if hint}
    <span class="pencil block">{hint}</span>
  {/if}
</label>

<style>
  label :global(input),
  label :global(select),
  label :global(textarea) {
    display: block;
    width: 100%;
    background: transparent;
    border: 0;
    border-bottom: 1px solid var(--color-rule-strong);
    padding: 0 0 3px;
    line-height: 20px;
    font-variant-numeric: tabular-nums;
  }
  label :global(input:focus),
  label :global(select:focus),
  label :global(textarea:focus) {
    outline: none;
    border-bottom-color: var(--color-red);
  }
</style>

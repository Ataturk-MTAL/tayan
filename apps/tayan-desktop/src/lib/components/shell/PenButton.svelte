<script lang="ts">
  type Props = {
    kind?: "ink" | "red" | "quiet";
    type?: "button" | "submit";
    disabled?: boolean;
    onclick?: () => void;
    children: import("svelte").Snippet;
  };
  let { kind = "ink", type = "button", disabled = false, onclick, children }: Props = $props();
</script>

<!-- Kart değil, cetvelli düğme. Yuvarlatma yok; kalemle çizilmiş bir kutu. -->
<button
  {type}
  {disabled}
  {onclick}
  class="border px-half py-quarter text-[13px] leading-rule transition-colors
         disabled:cursor-not-allowed disabled:opacity-40"
  class:border-ink={kind === "ink"}
  class:bg-ink={kind === "ink"}
  class:text-paper-lift={kind === "ink"}
  class:border-red={kind === "red"}
  class:text-red-deep={kind === "red"}
  class:bg-paper-lift={kind !== "ink"}
  class:border-rule-strong={kind === "quiet"}
  class:text-ink={kind === "quiet"}
>
  {@render children()}
</button>

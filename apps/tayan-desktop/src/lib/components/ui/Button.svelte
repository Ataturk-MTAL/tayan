<script lang="ts">
  import type { Snippet } from 'svelte';

  type Variant = 'primary' | 'secondary' | 'destructive' | 'ghost' | 'outline';
  type Size    = 'sm' | 'md' | 'lg';

  let {
    variant  = 'primary',
    size     = 'md',
    disabled = false,
    type     = 'button',
    class:   extraClass = '',
    onclick,
    children,
  }: {
    variant?:   Variant;
    size?:      Size;
    disabled?:  boolean;
    type?:      'button' | 'submit' | 'reset';
    class?:     string;
    onclick?:   () => void;
    children?:  Snippet;
  } = $props();

  const VARIANT_STYLES: Record<Variant, string> = {
    primary:     'bg-primary text-primary-foreground hover:bg-primary/90',
    secondary:   'bg-secondary text-secondary-foreground hover:bg-secondary/80',
    destructive: 'bg-destructive text-white hover:bg-destructive/90',
    ghost:       'hover:bg-accent hover:text-accent-foreground',
    outline:     'border border-input bg-background hover:bg-accent hover:text-accent-foreground',
  };

  const SIZE_STYLES: Record<Size, string> = {
    sm: 'h-8 px-3 text-xs',
    md: 'h-9 px-4 text-sm',
    lg: 'h-10 px-6 text-base',
  };
</script>

<button
  {type}
  {disabled}
  onclick={onclick}
  class="inline-flex items-center justify-center gap-2 rounded-md font-medium
         ring-offset-background transition-colors focus-visible:outline-none
         focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2
         disabled:pointer-events-none disabled:opacity-50
         {VARIANT_STYLES[variant]} {SIZE_STYLES[size]} {extraClass}"
>
  {@render children?.()}
</button>

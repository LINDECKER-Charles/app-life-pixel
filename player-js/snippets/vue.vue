<!--
  With Vite, tell Vue that <life-pixel> is a custom element, in vite.config.ts:
  vue({ template: { compilerOptions: { isCustomElement: (tag) => tag === 'life-pixel' } } })
-->
<script setup>
import '@life-pixel/player';
</script>

<template>
  <life-pixel src="{{src}}"{{tagAttribute}} alt="{{alt}}"></life-pixel>
</template>

<template>
  <div class="markdown-content" v-html="renderMarkdown(source)"></div>
</template>

<script setup>
import { renderMarkdown } from '@/utils/markdown.js'

defineProps({ source: { type: String, default: '' } })
</script>

<style scoped>
.markdown-content { font-size: 13px; line-height: 1.7; color: rgba(var(--text-color), .86); overflow-wrap: anywhere; }
.markdown-content :deep(h1), .markdown-content :deep(h2), .markdown-content :deep(h3), .markdown-content :deep(h4), .markdown-content :deep(h5), .markdown-content :deep(h6) { margin: 18px 0 8px; color: rgb(var(--text-color)); line-height: 1.4; }
.markdown-content :deep(h1) { font-size: 20px; }
.markdown-content :deep(h2) { font-size: 18px; }
.markdown-content :deep(h3) { font-size: 15px; }
.markdown-content :deep(h4), .markdown-content :deep(h5), .markdown-content :deep(h6) { font-size: 13px; }
.markdown-content :deep(img) { max-width: 100%; height: auto; }
.markdown-content :deep(> :first-child) { margin-top: 0; }
.markdown-content :deep(> :last-child) { margin-bottom: 0; }
.markdown-content :deep(p) { margin: 0 0 10px; }
.markdown-content :deep(ul), .markdown-content :deep(ol) { margin: 8px 0 12px; padding-left: 22px; }
.markdown-content :deep(li) { margin: 3px 0; }
.markdown-content :deep(code) { padding: 1px 5px; border-radius: 4px; background: rgba(var(--outline-color), .1); font-size: 12px; }
.markdown-content :deep(pre) { padding: 12px; border-radius: 8px; background: rgba(var(--outline-color), .08); overflow-x: auto; }
.markdown-content :deep(pre code) { padding: 0; background: transparent; }
.markdown-content :deep(blockquote) { margin: 12px 0; padding-left: 12px; border-left: 3px solid rgba(var(--primary-color), .45); color: rgba(var(--text-color), .65); }
.markdown-content :deep(a) { color: rgb(var(--primary-color)); text-decoration: underline; }
.markdown-content :deep(.markdown-table) { overflow-x: auto; margin: 12px 0; }
.markdown-content :deep(table) { border-collapse: collapse; width: 100%; font-size: 12px; }
.markdown-content :deep(th), .markdown-content :deep(td) { padding: 8px 10px; border: 1px solid rgba(var(--outline-color), .16); text-align: left; }
.markdown-content :deep(th) { background: rgba(var(--outline-color), .07); font-weight: 600; }
</style>

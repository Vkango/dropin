import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { test } from 'node:test'
import { renderMarkdown } from './markdown.js'

test('Energy Wiki renders localized range tables, emphasis and formula', () => {
  for (const locale of ['en-US', 'zh-CN', 'zh-CLASSICAL']) {
    const { wiki } = JSON.parse(readFileSync(new URL(`../../plugin-sdk/examples/tag-provider/i18n/${locale}.json`, import.meta.url), 'utf8'))
    const html = renderMarkdown(wiki)
    assert.match(html, /<table>/)
    assert.match(html, /<strong>Energy<\/strong>/)
    assert.match(html, /<code>min\(RMS \/ 0\.5, 1\) × 100<\/code>/)
    assert.equal((html.match(/<tr>/g) || []).length, 6)
    assert.match(html, /0–&lt;10/)
    assert.doesNotMatch(html, /<td>---<\/td>/)
  }
})

test('Wiki accepts standard lists, quotes, fenced code and links', () => {
  const html = renderMarkdown('1. First\n2. Second\n\n> A quote\n\n```js\nconst a = "<b>"\n```\n\n[Docs](https://example.com/docs)')
  assert.match(html, /<ol>/)
  assert.match(html, /<blockquote>/)
  assert.match(html, /<pre><code class="language-js">/)
  assert.match(html, /&lt;b&gt;/)
  assert.match(html, /target="_blank" rel="noopener noreferrer"/)
})

test('Wiki cannot inject HTML, scripts or executable links', () => {
  const html = renderMarkdown('<script>alert(1)</script>\n\n<img src=x onerror=alert(1)>\n\n[click](javascript:alert(1))\n\n![bad](data:text/html;base64,PHNjcmlwdD4=)')
  assert.doesNotMatch(html, /<script|<img|href="javascript:/)
  assert.match(html, /&lt;script&gt;/)
})

test('Markdown punctuation inside code stays literal', () => {
  assert.match(renderMarkdown('`**literal**`'), /<code>\*\*literal\*\*<\/code>/)
  assert.equal(renderMarkdown(null), '')
})

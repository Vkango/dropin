import MarkdownIt from 'markdown-it'

// Wiki text comes from plugins and user input. Keep raw HTML disabled while
// supporting standard Markdown (including tables, links and fenced code).
const markdown = new MarkdownIt({ html: false })
const renderLink = markdown.renderer.rules.link_open
  || ((tokens, index, options, env, renderer) => renderer.renderToken(tokens, index, options))

markdown.renderer.rules.link_open = (tokens, index, options, env, renderer) => {
  tokens[index].attrSet('target', '_blank')
  tokens[index].attrSet('rel', 'noopener noreferrer')
  return renderLink(tokens, index, options, env, renderer)
}

markdown.renderer.rules.table_open = () => '<div class="markdown-table"><table>\n'
markdown.renderer.rules.table_close = () => '</table></div>\n'

export const renderMarkdown = (source) => markdown.render(String(source ?? ''))

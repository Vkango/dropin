const escapeMap = {
  '&': '&amp;',
  '<': '&lt;',
  '>': '&gt;',
  '"': '&quot;',
  "'": '&#39;'
}

const escapeHtml = (value) =>
  String(value ?? '').replace(/[&<>"']/g, (char) => escapeMap[char])

export const sanitize = (html) => escapeHtml(html)

export default sanitize

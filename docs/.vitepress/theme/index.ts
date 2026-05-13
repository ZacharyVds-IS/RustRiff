import DefaultTheme from 'vitepress/theme'
import {EnhanceAppContext} from 'vitepress'
import './custom.css'
import 'katex/dist/katex.min.css'

export default {
  ...DefaultTheme,
  enhanceApp({ router }: EnhanceAppContext) {
    if (typeof window !== 'undefined') {
      const base = import.meta.env.BASE_URL || '/'
      const normalizedBase = base.endsWith('/') ? base : `${base}/`

      // Intercept feature card link clicks that target static .html files
      // and force a full browser navigation instead of a Vue Router push
      window.addEventListener('click', (e) => {
        const anchor = (e.target as HTMLElement).closest('a')
        if (!anchor) return
        const href = anchor.getAttribute('href')
        if (href && href.endsWith('.html') && href.startsWith('/')) {
          e.preventDefault()
          e.stopImmediatePropagation()

          const targetPath = href.startsWith(normalizedBase)
            ? href
            : `${normalizedBase}${href.replace(/^\/+/, '')}`

          // Replace current history entry with docs base so back returns home.
          history.replaceState(null, '', normalizedBase)
          window.location.href = targetPath
        }
      }, true) // capture phase so we run before the router
    }
  }
}


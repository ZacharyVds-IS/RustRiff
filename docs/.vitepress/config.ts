import {defineConfig} from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  base: '/Guitar-Amplifier/',
  title: "RustRiff documentation",
  description: "Documentation combining both frontend and backend docs for RustRiff",
  themeConfig: {
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Guide', link: '/guide/overview' },
      { text: 'Frontend Docs', link: '/frontend/index.html' },
      { text: 'Backend Docs', link: '/backend/doc/rustriff_lib/index.html' }
    ],

    sidebar: [
      {
        text: 'Project Docs',
        items: [
          { text: 'Overview', link: '/guide/overview' },
          { text: 'Project Structure', link: '/guide/project-structure' },
        ]
      },
      {
        text:'Amp Design',
        items:[
          {text:'Relation to real amp', link:'/guide/amp_design/structure'},
          {text:'Resampling', link:'/guide/amp_design/resampling.md'},
          {text:'Latency', link:'/guide/amp_design/latency'},
          {text:'Gain', link:'/guide/amp_design/gain'},
          {text:'Master Volume', link:'/guide/amp_design/master-volume'},
          {text:'Tone Stack',link:'/guide/amp_design/tone-stack'}
        ]
      },
      {
        text:'Persistency',
        items:[
          {text:'Storing data', link:'/guide/persistency/storing-data'}
        ]
      },
      {
        text:'Effects',
        items:[
          {text:'Effect chain', link:'/guide/effects/chain'},
        ]
      },
      {
        text: 'API References',
        items: [
          { text: 'Frontend API (TypeDoc)', link: '/frontend/index.html' },
          { text: 'Backend API (Rustdoc)', link: '/backend/doc/rustriff_lib/index.html' }
        ]
      },
      {
        text: 'Arc42 Descriptions',
        items:[
          {text: 'Why choose Rust?', link:'/arc42/programming-language-choice'}
        ]
      }
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/ZacharyVds-IS/Guitar-Amplifier' }
    ]
  }
})

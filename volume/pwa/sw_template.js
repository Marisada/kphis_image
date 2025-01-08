const CACHE_NAME = `DOM-AXUM-${VERSION}`
let browser

const start = [
  'manifest.webmanifest',
  'favicon.ico',
  'client.js',
  'client_bg.wasm',
  'app.css',
  'app.js'
]

self.addEventListener('install', event => {
  console.info(`installing service worker "${CACHE_NAME}"`)

  const requests = start.map(url => new Request(url, {cache: 'reload'}))
  event.waitUntil(
    caches.open(CACHE_NAME).then(cache => {
      return cache.addAll(requests)
    })
  )
})

self.addEventListener('activate', event => {
  console.info(`activating service worker "${CACHE_NAME}"`)
  const activate = async () => {
    await clients.claim()
    caches.keys().then(keyList => {
      return Promise.all(
        keyList.map(key => {
          if (key !== CACHE_NAME) {
            return caches.delete(key)
          }
        })
      )
    })
  }
  event.waitUntil(activate())
})

self.addEventListener('fetch', event => {
  const {headers} = event.request
  if (headers.get('Accept') === 'text/event-stream') {
    return
  }
  const response = caches.match(event.request)
    .then(response => response || lazyCache(event.request))
  event.respondWith(response)
})

self.addEventListener('message', event => {
  browser = event.source
  if (event.data.id === 'skipWaiting') {
    skipWaiting()
    browser.postMessage("reload")
  } else if (event.data.id === 'version') {
    console.info(`Service worker "${CACHE_NAME}" registered`)
    console.info(`App version "${event.data.value}" activated`)
  }
})

async function lazyCache(request) {
  const isLazy = request.url.split('/').includes('assets')
  const response = await fetch(request)
  const clone = response.clone()
  if (isLazy && response.ok) {
      caches.open(CACHE_NAME).then(cache => {
        cache.put(request, clone)
      })
  }
  return response
}
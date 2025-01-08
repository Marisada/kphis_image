const VERSION = '660731-1515'

import init from './client.js'
init('./client_bg.wasm').catch(console.error)

if ('serviceWorker' in navigator) {
    const checkUpdate = document.getElementById('checkUpdate')
    checkUpdate.style.display = 'none'

    const updateBanner = document.getElementById('updateBanner')
    const bannerContent = document.querySelector('#updateBanner .banner-content')
    const updateAvailable = sw => {
        updateBanner.dataset.state = 'updateavailable'
        document.querySelector('#updateBanner .banner-headline').innerHTML = 'Update Available'
        document.querySelector('#updateBanner .banner-subhead').innerHTML = 'Click here to update the app to the latest version'
        updateBanner.style.height = bannerContent.offsetHeight + 15 + 'px'
        updateBanner.onclick = () => {sw.postMessage({id:'skipWaiting'})}
    }
    updateBanner.addEventListener('click', () => {updateBanner.style.height = '0'})

    navigator.serviceWorker.register('/sw.js').then(reg => {
        reg.onupdatefound = () => {
            if (reg.active === null) return
            const installingWorker = reg.installing
            installingWorker.onstatechange = () => {
                if (installingWorker.state === 'installed') {
                if (navigator.serviceWorker.controller !== null) updateAvailable(reg.waiting)
                }
            }
        }
        checkUpdate.onclick = () => {
            updateBanner.style.height = bannerContent.offsetHeight + 15 + 'px'
            reg.update()
        }
        if (reg.waiting !== null) {
            updateAvailable(reg.waiting)
        }
        checkUpdate.style.display = 'block'
    }) 
    .catch(err => console.error('Service Worker Registration : ' + err))

    navigator.serviceWorker.ready.then(async reg => {
        if (reg.active !== null) {
            reg.active.postMessage({id:'version', value: VERSION})
        }
    }, 
    err => console.error('Service Worker Ready : ' + err))
    
    navigator.serviceWorker.addEventListener('message', e => {
        if (e.data === "reload"){
          window.location.reload()
        }
      })
}
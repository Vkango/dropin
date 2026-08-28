import { createDropinPluginClient } from '/sdk/dropin-sdk.js'

const dropin = createDropinPluginClient()
const status = document.querySelector('#status')
document.querySelector('#save').addEventListener('click', async () => {
  await dropin.storage.set('greeting', 'Hello from Dropin')
  status.textContent = await dropin.storage.get('greeting')
  await dropin.notification.show({
    title: 'Hello Plugin',
    body: status.textContent
  })
})

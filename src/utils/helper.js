import { t } from '../i18n/index.js'

export const getTimeInterval = (timestamp) => {
  const diff = Date.now() - Number(timestamp || Date.now())
  const seconds = Math.max(0, Math.floor(diff / 1000))
  if (seconds < 60) return t('notification.timeAgoNow')
  const minutes = Math.floor(seconds / 60)
  if (minutes < 60) return t('notification.timeAgoMin', { n: minutes })
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return t('notification.timeAgoHour', { n: hours })
  const days = Math.floor(hours / 24)
  return t('notification.timeAgoDay', { n: days })
}

export default { getTimeInterval }

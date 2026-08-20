import { nextTick, onActivated, onBeforeUnmount, onDeactivated, onMounted, ref, watch } from 'vue'
import { ALL_INITIAL } from './alphabet.js'

const STICKY_TOP_OFFSET = 72

export function useAlphabetNavigation(pageRef, availableInitials) {
  const activeInitial = ref(ALL_INITIAL)
  const alphabetTopOffset = ref(0)
  let scrollContainer = null
  let frameId = null
  let mutationObserver = null
  let pendingInitial = null
  let pendingTimeout = null

  const getGroupElements = () => {
    if (!pageRef.value?.isConnected) return []
    return [...pageRef.value.querySelectorAll('[data-group-initial]')]
  }

  const updateAlphabetTopOffset = () => {
    const pageElement = pageRef.value
    const firstGroup = getGroupElements()[0]
    if (!pageElement || !firstGroup) {
      alphabetTopOffset.value = 0
      return
    }

    alphabetTopOffset.value = Math.max(0,
      Math.round(firstGroup.getBoundingClientRect().top - pageElement.getBoundingClientRect().top)
    )
  }

  const updateActiveInitial = () => {
    frameId = null
    updateAlphabetTopOffset()
    if (pendingInitial !== null) {
      activeInitial.value = pendingInitial
      return
    }

    const groups = getGroupElements()
    if (!groups.length) {
      activeInitial.value = ALL_INITIAL
      return
    }

    const scrollTop = scrollContainer?.getBoundingClientRect().top || 0
    const threshold = scrollTop + STICKY_TOP_OFFSET
    const isAtBottom = scrollContainer
      && scrollContainer.scrollTop + scrollContainer.clientHeight >= scrollContainer.scrollHeight - 2
    const currentGroup = isAtBottom
      ? groups.at(-1)
      : groups.filter((group) => group.getBoundingClientRect().top <= threshold).at(-1)

    activeInitial.value = currentGroup?.dataset.groupInitial || ALL_INITIAL
  }

  const scheduleActiveUpdate = () => {
    if (frameId !== null) return
    frameId = requestAnimationFrame(updateActiveInitial)
  }

  const unbind = () => {
    scrollContainer?.removeEventListener('scroll', scheduleActiveUpdate)
    scrollContainer?.removeEventListener('scrollend', completePendingScroll)
    window.removeEventListener('resize', scheduleActiveUpdate)
    mutationObserver?.disconnect()
    scrollContainer = null
    mutationObserver = null
    if (frameId !== null) cancelAnimationFrame(frameId)
    frameId = null
    if (pendingTimeout !== null) clearTimeout(pendingTimeout)
    pendingTimeout = null
    pendingInitial = null
  }

  const completePendingScroll = () => {
    pendingInitial = null
    if (pendingTimeout !== null) clearTimeout(pendingTimeout)
    pendingTimeout = null
    scheduleActiveUpdate()
  }

  const bind = () => {
    unbind()
    scrollContainer = pageRef.value?.closest('.page-layout-scroll') || null
    scrollContainer?.addEventListener('scroll', scheduleActiveUpdate, { passive: true })
    scrollContainer?.addEventListener('scrollend', completePendingScroll, { passive: true })
    window.addEventListener('resize', scheduleActiveUpdate, { passive: true })
    if (pageRef.value) {
      mutationObserver = new MutationObserver(scheduleActiveUpdate)
      mutationObserver.observe(pageRef.value, { childList: true, subtree: true })
    }
    scheduleActiveUpdate()
  }

  const scrollToInitial = (initial) => {
    activeInitial.value = initial
    pendingInitial = initial || null
    nextTick(() => {
      if (!pageRef.value?.isConnected) return

      if (!initial) {
        pageRef.value.scrollIntoView({ behavior: 'smooth', block: 'start' })
        return
      }

      const target = getGroupElements()
        .find((element) => element.dataset.groupInitial === initial)
      target?.scrollIntoView({ behavior: 'smooth', block: 'start' })
      if (pendingTimeout !== null) clearTimeout(pendingTimeout)
      pendingTimeout = window.setTimeout(completePendingScroll, 1600)
      scheduleActiveUpdate()
    })
  }

  const handleGroupLabelClick = (initial) => {
    activeInitial.value = initial
    scheduleActiveUpdate()
  }

  const validateActiveInitial = (initials) => {
    if (activeInitial.value && !initials.includes(activeInitial.value)) {
      activeInitial.value = ALL_INITIAL
    }
    nextTick(bind)
  }

  onMounted(() => nextTick(bind))
  onActivated(() => nextTick(bind))
  onDeactivated(unbind)
  onBeforeUnmount(unbind)

  watch(availableInitials, validateActiveInitial)

  return {
    activeInitial,
    alphabetTopOffset,
    handleAlphabetSelect: scrollToInitial,
    handleGroupLabelClick
  }
}

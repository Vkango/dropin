<template>
  <div class="notification-container">
    <transition-group name="fade-notify" tag="div" class="notification-list">
      <div v-for="item in displayedNotifications" :key="item.id" :data-id="item.id" class="notification"
        @mouseenter="pauseTimer(item.id)" @mouseleave="startTimer(item.id, item.duration)"
        @mousedown="startDrag($event, item.id)" @mouseup="handleMouseUp($event, item.id)">
        <div class="notification-card">
          <div class="notification-icon">
            <img src="/assets/sys_music.svg" alt="" />
          </div>
          <div class="notification-body">
            <div class="notification-head">
              <span class="notification-title">{{ item.title }}</span>
              <span class="notification-source" v-if="item.source">{{ item.source }}</span>
            </div>
            <div class="notification-message">
              <component :is="item.component" v-bind="item.props"></component>
            </div>
          </div>
          <button class="notification-close" @mousedown.stop @click.stop="close(item.id)" aria-label="close">
            <img src="/assets/close.svg" alt="close" />
          </button>
        </div>
      </div>
    </transition-group>
  </div>
</template>

<script setup lang="ts">
import { ref, onUnmounted, markRaw, nextTick, computed, type Component } from 'vue';

const notifications = ref<NotificationItem[]>([]);
const timers = ref<Record<string, ReturnType<typeof setTimeout>>>({});
const hiddenNotifications = ref<NotificationItem[]>([]);
const _dragging = ref(false);
const dragStartX = ref(0);
const currentDragId = ref<string | null>(null);
const isClosing = ref(false);
const _dragStartTime = ref(0);
const isDragging = ref(false);
const dragThreshold = 5;
let deltaX = 0;
let notificationCounter = 0;

const displayedNotifications = computed(() =>
  notifications.value.filter(item => item && item.visible && !item.hidden)
);

function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

const handleMouseUp = (event: MouseEvent, id: string): void => {
  if ((event.target as HTMLElement).closest('.notification-close')) {
    return;
  }
  const dragDistance = Math.abs(deltaX);
  if (!isDragging.value && dragDistance < dragThreshold) {
    const notification = notifications.value.find(item => item.id === id);
    if (notification && notification.click) {
      notification.click();
    }
  }
  endDrag();
};

const addNotification = async (
  title: string,
  source: string,
  component: Component,
  clickHandler: (() => void) | null,
  props: Record<string, unknown> = {},
  duration = 5000
): Promise<string> => {
  while (isClosing.value) {
    await delay(100);
  }
  const id = `${Date.now()}-${notificationCounter++}`;
  const notification = {
    id,
    source,
    title,
    component: markRaw(component),
    props,
    duration,
    visible: false,
    hidden: false,
    timestamp: Date.now(),
    click: clickHandler || undefined
  };

  notifications.value.unshift(notification);
  notification.visible = true;

  if (duration > 0) {
    startTimer(id, duration);
  }
  return id;
};

const updateNotification = (
  id: string,
  patch: {
    title?: string;
    source?: string;
    component?: Component;
    props?: Record<string, unknown>;
    duration?: number;
  }
): void => {
  const notification = notifications.value.find(item => item.id === id);
  if (!notification) return;
  if (patch.title !== undefined) notification.title = patch.title;
  if (patch.source !== undefined) notification.source = patch.source;
  if (patch.component !== undefined) notification.component = markRaw(patch.component);
  if (patch.props !== undefined) notification.props = patch.props;
  if (patch.duration !== undefined) {
    notification.duration = patch.duration;
    if (patch.duration > 0) {
      startTimer(id, patch.duration);
    } else if (timers.value[id]) {
      clearTimeout(timers.value[id]);
      delete timers.value[id];
    }
  }
};

const hideNotification = async (id: string): Promise<void> => {
  const index = notifications.value.findIndex(item => item.id === id);
  if (index === -1) return;

  isClosing.value = true;
  notifications.value[index].visible = false;
  await delay(300);
  const notification: NotificationItem = { ...notifications.value[index] };
  notification.hidden = true;
  hiddenNotifications.value.push(notification);
  notifications.value.splice(index, 1);
  isClosing.value = false;
};

const deleteNotification = async (id: string): Promise<void> => {
  const visibleIndex = notifications.value.findIndex(item => item.id === id);
  if (visibleIndex !== -1) {
    notifications.value.splice(visibleIndex, 1);
  }
  const hiddenIndex = hiddenNotifications.value.findIndex(item => item.id === id);
  if (hiddenIndex !== -1) {
    hiddenNotifications.value.splice(hiddenIndex, 1);
  }
};


const restoreNotification = (id: string): void => {
  const index = hiddenNotifications.value.findIndex(item => item.id === id);
  if (index === -1) return;

  const notification: NotificationItem = { ...hiddenNotifications.value[index] };
  notification.hidden = false;
  notification.visible = true;
  hiddenNotifications.value.splice(index, 1);
  notifications.value.unshift(notification);
  nextTick(() => {
    const notificationElement = document.querySelector(`.notification[data-id="${id}"]`);
    if (notificationElement) {
      _dragging.value = false;
      dragStartX.value = 0;
      currentDragId.value = null;
      deltaX = 0;
    }
  });
};

const close = async (id: string): Promise<void> => {
  const index = notifications.value.findIndex(item => item.id === id);
  if (index === -1) return;

  isClosing.value = true;
  notifications.value[index].visible = false;
  await delay(300);
  notifications.value.splice(index, 1);
  isClosing.value = false;
};

const startTimer = (id: string, duration: number): void => {
  if (timers.value[id]) {
    clearTimeout(timers.value[id]);
  }
  if (duration > 0) {
    timers.value[id] = setTimeout(() => {
      hideNotification(id);
      delete timers.value[id];
    }, duration);
  }
};

const pauseTimer = (id: string): void => {
  if (timers.value[id]) {
    clearTimeout(timers.value[id]);
    delete timers.value[id];
  }
};

const startDrag = (event: MouseEvent, id: string): void => {
  if ((event.target as HTMLElement).closest('.notification-close')) {
    return;
  }

  _dragStartTime.value = Date.now();
  dragStartX.value = event.clientX;
  currentDragId.value = id;
  isDragging.value = false;
  deltaX = 0;

  document.addEventListener('mousemove', onDrag);
  document.addEventListener('mouseup', endDrag);
};

const onDrag = (event: MouseEvent): void => {
  if (!currentDragId.value) return;

  const dragDistance = Math.abs(event.clientX - dragStartX.value);
  if (dragDistance > dragThreshold) {
    isDragging.value = true;
    deltaX = event.clientX - dragStartX.value;
    const notificationElement = document.querySelector(`.notification[data-id="${currentDragId.value}"]`) as HTMLElement | null;
    if (notificationElement) {
      notificationElement.style.transition = 'none';
      notificationElement.style.transform = `translateX(${deltaX}px)`;
      notificationElement.style.opacity = `${1 - Math.abs(deltaX / 100)}`;
    }
  }
};

const endDrag = (): void => {
  if (currentDragId.value === null) return;
  const notificationElement = document.querySelector(`.notification[data-id="${currentDragId.value}"]`) as HTMLElement | null;
  if (notificationElement) {
    if (Math.abs(deltaX) > 100) {
      close(currentDragId.value);
    } else {
      notificationElement.style.transition = 'transform 0.3s ease, opacity 0.3s ease';
      notificationElement.style.transform = `translateX(0px)`;
      notificationElement.style.opacity = '1';
    }
  }

  _dragging.value = false;
  currentDragId.value = null;
  document.removeEventListener('mousemove', onDrag);
  document.removeEventListener('mouseup', endDrag);
};

onUnmounted(() => {
  for (const id in timers.value) {
    if (timers.value[id]) clearTimeout(timers.value[id]);
  }
});

defineExpose({
  addNotification,
  updateNotification,
  hideNotification,
  deleteNotification,
  restoreNotification,
  notifications,
  hiddenNotifications
});
</script>
<style scoped>
.notification-list {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.notification {
  position: relative;
  transition: transform 0.3s ease, opacity 0.3s ease;
}

.notification-card {
  position: relative;
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 12px 14px;
  border-radius: 14px;
  background: rgba(var(--background-color), 0.72);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(var(--outline-color), 0.18);
  box-shadow:
    0 8px 24px rgba(0, 0, 0, 0.18),
    0 1px 2px rgba(0, 0, 0, 0.12);
  overflow: hidden;
}

.notification-icon {
  flex: 0 0 auto;
  width: 34px;
  height: 34px;
  display: grid;
  place-items: center;
  border-radius: 10px;
  background: rgba(var(--primary-color), 0.16);
}

.notification-icon img {
  width: 18px;
  height: 18px;
  filter: invert(var(--invert, 0));
  opacity: 0.9;
}

.notification-body {
  flex: 1 1 auto;
  min-width: 0;
  padding-top: 1px;
}

.notification-head {
  display: flex;
  align-items: baseline;
  gap: 8px;
}

.notification-title {
  font-size: 13.5px;
  font-weight: 600;
  color: rgb(var(--text-color));
  letter-spacing: 0.1px;
  white-space: nowrap;
}

.notification-source {
  font-size: 11px;
  color: rgba(var(--text-color), 0.45);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.notification-message {
  margin-top: 4px;
  font-size: 12.5px;
  line-height: 1.5;
  color: rgba(var(--text-color), 0.65);
}

.notification-close {
  flex: 0 0 auto;
  width: 24px;
  height: 24px;
  display: grid;
  place-items: center;
  margin: -2px -4px 0 0;
  border: none;
  border-radius: 8px;
  background: transparent;
  cursor: pointer;
  opacity: 0.4;
  transition: opacity 0.2s ease, background-color 0.2s ease;
}

.notification-close:hover {
  opacity: 1;
  background: rgba(var(--text-color), 0.08);
}

.notification-close img {
  width: 13px;
  height: 13px;
  filter: invert(var(--invert, 0));
}

/* Transition animations */
.fade-notify-move,
.fade-notify-enter-active,
.fade-notify-leave-active {
  transition: all 0.3s cubic-bezier(0.22, 1, 0.36, 1);
}

.fade-notify-enter-from,
.fade-notify-leave-to {
  opacity: 0;
  transform: translateX(24px) scale(0.98);
}

.fade-notify-leave-active {
  position: absolute;
  width: 100%;
}
</style>

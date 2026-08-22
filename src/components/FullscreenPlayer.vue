<template>
    <div class="fullscreen-player" :style="lyricsStyle" @click.self="$emit('close')">
        <div class="player-background" :class="`background-mode-${backgroundMode}`" aria-hidden="true">
            <FlowingBackground v-if="backgroundMode === 'flowing'" :cover="currentSong.cover" :bands="audioBands" />
            <div v-else class="backdrop-image" :style="{ backgroundImage: `url(${currentSong.cover})` }"></div>
            <div class="backdrop-wash"></div>
            <div class="backdrop-vignette"></div>
        </div>

        <div class="player-container" :class="{ 'is-narrow': isNarrow }" @click.stop>
            <!-- 宽屏：左封面+歌曲信息 / 右歌词（或播放列表） -->
            <main v-if="!isNarrow" class="player-main">
                <section class="visual-column" :aria-label="t('generic.songInfo')">
                    <div ref="albumStageRef" class="album-stage">
                        <div ref="albumVisualRef" class="album-visual" :class="albumShapeClass"
                            :style="albumVisualStyle">
                            <div class="disc-shell" :class="albumShapeClass">
                                <MotionTransition variant="albumCover" mode="out-in">
                                    <div :key="currentSong.cover" class="album-cover-frame">
                                        <img :src="currentSong.cover" :alt="currentSong.title" class="album-cover"
                                            :class="albumShapeClass" />
                                    </div>
                                </MotionTransition>
                            </div>
                        </div>
                    </div>

                    <MotionTransition variant="songInfo" mode="out-in">
                        <div :key="currentSong.title" class="song-details">
                            <h1 class="song-title">{{ currentSong.title }}</h1>
                            <h2 class="song-artist">
                                <User2Icon style="scale: 0.7;" />{{ currentSong.artist }}
                            </h2>
                            <p class="song-album">
                                <DiscAlbum style="scale: 0.7;" />{{ currentSong.album }}
                            </p>

                            <div class="song-tags" aria-live="polite">
                                <AnimatePresence :initial="false">
                                    <MotionSpan v-for="tag in songTags" :key="tag.key" class="tag" :initial="tagInitial"
                                        :animate="tagAnimate" :exit="tagExit" :transition="tagTransition">
                                        {{ tag.label }}
                                    </MotionSpan>
                                </AnimatePresence>
                            </div>
                        </div>
                    </MotionTransition>
                </section>

                <section class="lyrics-column" :aria-label="t('generic.lyricsAndPlaylist')">
                    <AnimatePresence mode="wait" :initial="false">
                        <MotionDiv v-if="!isPlaylistOpen" key="lyrics-view" class="lyrics-view"
                            :initial="playlistViewInitial" :animate="playlistViewAnimate" :exit="playlistViewExit"
                            :transition="playlistTransition" @animation-complete="handleLyricsViewAnimationComplete">
                            <div ref="lyricsWindowRef" class="lyrics-window" tabindex="0"
                                :aria-label="t('player.lyrics')" @wheel="handleLyricsWheel"
                                @pointerdown="handleLyricsPointerDown" @scroll="handleLyricsScroll">
                                <MotionDiv v-if="lyricRows.length" class="lyrics-track"
                                    :aria-label="t('player.lyrics')">
                                    <MotionDiv v-for="(row, index) in lyricRows" :ref="setLyricRowRef(row.key)"
                                        :key="row.key" class="lyric-line"
                                        :class="{ 'lyric-interlude-row': row.type === 'interlude' }" role="button"
                                        tabindex="0" :aria-current="activeLyricRowIndex === index ? 'true' : undefined"
                                        :aria-label="row.type === 'interlude' ? t('player.interlude') : undefined"
                                        :animate="getLyricState(index)"
                                        :transition="row.type === 'interlude' ? instantTransition : contentTransition"
                                        @pointerdown.stop @click.stop="handleLyricClick(row)"
                                        @keydown.stop="handleLyricKeydown($event, row)">
                                        <template v-if="row.type === 'interlude'">
                                            <MoreHorizontal v-if="activeLyricRowIndex === index"
                                                class="lyric-interlude-icon" :size="32" :stroke-width="2.5"
                                                fill="currentColor" aria-hidden="true" />
                                        </template>
                                        <template v-else>
                                            <div class="lyric-primary">{{ row.line.text }}</div>
                                            <template v-if="showSecondaryLyrics">
                                                <div v-for="secondary in row.line.secondary" :key="secondary"
                                                    class="lyric-secondary">
                                                    {{ secondary }}
                                                </div>
                                            </template>
                                        </template>
                                    </MotionDiv>
                                </MotionDiv>
                                <MotionDiv v-else-if="lyricsLoading" class="lyrics-status" :initial="{ opacity: 0 }"
                                    :animate="{ opacity: 1 }" :transition="microTransition">
                                    {{ t('player.loadingLyrics') }}
                                </MotionDiv>
                                <div v-else-if="plainLyrics.length" class="plain-lyrics">
                                    <div v-for="line in plainLyrics" :key="line" class="plain-lyric-line">
                                        {{ line }}
                                    </div>
                                </div>
                                <MotionDiv v-else class="lyrics-status" :initial="{ opacity: 0 }"
                                    :animate="{ opacity: 1 }" :transition="microTransition">
                                    {{ t('player.noSyncedLyrics') }}
                                </MotionDiv>
                            </div>
                        </MotionDiv>

                        <MotionDiv v-else key="playlist-view" class="fullscreen-playlist-view"
                            :initial="playlistViewInitial" :animate="playlistViewAnimate" :exit="playlistViewExit"
                            :transition="playlistTransition">
                            <div class="fullscreen-playlist-scroll">
                                <div class="fullscreen-playlist-header">
                                    <MotionButton class="fullscreen-playlist-back" :while-hover="buttonHover"
                                        :while-press="buttonPress" :transition="microTransition"
                                        :aria-label="t('player.backToLyrics')" @click="closePlaylist">
                                        <ArrowLeft :size="18" :stroke-width="1.8" />
                                    </MotionButton>
                                    <strong>{{ t('player.queue') }}</strong>
                                </div>
                                <Playlist :songs="queueSongs" :current-song="currentSong" :is-playing="isPlaying"
                                    :show-header="false" :title="t('player.queue')"
                                    @song-select="$emit('playlist-song-select', $event)" />
                            </div>
                        </MotionDiv>
                    </AnimatePresence>
                </section>
            </main>

            <!-- 窄屏双页：上=动区（专辑页 / 歌词页 / 播放列表页），下=固定播放控制区 -->
            <div v-else class="narrow-body">
                <AnimatePresence mode="popLayout" :initial="false">
                    <MotionDiv v-if="!isPlaylistOpen && narrowPage === 'album'" key="narrow-album"
                        class="narrow-page narrow-album-page" :style="lyricsStyle" :initial="narrowInitial"
                        :animate="narrowAnimate" :exit="narrowExit" :transition="pageTransition">
                        <div ref="albumStageRef" class="album-stage">
                            <div ref="albumVisualRef" class="album-visual" :class="albumShapeClass"
                                :style="albumVisualStyle">
                                <div class="disc-shell" :class="albumShapeClass">
                                    <MotionTransition variant="albumCover" mode="out-in">
                                        <div :key="currentSong.cover" class="album-cover-frame">
                                            <img :src="currentSong.cover" :alt="currentSong.title" class="album-cover"
                                                :class="albumShapeClass" />
                                        </div>
                                    </MotionTransition>
                                </div>
                            </div>
                        </div>

                        <!-- 歌曲信息大按钮：悬停亮起、圆角，点击滑动到歌词页 -->
                        <MotionButton class="song-details song-info-nav" :while-hover="songInfoHover"
                            :while-press="buttonPress" :transition="microTransition"
                            :aria-label="t('player.showLyrics')" @click="goLyricsPage">
                            <MotionTransition variant="songInfo" mode="out-in">
                                <div :key="currentSong.title" class="song-details-inner">
                                    <h1 class="song-title">{{ currentSong.title }}</h1>
                                    <h2 class="song-artist">
                                        <User2Icon style="scale: 0.7;" />{{ currentSong.artist }}
                                    </h2>
                                    <p class="song-album">
                                        <DiscAlbum style="scale: 0.7;" />{{ currentSong.album }}
                                    </p>

                                    <div class="song-tags" aria-live="polite">
                                        <AnimatePresence :initial="false">
                                            <MotionSpan v-for="tag in songTags" :key="tag.key" class="tag"
                                                :initial="tagInitial" :animate="tagAnimate" :exit="tagExit"
                                                :transition="tagTransition">
                                                {{ tag.label }}
                                            </MotionSpan>
                                        </AnimatePresence>
                                    </div>
                                </div>
                            </MotionTransition>
                        </MotionButton>
                    </MotionDiv>

                    <MotionDiv v-else-if="!isPlaylistOpen" key="narrow-lyrics" class="narrow-page narrow-lyrics-page"
                        :style="lyricsStyle" :initial="narrowInitial" :animate="narrowAnimate" :exit="narrowExit"
                        :transition="pageTransition" @animation-complete="handleLyricsViewAnimationComplete">
                        <!-- 歌词页顶部标题/副标题大按钮：点击滑动回专辑页 -->
                        <MotionButton class="lyrics-heading nav" :while-hover="songInfoHover" :while-press="buttonPress"
                            :transition="microTransition" :aria-label="t('player.backToAlbum')" @click="goAlbumPage">
                            <div class="lyrics-heading-title" :title="currentSong.title">{{ currentSong.title }}</div>
                            <div class="lyrics-heading-sub" :title="currentSong.artist">{{ currentSong.artist }}</div>
                        </MotionButton>

                        <div ref="lyricsWindowRef" class="lyrics-window" tabindex="0" :aria-label="t('player.lyrics')"
                            @wheel="handleLyricsWheel" @pointerdown="handleLyricsPointerDown"
                            @scroll="handleLyricsScroll">
                            <MotionDiv v-if="lyricRows.length" class="lyrics-track" :aria-label="t('player.lyrics')">
                                <MotionDiv v-for="(row, index) in lyricRows" :ref="setLyricRowRef(row.key)"
                                    :key="row.key" class="lyric-line"
                                    :class="{ 'lyric-interlude-row': row.type === 'interlude' }" role="button"
                                    tabindex="0" :aria-current="activeLyricRowIndex === index ? 'true' : undefined"
                                    :aria-label="row.type === 'interlude' ? t('player.interlude') : undefined"
                                    :animate="getLyricState(index)"
                                    :transition="row.type === 'interlude' ? instantTransition : contentTransition"
                                    @pointerdown.stop @click.stop="handleLyricClick(row)"
                                    @keydown.stop="handleLyricKeydown($event, row)">
                                    <template v-if="row.type === 'interlude'">
                                        <MoreHorizontal v-if="activeLyricRowIndex === index"
                                            class="lyric-interlude-icon" :size="32" :stroke-width="2.5"
                                            fill="currentColor" aria-hidden="true" />
                                    </template>
                                    <template v-else>
                                        <div class="lyric-primary">{{ row.line.text }}</div>
                                        <template v-if="showSecondaryLyrics">
                                            <div v-for="secondary in row.line.secondary" :key="secondary"
                                                class="lyric-secondary">
                                                {{ secondary }}
                                            </div>
                                        </template>
                                    </template>
                                </MotionDiv>
                            </MotionDiv>
                            <MotionDiv v-else-if="lyricsLoading" class="lyrics-status" :initial="{ opacity: 0 }"
                                :animate="{ opacity: 1 }" :transition="microTransition">
                                {{ t('player.loadingLyrics') }}
                            </MotionDiv>
                            <div v-else-if="plainLyrics.length" class="plain-lyrics">
                                <div v-for="line in plainLyrics" :key="line" class="plain-lyric-line">
                                    {{ line }}
                                </div>
                            </div>
                            <MotionDiv v-else class="lyrics-status" :initial="{ opacity: 0 }" :animate="{ opacity: 1 }"
                                :transition="microTransition">
                                {{ t('player.noSyncedLyrics') }}
                            </MotionDiv>
                        </div>
                    </MotionDiv>

                    <MotionDiv v-else key="narrow-playlist" class="narrow-page narrow-playlist-page"
                        :style="lyricsStyle" :initial="narrowInitial" :animate="narrowAnimate" :exit="narrowExit"
                        :transition="pageTransition">
                        <div class="fullscreen-playlist-scroll">
                            <div class="fullscreen-playlist-header">
                                <MotionButton class="fullscreen-playlist-back" :while-hover="buttonHover"
                                    :while-press="buttonPress" :transition="microTransition"
                                    :aria-label="t('player.backToLyrics')" @click="closePlaylist">
                                    <ArrowLeft :size="18" :stroke-width="1.8" />
                                </MotionButton>
                                <strong>{{ t('player.queue') }}</strong>
                            </div>
                            <Playlist :songs="queueSongs" :current-song="currentSong" :is-playing="isPlaying"
                                :show-header="false" :title="t('player.queue')"
                                @song-select="$emit('playlist-song-select', $event)" />
                        </div>
                    </MotionDiv>
                </AnimatePresence>
            </div>

            <footer class="player-footer">
                <AnimatePresence mode="wait" :initial="false">
                    <MotionDiv v-if="!isPlaybackOptionsOpen" key="transport-bar" class="footer-view transport-view"
                        :initial="footerViewInitial" :animate="footerViewAnimate" :exit="footerViewExit"
                        :transition="footerTransition">
                        <div class="footer-actions">
                            <div class="footer-side footer-side-left">
                                <MotionButton class="footer-button" :while-hover="buttonHover"
                                    :while-press="buttonPress" :transition="microTransition"
                                    :aria-label="t('player.collapsePlayer')" @click="$emit('close')">
                                    <ChevronDown :size="18" :stroke-width="1.5" />
                                </MotionButton>
                                <!-- 宽屏：编辑播放页 + 全屏；窄屏收纳进溢出菜单 -->
                                <template v-if="!isNarrow">
                                    <MotionButton class="footer-button" :while-hover="buttonHover"
                                        :while-press="buttonPress" :transition="microTransition"
                                        :aria-label="t('player.openPlayerOptions')" @click="openPlaybackOptions">
                                        <SquarePen :size="18" :stroke-width="1.5" />
                                    </MotionButton>
                                    <MotionButton class="footer-button" :while-hover="buttonHover"
                                        :while-press="buttonPress" :transition="microTransition"
                                        :aria-label="isBrowserFullscreen ? t('player.exitFullscreen') : t('player.enterFullscreen')"
                                        @click="toggleBrowserFullscreen">
                                        <Minimize2 v-if="isBrowserFullscreen" :size="18" :stroke-width="1.5" />
                                        <Maximize2 v-else :size="18" :stroke-width="1.5" />
                                    </MotionButton>
                                </template>
                                <!-- 窄屏溢出菜单：收纳编辑播放页 / 全屏 / 歌词面板 / 更多 -->
                                <template v-else>
                                    <div id="narrow-overflow-anchor" ref="overflowAnchorRef"
                                        class="footer-popover-anchor overflow-anchor">
                                        <MotionButton class="footer-button" :while-hover="buttonHover"
                                            :while-press="buttonPress" :transition="microTransition"
                                            :aria-label="t('player.more')" :aria-expanded="isOverflowMenuOpen"
                                            @click="isOverflowMenuOpen = !isOverflowMenuOpen; isVolumePopoverOpen = false; isPlaybackModePopoverOpen = false">
                                            <MoreVertical :size="18" :stroke-width="1.5" />
                                        </MotionButton>
                                        <Transition name="overflow-menu">
                                            <div v-if="isOverflowMenuOpen" class="overflow-menu" role="menu">
                                                <MotionButton class="overflow-menu-item"
                                                    :aria-label="t('player.openPlayerOptions')" role="menuitem"
                                                    @click="openPlaybackOptions(); isOverflowMenuOpen = false">
                                                    <SquarePen :size="16" :stroke-width="1.5" />
                                                    <span>{{ t('player.playbackOptions') }}</span>
                                                </MotionButton>
                                                <MotionButton class="overflow-menu-item"
                                                    :aria-label="isBrowserFullscreen ? t('player.exitFullscreen') : t('player.enterFullscreen')"
                                                    role="menuitem"
                                                    @click="toggleBrowserFullscreen(); isOverflowMenuOpen = false">
                                                    <Minimize2 v-if="isBrowserFullscreen" :size="16" :stroke-width="1.5" />
                                                    <Maximize2 v-else :size="16" :stroke-width="1.5" />
                                                    <span>{{ isBrowserFullscreen ? t('player.exitFullscreen') :
                                                        t('player.enterFullscreen') }}</span>
                                                </MotionButton>
                                                <MotionButton class="overflow-menu-item"
                                                    :aria-label="t('player.lyricsPanel')" role="menuitem"
                                                    @click="isOverflowMenuOpen = false">
                                                    <PanelTop :size="16" :stroke-width="1.5" />
                                                    <span>{{ t('player.lyricsPanel') }}</span>
                                                </MotionButton>
                                                <MotionButton class="overflow-menu-item" :aria-label="t('player.more')"
                                                    role="menuitem" @click="isOverflowMenuOpen = false">
                                                    <MoreHorizontal :size="16" :stroke-width="1.5" />
                                                    <span>{{ t('player.more') }}</span>
                                                </MotionButton>
                                            </div>
                                        </Transition>
                                    </div>
                                </template>
                            </div>

                            <div class="transport-column">
                                <div class="transport-controls">
                                    <div id="fullscreen-volume-anchor" ref="volumePopoverAnchorRef"
                                        class="footer-popover-anchor volume-anchor">
                                        <MotionButton class="footer-button volume-button" :while-hover="buttonHover"
                                            :while-press="buttonPress" :transition="microTransition"
                                            :aria-label="t('player.volume')" :aria-expanded="isVolumePopoverOpen"
                                            @click="isVolumePopoverOpen = !isVolumePopoverOpen; isPlaybackModePopoverOpen = false">
                                            <Volume2 :size="18" :stroke-width="1.5" />
                                        </MotionButton>
                                        <VolumePopover :open="isVolumePopoverOpen" :volume="props.volume"
                                            :muted="props.muted" :anchor="volumePopoverAnchorRef" placement="above"
                                            anchor-id="fullscreen-volume-anchor"
                                            @update:volume="$emit('volume-change', $event)"
                                            @mute-change="$emit('mute-change', $event)"
                                            @close="isVolumePopoverOpen = false" />
                                    </div>
                                    <MotionButton class="footer-button" :while-hover="buttonHover"
                                        :while-press="buttonPress" :transition="microTransition"
                                        :aria-label="t('player.previous')" @click="$emit('previous')">
                                        <SkipBack :size="18" :stroke-width="1.5" />
                                    </MotionButton>
                                    <MotionButton class="play-button" :while-hover="{ scale: 1.06 }"
                                        :while-press="{ scale: 0.94 }" :transition="microTransition"
                                        :aria-label="isPlaying ? t('player.pause') : t('player.play')"
                                        @click="$emit('toggle-play')">
                                        <Pause v-if="isPlaying" :size="18" :stroke-width="1.8" />
                                        <Play v-else :size="18" :stroke-width="1.8" />
                                    </MotionButton>
                                    <MotionButton class="footer-button" :while-hover="buttonHover"
                                        :while-press="buttonPress" :transition="microTransition"
                                        :aria-label="t('player.next')" @click="$emit('next')">
                                        <SkipForward :size="18" :stroke-width="1.5" />
                                    </MotionButton>
                                    <div id="fullscreen-mode-anchor" ref="modePopoverAnchorRef"
                                        class="footer-popover-anchor mode-anchor">
                                        <MotionButton class="footer-button" :while-hover="buttonHover"
                                            :while-press="buttonPress" :transition="microTransition"
                                            :aria-label="t('player.playbackOrder')"
                                            :aria-expanded="isPlaybackModePopoverOpen"
                                            @click="isPlaybackModePopoverOpen = !isPlaybackModePopoverOpen; isVolumePopoverOpen = false">
                                            <Shuffle v-if="props.playbackMode === 'shuffle'" :size="18"
                                                :stroke-width="1.5" />
                                            <Repeat1 v-else-if="props.playbackMode === 'repeat-one'" :size="18"
                                                :stroke-width="1.5" />
                                            <ListOrdered v-else :size="18" :stroke-width="1.5" />
                                        </MotionButton>
                                        <PlaybackModePopover :open="isPlaybackModePopoverOpen"
                                            :mode="props.playbackMode" anchor-id="fullscreen-mode-anchor"
                                            placement="above" :list-loop="props.listLoop"
                                            @update:mode="$emit('playback-mode-change', $event)"
                                            @update:list-loop="$emit('list-loop-change', $event)"
                                            @close="isPlaybackModePopoverOpen = false" />
                                    </div>
                                </div>

                                <div class="progress-section">
                                    <span class="time-display">{{ currentTime }}</span>
                                    <div ref="progressRef" class="progress-container"
                                        :class="{ 'is-dragging': isProgressDragging }" role="slider"
                                        :aria-valuenow="Math.round(progress)" aria-valuemin="0" aria-valuemax="100"
                                        tabindex="0" @keydown="handleProgressKeydown"
                                        @pointerdown="handleProgressPointerDown">
                                        <div class="progress-track">
                                            <MotionDiv class="progress-fill" :animate="{ width: `${progress}%` }"
                                                :transition="progressTransition"></MotionDiv>
                                            <MotionDiv class="progress-thumb" :animate="{ left: `${progress}%` }"
                                                :transition="progressTransition"></MotionDiv>
                                        </div>
                                    </div>
                                    <span class="time-display">{{ totalTime }}</span>
                                </div>
                            </div>

                            <div class="footer-side footer-side-right">
                                <!-- 窄屏收纳进溢出菜单：均衡器 / 歌词面板 / 更多 -->
                                <template v-if="!isNarrow">
                                    <MotionButton class="footer-button" :while-hover="buttonHover"
                                        :while-press="buttonPress" :transition="microTransition"
                                        :aria-label="t('player.playbackSettings')">
                                        <SlidersHorizontal :size="18" :stroke-width="1.5" />
                                    </MotionButton>
                                    <MotionButton class="footer-button" :while-hover="buttonHover"
                                        :while-press="buttonPress" :transition="microTransition"
                                        :aria-label="t('player.lyricsPanel')">
                                        <PanelTop :size="18" :stroke-width="1.5" />
                                    </MotionButton>
                                    <MotionButton class="footer-button" :while-hover="buttonHover"
                                        :while-press="buttonPress" :transition="microTransition"
                                        :aria-label="t('player.more')">
                                        <MoreHorizontal :size="18" :stroke-width="1.5" />
                                    </MotionButton>
                                </template>
                                <!-- 播放列表按钮：两种模式都保留在右侧，视觉对称 -->
                                <MotionButton class="footer-button" :while-hover="buttonHover"
                                    :while-press="buttonPress" :transition="microTransition"
                                    :aria-label="t('player.playbackQueue')" @click="togglePlaylist">
                                    <ListMusic :size="18" :stroke-width="1.5" />
                                </MotionButton>
                            </div>
                        </div>
                    </MotionDiv>

                    <MotionDiv v-else key="playback-options" class="footer-view playback-options" role="group"
                        :aria-label="t('player.playbackOptions')" :initial="footerViewInitial"
                        :animate="footerViewAnimate" :exit="footerViewExit" :transition="footerTransition"
                        @animation-complete="handlePlaybackOptionsAnimationComplete">
                        <div class="playback-options-leading">
                            <MotionButton class="footer-button" :while-hover="buttonHover" :while-press="buttonPress"
                                :transition="microTransition" :aria-label="t('player.collapsePlayer')"
                                @click="$emit('close')">
                                <ChevronDown :size="18" :stroke-width="1.5" />
                            </MotionButton>
                            <MotionButton class="footer-button" :while-hover="buttonHover" :while-press="buttonPress"
                                :transition="microTransition" :aria-label="t('player.backToLyrics')"
                                @click="closePlaybackOptions">
                                <ArrowLeft :size="18" :stroke-width="1.5" />
                            </MotionButton>
                            <span class="playback-options-title">{{ t('player.playbackOptions') }}</span>
                            <SettingsComboBox class="settings-panel-combobox" :model-value="settingsPanelIndex"
                                :options="settingsPanelOptions" :label="t('player.settingsPanelNav')"
                                :trigger-label="settingsPanelLabel" :open="isSettingsPanelOpen" placement="above"
                                :width="210" @update:open="isSettingsPanelOpen = $event"
                                @select="handleSettingsPanelSelect" />
                        </div>

                        <AnimatePresence mode="wait" :initial="false">
                            <MotionDiv v-if="settingsPanelIndex === 0" key="panel-lyrics" class="playback-panel"
                                :initial="panelInitial" :animate="panelAnimate" :exit="panelExit"
                                :transition="panelTransition">
                                <div class="playback-option lyrics-size-option">
                                    <output class="option-value" aria-live="polite">{{ lyricsFontSizeValue
                                        }}px</output>
                                    <div class="lyrics-size-slider">
                                        <span class="size-mark size-mark-small" aria-hidden="true">A</span>
                                        <RangeSlider ref="lyricsRangeRef" :model-value="lyricsFontSizeValue" :min="20"
                                            :max="56" :aria-label="t('player.lyricsSize')"
                                            :aria-value-text="`${lyricsFontSizeValue}px`"
                                            @update:model-value="handleLyricsFontSizeInput"
                                            @keydown="handleLyricsRangeKeydown" />
                                        <span class="size-mark size-mark-large" aria-hidden="true">A</span>
                                    </div>
                                </div>
                            </MotionDiv>

                            <MotionDiv v-else-if="settingsPanelIndex === 1" key="panel-appearance"
                                class="playback-panel" :initial="panelInitial" :animate="panelAnimate" :exit="panelExit"
                                :transition="panelTransition">
                                <label class="playback-option toggle-option" for="secondary-lyrics-toggle">
                                    <span class="option-label">{{ t('player.secondaryLyrics') }}</span>
                                    <span class="switch-wrap">
                                        <input id="secondary-lyrics-toggle" type="checkbox"
                                            :checked="showSecondaryLyrics" @change="handleToggleSecondaryLyrics" />
                                        <span class="switch-track" aria-hidden="true">
                                            <span class="switch-thumb"></span>
                                        </span>
                                    </span>
                                    <SettingsComboBox :model-value="albumShape" :options="albumShapeOptions"
                                        :label="t('player.discShape')" :trigger-label="albumShapeLabel"
                                        :open="isDiscShapeOpen" placement="above" :width="190"
                                        @update:open="isDiscShapeOpen = $event" @select="handleAlbumShapeSelect" />
                                    <SettingsComboBox v-if="isCircleShape"
                                        :model-value="albumRotationEnabled ? 'on' : 'off'"
                                        :options="albumRotationOptions" :label="t('player.discRotation')"
                                        :trigger-label="albumRotationLabel" :open="isDiscRotationOpen" placement="above"
                                        :width="190" @update:open="isDiscRotationOpen = $event"
                                        @select="handleAlbumRotationSelect" />
                                </label>

                            </MotionDiv>

                            <MotionDiv v-else key="panel-background" class="playback-panel" :initial="panelInitial"
                                :animate="panelAnimate" :exit="panelExit" :transition="panelTransition">
                                <div class="playback-option background-option-group" role="group"
                                    :aria-label="t('player.background')">
                                    <span class="option-label">{{ t('player.background') }}</span>
                                    <div class="background-options" role="radiogroup"
                                        :aria-label="t('player.background')">
                                        <MotionButton class="background-option"
                                            :class="{ active: normalizedBackgroundMode === 'flowing' }"
                                            :while-hover="buttonHover" :while-press="buttonPress"
                                            :transition="microTransition" role="radio"
                                            :aria-checked="normalizedBackgroundMode === 'flowing'"
                                            @click="setBackgroundMode('flowing')">
                                            {{ t('player.flowing') }}
                                        </MotionButton>
                                        <MotionButton class="background-option"
                                            :class="{ active: normalizedBackgroundMode === 'blur' }"
                                            :while-hover="buttonHover" :while-press="buttonPress"
                                            :transition="microTransition" role="radio"
                                            :aria-checked="normalizedBackgroundMode === 'blur'"
                                            @click="setBackgroundMode('blur')">
                                            {{ t('player.blur') }}
                                        </MotionButton>
                                    </div>
                                </div>
                            </MotionDiv>
                        </AnimatePresence>
                    </MotionDiv>
                </AnimatePresence>
            </footer>
        </div>
    </div>
</template>

<script setup>
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { ArrowLeft, Circle, ChevronDown, ListMusic, ListOrdered, Maximize2, Minimize2, MoreHorizontal, MoreVertical, PanelTop, Pause, Play, Repeat1, RotateCw, Settings2, Shuffle, SkipBack, SkipForward, SlidersHorizontal, Square, SquarePen, Volume2 } from '@lucide/vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { AnimatePresence, motion, useReducedMotion } from 'motion-v'
import MotionTransition from './MotionTransition.vue'
import FlowingBackground from './FlowingBackground.vue'
import Playlist from './Playlist.vue'
import PlaybackModePopover from './PlaybackModePopover.vue'
import VolumePopover from './VolumePopover.vue'
import RangeSlider from './RangeSlider.vue'
import SettingsComboBox from './SettingsComboBox.vue'
import { useAppSettingsStore } from '../stores/appSettingsStore.js'
import { bassCall } from '../services/bassApi.js'
import { APPLE_SPRING, INSTANT_MOTION, MICRO_SPRING, SOFT_SPRING } from '../utils/motion.js'
import { User2Icon } from '@lucide/vue'
import { DiscAlbum } from '@lucide/vue'
import { useI18n } from '../i18n/index.js'

const { t } = useI18n()

const props = defineProps({
    isVisible: {
        type: Boolean,
        default: false
    },
    currentSong: {
        type: Object,
        required: true
    },
    isPlaying: {
        type: Boolean,
        default: false
    },
    currentTime: {
        type: String,
        default: '00:00'
    },
    currentTimeMs: {
        type: Number,
        default: 0
    },
    totalTime: {
        type: String,
        default: '00:00'
    },
    progress: {
        type: Number,
        default: 0
    },
    lyrics: {
        type: Object,
        default: null
    },
    lyricsLoading: {
        type: Boolean,
        default: false
    },
    channelId: {
        type: Number,
        default: null
    },
    backgroundMode: {
        type: String,
        default: 'flowing'
    },
    queueSongs: {
        type: Array,
        default: () => []
    },
    volume: {
        type: Number,
        default: 75
    },
    muted: {
        type: Boolean,
        default: false
    },
    playbackMode: {
        type: String,
        default: 'sequential'
    },
    listLoop: {
        type: Boolean,
        default: false
    }
})

const emit = defineEmits([
    'close',
    'toggle-play',
    'previous',
    'next',
    'progress-change',
    'progress-commit',
    'volume-change',
    'mute-change',
    'playback-mode-change',
    'list-loop-change',
    'add-to-playlist',
    'playlist-song-select',
    'background-mode-change'
])

const MotionDiv = motion.div
const MotionButton = motion.button
const MotionSpan = motion.span
const reducedMotion = useReducedMotion()
const isProgressDragging = ref(false)
const progressRef = ref(null)
const activeProgressPointerId = ref(null)
const isVolumePopoverOpen = ref(false)
const isPlaybackModePopoverOpen = ref(false)
const volumePopoverAnchorRef = ref(null)
const modePopoverAnchorRef = ref(null)
const isOverflowMenuOpen = ref(false)
const overflowAnchorRef = ref(null)
const microTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)
const contentTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : SOFT_SPRING)
const playlistTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)
const instantTransition = computed(() => INSTANT_MOTION)
const progressTransition = computed(() => isProgressDragging.value ? INSTANT_MOTION : microTransition.value)
const footerTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : APPLE_SPRING)
const footerViewInitial = { opacity: 0, y: 96, filter: 'blur(8px)' }
const footerViewAnimate = { opacity: 1, y: 0, filter: 'blur(0px)' }
const footerViewExit = { opacity: 0, y: 96, filter: 'blur(8px)' }
const playlistViewInitial = { opacity: 0, y: 44 }
const playlistViewAnimate = { opacity: 1, y: 0 }
const playlistViewExit = { opacity: 0, y: -44 }
const settingsItemInitial = { opacity: 0, y: 16 }
const settingsItemAnimate = { opacity: 1, y: 0 }
const settingsItemTransition = (index) => reducedMotion.value
    ? INSTANT_MOTION
    : { ...APPLE_SPRING, delay: index * 0.06 }
const tagInitial = { opacity: 0, y: 6, scale: 0.9, filter: 'blur(3px)' }
const tagAnimate = { opacity: 1, y: 0, scale: 1, filter: 'blur(0px)' }
const tagExit = { opacity: 0, y: -4, scale: 0.9, filter: 'blur(3px)' }
const tagTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)
const buttonHover = { scale: 1.08 }
const buttonPress = { scale: 0.92 }
const songInfoHover = { backgroundColor: 'rgba(var(--primary-color), 0.16)' }
const isBrowserFullscreen = ref(false)
const appWindow = getCurrentWindow()
const wasMaximizedBeforeFullscreen = ref(false)

// 窄屏双页模式：专辑页 / 歌词页（播放列表临时替换当前页）
const NARROW_BREAKPOINT = 720
const isNarrow = ref(typeof window !== 'undefined' && window.matchMedia(`(max-width: ${NARROW_BREAKPOINT}px)`).matches)
const narrowPage = ref('album')
const narrowDirection = ref(1)
const pageTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : APPLE_SPRING)
const narrowInitial = computed(() => ({ x: `${-100 * narrowDirection.value}%`, opacity: 0.3 }))
const narrowAnimate = { x: 0, opacity: 1 }
const narrowExit = computed(() => ({ x: `${100 * narrowDirection.value}%`, opacity: 0.3 }))

const goLyricsPage = () => {
    narrowDirection.value = 1
    narrowPage.value = 'lyrics'
}

const goAlbumPage = () => {
    narrowDirection.value = -1
    narrowPage.value = 'album'
}

const syncBrowserFullscreen = async () => {
    if (document.fullscreenElement) {
        isBrowserFullscreen.value = true
        return
    }
    try {
        isBrowserFullscreen.value = await appWindow.isFullscreen()
    } catch {
        isBrowserFullscreen.value = false
    }
}

const toggleBrowserFullscreen = async () => {
    try {
        if (document.fullscreenElement) {
            await document.exitFullscreen()
        } else {
            const isWindowFullscreen = await appWindow.isFullscreen()
            if (isWindowFullscreen) {
                await appWindow.setFullscreen(false)
            } else {
                wasMaximizedBeforeFullscreen.value = await appWindow.isMaximized()
                if (wasMaximizedBeforeFullscreen.value) await appWindow.unmaximize()
                await appWindow.setFullscreen(true)
            }
        }
        if (!document.fullscreenElement && wasMaximizedBeforeFullscreen.value) {
            await appWindow.maximize()
            wasMaximizedBeforeFullscreen.value = false
        }
        await syncBrowserFullscreen()
    } catch (error) {
        if (wasMaximizedBeforeFullscreen.value) {
            await appWindow.maximize().catch(() => { })
            wasMaximizedBeforeFullscreen.value = false
        }
        try {
            if (!document.fullscreenElement && document.documentElement.requestFullscreen) {
                await document.documentElement.requestFullscreen()
                await syncBrowserFullscreen()
            }
        } catch (fallbackError) {
            console.warn('切换全屏失败:', fallbackError || error)
        }
    }
}

const isPlaybackOptionsOpen = ref(false)
const lyricsRangeRef = ref(null)

// 播放器偏好：字号 / 副歌词 / 唱片形状，统一持久化到 settings.json
const appSettings = useAppSettingsStore()
const lyricsFontSizeValue = computed(() => appSettings.state.lyricsFontSize)
const showSecondaryLyrics = computed(() => appSettings.state.showSecondaryLyrics)
const albumShape = computed(() => appSettings.state.albumShape)
// 仅圆形唱片会旋转；圆角矩形保持静止
const albumRotationEnabled = computed(() => appSettings.state.albumRotation && albumShape.value === 'circle')
const lyricsStyle = computed(() => ({ '--lyrics-font-size': `${lyricsFontSizeValue.value}px` }))
const albumShapeClass = computed(() => `album-visual--${albumShape.value}`)
const isCircleShape = computed(() => albumShape.value === 'circle')

// 播放页选项的分区导航：字号 → 外观 → 背景
const settingsPanelIndex = ref(0)
const SETTINGS_PANELS = 3
const settingsPanelOptions = computed(() => [
    { value: 0, label: t('player.lyricsSize'), icon: Settings2 },
    { value: 1, label: t('player.appearance'), icon: Square },
    { value: 2, label: t('player.background'), icon: MoreHorizontal }
])
const settingsPanelLabel = computed(() => settingsPanelOptions.value[settingsPanelIndex.value]?.label ?? '')
const isSettingsPanelOpen = ref(false)

const handleLyricsFontSizeInput = (nextSize) => {
    appSettings.updateLyricsFontSize(nextSize)
}

const handleToggleSecondaryLyrics = () => {
    appSettings.updateShowSecondaryLyrics(!showSecondaryLyrics.value)
}

const handleAlbumShapeSelect = (value) => {
    appSettings.updateAlbumShape(value)
}

const handleAlbumRotationSelect = (value) => {
    appSettings.updateAlbumRotation(value === 'on')
}

const albumShapeOptions = computed(() => [
    { value: 'circle', label: t('player.shapeCircle'), icon: Circle },
    { value: 'rounded-rect', label: t('player.shapeRoundedRect'), icon: Square }
])

const albumShapeLabel = computed(() =>
    albumShapeOptions.value.find((option) => option.value === albumShape.value)?.label ?? '')
const albumRotationOptions = computed(() => [
    { value: 'on', label: t('player.rotationOn'), icon: RotateCw },
    { value: 'off', label: t('player.rotationOff'), icon: Pause }
])

const albumRotationLabel = computed(() =>
    albumRotationOptions.value.find((option) => option.value === (albumRotationEnabled.value ? 'on' : 'off'))?.label ?? '')

const handleSettingsPanelSelect = (value) => {
    settingsPanelIndex.value = Number(value)
}

const isDiscShapeOpen = ref(false)
const isDiscRotationOpen = ref(false)
const panelInitial = { opacity: 0, x: 24 }
const panelAnimate = { opacity: 1, x: 0 }
const panelExit = { opacity: 0, x: -24 }
const panelTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)

// 形状切换为圆角矩形时立即停止旋转并复位角度
watch(albumShape, (shape) => {
    if (shape !== 'circle') {
        stopAlbumRotation()
        albumRotationAngle.value = 0
        if (albumVisualRef.value) albumVisualRef.value.style.transform = 'rotate(0deg)'
    }
})

const ZERO_BANDS = { bass: 0, mid: 0, treble: 0, level: 0 }
const audioBands = ref({ ...ZERO_BANDS })
let audioBandsTimer
let audioBandsRequest = 0
let audioBandsInFlight = false

const bassTrackInfo = ref(null)
let bassTrackInfoRequest = 0
let bassTrackInfoInFlight = false

const formatSampleRate = (value) => {
    const sampleRate = Number(value)
    if (!Number.isFinite(sampleRate) || sampleRate <= 0) return ''
    const kiloHertz = sampleRate / 1000
    return `${Number.isInteger(kiloHertz) ? kiloHertz : kiloHertz.toFixed(1).replace(/\.0$/, '')}KHz`
}

const formatBitrate = (value) => {
    const bitrate = Number(value)
    if (!Number.isFinite(bitrate) || bitrate <= 0) return ''
    return `${Math.round(bitrate)}Kbps`
}

const bassFormatName = (info) => {
    const bassFormat = String(info?.format || '').trim()
    if (bassFormat) return `${bassFormat.toUpperCase()} Audio`

    const source = String(info?.filename || '')
        .split(/[?#]/, 1)[0]
        .split(/[\\/]/).pop() || ''
    const extension = source.includes('.') ? source.split('.').pop().toUpperCase() : ''
    const fallback = String(props.currentSong?.format || props.currentSong?.codec || '')
        .replace(/^\./, '')
        .replace(/\s+Audio$/i, '')
        .trim()
    const format = extension || fallback
    return format ? `${format.toUpperCase()} Audio` : ''
}

const songTags = computed(() => {
    const info = bassTrackInfo.value
    if (!info) return []

    return [
        { key: 'sample-rate', label: formatSampleRate(info.frequency) },
        { key: 'bitrate', label: formatBitrate(info.bitrate ?? props.currentSong?.bitrate) },
        {
            key: 'channels',
            label: Number(info.channels) > 0
                ? `${info.channels} ${Number(info.channels) === 1 ? 'Channel' : 'Channels'}`
                : ''
        },
        { key: 'format', label: bassFormatName(info) }
    ].filter((tag) => tag.label)
})

const normalizedBackgroundMode = computed(() => props.backgroundMode === 'blur' ? 'blur' : 'flowing')
const isPlaylistOpen = ref(false)
const togglePlaylist = () => {
    isPlaylistOpen.value = !isPlaylistOpen.value
    if (isPlaylistOpen.value) isPlaybackOptionsOpen.value = false
}
const closePlaylist = () => {
    isPlaylistOpen.value = false
}

const handleLyricsViewAnimationComplete = () => {
    scheduleLyricsMeasure()
}

const stopAudioBands = () => {
    if (audioBandsTimer) window.clearInterval(audioBandsTimer)
    audioBandsTimer = undefined
    audioBandsRequest++
    audioBandsInFlight = false
    audioBands.value = { ...ZERO_BANDS }
}

const refreshAudioBands = async () => {
    if (!props.isVisible || normalizedBackgroundMode.value !== 'flowing' || !props.channelId || audioBandsInFlight) return
    const requestId = ++audioBandsRequest
    const channelId = props.channelId
    audioBandsInFlight = true
    try {
        const result = await bassCall('bass_channel_fft', { channelId, fftSize: 512 })
        if (requestId === audioBandsRequest && channelId === props.channelId) {
            audioBands.value = result?.bands ? { ...ZERO_BANDS, ...result.bands } : { ...ZERO_BANDS }
        }
    } catch (error) {
        if (requestId === audioBandsRequest) audioBands.value = { ...ZERO_BANDS }
    } finally {
        audioBandsInFlight = false
    }
}

const startAudioBands = () => {
    stopAudioBands()
    if (!props.isVisible || normalizedBackgroundMode.value !== 'flowing' || !props.channelId) return
    refreshAudioBands()
    audioBandsTimer = window.setInterval(refreshAudioBands, 100)
}

const stopBassTrackInfo = () => {
    bassTrackInfoRequest++
    bassTrackInfoInFlight = false
    bassTrackInfo.value = null
}

const refreshBassTrackInfo = async () => {
    if (!props.isVisible || !props.channelId || bassTrackInfoInFlight) return
    const requestId = ++bassTrackInfoRequest
    const channelId = props.channelId
    bassTrackInfoInFlight = true
    try {
        const result = await bassCall('bass_channel_info', { channelId })
        if (requestId === bassTrackInfoRequest && channelId === props.channelId) {
            bassTrackInfo.value = result
        }
    } catch (error) {
        if (requestId === bassTrackInfoRequest) bassTrackInfo.value = null
    } finally {
        bassTrackInfoInFlight = false
    }
}

const startBassTrackInfo = () => {
    stopBassTrackInfo()
    if (!props.isVisible || !props.channelId) return
    refreshBassTrackInfo()
}

const openPlaybackOptions = () => {
    isVolumePopoverOpen.value = false
    isPlaybackModePopoverOpen.value = false
    isPlaybackOptionsOpen.value = true
}

const closePlaybackOptions = () => {
    isPlaybackOptionsOpen.value = false
}

const setBackgroundMode = (mode) => {
    if (mode === normalizedBackgroundMode.value) return
    emit('background-mode-change', mode)
}

const handleLyricsRangeKeydown = (event) => {
    if (event.key !== 'Escape') event.stopPropagation()
}

const updateCompactViewport = () => {
    isNarrow.value = window.matchMedia(`(max-width: ${NARROW_BREAKPOINT}px)`).matches
    // 不重置 narrowPage / isPlaylistOpen，避免调节窗口尺寸时丢失当前页面状态
}

const handlePlaybackOptionsAnimationComplete = () => {
    if (isPlaybackOptionsOpen.value) lyricsRangeRef.value?.focus()
}

const albumStageRef = ref(null)
const albumVisualRef = ref(null)
const lyricsWindowRef = ref(null)
const lyricRowRefs = new Map()
const albumSize = ref(0)
const ALBUM_ROTATION_SPEED = 360 / 48000
const ALBUM_MAX_SIZE = 560
let albumRotationFrame
let albumRotationLastTime = 0
const albumRotationAngle = ref(0)
let albumResizeObserver
let lyricsResizeObserver
let lyricsMeasureFrame
let lyricsIdleTimer
let lyricsProgrammaticScrollTimer
let lyricsProgrammaticScroll = false
const LYRICS_IDLE_TIMEOUT = 5000
const tickCount = 52
const ticks = Array.from({ length: tickCount }, (_, index) => (index * 360) / tickCount)
const syncedLyrics = computed(() => props.lyrics?.lines || [])
const interludes = computed(() => props.lyrics?.interludes || [])
const plainLyrics = computed(() => props.lyrics?.plainLines || [])
const lyricTimelineRows = computed(() => {
    const rows = [
        ...syncedLyrics.value.map((line, index) => ({
            type: 'line',
            key: `line-${line.startTimeMs}-${index}`,
            startTimeMs: line.startTimeMs,
            endTimeMs: line.endTimeMs,
            line
        })),
        ...interludes.value.map((interlude, index) => ({
            type: 'interlude',
            key: `interlude-${interlude.startTimeMs}-${index}`,
            startTimeMs: interlude.startTimeMs,
            endTimeMs: interlude.endTimeMs
        }))
    ]

    return rows.sort((left, right) => left.startTimeMs - right.startTimeMs)
})
const activeLyricTimelineRow = computed(() => lyricTimelineRows.value.find((row) =>
    props.currentTimeMs >= row.startTimeMs && props.currentTimeMs < row.endTimeMs
))
const lyricRows = computed(() => {
    const activeInterludeKey = activeLyricTimelineRow.value?.type === 'interlude'
        ? activeLyricTimelineRow.value.key
        : null

    return lyricTimelineRows.value.filter((row) =>
        row.type !== 'interlude' || row.key === activeInterludeKey
    )
})
const activeLyricRowIndex = computed(() => lyricRows.value.findIndex((row) =>
    props.currentTimeMs >= row.startTimeMs && props.currentTimeMs < row.endTimeMs
))
const isLyricsManualScrolling = ref(false)

const setLyricRowRef = (key) => (value) => {
    const element = value?.$el ?? value
    if (element) lyricRowRefs.set(key, element)
    else lyricRowRefs.delete(key)
}

const clearLyricsIdleTimer = () => {
    if (lyricsIdleTimer) window.clearTimeout(lyricsIdleTimer)
    lyricsIdleTimer = undefined
}

const clearProgrammaticLyricsScroll = () => {
    if (lyricsProgrammaticScrollTimer) window.clearTimeout(lyricsProgrammaticScrollTimer)
    lyricsProgrammaticScrollTimer = undefined
    lyricsProgrammaticScroll = false
}

const markProgrammaticLyricsScroll = () => {
    if (lyricsProgrammaticScrollTimer) window.clearTimeout(lyricsProgrammaticScrollTimer)
    lyricsProgrammaticScroll = true
    lyricsProgrammaticScrollTimer = window.setTimeout(() => {
        lyricsProgrammaticScrollTimer = undefined
        lyricsProgrammaticScroll = false
    }, reducedMotion.value ? 100 : 1200)
}

const scrollLyricsToActive = async (behavior = 'auto') => {
    await nextTick()
    const windowElement = lyricsWindowRef.value
    const activeRowKey = activeLyricTimelineRow.value?.key
    const lineElement = activeRowKey ? lyricRowRefs.get(activeRowKey) : null
    if (!windowElement) return

    const targetTop = lineElement && activeRowKey
        ? lineElement.offsetTop + lineElement.offsetHeight / 2 - windowElement.clientHeight / 2
        : 0
    const maxScrollTop = Math.max(0, windowElement.scrollHeight - windowElement.clientHeight)
    markProgrammaticLyricsScroll()
    windowElement.scrollTo({
        top: Math.max(0, Math.min(maxScrollTop, targetTop)),
        behavior
    })
}

const restoreLyricsAutoScroll = () => {
    lyricsIdleTimer = undefined
    if (!isLyricsManualScrolling.value) return
    isLyricsManualScrolling.value = false
    scheduleLyricsMeasure()
}

const markLyricsAsManuallyScrolled = () => {
    if (isPlaylistOpen.value) return
    clearProgrammaticLyricsScroll()
    isLyricsManualScrolling.value = true
    clearLyricsIdleTimer()
    lyricsIdleTimer = window.setTimeout(restoreLyricsAutoScroll, LYRICS_IDLE_TIMEOUT)
}

const handleLyricsWheel = (event) => {
    if (event.deltaX || event.deltaY) markLyricsAsManuallyScrolled()
}

const handleLyricsPointerDown = (event) => {
    if (event.button === 0) markLyricsAsManuallyScrolled()
}

const handleLyricsScroll = () => {
    if (lyricsProgrammaticScroll) return
    markLyricsAsManuallyScrolled()
}

const parseDurationMs = (value) => {
    const parts = String(value || '').trim().split(':').map(Number)
    if (!parts.length || parts.some((part) => !Number.isFinite(part))) return 0

    const seconds = parts.length === 3
        ? parts[0] * 3600 + parts[1] * 60 + parts[2]
        : parts.length === 2
            ? parts[0] * 60 + parts[1]
            : parts[0]
    return seconds > 0 ? seconds * 1000 : 0
}

const handleLyricClick = (row) => {
    const durationMs = parseDurationMs(props.totalTime)
    if (!Number.isFinite(row.startTimeMs) || durationMs <= 0) return

    const targetProgress = Math.max(0, Math.min(100, row.startTimeMs / durationMs * 100))
    emit('progress-change', targetProgress)
    emit('progress-commit', targetProgress)
}

const handleLyricKeydown = (event, row) => {
    if (event.key !== 'Enter' && event.key !== ' ') return
    event.preventDefault()
    handleLyricClick(row)
}

const scheduleLyricsMeasure = () => {
    if (lyricsMeasureFrame) window.cancelAnimationFrame(lyricsMeasureFrame)
    lyricsMeasureFrame = window.requestAnimationFrame(() => {
        lyricsMeasureFrame = undefined
        if (!isLyricsManualScrolling.value && !isPlaylistOpen.value) {
            scrollLyricsToActive(reducedMotion.value ? 'auto' : 'smooth')
        }
    })
}
const albumVisualStyle = computed(() => {
    const size = albumSize.value || ALBUM_MAX_SIZE
    return {
        ...(albumSize.value > 0 ? {
            width: `${albumSize.value}px`,
            height: `${albumSize.value}px`
        } : {}),
        '--tick-radius': `${size * 0.46}px`,
        '--tick-width': `${Math.max(4, size * 0.018)}px`,
        '--tick-height': `${Math.max(12, size * 0.065)}px`
    }
})

const stopAlbumRotation = () => {
    if (albumRotationFrame) window.cancelAnimationFrame(albumRotationFrame)
    albumRotationFrame = undefined
    albumRotationLastTime = 0
    // 暂停只停帧、保留当前角度，继续播放时从原位置接着转，不再复位到 0 度
}

const updateAlbumRotation = (now) => {
    albumRotationFrame = undefined
    // 仅在播放且可见且开启旋转时推进角度；元素缺失（窄/宽屏切换瞬间）时保留角度并停帧，
    // 由 syncAlbumRotation 在元素恢复后重启，避免切换后停止旋转。
    if (props.isVisible && props.isPlaying && !reducedMotion.value && albumRotationEnabled.value) {
        if (!albumRotationLastTime) albumRotationLastTime = now
        const elapsed = Math.min(100, now - albumRotationLastTime)
        albumRotationLastTime = now
        albumRotationAngle.value = (albumRotationAngle.value + elapsed * ALBUM_ROTATION_SPEED) % 360
    }

    const element = albumVisualRef.value
    if (element) {
        element.style.transform = `rotate(${albumRotationAngle.value}deg)`
    }
    albumRotationFrame = window.requestAnimationFrame(updateAlbumRotation)
}

const syncAlbumRotation = () => {
    if (!props.isVisible || !props.isPlaying || reducedMotion.value || !albumRotationEnabled.value) {
        stopAlbumRotation()
        return
    }

    if (!albumRotationFrame) {
        albumRotationLastTime = 0
        albumRotationFrame = window.requestAnimationFrame(updateAlbumRotation)
    }
}

const getLyricState = (index) => {
    const distance = activeLyricRowIndex.value < 0
        ? index + 1
        : Math.abs(index - activeLyricRowIndex.value)
    return {
        opacity: distance === 0 ? 1 : Math.max(0.22, 0.7 - distance * 0.13),
        scale: distance === 0 ? 1 : Math.max(0.88, 1 - distance * 0.035),
        filter: isLyricsManualScrolling.value || distance === 0
            ? 'blur(0px)'
            : `blur(${Math.min(6, distance * 1.5)}px)`,
        color: distance === 0 ? '#ffffff' : 'rgba(255, 255, 255, 0.5)',
        fontWeight: distance === 0 ? 750 : 600
    }
}

const progressFromPointer = (event) => {
    const rect = progressRef.value?.getBoundingClientRect()
    if (!rect?.width) return 0
    const percent = ((event.clientX - rect.left) / rect.width) * 100
    return Math.max(0, Math.min(100, percent))
}

const handleProgressPointerDown = (event) => {
    if (event.button !== 0) return
    isProgressDragging.value = true
    activeProgressPointerId.value = event.pointerId
    progressRef.value?.setPointerCapture?.(event.pointerId)
    emit('progress-change', progressFromPointer(event))
    window.addEventListener('pointermove', handleProgressPointerMove)
    window.addEventListener('pointerup', handleProgressPointerUp)
    window.addEventListener('pointercancel', handleProgressPointerUp)
}

const handleProgressPointerMove = (event) => {
    if (!isProgressDragging.value || event.pointerId !== activeProgressPointerId.value) return
    emit('progress-change', progressFromPointer(event))
}

const handleProgressPointerUp = (event) => {
    if (!isProgressDragging.value || event.pointerId !== activeProgressPointerId.value) return
    const nextProgress = progressFromPointer(event)
    emit('progress-change', nextProgress)
    emit('progress-commit', nextProgress)
    if (progressRef.value?.hasPointerCapture?.(event.pointerId)) {
        progressRef.value.releasePointerCapture(event.pointerId)
    }
    isProgressDragging.value = false
    activeProgressPointerId.value = null
    window.removeEventListener('pointermove', handleProgressPointerMove)
    window.removeEventListener('pointerup', handleProgressPointerUp)
    window.removeEventListener('pointercancel', handleProgressPointerUp)
}

const handleProgressKeydown = (event) => {
    let nextProgress = props.progress
    if (event.key === 'ArrowLeft' || event.key === 'ArrowDown') nextProgress -= 5
    else if (event.key === 'ArrowRight' || event.key === 'ArrowUp') nextProgress += 5
    else if (event.key === 'Home') nextProgress = 0
    else if (event.key === 'End') nextProgress = 100
    else return

    event.preventDefault()
    event.stopPropagation()
    const boundedProgress = Math.max(0, Math.min(100, nextProgress))
    emit('progress-change', boundedProgress)
    emit('progress-commit', boundedProgress)
}

const handleKeydown = (event) => {
    if (!props.isVisible) return

    switch (event.key) {
        case 'Escape':
            if (isPlaylistOpen.value) closePlaylist()
            else if (isOverflowMenuOpen.value) isOverflowMenuOpen.value = false
            else if (isPlaybackOptionsOpen.value) closePlaybackOptions()
            else emit('close')
            break
        case ' ':
            event.preventDefault()
            emit('toggle-play')
            break
        case 'ArrowLeft':
            emit('previous')
            break
        case 'ArrowRight':
            emit('next')
            break
    }
}

const handleOverflowOutside = (event) => {
    if (!isOverflowMenuOpen.value) return
    const anchor = overflowAnchorRef.value?.$el ?? overflowAnchorRef.value
    if (anchor && !anchor.contains(event.target)) isOverflowMenuOpen.value = false
}

const updateAlbumSize = () => {
    const stage = albumStageRef.value
    if (!stage) return

    // 封面尺寸取舞台宽高与上限的最小值，保证始终是正方形、等比缩放、绝不被压扁。
    // 舞台是 flex 项，其高度即封面可占用的剩余空间，故直接取其 clientWidth/clientHeight。
    const nextSize = Math.max(0, Math.min(stage.clientWidth, stage.clientHeight, ALBUM_MAX_SIZE))
    if (nextSize > 0) albumSize.value = nextSize
}

onMounted(() => {
    document.addEventListener('keydown', handleKeydown)
    document.addEventListener('fullscreenchange', syncBrowserFullscreen)
    document.addEventListener('pointerdown', handleOverflowOutside, true)
    syncBrowserFullscreen()
    window.addEventListener('resize', updateCompactViewport)
    albumResizeObserver = new ResizeObserver(updateAlbumSize)
    if (albumStageRef.value) albumResizeObserver.observe(albumStageRef.value)
    lyricsResizeObserver = new ResizeObserver(scheduleLyricsMeasure)
    if (lyricsWindowRef.value) lyricsResizeObserver.observe(lyricsWindowRef.value)
    requestAnimationFrame(updateAlbumSize)
    scheduleLyricsMeasure()
    startAudioBands()
    syncAlbumRotation()
})

onUnmounted(() => {
    window.removeEventListener('pointermove', handleProgressPointerMove)
    window.removeEventListener('pointerup', handleProgressPointerUp)
    window.removeEventListener('pointercancel', handleProgressPointerUp)
    document.removeEventListener('keydown', handleKeydown)
    document.removeEventListener('fullscreenchange', syncBrowserFullscreen)
    document.removeEventListener('pointerdown', handleOverflowOutside, true)
    window.removeEventListener('resize', updateCompactViewport)
    albumResizeObserver?.disconnect()
    lyricsResizeObserver?.disconnect()
    if (lyricsMeasureFrame) window.cancelAnimationFrame(lyricsMeasureFrame)
    clearLyricsIdleTimer()
    clearProgrammaticLyricsScroll()
    lyricRowRefs.clear()
    stopAlbumRotation()
    stopAudioBands()
    stopBassTrackInfo()
    if (document.fullscreenElement === document.documentElement) document.exitFullscreen().catch(() => { })
    appWindow.isFullscreen().then(async (isFullscreen) => {
        if (!isFullscreen) return
        await appWindow.setFullscreen(false).catch(() => { })
        if (wasMaximizedBeforeFullscreen.value) await appWindow.maximize().catch(() => { })
    }).catch(() => { })
})

watch([lyricRows, activeLyricRowIndex, activeLyricTimelineRow, lyricsFontSizeValue], scheduleLyricsMeasure, {
    deep: true,
    flush: 'post'
})
watch(isPlaylistOpen, async () => {
    await nextTick()
    lyricsResizeObserver?.disconnect()
    if (!isPlaylistOpen.value && lyricsWindowRef.value) {
        lyricsResizeObserver?.observe(lyricsWindowRef.value)
        scheduleLyricsMeasure()
    }
}, { flush: 'post' })

// 宽屏/窄屏切换会改变 albumStage / lyricsWindow 的渲染实例，需重新挂载观测器
watch([isNarrow, narrowPage, isPlaylistOpen], async () => {
    await nextTick()
    albumResizeObserver?.disconnect()
    if (albumStageRef.value) albumResizeObserver?.observe(albumStageRef.value)
    requestAnimationFrame(updateAlbumSize)

    lyricsResizeObserver?.disconnect()
    const lyricsVisible = !isPlaylistOpen.value && (narrowPage.value === 'lyrics' || !isNarrow.value)
    if (lyricsVisible && lyricsWindowRef.value) lyricsResizeObserver?.observe(lyricsWindowRef.value)
    scheduleLyricsMeasure()

    // 封面元素在窄/宽屏间切换后是新的 DOM 实例，重启旋转以立即恢复
    syncAlbumRotation()
}, { flush: 'post' })
watch([() => props.isVisible, () => props.currentSong?.id, () => props.lyrics], () => {
    clearLyricsIdleTimer()
    clearProgrammaticLyricsScroll()
    isLyricsManualScrolling.value = false
}, { flush: 'post' })
watch(() => [props.isVisible, props.channelId, normalizedBackgroundMode.value], startAudioBands)
watch(() => [props.isVisible, props.channelId, props.currentSong?.id, props.currentSong?.title], startBassTrackInfo, { immediate: true })
watch([() => props.isVisible, () => props.isPlaying, reducedMotion, albumRotationEnabled], syncAlbumRotation, { immediate: true })
</script>

<style scoped>
.fullscreen-player {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    height: 100dvh;
    overflow: hidden;
    color: #f7f5f3;
    background: #090807;
    isolation: isolate;
}

.player-background,
.backdrop-image,
.backdrop-wash,
.backdrop-vignette {
    position: absolute;
    inset: 0;
}

.player-background {
    z-index: 0;
    overflow: hidden;
    background: #090807;
}

.backdrop-image {
    background-position: center;
    background-size: cover;
    filter: blur(48px) saturate(1.15) brightness(0.65);
    opacity: 0.62;
    transform: scale(1.14);
}

.backdrop-wash {
    background: rgba(7, 5, 5, 0.18);
}

.backdrop-vignette {
    box-shadow: inset 0 0 220px rgba(0, 0, 0, 0.6);
}

.player-container {
    position: relative;
    z-index: 1;
    width: 100%;
    height: 100%;
    overflow: hidden;
}

.player-main {
    display: grid;
    position: relative;
    z-index: 1;
    grid-template-columns: minmax(420px, 50%) minmax(0, 50%);
    width: 100%;
    height: calc(100% - 156px);
    min-height: 0;
}

.visual-column {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    justify-content: flex-start;
    padding: clamp(80px, 18vh, 250px) clamp(32px, 8.8vw, 230px) 0;
    min-height: 0;
    overflow: hidden;
}

.album-stage {
    flex: 1 1 auto;
    min-width: 0;
    min-height: 0;
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
}

.album-visual {
    position: relative;
    flex: 0 1 auto;
    min-width: 0;
    min-height: 0;
    flex: 0 0 auto;
    width: min(100%, 560px);
    height: auto;
    aspect-ratio: 1;
    margin-left: 0;
    display: grid;
    place-items: center;
    transform-origin: center;
    will-change: transform;
    --tick-radius: 138px;
}

.tick-ring {
    position: absolute;
    inset: 0;
}

.tick {
    position: absolute;
    top: 50%;
    left: 50%;
    width: var(--tick-width, 5px);
    height: var(--tick-height, 18px);
    border-radius: 99px;
    background: rgba(115, 105, 99, 0.52);
    transform-origin: center;
}

.disc-shell {
    position: relative;
    width: 100%;
    height: 100%;
    display: grid;
    place-items: center;
    border: clamp(7px, 0.7vw, 13px) solid rgba(0, 0, 0, 0.1);
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.2);
    /* box-shadow: 0 18px 38px rgba(66, 45, 39, 0.22); */
}

.disc-shell::before {
    content: '';
    position: absolute;
    inset: 8%;
    border-radius: 50%;
    border: 2px solid rgba(101, 87, 80, 0.44);
    pointer-events: none;
}

/* 圆角矩形唱片：外框、内环与封面同步变为圆角矩形 */
.disc-shell.album-visual--rounded-rect {
    border-radius: 20px;
}

.disc-shell.album-visual--rounded-rect::before {
    border-radius: 20px;
}

.album-cover.album-visual--rounded-rect {
    border-radius: 10px;
}

.album-cover-frame {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    transform-origin: 50% 50%;
}

.album-cover {
    position: absolute;
    inset: 2.5%;
    width: 95%;
    height: 95%;
    border-radius: 50%;
    object-fit: cover;
    transform-origin: 50% 50%;
    box-shadow: 0 8px 20px rgba(48, 31, 26, 0.28);
}

.song-details {
    flex: 0 0 auto;
    width: min(100%, 500px);
    margin: 50px 0 52px clamp(0px, 3vw, 75px);
    text-align: left;
}

.song-title,
.song-artist,
.song-album {
    margin: 0;
    color: #f7f5f3;
    display: flex;
    gap: 10px;
}

.song-title {
    font-size: 20px;
    font-weight: bold;
}

.song-artist {
    margin-top: 20px;
    font-size: 16px;
    font-weight: normal;
    opacity: 0.5;
}

.song-album {
    margin-top: 10px;
    font-size: 16px;
    opacity: 0.5;
}

.song-tags {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-start;
    gap: 5px;
    margin-top: 20px;
    min-height: 17px;
}

.tag {
    display: inline-block;
    padding: 3px 5px;
    border-radius: 5px;
    color: rgba(255, 255, 255, 0.3);
    background: rgba(255, 255, 255, 0.12);
    font-size: 11px;
    /* font-weight: 650; */
    line-height: 1;
}

.lyrics-column {
    position: relative;
    min-width: 0;
    height: 100%;
    min-height: 0;
    overflow: hidden;
}

.lyrics-view,
.fullscreen-playlist-view {
    position: absolute;
    inset: 0;
    display: flex;
    min-width: 0;
    min-height: 0;
    flex-direction: column;
    will-change: transform, opacity;
}

.fullscreen-playlist-view {
    overflow: hidden;
}

.fullscreen-playlist-header {
    display: flex;
    align-items: center;
    flex: 0 0 auto;
    gap: 14px;
    margin: 0 0 12px;
    color: #ffffff;
}

.fullscreen-playlist-header h2 {
    font-size: 22px;
    font-weight: 700;
    letter-spacing: -0.03em;
}

.fullscreen-playlist-header span {
    display: block;
    margin-top: 4px;
    color: rgba(255, 255, 255, 0.56);
    font-size: 12px;
}

.fullscreen-playlist-back {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex: 0 0 auto;
    width: 38px;
    height: 38px;
    padding: 0;
    border-radius: 50%;
    color: #ffffff;
    background: transparent;
    border: none;
    cursor: pointer;
}

.fullscreen-playlist-scroll {
    position: absolute;
    inset: 6% 8% 10% 2%;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
    scrollbar-gutter: stable;
    padding: 48px 0 64px;
    overscroll-behavior: contain;
    touch-action: pan-y;
    padding-right: 30px;
    mask-image: linear-gradient(to bottom, transparent 0%, #000 11%, #000 88%, transparent 100%);
    -webkit-mask-image: linear-gradient(to bottom, transparent 0%, #000 11%, #000 88%, transparent 100%);
    height: 90%
}

.fullscreen-playlist-scroll:hover {
    overflow: auto
}

.fullscreen-playlist-scroll::-webkit-scrollbar-track {
    margin-top: 48px;
    margin-bottom: 64px;
}

.fullscreen-playlist-scroll :deep(.playlist) {
    padding-right: 0;
    padding-left: 0;
}

.fullscreen-playlist-scroll :deep(.playlist-item) {
    color: rgba(255, 255, 255, 0.9);
}

.fullscreen-playlist-scroll :deep(.playlist-item-artist),
.fullscreen-playlist-scroll :deep(.playlist-item-duration) {
    color: rgba(255, 255, 255, 0.52);
}

.fullscreen-playlist-scroll :deep(.playlist-item.is-current) {
    color: #ffffff;
    background: rgba(var(--primary-color), 0.22);
}

.lyrics-window {
    position: absolute;
    inset: 6% 8% 10% 2%;
    overflow-x: hidden;
    overflow-y: hidden;
    display: flex;
    align-items: stretch;
    justify-content: center;
    scrollbar-gutter: stable;
    scrollbar-width: thin;
    scrollbar-color: rgba(255, 255, 255, 0.38) transparent;
    overscroll-behavior: contain;
    touch-action: pan-y;
    mask-image: linear-gradient(to bottom, transparent 0%, #000 16%, #000 84%, transparent 100%);
    -webkit-mask-image: linear-gradient(to bottom, transparent 0%, #000 16%, #000 84%, transparent 100%);
    height: 90%;
}

.lyrics-window:hover {
    overflow-y: auto;
}

.lyrics-window::-webkit-scrollbar {
    width: 5px;
}

.lyrics-window::-webkit-scrollbar-track {
    background: transparent;
}

.lyrics-window::-webkit-scrollbar-thumb {
    border-radius: 3px;
    background: rgba(255, 255, 255, 0.38);
}

.lyrics-window::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.58);
}

.lyrics-track {
    position: relative;
    top: 0;
    left: 0;
    width: 100%;
    height: max-content;
    min-height: 100%;
    flex: 0 0 auto;
    align-self: flex-start;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 18px;
    padding: 38% 0;
    transform-origin: center;
    padding-left: 10px;
}

.lyric-line {
    width: fit-content;
    max-width: 100%;
    flex: 0 0 auto;
    min-height: 44px;
    padding: 5px 10px;
    border-radius: 10px;
    color: rgba(255, 255, 255, 0.5);
    font-size: var(--lyrics-font-size, 32px);
    line-height: 1.18;
    letter-spacing: -0.035em;
    text-align: left;
    transform-origin: left center;
    will-change: transform, opacity, filter;
    cursor: pointer;
    transition: background-color 160ms ease, color 160ms ease;
}

.lyric-line:hover,
.lyric-line:focus-visible {
    color: rgba(255, 255, 255, 0.9);
    background: rgba(var(--primary-color), 0.14);
}

.lyric-primary {
    font-weight: inherit;
}

.lyric-secondary {
    margin-top: 8px;
    color: rgba(255, 255, 255, 0.58);
    font-size: 1em;
    font-weight: 550;
    line-height: 1.25;
    letter-spacing: -0.015em;
}

.lyrics-status,
.plain-lyrics {
    width: 100%;
    color: rgba(255, 255, 255, 0.52);
    text-align: left;
}

.lyrics-status {
    align-self: center;
    padding: 0 8%;
    font-size: 22px;
}

.lyric-interlude-row {
    display: flex;
    align-items: center;
    justify-content: flex-start;
}

.lyric-interlude-icon {
    flex: 0 0 auto;
}

.plain-lyrics {
    align-self: center;
    display: flex;
    flex-direction: column;
    gap: 18px;
    max-height: 76%;
    overflow: hidden;
    padding: 0 4%;
}

.plain-lyric-line {
    font-size: var(--lyrics-font-size, 32px);
    line-height: 1.35;
}

@media (max-height: 820px) and (min-width: 721px) {
    .song-tags {
        gap: 7px;
        margin-top: 14px;
        min-height: 21px;
    }

    .tag {
        padding: 5px 9px;
    }
}

.player-footer {
    position: absolute;
    z-index: 2;
    right: 0;
    bottom: 0;
    left: 0;
    height: 95px;
    overflow: visible;
    padding: 0 20px;
}

.footer-view {
    width: 100%;
    min-height: 62px;
    will-change: transform, opacity, filter;
}

.transport-view {
    display: flex;
    align-items: center;
}

.footer-actions {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    width: 100%;
    height: 62px;
}

.playback-options {
    display: grid;
    grid-template-columns: auto 1fr 20px;
    align-items: center;
    gap: clamp(26px, 5vw, 88px);
}

.playback-options-leading,
.playback-option,
.background-options,
.lyrics-size-slider,
.option-label-row {
    display: flex;
    align-items: center;
}

.playback-options-leading {
    gap: 14px;
    min-width: 0;
}

.playback-panel {
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: flex-start;
    flex-wrap: wrap;
    gap: 16px clamp(22px, 3vw, 56px);
    min-width: 0;
}

.combo-row {
    flex: 0 0 auto;
    justify-content: flex-start;
    gap: 12px;
}

.toggle-option {
    flex: 0 0 auto;
}

.playback-options-title {
    overflow: hidden;
    color: rgba(255, 255, 255, 0.86);
    font-size: 14px;
    font-weight: 650;
    white-space: nowrap;
    text-overflow: ellipsis;
}

.playback-option {
    min-width: 0;
    gap: 18px;
    width: 100%;
}

.lyrics-size-option {
    display: flex;
    gap: 10px;
}

.option-label-row {
    justify-content: space-between;
    gap: 16px;
}

.option-label {
    color: rgba(255, 255, 255, 0.58);
    font-size: 12px;
    white-space: nowrap;
}

.option-label-row .option-label {
    margin-left: auto;
}

.option-value {
    min-width: 42px;
    color: rgba(255, 255, 255, 0.86);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
    text-align: right;
}

.lyrics-size-slider {
    gap: 10px;
    width: 100%;
}

.size-mark {
    flex: 0 0 auto;
    color: rgba(255, 255, 255, 0.5);
    font-weight: 700;
    line-height: 1;
}

.size-mark-small {
    font-size: 11px;
}

.size-mark-large {
    font-size: 18px;
}

.background-options {
    gap: 8px;
}

.background-option {
    min-width: 76px;
    padding: 8px 12px;
    border: 1px solid rgba(255, 255, 255, 0.16);
    border-radius: 9px;
    color: rgba(255, 255, 255, 0.66);
    background: rgba(255, 255, 255, 0.06);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
}

.background-option.active {
    border-color: rgba(var(--primary-color), 0.72);
    color: #ffffff;
    background: rgba(var(--primary-color), 0.22);
}

.toggle-option {
    gap: 12px;
    cursor: pointer;
}

.switch-wrap {
    position: relative;
    display: inline-flex;
    flex: 0 0 auto;
}

.switch-wrap input {
    position: absolute;
    inset: 0;
    z-index: 1;
    width: 100%;
    height: 100%;
    margin: 0;
    opacity: 0;
    cursor: pointer;
}

.switch-track {
    position: relative;
    display: block;
    width: 40px;
    height: 22px;
    border-radius: 99px;
    background: rgba(255, 255, 255, 0.14);
    transition: background-color 160ms ease;
}

.switch-thumb {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.82);
    transition: transform 160ms ease;
}

.switch-wrap input:checked+.switch-track {
    background: rgb(var(--primary-color));
}

.switch-wrap input:checked+.switch-track .switch-thumb {
    transform: translateX(18px);
}

.footer-side {
    display: flex;
    align-items: center;
    align-self: center;
    height: 48px;
    gap: 16px;
}

.footer-side-right {
    justify-content: flex-end;
}

.transport-controls {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    align-self: center;
    height: 48px;
    gap: 28px;
}

.footer-popover-anchor {
    position: relative;
    display: inline-flex;
}

.transport-column {
    display: flex;
    align-items: center;
    justify-content: center;
    align-self: center;
    flex-direction: column;
}

.footer-button,
.play-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    border: 0;
    color: #f7f5f3;
    background: transparent;
    cursor: pointer;
}

.footer-button {
    width: 32px;
    height: 32px;
}

.play-button {
    width: 44px;
    height: 44px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.14);
}

.progress-section {
    display: flex;
    align-items: center;
    gap: 24px;
    width: min(48vw, 900px);
    margin: 13px auto 0;
}

.time-display {
    min-width: 42px;
    color: rgba(255, 255, 255, 0.68);
    font-size: 12px;
    text-align: center;
}

.progress-container {
    flex: 1;
    min-width: 0;
    padding: 9px 0;
    cursor: grab;
    outline: none;
    touch-action: none;
    user-select: none;
}

.progress-container.is-dragging {
    cursor: grabbing;
}

.progress-track {
    position: relative;
    width: 100%;
    height: 3px;
    overflow: visible;
    border-radius: 99px;
    background: rgba(255, 255, 255, 0.3);
}

.progress-fill {
    height: 100%;
    border-radius: inherit;
    background: rgb(var(--primary-color));
}

.progress-thumb {
    position: absolute;
    top: 50%;
    width: 11px;
    height: 11px;
    border-radius: 50%;
    background: rgb(var(--primary-color));
    box-shadow: 0 2px 7px rgba(0, 0, 0, 0.4);
    opacity: 0;
    pointer-events: none;
    transform: translate(-50%, -50%) scale(0.5);
    transition: opacity 120ms ease, transform 120ms ease;
}

.progress-container:hover .progress-thumb,
.progress-container:focus-visible .progress-thumb,
.progress-container.is-dragging .progress-thumb {
    opacity: 1;
    transform: translate(-50%, -50%) scale(1);
}

@media (max-width: 1050px) {
    .player-main {
        grid-template-columns: minmax(340px, 46%) minmax(0, 54%);
    }

    .visual-column {
        padding-left: 7vw;
        padding-right: 3vw;
    }

    .lyric-line {
        font-size: var(--lyrics-font-size, 32px);
    }

    .playback-options {
        grid-template-columns: minmax(155px, 0.65fr) minmax(220px, 1.25fr) minmax(200px, 0.85fr);
        gap: 18px;
        padding: 0 20px;
    }

    .transport-controls {
        gap: 14px;
    }

    .play-button {
        width: 48px;
        height: 48px;
    }
}

/* ============ 窄屏双页模式 ============ */
@media (max-width: 720px) {
    .player-container.is-narrow {
        display: flex;
        flex-direction: column;
    }

    /* 上=动区（页），下=固定播放控制区 */
    .narrow-body {
        position: relative;
        flex: 1 1 auto;
        min-height: 0;
        overflow: hidden;
    }

    .narrow-page {
        position: absolute;
        inset: 0;
        display: flex;
        flex-direction: column;
        min-height: 0;
        overflow: hidden;
        will-change: transform, opacity;
    }

    /* 第一页：旋转封面 + 居中歌曲信息 */
    .narrow-album-page {
        align-items: center;
        justify-content: center;
        padding: 14px 20px;
        gap: 14px;
    }

    .narrow-album-page .album-stage {
        margin: 70px 0 20px;
        width: 100%;
        min-height: 0;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .narrow-album-page .album-visual {
        width: auto;
        height: auto;
        min-width: 0;
        aspect-ratio: auto;
    }

    /* 歌曲信息大按钮：默认纯透明，仅悬停时亮起 + 圆角 */
    .song-info-nav {
        flex: 0 0 auto;
        width: 100%;
        max-width: 420px;
        margin: 0;
        padding: 12px 18px;
        border: 1px solid transparent;
        border-radius: 10px;
        background: transparent;
        color: inherit;
        text-align: center;
        cursor: pointer;
        font: inherit;
        margin-bottom: 30px;
    }

    .song-info-nav .song-details-inner {
        display: block;
    }

    .song-info-nav .song-title {
        font-size: 20px;
        justify-content: center;
    }

    .song-info-nav .song-artist,
    .song-info-nav .song-album {
        justify-content: center;
    }

    .song-info-nav .song-tags {
        justify-content: center;
        margin-top: 12px;
    }

    /* 第二页：歌词（居左显示，整体不居中） */
    .narrow-lyrics-page {
        padding: 0;
    }

    /* 歌词页标题按钮：下移避开顶部标题栏；默认透明，悬停亮起 */
    .lyrics-heading {
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        flex: 0 0 auto;
        gap: 2px;
        padding: 12px;
        margin: 72px 12px 12px;
        border: 1px solid transparent;
        border-radius: 10px;
        background: transparent;
        color: #f7f5f3;
        text-align: left;
        cursor: pointer;
        font: inherit;
    }

    .lyrics-heading-title {
        max-width: 100%;
        overflow: hidden;
        font-size: 18px;
        font-weight: 700;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .lyrics-heading-sub {
        max-width: 100%;
        overflow: hidden;
        font-size: 13px;
        font-weight: 500;
        color: rgba(255, 255, 255, 0.55);
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    /* 歌词窗口：填满标题按钮下方到页面底部的空间，歌词文本居左、整体左对齐 */
    .narrow-lyrics-page .lyrics-window {
        position: relative;
        inset: auto;
        flex: 1 1 auto;
        height: auto;
        min-height: 0;
        justify-content: flex-start;
        mask-image: linear-gradient(to bottom, transparent 0%, #000 6%, #000 88%, transparent 100%);
        -webkit-mask-image: linear-gradient(to bottom, transparent 0%, #000 6%, #000 88%, transparent 100%);
    }

    .narrow-lyrics-page .lyrics-track {
        align-items: flex-start;
        padding: 8% 22px 18%;
    }

    .narrow-lyrics-page .lyric-line {
        min-height: 40px;
        font-size: var(--lyrics-font-size, 22px);
        text-align: left;
    }

    .narrow-lyrics-page .plain-lyrics,
    .narrow-lyrics-page .lyrics-status {
        padding: 0 22px;
    }

    /* 播放列表页 */
    .narrow-playlist-page .fullscreen-playlist-scroll {
        margin-top: 30px;
        inset: 0 16px 16px 16px;
        padding: 40px 0 24px;
    }

    .narrow-playlist-page .fullscreen-playlist-header {
        margin-bottom: 8px;
    }

    /* 固定播放控制区：保持透明背景，左右按钮对称 */
    .player-footer {
        position: relative;
        flex: 0 0 auto;
        bottom: auto;
        height: auto;
        padding: 10px 16px 14px;
        background: transparent;
        backdrop-filter: none;
        -webkit-backdrop-filter: none;
    }

    .player-footer .footer-view {
        min-height: 0;
    }

    .footer-actions {
        grid-template-columns: auto minmax(0, 1fr) auto;
        height: auto;
        gap: 8px;
    }

    .footer-side {
        gap: 4px;
    }

    .transport-controls {
        gap: 12px;
    }

    .footer-button {
        width: 30px;
        height: 30px;
        flex: 0 0 auto;
    }

    .play-button {
        width: 46px;
        height: 46px;
    }

    .transport-column {
        height: auto;
    }

    /* 进度条适当收窄，为左右按钮留白 */
    .progress-section {
        width: min(72%, 420px);
        gap: 8px;
        margin-top: 6px;
    }

    .time-display {
        min-width: 32px;
        font-size: 11px;
    }

    /* 溢出菜单：收纳窄屏下无法显示的按钮 */
    .overflow-anchor {
        position: relative;
    }

    .overflow-menu {
        position: absolute;
        bottom: calc(100% + 8px);
        left: 0;
        z-index: 50;
        display: flex;
        flex-direction: column;
        gap: 2px;
        min-width: 168px;
        padding: 6px;
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 12px;
        background: rgba(18, 16, 14, 0.82);
        backdrop-filter: blur(16px) saturate(1.1);
        -webkit-backdrop-filter: blur(16px) saturate(1.1);
        box-shadow: 0 12px 34px rgba(0, 0, 0, 0.4);
        transform-origin: bottom left;
    }

    .overflow-menu-enter-active,
    .overflow-menu-leave-active {
        transition: opacity 160ms ease, transform 160ms ease;
    }

    .overflow-menu-enter-from,
    .overflow-menu-leave-to {
        opacity: 0;
        transform: translateY(6px) scale(0.98);
    }

    .overflow-menu-item {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 8px 10px;
        border: none;
        border-radius: 8px;
        color: rgba(255, 255, 255, 0.82);
        background: transparent;
        font: inherit;
        font-size: 13px;
        cursor: pointer;
        white-space: nowrap;
    }

    .overflow-menu-item:hover {
        color: #ffffff;
        background: rgba(var(--primary-color), 0.16);
    }

    .playback-options {
        grid-template-columns: 1fr;
        grid-auto-rows: min-content;
        gap: 10px;
        align-content: start;
        padding: 0;
    }

    .playback-options .playback-option {
        gap: 8px;
    }

    .playback-options-leading {
        flex-wrap: wrap;
        row-gap: 8px;
    }

    .settings-panel-combobox {
        margin-left: 0;
    }

    .playback-panel {
        flex-wrap: wrap;
        gap: 12px 16px;
    }
}
</style>

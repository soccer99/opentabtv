# TypeScript Reference Implementation

Translated from tablo-tools-electron and Tablo_Lite for Vue.js frontend.

## Core Types

```typescript
// src/types/tablo.ts

// ─────────────────────────────────────────────────────────────────────
// Device Types
// ─────────────────────────────────────────────────────────────────────

export interface TabloDevice {
  sid: string
  name: string
  privateIp: string
  localUrl: string
  serverId: string
  serverVersion?: string
  board?: string
  model?: DeviceModel
  accountToken?: string
  lighthouseToken?: string
}

export interface DeviceModel {
  name: string
  tuners?: number
}

export interface ServerInfo {
  name: string
  private_ip: string
  server_id: string
  server_version: string
  timezone: string
  model: DeviceModel
}

export interface TunerStatus {
  number: number
  in_use: boolean
  recording: boolean
}

export interface HardDrive {
  name: string
  free_mib: number
  size_mib: number
}

export interface DeviceSettings {
  led?: string
  recording_quality: string
  live_tv_quality?: string
  commercial_skip?: string
  auto_delete?: boolean
}

export interface Subscription {
  title: string
  status: string
  description?: string
}

// ─────────────────────────────────────────────────────────────────────
// Channel Types
// ─────────────────────────────────────────────────────────────────────

export interface Channel {
  path: string
  object_id: string
  channel: ChannelInfo
}

export interface ChannelInfo {
  major: number
  minor: number
  network?: string
  call_sign: string
  resolution?: string
  channel_identifier?: string
  logos?: ChannelLogo[]
}

export interface ChannelLogo {
  kind: 'originalLarge' | 'lightLarge' | string
  url: string
}

export interface GuideChannel {
  identifier: string
  callSign: string
  major: number
  minor: number
  network: string
  kind: 'ota' | 'ott'
  displayName: string
  logoUrl?: string
  currentProgram?: CurrentProgram
}

export interface CurrentProgram {
  title: string
  description?: string
  start: string
  duration: number
  genres?: string[]
  kind?: string
}

// ─────────────────────────────────────────────────────────────────────
// Recording Types
// ─────────────────────────────────────────────────────────────────────

export interface Recording {
  path: string
  object_id: number
  airing_details: AiringDetails
  video_details?: VideoDetails
  user_info?: UserInfo
  episode?: EpisodeInfo
  event?: EventInfo
  series_path?: string
  movie_path?: string
  sport_path?: string
  snapshot_image?: SnapshotImage
}

export interface AiringDetails {
  datetime: string
  show_title: string
  duration: number
  channel_path?: string
  genres?: string[]
  event_type?: string
}

export interface VideoDetails {
  duration: number
  state: 'finished' | 'recording' | 'failed'
  size: number
  width?: number
  height?: number
  clean: boolean
  audio?: 'aac' | 'ac3'
  comskip?: ComskipInfo
}

export interface ComskipInfo {
  state: 'ready' | 'processing' | string
  error?: string
}

export interface UserInfo {
  watched: boolean
  protected: boolean
  position?: number
}

export interface EpisodeInfo {
  title?: string
  description?: string
  season_number: number
  number: number
}

export interface EventInfo {
  title: string
  description?: string
}

export interface SnapshotImage {
  image_id: number
}

export type RecordingType = 'series' | 'movie' | 'event' | 'program'

// ─────────────────────────────────────────────────────────────────────
// Streaming Types
// ─────────────────────────────────────────────────────────────────────

export interface WatchSession {
  playlist_url: string
  token?: string
  expires?: string
  keepalive?: number
  bif_url_sd?: string
  bif_url_hd?: string
  cutlist?: CutlistEntry[]
}

export interface CutlistEntry {
  start: number
  end: number
}

export interface StreamSession {
  sessionId: string
  playlistUrl: string
  proxyUrl: string
  transcoded: boolean
}

// ─────────────────────────────────────────────────────────────────────
// Schedule Types
// ─────────────────────────────────────────────────────────────────────

export interface ScheduleRequest {
  schedule: ScheduleRule
  schedule_rule: string
  config: ScheduleConfig
  program: { title: string }
  show_counts: ShowCounts
  keep: KeepRule
  recordings_path: null
}

export interface ScheduleRule {
  rule: 'all' | 'new' | 'none'
  offsets: ScheduleOffsets
}

export interface ScheduleOffsets {
  start: number
  end: number
  source: 'none' | 'show'
}

export interface ScheduleConfig {
  title: string
  channel_path: string
  duration: number
  kind: 'once' | 'series'
  once?: OnceSchedule
}

export interface OnceSchedule {
  year: number
  month: number
  day: number
  hour: number
  minute: number
  timezone: string
}

export interface ShowCounts {
  airing_count: number
  conflicted_count: number
  scheduled_count: number
  unwatched_count?: number
  protected_count?: number
  watched_and_protected_count?: number
  failed_count?: number
}

export interface KeepRule {
  rule: 'none' | 'count' | 'all'
  count: number | null
}

// ─────────────────────────────────────────────────────────────────────
// Error Types
// ─────────────────────────────────────────────────────────────────────

export interface TabloError {
  error: {
    code: string
    description: string
    details?: string
  }
}

// ─────────────────────────────────────────────────────────────────────
// Guide Grid Types
// ─────────────────────────────────────────────────────────────────────

export interface GuideGridChannel {
  identifier: string
  callSign: string
  major: number
  minor: number
  network: string
  displayName: string
  logoUrl?: string
  airings: GuideAiring[]
}

export interface GuideAiring {
  title: string
  description?: string
  start: string
  duration: number
  genres?: string[]
  kind?: string
}
```

## Tablo Service

```typescript
// src/services/tablo.ts

import { invoke } from '@tauri-apps/api/core'
import type {
  TabloDevice,
  ServerInfo,
  Channel,
  Recording,
  WatchSession,
  TunerStatus,
  HardDrive,
  DeviceSettings,
  Subscription,
  GuideChannel,
  GuideGridChannel,
  ScheduleRequest
} from '@/types/tablo'

const BATCH_LIMIT = 50

class TabloService {
  private deviceIp: string | null = null

  setDevice(ip: string) {
    this.deviceIp = ip
  }

  private get baseUrl(): string {
    if (!this.deviceIp) throw new Error('No device selected')
    return `http://${this.deviceIp}:8885`
  }

  // ─────────────────────────────────────────────────────────────────────
  // HTTP Helpers (for direct browser calls - legacy mode)
  // ─────────────────────────────────────────────────────────────────────

  private async get<T>(path: string): Promise<T> {
    const response = await fetch(`${this.baseUrl}${path}`)
    if (!response.ok) {
      const error = await response.json()
      throw new Error(error.error?.description || `HTTP ${response.status}`)
    }
    return response.json()
  }

  private async post<T>(path: string, body?: unknown): Promise<T> {
    const response = await fetch(`${this.baseUrl}${path}`, {
      method: 'POST',
      cache: 'no-cache',
      body: body ? JSON.stringify(body) : undefined,
    })
    if (!response.ok) {
      const error = await response.json()
      throw new Error(error.error?.description || `HTTP ${response.status}`)
    }
    return response.json()
  }

  private async patch<T>(path: string, body: unknown): Promise<T> {
    const response = await fetch(`${this.baseUrl}${path}`, {
      method: 'PATCH',
      body: JSON.stringify(body),
    })
    if (!response.ok) {
      const error = await response.json()
      throw new Error(error.error?.description || `HTTP ${response.status}`)
    }
    return response.json()
  }

  private async delete(path: string): Promise<void> {
    const response = await fetch(`${this.baseUrl}${path}`, {
      method: 'DELETE',
    })
    if (!response.ok && response.status !== 204) {
      const error = await response.json()
      throw new Error(error.error?.description || `HTTP ${response.status}`)
    }
  }

  // ─────────────────────────────────────────────────────────────────────
  // Discovery (via Tauri backend)
  // ─────────────────────────────────────────────────────────────────────

  async discoverDevices(): Promise<TabloDevice[]> {
    return invoke<TabloDevice[]>('discover_devices')
  }

  // ─────────────────────────────────────────────────────────────────────
  // Server Info
  // ─────────────────────────────────────────────────────────────────────

  async getServerInfo(): Promise<ServerInfo> {
    return invoke<ServerInfo>('get_server_info', { deviceIp: this.deviceIp })
  }

  async getTuners(): Promise<TunerStatus[]> {
    return this.get<TunerStatus[]>('/server/tuners')
  }

  async getHardDrives(): Promise<HardDrive[]> {
    return this.get<HardDrive[]>('/server/harddrives')
  }

  async getSettings(): Promise<DeviceSettings> {
    return this.get<DeviceSettings>('/settings/info')
  }

  async getSubscriptions(): Promise<Subscription[]> {
    return this.get<Subscription[]>('/account/subscription')
  }

  // ─────────────────────────────────────────────────────────────────────
  // Channels
  // ─────────────────────────────────────────────────────────────────────

  async getChannelPaths(): Promise<string[]> {
    return this.get<string[]>('/guide/channels')
  }

  async getChannel(path: string): Promise<Channel> {
    return this.get<Channel>(path)
  }

  async batchGetChannels(paths: string[]): Promise<Record<string, Channel | null>> {
    const results: Record<string, Channel | null> = {}

    for (let i = 0; i < paths.length; i += BATCH_LIMIT) {
      const chunk = paths.slice(i, i + BATCH_LIMIT)
      const batch = await this.post<Record<string, Channel | null>>('/batch', chunk)
      Object.assign(results, batch)
    }

    return results
  }

  async getAllChannels(): Promise<Channel[]> {
    const paths = await this.getChannelPaths()
    const batch = await this.batchGetChannels(paths)
    return Object.values(batch).filter((c): c is Channel => c !== null)
  }

  // ─────────────────────────────────────────────────────────────────────
  // Recordings
  // ─────────────────────────────────────────────────────────────────────

  async getRecordingPaths(): Promise<string[]> {
    return this.get<string[]>('/recordings/airings')
  }

  async getRecording(path: string): Promise<Recording> {
    return this.get<Recording>(path)
  }

  async batchGetRecordings(paths: string[]): Promise<Record<string, Recording | null>> {
    const results: Record<string, Recording | null> = {}

    for (let i = 0; i < paths.length; i += BATCH_LIMIT) {
      const chunk = paths.slice(i, i + BATCH_LIMIT)
      const batch = await this.post<Record<string, Recording | null>>('/batch', chunk)
      Object.assign(results, batch)
    }

    return results
  }

  async getAllRecordings(): Promise<Recording[]> {
    const paths = await this.getRecordingPaths()
    const batch = await this.batchGetRecordings(paths)
    return Object.values(batch).filter((r): r is Recording => r !== null)
  }

  async deleteRecording(path: string): Promise<void> {
    return this.delete(path)
  }

  async setWatched(path: string, watched: boolean): Promise<void> {
    await this.patch(path, { watched })
  }

  async setProtected(path: string, protected_: boolean): Promise<void> {
    await this.patch(path, { protected: protected_ })
  }

  async setPosition(path: string, position: number): Promise<void> {
    await this.patch(path, { position })
  }

  // ─────────────────────────────────────────────────────────────────────
  // Streaming
  // ─────────────────────────────────────────────────────────────────────

  async watch(path: string): Promise<WatchSession> {
    return this.post<WatchSession>(`${path}/watch`, '')
  }

  async watchChannel(channelPath: string): Promise<WatchSession> {
    return invoke<WatchSession>('watch_channel', {
      deviceIp: this.deviceIp,
      channelPath
    })
  }

  async watchRecording(recordingPath: string): Promise<WatchSession> {
    return invoke<WatchSession>('watch_recording', {
      deviceIp: this.deviceIp,
      recordingPath
    })
  }

  // ─────────────────────────────────────────────────────────────────────
  // Guide / Airings
  // ─────────────────────────────────────────────────────────────────────

  async getAiringPaths(): Promise<string[]> {
    return this.get<string[]>('/guide/airings')
  }

  async scheduleAiring(path: string): Promise<void> {
    await this.patch(path, { scheduled: true })
  }

  async unscheduleAiring(path: string): Promise<void> {
    await this.patch(path, { scheduled: false })
  }

  async scheduleManual(
    title: string,
    channelPath: string,
    datetime: Date,
    durationMinutes: number
  ): Promise<void> {
    const timezone = Intl.DateTimeFormat().resolvedOptions().timeZone

    const request: ScheduleRequest = {
      schedule: {
        rule: 'all',
        offsets: { start: 0, end: 0, source: 'none' }
      },
      schedule_rule: 'all',
      config: {
        title,
        channel_path: channelPath,
        duration: durationMinutes * 60,
        kind: 'once',
        once: {
          year: datetime.getFullYear(),
          month: datetime.getMonth() + 1,
          day: datetime.getDate(),
          hour: datetime.getHours(),
          minute: datetime.getMinutes(),
          timezone
        }
      },
      program: { title },
      show_counts: {
        airing_count: 1,
        conflicted_count: 0,
        scheduled_count: 1
      },
      keep: { rule: 'none', count: null },
      recordings_path: null
    }

    await this.post('/guide/programs', request)
  }

  // ─────────────────────────────────────────────────────────────────────
  // Series Scheduling
  // ─────────────────────────────────────────────────────────────────────

  async scheduleSeriesAll(seriesPath: string): Promise<void> {
    await this.patch(seriesPath, { schedule: 'all' })
  }

  async scheduleSeriesNew(seriesPath: string): Promise<void> {
    await this.patch(seriesPath, { schedule: 'new' })
  }

  async unscheduleSeries(seriesPath: string): Promise<void> {
    await this.patch(seriesPath, { schedule: 'none' })
  }

  async setSeriesKeep(seriesPath: string, count: number | null): Promise<void> {
    const keep = count !== null
      ? { keep: { rule: 'count', count } }
      : { keep: { rule: 'all', count: null } }
    await this.patch(seriesPath, keep)
  }
}

export const tabloService = new TabloService()
export default tabloService
```

## Pinia Store

```typescript
// src/stores/devices.ts

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { tabloService } from '@/services/tablo'
import type { TabloDevice, ServerInfo, TunerStatus, HardDrive } from '@/types/tablo'

export const useDevicesStore = defineStore('devices', () => {
  // State
  const devices = ref<TabloDevice[]>([])
  const activeDevice = ref<TabloDevice | null>(null)
  const serverInfo = ref<ServerInfo | null>(null)
  const tuners = ref<TunerStatus[]>([])
  const hardDrives = ref<HardDrive[]>([])
  const isDiscovering = ref(false)
  const isConnected = ref(false)
  const error = ref<string | null>(null)

  // Getters
  const hasDevices = computed(() => devices.value.length > 0)
  const deviceCount = computed(() => devices.value.length)
  const tunerCount = computed(() => tuners.value.length)
  const tunersInUse = computed(() => tuners.value.filter(t => t.in_use).length)
  const tunersRecording = computed(() => tuners.value.filter(t => t.recording).length)

  const totalStorage = computed(() =>
    hardDrives.value.reduce((acc, d) => acc + d.size_mib, 0)
  )
  const freeStorage = computed(() =>
    hardDrives.value.reduce((acc, d) => acc + d.free_mib, 0)
  )
  const usedStoragePercent = computed(() =>
    totalStorage.value > 0
      ? Math.round(((totalStorage.value - freeStorage.value) / totalStorage.value) * 100)
      : 0
  )

  // Actions
  async function discover() {
    isDiscovering.value = true
    error.value = null

    try {
      devices.value = await tabloService.discoverDevices()

      // Auto-select if only one device
      if (devices.value.length === 1) {
        await selectDevice(devices.value[0])
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Discovery failed'
      throw e
    } finally {
      isDiscovering.value = false
    }
  }

  async function selectDevice(device: TabloDevice) {
    activeDevice.value = device
    tabloService.setDevice(device.privateIp)
    isConnected.value = false
    error.value = null

    try {
      // Fetch device info
      const [info, tunerStatus, drives] = await Promise.all([
        tabloService.getServerInfo(),
        tabloService.getTuners(),
        tabloService.getHardDrives()
      ])

      serverInfo.value = info
      tuners.value = tunerStatus
      hardDrives.value = drives
      isConnected.value = true
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Connection failed'
      isConnected.value = false
      throw e
    }
  }

  async function refreshStatus() {
    if (!activeDevice.value) return

    try {
      const [tunerStatus, drives] = await Promise.all([
        tabloService.getTuners(),
        tabloService.getHardDrives()
      ])

      tuners.value = tunerStatus
      hardDrives.value = drives
    } catch (e) {
      console.error('Failed to refresh device status:', e)
    }
  }

  function disconnect() {
    activeDevice.value = null
    serverInfo.value = null
    tuners.value = []
    hardDrives.value = []
    isConnected.value = false
  }

  return {
    // State
    devices,
    activeDevice,
    serverInfo,
    tuners,
    hardDrives,
    isDiscovering,
    isConnected,
    error,
    // Getters
    hasDevices,
    deviceCount,
    tunerCount,
    tunersInUse,
    tunersRecording,
    totalStorage,
    freeStorage,
    usedStoragePercent,
    // Actions
    discover,
    selectDevice,
    refreshStatus,
    disconnect
  }
})
```

```typescript
// src/stores/recordings.ts

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { tabloService } from '@/services/tablo'
import type { Recording, RecordingType } from '@/types/tablo'

export const useRecordingsStore = defineStore('recordings', () => {
  // State
  const recordings = ref<Recording[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // Getters
  const recordingCount = computed(() => recordings.value.length)

  const byType = computed(() => {
    const groups: Record<RecordingType, Recording[]> = {
      series: [],
      movie: [],
      event: [],
      program: []
    }

    for (const rec of recordings.value) {
      if (rec.series_path) groups.series.push(rec)
      else if (rec.movie_path) groups.movie.push(rec)
      else if (rec.sport_path) groups.event.push(rec)
      else groups.program.push(rec)
    }

    return groups
  })

  const unwatchedCount = computed(() =>
    recordings.value.filter(r => !r.user_info?.watched).length
  )

  const protectedCount = computed(() =>
    recordings.value.filter(r => r.user_info?.protected).length
  )

  const failedCount = computed(() =>
    recordings.value.filter(r => r.video_details?.state === 'failed').length
  )

  const totalSize = computed(() =>
    recordings.value.reduce((acc, r) => acc + (r.video_details?.size || 0), 0)
  )

  // Actions
  async function fetchAll() {
    isLoading.value = true
    error.value = null

    try {
      recordings.value = await tabloService.getAllRecordings()
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load recordings'
      throw e
    } finally {
      isLoading.value = false
    }
  }

  async function deleteRecording(recording: Recording) {
    await tabloService.deleteRecording(recording.path)
    recordings.value = recordings.value.filter(r => r.object_id !== recording.object_id)
  }

  async function setWatched(recording: Recording, watched: boolean) {
    await tabloService.setWatched(recording.path, watched)
    const rec = recordings.value.find(r => r.object_id === recording.object_id)
    if (rec?.user_info) {
      rec.user_info.watched = watched
    }
  }

  async function setProtected(recording: Recording, protected_: boolean) {
    await tabloService.setProtected(recording.path, protected_)
    const rec = recordings.value.find(r => r.object_id === recording.object_id)
    if (rec?.user_info) {
      rec.user_info.protected = protected_
    }
  }

  function getById(id: number): Recording | undefined {
    return recordings.value.find(r => r.object_id === id)
  }

  function getByPath(path: string): Recording | undefined {
    return recordings.value.find(r => r.path === path)
  }

  return {
    // State
    recordings,
    isLoading,
    error,
    // Getters
    recordingCount,
    byType,
    unwatchedCount,
    protectedCount,
    failedCount,
    totalSize,
    // Actions
    fetchAll,
    deleteRecording,
    setWatched,
    setProtected,
    getById,
    getByPath
  }
})
```

## Utility Functions

```typescript
// src/utils/recording.ts

import type { Recording, RecordingType } from '@/types/tablo'

export function getRecordingType(recording: Recording): RecordingType {
  if (recording.series_path) return 'series'
  if (recording.movie_path) return 'movie'
  if (recording.sport_path) return 'event'
  return 'program'
}

export function getEpisodeString(recording: Recording): string {
  if (!recording.episode) return ''
  const { season_number, number } = recording.episode
  return `S${String(season_number).padStart(2, '0')}E${String(number).padStart(2, '0')}`
}

export function formatDuration(seconds: number): string {
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)

  if (hours > 0) {
    return `${hours}h ${minutes}m`
  }
  return `${minutes}m`
}

export function formatFileSize(bytes: number): string {
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let size = bytes
  let unitIndex = 0

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024
    unitIndex++
  }

  return `${size.toFixed(1)} ${units[unitIndex]}`
}

export function formatDateTime(isoString: string): string {
  const date = new Date(isoString)
  return date.toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: 'numeric',
    minute: '2-digit'
  })
}

export function isRecordingComplete(recording: Recording): boolean {
  return recording.video_details?.state === 'finished'
}

export function isRecordingFailed(recording: Recording): boolean {
  return recording.video_details?.state === 'failed'
}

export function isRecordingInProgress(recording: Recording): boolean {
  return recording.video_details?.state === 'recording'
}

export function hasComskip(recording: Recording): boolean {
  return recording.video_details?.comskip?.state === 'ready'
}
```

```typescript
// src/utils/channel.ts

import type { Channel, ChannelInfo } from '@/types/tablo'

export function getChannelNumber(channel: ChannelInfo): string {
  return channel.minor > 0
    ? `${channel.major}.${channel.minor}`
    : String(channel.major)
}

export function getChannelDisplayName(channel: ChannelInfo): string {
  const number = getChannelNumber(channel)
  const name = channel.network || channel.call_sign
  return `${number} ${name}`
}

export function getChannelLogo(channel: ChannelInfo): string | undefined {
  if (!channel.logos?.length) return undefined

  // Prefer originalLarge, then lightLarge, then any
  const logo =
    channel.logos.find(l => l.kind === 'originalLarge') ||
    channel.logos.find(l => l.kind === 'lightLarge') ||
    channel.logos[0]

  return logo?.url
}

export function sortChannels(channels: Channel[]): Channel[] {
  return [...channels].sort((a, b) => {
    const majorDiff = a.channel.major - b.channel.major
    if (majorDiff !== 0) return majorDiff
    return a.channel.minor - b.channel.minor
  })
}
```

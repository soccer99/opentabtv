# Feature Specifications

## Core Features

### 1. Device Discovery & Connection

**Priority**: P0 (Critical)

#### Requirements
- Auto-discover Tablo devices on LAN via UDP broadcast (port 8881)
- Support manual IP entry for segmented networks
- Cloud-based discovery as fallback (4th Gen)
- Remember previously connected devices
- Display device info: name, model, storage, tuner count

#### User Flow
1. App launches → scans for devices automatically
2. Found devices shown in list with name/IP
3. User clicks device → authenticates → connected
4. OR user clicks "Enter IP manually" → enters IP → connects

#### Technical Notes
- UDP discovery on port 8881
- Cloud API: `https://api.tablotv.com/assocserver/getipinfo/`
- Store device cache in Pinia persisted state
- Handle network errors gracefully

---

### 2. Live TV

**Priority**: P0 (Critical)

#### Requirements
- Display channel list with logos, numbers, call signs
- Show "now playing" info for each channel
- Single-click to start watching
- Full-screen video player with controls
- Support for HLS playback (h.264)
- FFmpeg transcoding for incompatible codecs
- Optional VLC launch for external playback

#### User Flow
1. Navigate to Live TV view
2. See grid/list of channels with current program
3. Click channel → video player opens → playback starts
4. Overlay controls: play/pause, volume, fullscreen, quality
5. Press ESC or click X → return to channel list

#### Technical Notes
- hls.js for in-browser playback
- Detect codec issues → fallback to FFmpeg transcode
- Stream URL format: `http://{ip}/live/{session}.m3u8`
- Buffer 2-3 seconds for smooth playback

---

### 3. TV Guide

**Priority**: P1 (High)

#### Requirements
- Grid layout: channels (rows) × time (columns)
- Scroll horizontally through time
- Scroll vertically through channels
- Current time indicator
- Click program → show details
- Jump to "Now" button
- Filter by category (movies, sports, news, etc.)

#### UI Design
```
┌──────┬─────────┬─────────┬─────────┬─────────┐
│      │  8:00   │  8:30   │  9:00   │  9:30   │
├──────┼─────────┴─────────┼─────────┴─────────┤
│ CBS  │    Morning Show   │    The Price...   │
├──────┼─────────┬─────────┼─────────────────────┤
│ NBC  │  Today  │  cont.  │      Dateline      │
├──────┼─────────┴─────────┴─────────┬──────────┤
│ ABC  │        Good Morning         │  Local   │
└──────┴─────────────────────────────┴──────────┘
```

#### Technical Notes
- Fetch guide data from Tablo API (7+ days)
- Virtual scrolling for performance
- Cache guide data (TTL: 1 hour)
- Time zone handling

---

### 4. Recordings Library

**Priority**: P1 (High)

#### Requirements
- Browse all recordings
- Group by: show, movie, sports, date
- Show metadata: title, episode, duration, date recorded
- Thumbnail previews
- Play recording
- Delete recording (with confirmation)
- Search/filter recordings

#### User Flow
1. Navigate to Recordings
2. View grouped by series/movies
3. Click series → see episodes
4. Click episode → play
5. Long-press/right-click → delete option

---

### 5. Settings

**Priority**: P2 (Medium)

#### Preferences
- Theme: Dark / Light / System
- Video quality preference
- Default view on launch
- FFmpeg path (if not in PATH)
- VLC integration toggle
- Clear device cache

#### Device Settings (Read-only initially)
- Device name
- Storage info
- Tuner status
- Firmware version

---

## Future Features (Post-MVP)

### Scheduling
- View scheduled recordings
- Set manual recordings
- Manage series rules

### Remote Access
- Tablo Connect integration
- Out-of-home streaming

### Multi-Device
- Switch between multiple Tablo devices
- Unified library view

### Search
- Global search across guide and recordings
- Voice search (macOS dictation)

### Picture-in-Picture
- Mini player while browsing
- System PiP support

### Keyboard Shortcuts
| Key | Action |
|-----|--------|
| Space | Play/Pause |
| F | Toggle fullscreen |
| M | Mute/Unmute |
| ←/→ | Seek 10s |
| ↑/↓ | Volume |
| Esc | Exit fullscreen / Close player |
| G | Go to Guide |
| L | Go to Live TV |
| R | Go to Recordings |

---

## Non-Functional Requirements

### Performance
- App launch: < 2 seconds
- Device discovery: < 5 seconds
- Channel switch: < 3 seconds (live TV)
- Guide load: < 2 seconds (cached)

### Compatibility
- macOS 12+ (Monterey and later)
- Apple Silicon + Intel support
- Screen sizes: 1280x720 minimum

### Accessibility
- VoiceOver support
- Keyboard navigation
- High contrast mode support
- Reduced motion support

### Security
- No telemetry/analytics without consent
- Credentials in system keychain
- Local network only (no external server)

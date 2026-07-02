# Tablo API Reference

Comprehensive API documentation for Tablo DVR devices (based on unofficial API docs and reference implementations).

## Overview

Tablo devices expose a REST API on port 8885. The API requires **no authentication** for local access (public endpoints). A private Meteor-based API exists for the official Tablo Connect web app but is not publicly accessible.

## Network Ports

| Port  | Protocol | Purpose                                    |
|-------|----------|--------------------------------------------|
| 80    | HTTP     | Video/audio streaming data                 |
| 8881  | UDP      | Device discovery (broadcast)               |
| 8885  | HTTP     | REST API endpoints                         |
| 8887  | WebSocket| WAMP API (real-time events)               |
| 18080 | HTTP     | Read-only file access (pseudo-FTP)         |

## Device Discovery

### Cloud Discovery (4th Gen Primary)
```
GET https://api.tablotv.com/assocserver/getipinfo/
```
Returns JSON array of registered devices with private/public IPs.

### UDP Broadcast (Legacy/Fallback)
- **Send**: UDP broadcast to port 8881
- **Listen**: Port 8882 for response
- Returns device info including private IP address

### Manual IP Entry
Allow users to enter IP directly for network segmentation scenarios.

## API Endpoints

### Server Information

#### Get Device Info
```
GET /server/info
```
Returns device name, model, firmware version, timezone.

#### Get Capabilities
```
GET /server/capabilities
```
Returns list of available features.

#### Get Guide Status
```
GET /server/guide/status
```
Returns guide data status and last update time.

#### Get Tuner Status
```
GET /server/tuners
```
Returns array of tuner objects with `in_use` and `recording` flags.

**Response:**
```json
[
  { "number": 0, "in_use": true, "recording": true },
  { "number": 1, "in_use": false, "recording": false }
]
```

#### Get Hard Drives
```
GET /server/harddrives
```
Returns connected storage with `name`, `free_mib`, `size_mib`.

### Account & Settings

#### Get Subscription
```
GET /account/subscription
```
Returns active services, subscription state, expiration dates.

#### Get Settings
```
GET /settings/info
```
Returns LED, recording quality, auto-delete preferences.

#### Get Quality Options
```
GET /settings/recording_qualities
GET /settings/recording_qualities/<NUM>
```
Returns available quality options with Mbps, resolution, hourly size.

### Guide Data

#### Get All Channel Paths
```
GET /guide/channels
```
Returns array of channel paths (e.g., `["/guide/channels/357025", ...]`).

#### Get Channel Details
```
GET /guide/channels/<id>
```
Returns channel info: call sign, network, resolution.

**Response:**
```json
{
  "path": "/guide/channels/357025",
  "object_id": "357025",
  "channel": {
    "major": 7,
    "minor": 1,
    "network": "FOX",
    "call_sign": "WXYZ",
    "resolution": "1080i",
    "channel_identifier": "abc123",
    "logos": [
      { "kind": "originalLarge", "url": "https://..." }
    ]
  }
}
```

#### Get All Airing Paths
```
GET /guide/airings
```
Returns paths with optional `state` filter (scheduled, conflicted, none).

#### Get Shows
```
GET /guide/shows
```
Supports `qualifier` (new, premiering, primetime) and `state` filters.

#### Get Movies
```
GET /guide/movies
```
Supports `channel`, `minrating`, `maxrating`, `unrated` filters.

#### Get Series
```
GET /guide/series
GET /guide/series/<id>
GET /guide/series/<id>/seasons
GET /guide/series/<id>/episodes
```

#### Get Sports
```
GET /guide/sports
GET /guide/sports/<id>
GET /guide/sports/events/<id>
```

### Recordings

#### Get All Recording Paths
```
GET /recordings/airings
```

#### Get Recording Details
```
GET /recordings/airings/<id>
```

**Response:**
```json
{
  "path": "/recordings/airings/12345",
  "object_id": 12345,
  "airing_details": {
    "datetime": "2024-09-15T20:00:00.000Z",
    "show_title": "Breaking News",
    "duration": 3600,
    "channel_path": "/guide/channels/357025"
  },
  "video_details": {
    "state": "finished",
    "duration": 3600,
    "size": 2147483648,
    "width": 1920,
    "height": 1080,
    "clean": true,
    "audio": "aac",
    "comskip": { "state": "ready", "error": null }
  },
  "user_info": {
    "watched": false,
    "protected": false,
    "position": 0
  },
  "episode": {
    "title": "Episode Title",
    "description": "...",
    "season_number": 1,
    "number": 5
  },
  "series_path": "/guide/series/123"
}
```

#### Delete Recording
```
DELETE /recordings/<path>
```
Returns HTTP 204 on success.

#### Modify Recording
```
PATCH /recordings/<path>
```

**Mark watched:**
```json
{ "watched": true }
```

**Mark protected:**
```json
{ "protected": true }
```

**Set playback position:**
```json
{ "position": 1234 }
```

### Batch Operations

```
POST /batch
Content-Type: application/json

["/recordings/series/episodes/2267413", "/recordings/series/episodes/2265212"]
```

Returns object with path keys containing full records or `null`. **Maximum 50 entries per request.**

### Streaming / Watch Sessions

#### Start Watch Session
```
POST /recordings/<path>/watch
POST /guide/channels/<id>/watch
```

**Response:**
```json
{
  "playlist_url": "http://192.168.1.12:80/watch/...",
  "token": "abc123",
  "expires": "2024-09-15T21:00:00Z",
  "keepalive": 30,
  "bif_url_sd": "http://...",
  "bif_url_hd": "http://...",
  "cutlist": [
    { "start": 120.5, "end": 180.0 },
    { "start": 900.0, "end": 960.0 }
  ]
}
```

The `playlist_url` is an HLS m3u8 stream. The `cutlist` contains commercial skip segments.

### Scheduling Recordings

#### Schedule Single Airing
```
PATCH /guide/<path>
```
```json
{ "scheduled": true }
```
or
```json
{ "scheduled": false }
```

#### Schedule Series
```
PATCH /guide/series/<id>
```

**Record all episodes:**
```json
{ "schedule": "all" }
```

**Record new episodes only:**
```json
{ "schedule": "new" }
```

**Cancel recording:**
```json
{ "schedule": "none" }
```

#### Modify Recording Rules
```
PATCH /guide/series/<id>
```

**Keep last N recordings:**
```json
{ "keep": { "rule": "count", "count": 5 } }
```

**Keep all:**
```json
{ "keep": { "rule": "all", "count": null } }
```

**Early start (seconds before):**
```json
{ "schedule": { "offsets": { "source": "show", "start": -120 } } }
```

**Extended end (seconds after):**
```json
{ "schedule": { "offsets": { "source": "show", "end": 300 } } }
```

#### Schedule Manual Recording
```
POST /guide/programs
```

```json
{
  "schedule": {
    "rule": "all",
    "offsets": { "start": 0, "end": 0, "source": "none" }
  },
  "schedule_rule": "all",
  "config": {
    "title": "Show Name",
    "channel_path": "/guide/channels/357025",
    "duration": 3600,
    "kind": "once",
    "once": {
      "year": 2024,
      "month": 9,
      "day": 15,
      "hour": 20,
      "minute": 0,
      "timezone": "America/New_York"
    }
  },
  "program": { "title": "Show Name" },
  "show_counts": {
    "airing_count": 1,
    "conflicted_count": 0,
    "scheduled_count": 1
  },
  "keep": { "rule": "none", "count": null },
  "recordings_path": null
}
```

## Data Structures

### Video Details
| Field | Type | Description |
|-------|------|-------------|
| state | string | `finished`, `recording`, `failed` |
| clean | boolean | Commercial skip ready |
| cloud | boolean | Cloud-stored |
| audio | string | `aac` or `ac3` |
| size | number | Bytes |
| duration | number | Seconds |
| width/height | number | Pixels |
| comskip.state | string | `ready`, `processing` |

### Airing Schedule
| Field | Type | Description |
|-------|------|-------------|
| state | string | `none`, `scheduled`, `conflicted` |
| qualifier | string | `none`, `user`, `series` |
| offsets.start | number | Seconds before |
| offsets.end | number | Seconds after |

### Show Counts
| Field | Type | Description |
|-------|------|-------------|
| airing_count | number | Total airings |
| conflicted_count | number | Schedule conflicts |
| scheduled_count | number | Scheduled to record |
| unwatched_count | number | Not yet watched |
| protected_count | number | Delete-protected |
| failed_count | number | Failed recordings |

## Error Responses

```json
{
  "error": {
    "code": "ERROR_CODE",
    "description": "Human-readable error message",
    "details": "Additional details"
  }
}
```

## HLS Streaming

Tablo streams use HLS (HTTP Live Streaming):
- **Live TV**: Transcoded on-device, served as .m3u8 playlist
- **Recordings**: Pre-encoded, accessible via watch endpoint
- **Older devices**: H.264 encoding (browser compatible)
- **Newer devices**: May use H.265/HEVC (requires FFmpeg transcoding)

### Deinterlacing
OTA broadcasts are typically 1080i (interlaced). For smooth browser playback:
```
ffmpeg -i input.m3u8 -vf yadif -c:v libx264 -preset ultrafast output.m3u8
```

## Best Practices

1. **Use batch endpoint** for efficiency (max 50 items per request)
2. **Limit concurrent requests** (semaphore of ~30)
3. **Cache guide data** (10+ minute TTL)
4. **Parallel fetch** channel details and airings
5. **Rewrite HLS manifests** for CORS proxy
6. **Handle 1080i** with deinterlacing filter

## Reference Projects

- [tablo-web](https://github.com/trevor-viljoen/tablo-web) - React + FastAPI + FFmpeg
- [tablo-tools-electron](https://github.com/jessedp/tablo-tools-electron) - Electron + React (archived)
- [Tablo_Lite](https://github.com/Epchk/Tablo_Lite) - Simple vanilla JS web UI
- [tablo-api-docs](https://jessedp.github.io/tablo-api-docs/) - Unofficial API documentation

# Reference Code Analysis

Analysis of three reference Tablo projects for porting to our Tauri + Vue.js app.

## Source Projects

| Project | Stack | Key Features |
|---------|-------|--------------|
| [tablo-web](https://github.com/trevor-viljoen/tablo-web) | FastAPI + React | Cloud auth, HLS proxy, FFmpeg transcoding |
| [tablo-tools-electron](https://github.com/jessedp/tablo-tools-electron) | Electron + React | Recording export, batch operations, NeDB |
| [Tablo_Lite](https://github.com/Epchk/Tablo_Lite) | Vanilla JS | Simplest API implementation |

## Key Findings

### Authentication (4th Gen Tablo)

**tablo-web** uses the `tablo-api` Python package which handles:
1. Cloud login to `lighthousetv.ewscloud.com`
2. HMAC-MD5 signed requests to local device
3. Bearer + Lighthouse tokens for cloud API

```python
# From tablo-web/backend/app/state.py
from tablo_api import TabloAuth, TabloClient

# Cloud auth headers
def _cloud_headers(self) -> tuple[str, dict]:
    dev = self.active_device
    return "https://lighthousetv.ewscloud.com", {
        "Authorization": f"Bearer {dev.account_token}",
        "Lighthouse": dev.lighthouse_token,
        "User-Agent": "Tablo-FAST/2.0.0 (Mobile; iPhone; iOS 16.6)",
    }

# Local device auth
async def request_device(self, method: str, path: str, body: str = "") -> dict:
    auth_header, date_header = TabloAuth.make_device_auth(method, path, body)

    url = self.active_device.local_url.rstrip("/") + path
    resp = await self._http.request(
        method,
        url,
        headers={
            "Authorization": auth_header,
            "Date": date_header,
            "User-Agent": "Tablo-FAST/1.7.0 (Mobile; iPhone; iOS 18.4)",
        }
    )
```

### Device Discovery

**tablo-tools-electron** uses the `tablo-api` npm package:
```typescript
// src/main/utils/Tablo.ts
import Tablo from 'tablo-api';

export const discover = async (): Promise<void> => {
  const devices = await globalThis.Api.discover();

  await asyncForEach(devices, async (device: any) => {
    if (device.via === 'broadcast') {
      // UDP discovery on port 8881
      globalThis.Api.device = device;
      const info = await globalThis.Api.get('/server/info');
    } else {
      // Cloud discovery via api.tablotv.com
      device.server_id = device.serverid;
    }
  });
};

// Connection check via TCP socket
export async function checkConnection(): Promise<boolean> {
  const client = new net.Socket();
  client.setTimeout(750);
  client.connect({ port: 8885, host: device.private_ip });
  // ...
}
```

### Core API Calls

**Tablo_Lite** shows the simplest implementation:

```javascript
// tablo_ota.js - Base configuration
var tablo_ipv4_addr = "192.168.1.12";
var tbaseurl = `http://${tablo_ipv4_addr}:8885`;

// GET request wrapper
async function tgetRESTData(url) {
    const response = await fetch(url);
    if (!response.ok) {
        const error_text = await response.json();
        throw new Error(error_text.error.description);
    }
    return response.json();
}

// POST request wrapper (for /watch endpoints)
async function tpostRESTData(url, data) {
    const response = await fetch(url, {
        method: "POST",
        cache: "no-cache",
        body: data
    });
    return response.json();
}

// Key API functions
function tgetServerInfo() {
    return tgetRESTData(`${tbaseurl}/server/info`);
}

function tgetChannelList() {
    return tgetRESTData(`${tbaseurl}/guide/channels`);
}

function tgetRecordings() {
    return tgetRESTData(`${tbaseurl}/recordings/airings`);
}

// Watch endpoint - returns HLS playlist URL
function tgetWatchDetails(path) {
    return tpostRESTData(`${tbaseurl}${path}/watch`, '');
}

// Batch endpoint - get details for up to 50 items
function tbatchDetails(list) {
    return tpostRESTData(`${tbaseurl}/batch`, JSON.stringify(list));
}
```

### HLS Streaming

**tablo-web** implements HLS proxying with manifest rewriting:

```python
# stream.py - HLS proxy with manifest rewriting
@router.get("/hls/{session_id}/{path:path}")
async def hls_proxy(session_id: str, path: str, request: Request):
    sess = state.get_session(session_id)

    if path == "playlist.m3u8":
        target_url = sess.stream.playlist_url
    else:
        target_url = sess.base_url + "/" + path.lstrip("/")

    async with state.http.stream("GET", target_url) as resp:
        if "mpegurl" in resp.headers.get("content-type", "").lower():
            body = await resp.aread()
            rewritten = _rewrite_manifest(body.decode(), session_id, target_url)
            return Response(content=rewritten, media_type="application/vnd.apple.mpegurl")
        # Stream segments directly
        async for chunk in resp.aiter_bytes():
            yield chunk

def _rewrite_manifest(manifest: str, session_id: str, playlist_url: str) -> str:
    playlist_base = playlist_url.rsplit("/", 1)[0] + "/"
    lines = []
    for line in manifest.splitlines():
        if line.startswith("#") or line == "":
            lines.append(line)
            continue
        # Rewrite segment URLs to proxy path
        if line.startswith(("http://", "https://")):
            parsed = urlparse(line)
            rel = parsed.path.lstrip("/")
        else:
            full_url = urljoin(playlist_base, line)
            rel = urlparse(full_url).path.lstrip("/")
        lines.append(f"/api/hls/{session_id}/{rel}")
    return "\n".join(lines)
```

### FFmpeg Transcoding

**tablo-web** handles live transcoding for browser compatibility:

```python
# stream.py - FFmpeg transcoding for 1080i deinterlacing
async def start_transcoder(session_id: str, input_url: str):
    session_dir = TRANSCODE_DIR / session_id
    session_dir.mkdir(exist_ok=True)

    cmd = [
        "ffmpeg", "-y",
        "-protocol_whitelist", "file,http,https,tcp,tls,crypto",
        "-i", input_url,
        # Deinterlace 1080i OTA broadcast
        "-vf", "yadif",
        "-c:v", "libx264", "-preset", "ultrafast", "-crf", "28",
        "-maxrate", "2000k", "-bufsize", "4000k",
        "-pix_fmt", "yuv420p", "-g", "60",
        "-c:a", "aac", "-b:a", "128k", "-ac", "2",
        "-f", "hls",
        "-hls_time", "6",
        "-hls_list_size", "6",
        "-hls_segment_filename", "%03d.ts",
        "-hls_flags", "delete_segments+independent_segments",
        "playlist.m3u8"
    ]

    proc = subprocess.Popen(cmd, cwd=session_dir)
```

**tablo-tools-electron** uses fluent-ffmpeg for recording export:

```typescript
// exportVideo.ts
import FfmpegCommand from 'fluent-ffmpeg';

export const exportVideo = async (airingData, actionOnDuplicate, progressCallback) => {
    const airing = await Airing.create(airingData);

    // Get playlist URL from Tablo
    const watchPath = await airing.watch();
    const input = watchPath.playlist_url;

    FfmpegCommand.setFfmpegPath(findFfmpegPath());

    airing.cmd = FfmpegCommand();
    airing.cmd
        .input(input)
        .output(outFile)
        .addOutputOptions(ffmpegOpts)
        .on('progress', (progress) => progressCallback(airing.object_id, progress))
        .on('end', () => progressCallback(airing.object_id, { finished: true }))
        .on('error', (err) => progressCallback(airing.object_id, { failed: true, failedMsg: err }))
        .run();
};
```

### Recording Data Structures

**tablo-tools-electron** Airing class shows all recording fields:

```typescript
// Airing.ts - Core recording data structure
class Airing {
    object_id: number;
    path: string;

    airing_details: {
        datetime: string;      // ISO 8601
        show_title: string;
        duration: number;      // seconds
        channel_path: string;
    };

    video_details: {
        duration: number;      // actual duration
        state: 'finished' | 'recording' | 'failed';
        size: number;         // bytes
        width: number;
        height: number;
        clean: boolean;       // commercial skip ready
    };

    episode?: {
        title: string;
        description: string;
        season_number: number;
        number: number;       // episode number
    };

    event?: {
        title: string;
        description: string;
    };

    user_info: {
        watched: boolean;
        protected: boolean;
        position: number;     // playback position in seconds
    };

    series_path?: string;
    movie_path?: string;
    sport_path?: string;

    // Methods
    async watch(): Promise<{ playlist_url: string }>;
    async delete(): Promise<void>;
    async setProtected(protect: boolean): Promise<boolean>;
    async setWatched(watched: boolean): Promise<boolean>;
}
```

### Guide/EPG Data

**tablo-web** shows guide enrichment with parallel fetching:

```python
# state.py - Guide data aggregation
async def get_guide_data(self) -> list[dict]:
    channels = await self.channels()
    logo_map, path_to_ident, channel_airing_map, cloud_airing_map = \
        await self._fetch_guide_enrichment()

    guide = []
    for c in channels:
        c_path = next((p for p, ident in path_to_ident.items()
                       if ident == c.identifier), None)
        current_program = (channel_airing_map.get(c_path)
                          if c_path else None) or cloud_airing_map.get(c.identifier)
        guide.append({
            "identifier": c.identifier,
            "call_sign": c.call_sign,
            "major": c.major,
            "minor": c.minor,
            "network": c.network,
            "kind": c.kind,
            "display_name": c.display_name,
            "logo_url": logo_map.get(c.identifier),
            "current_program": current_program,
        })
    return guide

async def _build_grid_enrichment(self, max_airings: int = 1000):
    # Fetch channel details and airings in parallel
    path_results = await asyncio.gather(
        self.request_device("GET", "/guide/channels"),
        self.request_device("GET", "/guide/airings"),
        self._fetch_cloud_data(),
        return_exceptions=True,
    )

    # Use semaphore for concurrent detail fetches
    sem = asyncio.Semaphore(30)
    async def fetch_detail(path):
        async with sem:
            return await self.request_device("GET", path)
```

## Scheduling Recordings

**Tablo_Lite** shows manual recording scheduling:

```javascript
// tablo_ota.js - Schedule a one-time recording
function tScheduleNewRecording(title, year, month, day, hour, minute, duration, channel_path) {
    var data = {
        "schedule": {
            "rule": "all",
            "offsets": {"start": 0, "end": 0, "source": "none"}
        },
        "schedule_rule": "all",
        "config": {
            "title": title,
            "channel_path": channel_path,
            "duration": duration * 60,  // Convert to seconds
            "kind": "once",
            "once": {
                "year": parseInt(year),
                "month": parseInt(month),
                "day": parseInt(day),
                "hour": parseInt(hour),
                "minute": parseInt(minute),
                "timezone": Intl.DateTimeFormat().resolvedOptions().timeZone
            }
        },
        "program": {"title": title},
        "show_counts": {"airing_count": 1, "conflicted_count": 0, "scheduled_count": 1},
        "keep": {"rule": "none", "count": null},
        "recordings_path": null
    };

    return tpostRESTData(`${tbaseurl}/guide/programs`, JSON.stringify(data));
}
```

## Key Patterns to Implement

1. **Cloud Auth Flow**: Login → device list → select device → get tokens
2. **Local Device Signing**: HMAC-MD5 signatures for local API
3. **Batch Operations**: Use `/batch` endpoint for efficiency (max 50 items)
4. **HLS Proxy**: Rewrite manifests for CORS, optionally transcode
5. **Parallel Fetching**: Semaphore-limited concurrent requests
6. **Pagination**: Channels/recordings return paths, batch-fetch details
7. **Watch Sessions**: POST to `{path}/watch` returns playlist URL

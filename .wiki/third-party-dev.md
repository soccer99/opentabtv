# Third-Party App Development for Tablo

## Official Status

Tablo does not provide official public API documentation. However:
- An NDA can be signed with Nuvyyo for access to real documentation
- Community has reverse-engineered the API extensively
- Legacy (pre-4th Gen) devices have an open local API
- 4th Gen devices require cloud authentication

## API Documentation Resources

### Primary Resources

| Resource | URL | Notes |
|----------|-----|-------|
| Unofficial API Docs | https://jessedp.github.io/tablo-api-docs/ | Comprehensive legacy API docs |
| TabloTV Community Forum | https://community.tablotv.com/c/third-party-apps/ | Official third-party apps forum |
| tablo-api (Python) | https://pypi.org/project/tablo-api/ | 4th Gen cloud auth wrapper |
| tablo-api-js (Node.js) | https://github.com/jessedp/tablo-api-js | Legacy API wrapper |
| @gibme/tablo.tv | https://github.com/gibme-npm/tablo.tv | Full-featured TypeScript API |

### Reference Implementations

| Project | Language | Features |
|---------|----------|----------|
| [tablo-web](https://github.com/trevor-viljoen/tablo-web) | Python/React | 4th Gen, FFmpeg transcoding, Plex integration |
| [tablo-tools-electron](https://github.com/jessedp/tablo-tools-electron) | TypeScript/Electron | Legacy, export/delete, live TV |
| [Tablo_Lite](https://github.com/Epchk/Tablo_Lite) | JavaScript | Simple web UI, vanilla JS |
| [script.tablo](https://github.com/Nuvyyo/script.tablo) | Python | Official Kodi addon (reference impl) |
| [tablo2plex](https://github.com/hearhellacopters/tablo2plex) | Python | HMAC-MD5 signing reverse engineering |
| [tablo_downloader](https://github.com/kjwilder/tablo_downloader) | Python | CLI download tool |

## API Differences: Legacy vs 4th Gen

### Legacy Devices (Pre-4th Gen)

No authentication required for local API:

```bash
# Discovery
curl "https://api.tablotv.com/assocserver/getipinfo/"

# Device info
curl "http://192.168.1.100:8885/server/info"

# Channels
curl "http://192.168.1.100:8885/guide/channels"

# Watch channel
curl -X POST "http://192.168.1.100:8885/guide/channels/612649/watch"
```

### 4th Gen Devices

Cloud authentication required, then HMAC-MD5 signed local requests:

```python
from tablo_api import TabloAuth, TabloClient

# 1. Cloud authentication
auth = TabloAuth("email@example.com", "password")
devices = auth.discover()

# 2. Connect to device
client = TabloClient(devices[0])

# 3. Get channels (from cloud guide)
channels = client.channels()

# 4. Watch channel (HMAC-MD5 signed local request)
stream = client.watch(channels[0])
print(stream.playlist_url)  # HLS URL
```

## Authentication Flow (4th Gen)

```
┌─────────────────────────────────────────────────────────────────┐
│                    Authentication Flow                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  1. POST /api/v2/login/                                         │
│     → lighthousetv.ewscloud.com                                 │
│     ← Bearer token                                               │
│                                                                  │
│  2. GET /api/v2/account/                                        │
│     → lighthousetv.ewscloud.com                                 │
│     ← Device list with local IPs                                │
│                                                                  │
│  3. POST /api/v2/account/select/                                │
│     → lighthousetv.ewscloud.com                                 │
│     ← Device-scoped Lighthouse token                            │
│                                                                  │
│  4. GET /api/v2/account/{token}/guide/channels/                 │
│     → lighthousetv.ewscloud.com                                 │
│     ← Channel guide data                                         │
│                                                                  │
│  5. POST http://{local_ip}:8885/guide/channels/{id}/watch      │
│     → Local device (HMAC-MD5 signed)                            │
│     ← HLS playlist URL                                           │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## HMAC-MD5 Signing (4th Gen)

The HMAC-MD5 signing scheme was reverse-engineered from the Tablo iOS app:

```python
import hmac
import hashlib
from datetime import datetime

def make_device_auth(method: str, path: str, body: str = "") -> tuple[str, str]:
    """Generate HMAC-MD5 auth headers for local device requests."""
    # Static keys from Tablo app (not cryptographically secret)
    ACCESS_KEY = "your-access-key"  # From reverse engineering
    SECRET_KEY = "your-secret-key"  # From reverse engineering

    # RFC 1123 date
    date = datetime.utcnow().strftime("%a, %d %b %Y %H:%M:%S GMT")

    # String to sign
    string_to_sign = f"{method}\n{path}\n{body}\n{date}"

    # HMAC-MD5 signature
    signature = hmac.new(
        SECRET_KEY.encode(),
        string_to_sign.encode(),
        hashlib.md5
    ).hexdigest()

    auth_header = f"TABLO {ACCESS_KEY}:{signature}"
    return auth_header, date
```

**Note:** Access/secret keys are embedded in official apps and must be extracted. They are static values, not per-user secrets.

## Legacy API Quick Reference

### Discovery

```bash
# Cloud discovery
GET https://api.tablotv.com/assocserver/getipinfo/

# UDP discovery (port 8881)
# Send broadcast, listen on port 8882
```

### Server

```bash
GET /server/info              # Device info
GET /server/capabilities      # Feature list
GET /server/guide/status      # Guide status
GET /server/harddrives        # Storage info
GET /server/tuners            # Tuner status
```

### Guide

```bash
GET /guide/channels           # All channels
GET /guide/channels/{id}      # Single channel
GET /guide/airings            # All upcoming airings
GET /guide/shows              # All shows in guide
GET /guide/movies             # All movies
GET /guide/series             # All series
GET /guide/sports             # All sports
```

### Recordings

```bash
GET /recordings/airings       # All recorded airings
GET /recordings/shows         # All recorded shows
GET /recordings/movies        # All recorded movies
GET /recordings/series        # All recorded series
```

### Watching

```bash
POST /{path}/watch            # Get HLS playlist URL
# Returns: { "playlist_url": "http://.../stream/pl.m3u8?token" }
```

### Scheduling

```bash
PATCH /guide/movies/airings/{id}  -d '{"scheduled": true}'
PATCH /guide/series/{id}          -d '{"schedule": "new"}'  # or "all" or "none"
```

### Modifying Recordings

```bash
PATCH /recordings/series/episodes/{id}  -d '{"watched": true}'
PATCH /recordings/series/episodes/{id}  -d '{"protected": true}'
DELETE /recordings/series/episodes/{id}
```

### Batch Operations

```bash
POST /batch -d '["/recordings/series/episodes/123", "/recordings/movies/456"]'
```

## Tools & Scripts

### Python Tools

```bash
# tablo-api (4th Gen)
pip install tablo-api

# tablo_downloader (Legacy)
pip install tablo_downloader
```

### Node.js Tools

```bash
# Legacy API
npm install tablo-api

# Full-featured (requires keys)
npm install @gibme/tablo.tv
```

### Third-Party Apps

| App | Platform | Status | Notes |
|-----|----------|--------|-------|
| Tablo Tools | Win/Mac/Linux | Archived | Electron, legacy only |
| Tablo Ripper | Windows | Discontinued | Closed source |
| Tablo Exporter | Java | Active | Export recordings |
| APL | Java | Active | Export recordings |
| SurLaTablo | Python | Legacy | CLI export |
| tut | Python | Active | CLI tool |

## Security Considerations

1. **Credentials**: Never store in code; use system keychain
2. **Cloud Traffic**: Always HTTPS
3. **Local Traffic**: Plain HTTP on port 8885 - LAN only
4. **HMAC Keys**: Static (not secret), but required for 4th Gen
5. **Sessions**: Watch URLs expire (~20 minutes)

## Community Resources

- [TabloTV Community Forum](https://community.tablotv.com/)
- [Third Party Apps Forum](https://community.tablotv.com/c/third-party-apps/)
- [Unofficial API Docs GitHub](https://github.com/jessedp/tablo-api-docs)

## Known Limitations

1. **4th Gen**: Cloud auth required, no offline mode
2. **OTT Channels**: May have restrictions
3. **DRM**: Some content may be protected
4. **Rate Limits**: Unknown, be respectful
5. **API Changes**: Unofficial, may break without notice

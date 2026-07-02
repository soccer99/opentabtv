// Device types

/** Device generation - Legacy (pre-4th Gen) or Gen4 (2024+) */
export type DeviceGeneration = "legacy" | "gen4";

export interface TabloDevice {
  id: string;
  name: string;
  localIp: string;
  model?: string;
  version?: string;
  tuners?: number;
  serverId?: string;
  /** Device generation (legacy or gen4) */
  generation: DeviceGeneration;
  /** 4th Gen: Device SID from cloud API */
  sid?: string;
  /** 4th Gen: Account bearer token from Lighthouse login */
  accountToken?: string;
  /** 4th Gen: Device-scoped token from Lighthouse select */
  lighthouseToken?: string;
}

/** Tablo account information from cloud login */
export interface TabloAccount {
  email: string;
  token: string;
  devices: CloudDevice[];
}

/** Device info from cloud API */
export interface CloudDevice {
  sid: string;
  name: string;
  localIp?: string;
  model?: string;
}

/** Login credentials for 4th Gen cloud auth */
export interface LoginCredentials {
  email: string;
  password: string;
}

export type ConnectionState = "disconnected" | "connecting" | "connected";

// Channel types
export interface Channel {
  id: string;
  objectId: number;
  callSign: string;
  major: number;
  minor: number;
  network: string | null;
  resolution: "hd_1080" | "hd_720" | "sd";
  favourite: boolean;
}

// Guide types
export interface Program {
  id: string;
  title: string;
  description?: string;
  startTime: Date;
  endTime: Date;
  duration: number;
  channelId: string;
  type: "series" | "movie" | "sports" | "program";
  episode?: {
    number: number;
    seasonNumber: number;
    title: string;
  };
  genres?: string[];
}

// Recording types
export type RecordingType = "series" | "movie" | "sports" | "program" | "manual" | "unknown";
export type RecordingState = "finished" | "recording" | "failed" | "unknown";

export interface Recording {
  id: string;
  path: string;
  title: string;
  recordingType: RecordingType;
  description?: string;
  recordedAt: Date;
  duration: number;
  size: number;
  channelId: string;
  episode?: {
    number: number;
    seasonNumber: number;
    title?: string;
  };
  videoDetails: {
    width: number;
    height: number;
    state: RecordingState;
    hasComskip: boolean;
  };
  userInfo: {
    watched: boolean;
    protected: boolean;
    position: number;
  };
}

// Playback types
export interface StreamInfo {
  url: string;
  token: string;
  expires: Date;
  transcoded: boolean;
}

// API response types
export interface ApiError {
  code: string;
  message: string;
  details?: unknown;
}

export interface ApiResult<T> {
  data?: T;
  error?: ApiError;
}

// Casting types
export type CastDeviceType = "chromecast";

export interface CastDevice {
  id: string;
  name: string;
  ip: string;
  port: number;
  deviceType: CastDeviceType;
  model?: string;
}

export type CastState = "idle" | "discovering" | "connecting" | "casting";

export interface CastSession {
  device: CastDevice;
  mediaUrl: string;
  title?: string;
  isPlaying: boolean;
}

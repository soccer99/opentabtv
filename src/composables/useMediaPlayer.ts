import { ref, onUnmounted, watch, type Ref } from "vue";
import Hls from "hls.js";

export interface MediaPlayerState {
  isPlaying: boolean;
  isPaused: boolean;
  isBuffering: boolean;
  isMuted: boolean;
  volume: number;
  currentTime: number;
  duration: number;
  error: string | null;
}

export interface MediaPlayerControls {
  play: () => void;
  pause: () => void;
  togglePlay: () => void;
  setVolume: (volume: number) => void;
  toggleMute: () => void;
  seek: (time: number) => void;
  requestFullscreen: () => void;
  exitFullscreen: () => void;
  toggleFullscreen: () => void;
}

export function useMediaPlayer(videoRef: Ref<HTMLVideoElement | null>) {
  const state = ref<MediaPlayerState>({
    isPlaying: false,
    isPaused: true,
    isBuffering: false,
    isMuted: false,
    volume: 1,
    currentTime: 0,
    duration: 0,
    error: null,
  });

  let hls: Hls | null = null;
  let isFullscreen = ref(false);

  // Initialize HLS.js when video element is available
  function initHls(): Hls | null {
    if (!Hls.isSupported()) {
      console.warn("HLS.js is not supported in this browser");
      return null;
    }

    const instance = new Hls({
      enableWorker: true,
      lowLatencyMode: false,
      backBufferLength: 30,
      maxBufferLength: 30,
      maxMaxBufferLength: 60,
      // Live stream optimizations
      liveSyncDurationCount: 3,
      liveMaxLatencyDurationCount: 10,
    });

    instance.on(Hls.Events.ERROR, (_event, data) => {
      if (data.fatal) {
        switch (data.type) {
          case Hls.ErrorTypes.NETWORK_ERROR:
            state.value.error = "Network error - trying to recover";
            instance.startLoad();
            break;
          case Hls.ErrorTypes.MEDIA_ERROR:
            state.value.error = "Media error - trying to recover";
            instance.recoverMediaError();
            break;
          default:
            state.value.error = `Fatal error: ${data.details}`;
            destroy();
            break;
        }
      }
    });

    instance.on(Hls.Events.MANIFEST_PARSED, () => {
      state.value.error = null;
      videoRef.value?.play().catch((e) => {
        console.warn("Autoplay prevented:", e);
      });
    });

    instance.on(Hls.Events.BUFFER_APPENDING, () => {
      state.value.isBuffering = true;
    });

    instance.on(Hls.Events.FRAG_BUFFERED, () => {
      state.value.isBuffering = false;
    });

    return instance;
  }

  // Load a stream URL
  function loadSource(url: string): void {
    const video = videoRef.value;
    if (!video) return;

    // Clean up existing HLS instance
    if (hls) {
      hls.destroy();
      hls = null;
    }

    state.value.error = null;
    state.value.isBuffering = true;

    if (Hls.isSupported()) {
      hls = initHls();
      if (hls) {
        hls.loadSource(url);
        hls.attachMedia(video);
      }
    } else if (video.canPlayType("application/vnd.apple.mpegurl")) {
      // Native HLS support (Safari)
      video.src = url;
      video.load();
    } else {
      state.value.error = "HLS playback not supported";
    }
  }

  // Set up video element event listeners
  function setupVideoEvents(video: HTMLVideoElement): void {
    video.addEventListener("play", () => {
      state.value.isPlaying = true;
      state.value.isPaused = false;
    });

    video.addEventListener("pause", () => {
      state.value.isPlaying = false;
      state.value.isPaused = true;
    });

    video.addEventListener("waiting", () => {
      state.value.isBuffering = true;
    });

    video.addEventListener("canplay", () => {
      state.value.isBuffering = false;
    });

    video.addEventListener("timeupdate", () => {
      state.value.currentTime = video.currentTime;
    });

    video.addEventListener("durationchange", () => {
      state.value.duration = video.duration;
    });

    video.addEventListener("volumechange", () => {
      state.value.volume = video.volume;
      state.value.isMuted = video.muted;
    });

    video.addEventListener("error", () => {
      const error = video.error;
      state.value.error = error ? `Video error: ${error.message}` : "Unknown video error";
    });
  }

  // Watch for video ref changes
  watch(
    videoRef,
    (video) => {
      if (video) {
        setupVideoEvents(video);
      }
    },
    { immediate: true }
  );

  // Controls
  const controls: MediaPlayerControls = {
    play() {
      videoRef.value?.play().catch(console.error);
    },
    pause() {
      videoRef.value?.pause();
    },
    togglePlay() {
      if (state.value.isPlaying) {
        controls.pause();
      } else {
        controls.play();
      }
    },
    setVolume(volume: number) {
      if (videoRef.value) {
        videoRef.value.volume = Math.max(0, Math.min(1, volume));
      }
    },
    toggleMute() {
      if (videoRef.value) {
        videoRef.value.muted = !videoRef.value.muted;
      }
    },
    seek(time: number) {
      if (videoRef.value && isFinite(time)) {
        videoRef.value.currentTime = time;
      }
    },
    requestFullscreen() {
      const container = videoRef.value?.parentElement;
      if (container?.requestFullscreen) {
        container.requestFullscreen();
        isFullscreen.value = true;
      }
    },
    exitFullscreen() {
      if (document.fullscreenElement) {
        document.exitFullscreen();
        isFullscreen.value = false;
      }
    },
    toggleFullscreen() {
      if (isFullscreen.value) {
        controls.exitFullscreen();
      } else {
        controls.requestFullscreen();
      }
    },
  };

  // Cleanup
  function destroy(): void {
    if (hls) {
      hls.destroy();
      hls = null;
    }
  }

  onUnmounted(destroy);

  // Listen for fullscreen changes
  document.addEventListener("fullscreenchange", () => {
    isFullscreen.value = !!document.fullscreenElement;
  });

  return {
    state,
    isFullscreen,
    controls,
    loadSource,
    destroy,
  };
}

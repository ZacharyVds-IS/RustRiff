import {createTauriTest} from "@srsholmes/tauri-playwright";
import type {AmpConfigDto, AudioSettingsDto} from "../src/domain/types";

// ── Binary path ────────────────────────────────────────────────────────────
// Set TAURI_BINARY to override; defaults to the release build location.
// Build the binary first:
//   npm run tauri build -- --no-bundle --features e2e-testing
const IS_WIN = process.platform === "win32";
const DEFAULT_BINARY = IS_WIN
  ? "./src-tauri/target/release/rustriff.exe"
  : "./src-tauri/target/release/rustriff";

const TAURI_BINARY = process.env.TAURI_BINARY ?? DEFAULT_BINARY;

// ── Minimal IPC stubs for browser-only mode ─────────────────────────────
// These prevent the real Tauri backend from being called in headless Chrome.
const mockAudioSettings: AudioSettingsDto = {
  input_device_name: "Mock Input",
  output_device_name: "Mock Output",
  input_sample_rate: 44100,
  output_sample_rate: 44100,
  input_channels: 2,
  output_channels: 2,
  audio_driver: "Mock",
};

const mockAmpConfig: AmpConfigDto = {
  master_volume: 0.8,
  is_active: true,
  channels: [
    {
      id: "mock-channel-id",
      name: "Clean",
      gain: 0.5,
      tone_stack: { bass: 0.5, middle: 0.5, treble: 0.5 },
      volume: 0.8,
      effect_chain: [],
    },
  ],
  current_channel: "mock-channel-id",
  audio_settings: mockAudioSettings,
};

export const { test, expect } = createTauriTest({
  // URL used in browser-only mode (Vite dev server must be running)
  devUrl: "http://localhost:1420",

  // ── IPC mocks (browser-only mode only) ──────────────────────────────────
  ipcMocks: {
    get_amp_config: () => mockAmpConfig,
    toggle_on_off: () => null,

    get_available_audio_drivers: () => ["Mock"],
    get_selected_audio_driver: () => "Mock",
    get_input_device_list: () => [],
    get_output_device_list: () => [],
    get_input_channel_options: () => [],
    get_output_channel_options: () => [],
    get_selected_input_channel_count: () => 2,
    get_selected_output_channel_count: () => 2,
    get_buffer_size_frames: () => 512,

    get_all_channels: () => [
      {
        id: "mock-channel-id",
        name: "Clean",
        gain: 0.5,
        tone_stack: { bass: 0.5, middle: 0.5, treble: 0.5 },
        volume: 0.8,
        effect_chain: [],
      },
    ],
    get_channel_id: () => "mock-channel-id",

    get_all_ir_profiles: () => [],
    get_default_ir_file: () => null,

    get_spectrum_contract: () => ({
      live_spectrum_event: "spectrum",
      min_db: -90,
      max_db: 0,
      min_frequency_hz: 20,
      max_frequency_hz: 20000,
    }),
    get_tuner_contract: () => ({ live_tuner_event: "tuner" }),

    set_gain: () => null,
    set_master_volume: () => null,
    set_bass: () => null,
    set_middle: () => null,
    set_treble: () => null,
    set_volume: () => null,
    set_tone_stack: () => null,
    set_channel_id: () => null,
    add_channel: () => null,
    remove_channel: () => null,
    add_effect: () => null,
    remove_effect: () => null,
    toggle_effect: () => null,
    apply_effect_order_change: () => null,
    set_audio_driver: () => null,
    set_input_device: () => null,
    set_output_device: () => null,
    set_buffer_size_frames: () => null,
  },

  // ── Tauri native mode ────────────────────────────────────────────────────
  // The fixture will spawn this binary and wait for the plugin socket.
  // Socket path (Linux/macOS). On Windows the plugin falls back to TCP.
  mcpSocket: "/tmp/tauri-playwright.sock",
  tauriCommand: TAURI_BINARY,
  tauriCwd: process.cwd(),
  startTimeout: 60,
});


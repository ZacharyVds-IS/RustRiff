import {createTauriTest} from "@srsholmes/tauri-playwright";

// ── Binary path ────────────────────────────────────────────────────────────
// Set TAURI_BINARY to override; defaults to the release build location.
// Build the binary first:
//   npm run tauri build -- --no-bundle --features e2e-testing
const IS_WIN = process.platform === "win32";
const DEFAULT_BINARY = IS_WIN
  ? "./src-tauri/target/release/rustriff.exe"
  : "./src-tauri/target/release/rustriff";

const TAURI_BINARY = process.env.TAURI_BINARY ?? DEFAULT_BINARY;

type IpcInvocation = {
  command: string;
  args: unknown[];
};

type MockEffect = {
  kind?: string;
  data?: {id?: string; [key: string]: unknown};
  [key: string]: unknown;
};

type MockAmpConfig = {
  master_volume: number;
  is_active: boolean;
  channels: Array<{
    id: string;
    name: string;
    gain: number;
    tone_stack: {bass: number; middle: number; treble: number};
    volume: number;
    effect_chain: MockEffect[];
  }>;
  current_channel: string;
  audio_settings: {
    input_device_name: string;
    output_device_name: string;
    input_sample_rate: number;
    output_sample_rate: number;
    input_channels: number;
    output_channels: number;
    audio_driver: string;
  };
};

const createInitialAmpConfig = (): MockAmpConfig => ({
  master_volume: 0.8,
  is_active: true,
  channels: [
    {
      id: "mock-channel-id",
      name: "Clean",
      gain: 0.5,
      tone_stack: {bass: 0.5, middle: 0.5, treble: 0.5},
      volume: 0.8,
      effect_chain: [],
    },
  ],
  current_channel: "mock-channel-id",
  audio_settings: {
    input_device_name: "Mock Input",
    output_device_name: "Mock Output",
    input_sample_rate: 48_000,
    output_sample_rate: 48_000,
    input_channels: 2,
    output_channels: 2,
    audio_driver: "Mock",
  },
});

let mockAmpConfig = createInitialAmpConfig();
let nextMockEffectId = 1;
const ipcInvocations: IpcInvocation[] = [];

const clone = <T>(value: T): T => JSON.parse(JSON.stringify(value)) as T;

const recordIpcInvocation = (command: string, args: unknown[]) => {
  ipcInvocations.push({command, args: clone(args)});
};

export const resetIpcMockState = () => {
  mockAmpConfig = createInitialAmpConfig();
  nextMockEffectId = 1;
  ipcInvocations.length = 0;
};

// ── IPC stubs for browser-only mode ─────────────────────────────────────
// All entries are intentional interceptors — they replace real Tauri IPC calls
// so browser-only tests work without a native backend. The runtime consumer is
// the createTauriTest fixture which reads each key dynamically at runtime.
// noinspection JSUnusedGlobalSymbols
const ipcMocks: Record<string, (...args: unknown[]) => unknown> = {
  get_amp_config: () => clone(mockAmpConfig),
  toggle_on_off: () => null,

  get_available_audio_drivers: () => ["Mock"],
  get_selected_audio_driver: () => "Mock",
  get_input_device_list: () => [
    {id: "mock-in-1", name: "Mock Input 1", sample_rate: 48_000},
  ],
  get_output_device_list: () => [
    {id: "mock-out-1", name: "Mock Output 1", sample_rate: 48_000},
  ],
  get_input_channel_options: () => [],
  get_output_channel_options: () => [],
  get_selected_input_channel_count: () => 2,
  get_selected_output_channel_count: () => 2,
  get_buffer_size_frames: () => 256,
  measure_buffer_latency: () => ({
    input_buffer_latency_ms: 2.67,
    output_buffer_latency_ms: 2.67,
    total_buffer_latency_ms: 5.34,
  }),
  measure_round_trip_latency: () => ({
    is_valid: true,
    latency_ms: 9.42,
    error: null,
  }),
  measure_all_dsp_algorithmic_latency: () => [
    {processor_name: "Volume", latency_ms: 0.2, latency_samples: 9},
    {processor_name: "Gain", latency_ms: 0.1, latency_samples: 4},
    {processor_name: "Tone Stack", latency_ms: 0.0, latency_samples: 0},
    {processor_name: "Master Volume", latency_ms: 0.0, latency_samples: 0},
  ],
  measure_all_dsp_cpu_timings: () => [
    {processor_name: "Volume", execution_us_per_sample: 0.012},
    {processor_name: "Gain", execution_us_per_sample: 0.021},
    {processor_name: "Tone Stack", execution_us_per_sample: 0.034},
    {processor_name: "Master Volume", execution_us_per_sample: 0.006},
  ],

  get_all_channels: () => [
    {
      id: "mock-channel-id",
      name: "Clean",
      gain: 0.5,
      tone_stack: {bass: 0.5, middle: 0.5, treble: 0.5},
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
    max_frequency_hz: 20_000,
  }),
  get_tuner_contract: () => ({live_tuner_event: "tuner"}),
  start_live_tuner_stream: () => null,
  stop_live_tuner_stream: () => null,
  get_midi_inputs: () => [],
  connect_midi_device: () => null,
  disconnect_midi_device: () => null,

  // MIDI settings/mapping screens rely on these commands; return safe defaults for browser-only runs.
  get_midi_inputs: () => [],
  connect_midi_device: () => null,
  disconnect_midi_device: () => null,
  get_midi_bindings: () => [],
  register_midi_binding: () => null,
  remove_midi_binding: () => null,

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
  add_effect: (...args: unknown[]) => {
    recordIpcInvocation("add_effect", args);

    const [params] = args as [{effectDto?: {kind?: string; data?: {id?: string}}}?];
    const effectToAdd = params?.effectDto ? clone(params.effectDto) : undefined;
    const currentChannel = mockAmpConfig.channels.find(
      (channel) => channel.id === mockAmpConfig.current_channel,
    );

    if (effectToAdd && currentChannel) {
      if (effectToAdd.data?.id === "0") {
        effectToAdd.data.id = `mock-effect-${nextMockEffectId++}`;
      }
      currentChannel.effect_chain.push(effectToAdd);
    }

    return null;
  },
  remove_effect: () => null,
  toggle_effect: () => null,
  apply_effect_order_change: () => null,
  set_audio_driver: () => null,
  set_asio_channel_config: () => null,
  set_input_device: () => null,
  set_output_device: () => null,
  set_buffer_size_frames: () => null,
};

export const {test, expect} = createTauriTest({
  // URL used in browser-only mode — Playwright manages the Vite dev server.
  devUrl: "http://127.0.0.1:1420",

  ipcMocks,

  // ── Tauri native mode ────────────────────────────────────────────────────
  // The fixture spawns the pre-built binary and waits for the plugin socket.
  // Windows falls back to TCP; Linux/macOS use the socket path below.
  mcpSocket: "/tmp/tauri-playwright.sock",
  tauriCommand: TAURI_BINARY,
  tauriCwd: process.cwd(),
  startTimeout: 60,
});

/* tslint:disable */
/* eslint-disable */

export class WasmSession {
    free(): void;
    [Symbol.dispose](): void;
    is_over(): boolean;
    constructor();
    send(input: string): string;
}

/**
 * A wasm-bindgen-exposed TUI session backed by ratatui + WasmBackend.
 * Returns ANSI escape sequences; pass directly to `term.write()` in xterm.js.
 */
export class WasmTuiSession {
    free(): void;
    [Symbol.dispose](): void;
    is_over(): boolean;
    constructor();
    resize(cols: number, rows: number): string;
    send(input: string): string;
    send_key(key: string): string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_wasmsession_free: (a: number, b: number) => void;
    readonly __wbg_wasmtuisession_free: (a: number, b: number) => void;
    readonly wasmsession_is_over: (a: number) => number;
    readonly wasmsession_new: () => number;
    readonly wasmsession_send: (a: number, b: number, c: number) => [number, number];
    readonly wasmtuisession_new: () => number;
    readonly wasmtuisession_resize: (a: number, b: number, c: number) => [number, number];
    readonly wasmtuisession_send: (a: number, b: number, c: number) => [number, number];
    readonly wasmtuisession_send_key: (a: number, b: number, c: number) => [number, number];
    readonly wasmtuisession_is_over: (a: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;

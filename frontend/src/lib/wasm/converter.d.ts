/* tslint:disable */
/* eslint-disable */
export function init(): void;
export function convert_bin_to_uf2(bin: Uint8Array, slot: number, name: string, autoclock_hz: number): ConversionOutput;
export class ConversionOutput {
  private constructor();
  free(): void;
  readonly data: Uint8Array;
  readonly startOffset: number;
  readonly slot: number;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_conversionoutput_free: (a: number, b: number) => void;
  readonly conversionoutput_data: (a: number) => any;
  readonly conversionoutput_startOffset: (a: number) => number;
  readonly conversionoutput_slot: (a: number) => number;
  readonly init: () => void;
  readonly convert_bin_to_uf2: (a: any, b: number, c: number, d: number, e: number) => [number, number, number];
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_3: WebAssembly.Table;
  readonly __externref_table_dealloc: (a: number) => void;
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

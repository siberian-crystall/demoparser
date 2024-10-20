/* tslint:disable */
/* eslint-disable */
/**
 * @param {Uint8Array} file
 * @param {string | undefined} [event_name]
 * @param {any[] | undefined} [wanted_player_props]
 * @param {any[] | undefined} [wanted_other_props]
 * @returns {any}
 */
export function parseEvent(file: Uint8Array, event_name?: string, wanted_player_props?: any[], wanted_other_props?: any[]): any;
/**
 * @param {Uint8Array} file
 * @param {any[] | undefined} [event_names]
 * @param {any[] | undefined} [wanted_player_props]
 * @param {any[] | undefined} [wanted_other_props]
 * @returns {any}
 */
export function parseEvents(file: Uint8Array, event_names?: any[], wanted_player_props?: any[], wanted_other_props?: any[]): any;
/**
 * @param {Uint8Array} fileBytes
 * @returns {any}
 */
export function listGameEvents(fileBytes: Uint8Array): any;
/**
 * @param {Uint8Array} file
 * @param {any[] | undefined} [wanted_props]
 * @param {Int32Array | undefined} [wanted_ticks]
 * @param {any[] | undefined} [wanted_players]
 * @param {boolean | undefined} [struct_of_arrays]
 * @returns {any}
 */
export function parseTicks(file: Uint8Array, wanted_props?: any[], wanted_ticks?: Int32Array, wanted_players?: any[], struct_of_arrays?: boolean): any;
/**
 * @param {Uint8Array} file
 * @returns {any}
 */
export function parseGrenades(file: Uint8Array): any;
/**
 * @param {Uint8Array} file
 * @returns {any}
 */
export function parseHeader(file: Uint8Array): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly parseEvent: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
  readonly parseEvents: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
  readonly listGameEvents: (a: number, b: number, c: number) => void;
  readonly parseTicks: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => void;
  readonly parseGrenades: (a: number, b: number, c: number) => void;
  readonly parseHeader: (a: number, b: number, c: number) => void;
  readonly __wbindgen_export_0: (a: number, b: number) => number;
  readonly __wbindgen_export_1: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_export_2: (a: number) => void;
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

import { dlopen, ptr, suffix } from "bun:ffi";
import { MinaCalcError, type CalcConfig, type CalcMode, DEFAULT_CONFIG, type DetailedResult, type Note, type SkillsetScores, modeValue, packConfig, packNotes, readScore, validate } from "./shared.js";

export * from "./shared.js";

const path = process.env.MINACALC_LIBRARY_PATH ?? `libminacalc_bindings.${suffix}`;
const library = dlopen(path, {
  minacalc_version: { args: [], returns: "i32" },
  minacalc_calc_at_rate: { args: ["ptr", "usize", "f32", "f32", "u32", "i32", "ptr"], returns: "i32" },
  minacalc_calc_all_rates: { args: ["ptr", "usize", "u32", "i32", "ptr"], returns: "i32" },
  minacalc_calc_at_rate_with_config: { args: ["ptr", "usize", "f32", "f32", "u32", "i32", "ptr", "ptr"], returns: "i32" },
  minacalc_calc_rates: { args: ["ptr", "usize", "ptr", "usize", "u32", "i32", "ptr", "ptr"], returns: "i32" },
});
function check(status: number): void { if (status !== 0) throw new MinaCalcError(`native calculation failed (status ${status})`, status); }
export function version(): number { return library.symbols.minacalc_version(); }
export function close(): void { library.close(); }
export function calcAtRate(notes: readonly Note[], rate: number, goal = 0.93, keys = 4, mode: CalcMode = "ssr"): SkillsetScores {
  validate(notes, keys); const input = new Uint8Array(packNotes(notes)); const output = new Uint8Array(32);
  check(library.symbols.minacalc_calc_at_rate(ptr(input), notes.length, rate, goal, keys, modeValue(mode), ptr(output)));
  return readScore(new DataView(output.buffer));
}
export function calcAllRates(notes: readonly Note[], keys = 4, mode: CalcMode = "msd"): readonly SkillsetScores[] {
  validate(notes, keys); const input = new Uint8Array(packNotes(notes)); const output = new Uint8Array(14 * 32);
  check(library.symbols.minacalc_calc_all_rates(ptr(input), notes.length, keys, modeValue(mode), ptr(output)));
  const view = new DataView(output.buffer); return Array.from({ length: 14 }, (_, index) => readScore(view, index * 32));
}
export function calcAtRateDetailed(notes: readonly Note[], rate: number, goal = 0.93, keys = 4, mode: CalcMode = "ssr", config: CalcConfig = DEFAULT_CONFIG): DetailedResult {
  validate(notes, keys); const input = new Uint8Array(packNotes(notes)); const cfg = new Uint8Array(packConfig(config)); const output = new Uint8Array(36);
  check(library.symbols.minacalc_calc_at_rate_with_config(ptr(input), notes.length, rate, goal, keys, modeValue(mode), ptr(cfg), ptr(output)));
  const view = new DataView(output.buffer); return { scores: readScore(view), grindScaler: view.getFloat32(32, true) };
}
export function calcRates(notes: readonly Note[], rates: readonly number[], keys = 4, mode: CalcMode = "msd", config: CalcConfig = DEFAULT_CONFIG): readonly SkillsetScores[] {
  validate(notes, keys); if (!rates.length) throw new MinaCalcError("rates must not be empty", 3); const input = new Uint8Array(packNotes(notes)); const rateArray = new Float32Array(rates); const cfg = new Uint8Array(packConfig(config)); const output = new Uint8Array(rates.length * 32);
  check(library.symbols.minacalc_calc_rates(ptr(input), notes.length, ptr(rateArray), rates.length, keys, modeValue(mode), ptr(cfg), ptr(output))); const view = new DataView(output.buffer); return rates.map((_, i) => readScore(view, i * 32));
}

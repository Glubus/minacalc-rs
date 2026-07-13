import { dlopen, ptr, suffix } from "bun:ffi";
import { MinaCalcError, type CalcMode, type Note, type SkillsetScores, modeValue, packNotes, readScore, validate } from "./shared.js";

export * from "./shared.js";

const path = process.env.MINACALC_LIBRARY_PATH ?? `libminacalc_bindings.${suffix}`;
const library = dlopen(path, {
  minacalc_version: { args: [], returns: "i32" },
  minacalc_calc_at_rate: { args: ["ptr", "usize", "f32", "f32", "u32", "i32", "ptr"], returns: "i32" },
  minacalc_calc_all_rates: { args: ["ptr", "usize", "u32", "i32", "ptr"], returns: "i32" },
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

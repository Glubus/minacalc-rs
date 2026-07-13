import koffi from "koffi";
import { MinaCalcError, type CalcMode, type Note, type SkillsetScores, modeValue, packNotes, readScore, validate } from "./shared.js";

export * from "./shared.js";

const libraryPath = process.env.MINACALC_LIBRARY_PATH ?? "minacalc_bindings";
const library = koffi.load(libraryPath);
const calculateAtRate = library.func("int minacalc_calc_at_rate(void *, size_t, float, float, uint32_t, int, void *)");
const calculateAllRates = library.func("int minacalc_calc_all_rates(void *, size_t, uint32_t, int, void *)");
const statusMessage = library.func("const char *minacalc_status_message(int)");
const nativeVersion = library.func("int minacalc_version()");

function check(status: number): void {
  if (status !== 0) throw new MinaCalcError(String(statusMessage(status)), status);
}

/** MinaCalc engine version linked by the loaded native library. */
export function version(): number { return nativeVersion(); }

export function calcAtRate(notes: readonly Note[], rate: number, goal = 0.93, keys = 4, mode: CalcMode = "ssr"): SkillsetScores {
  validate(notes, keys);
  if (!Number.isFinite(rate) || !Number.isFinite(goal)) throw new MinaCalcError("rate and goal must be finite", 3);
  const packed = Buffer.from(packNotes(notes));
  const output = Buffer.alloc(32);
  check(calculateAtRate(packed, notes.length, rate, goal, keys, modeValue(mode), output));
  return readScore(new DataView(output.buffer, output.byteOffset, output.byteLength));
}

export function calcAllRates(notes: readonly Note[], keys = 4, mode: CalcMode = "msd"): readonly SkillsetScores[] {
  validate(notes, keys);
  const packed = Buffer.from(packNotes(notes));
  const output = Buffer.alloc(14 * 32);
  check(calculateAllRates(packed, notes.length, keys, modeValue(mode), output));
  const view = new DataView(output.buffer, output.byteOffset, output.byteLength);
  return Array.from({ length: 14 }, (_, index) => readScore(view, index * 32));
}

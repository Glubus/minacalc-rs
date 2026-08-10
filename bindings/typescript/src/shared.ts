export type CalcMode = "msd" | "ssr";

export interface Note {
  /** Active columns as a bitmask. */
  notes: number;
  /** Time in seconds. */
  rowTime: number;
}

export interface SkillsetScores {
  overall: number;
  stream: number;
  jumpstream: number;
  handstream: number;
  stamina: number;
  jackspeed: number;
  chordjack: number;
  technical: number;
}

export interface CalcConfig {
  ssrGoalCap: number;
  lowAccCutoff: number;
  /** `null` disables the cap. */
  ssrRatingCap: number | null;
  defaultScoreGoal: number;
  grindScaling: boolean;
  skillsetScalers: Omit<SkillsetScores, "overall">;
}

export interface DetailedResult { scores: SkillsetScores; grindScaler: number; }

export const DEFAULT_CONFIG: Readonly<CalcConfig> = Object.freeze({
  ssrGoalCap: 0.965, lowAccCutoff: 0.9, ssrRatingCap: 40,
  defaultScoreGoal: 0.93, grindScaling: true,
  skillsetScalers: Object.freeze({ stream: 1, jumpstream: 1, handstream: 1, stamina: 1, jackspeed: 1, chordjack: 1, technical: 1 }),
});

export const RATES = Object.freeze(
  Array.from({ length: 14 }, (_, index) => Number((0.7 + index * 0.1).toFixed(1))),
) as readonly number[];

export class MinaCalcError extends Error {
  constructor(message: string, readonly status: number) {
    super(message);
    this.name = "MinaCalcError";
  }
}

export function modeValue(mode: CalcMode): 0 | 1 {
  return mode === "msd" ? 0 : 1;
}

export function validate(notes: readonly Note[], keys: number): void {
  if (notes.length === 0) throw new MinaCalcError("notes must not be empty", 2);
  if (![4, 6, 7].includes(keys)) throw new MinaCalcError("keys must be 4, 6, or 7", 3);
  for (const note of notes) {
    if (!Number.isInteger(note.notes) || note.notes < 0 || !Number.isFinite(note.rowTime)) {
      throw new MinaCalcError("each note must contain a uint32 bitmask and finite rowTime", 3);
    }
  }
}

/** Converts readable column lists into MinaCalc's compact bitmask rows. */
export function packNotes(notes: readonly Note[]): ArrayBuffer {
  const buffer = new ArrayBuffer(notes.length * 8);
  const view = new DataView(buffer);
  notes.forEach((note, index) => {
    view.setUint32(index * 8, note.notes, true);
    view.setFloat32(index * 8 + 4, note.rowTime, true);
  });
  return buffer;
}

export function readScore(view: DataView, offset = 0): SkillsetScores {
  const keys: (keyof SkillsetScores)[] = ["overall", "stream", "jumpstream", "handstream", "stamina", "jackspeed", "chordjack", "technical"];
  const score = {} as SkillsetScores;
  keys.forEach((key, index) => { score[key] = view.getFloat32(offset + index * 4, true); });
  return score;
}

export function packConfig(config: CalcConfig = DEFAULT_CONFIG): ArrayBuffer {
  const values = [config.ssrGoalCap, config.lowAccCutoff, config.ssrRatingCap ?? 0,
    config.defaultScoreGoal, config.skillsetScalers.stream, config.skillsetScalers.jumpstream,
    config.skillsetScalers.handstream, config.skillsetScalers.stamina,
    config.skillsetScalers.jackspeed, config.skillsetScalers.chordjack, config.skillsetScalers.technical];
  if (values.some((value) => !Number.isFinite(value)) || values.slice(0, 4).some((value, index) => index !== 2 && (value < 0 || value > 1)) || values.slice(4).some((value) => value <= 0) || (config.ssrRatingCap !== null && config.ssrRatingCap <= 0)) {
    throw new MinaCalcError("invalid calculator config", 3);
  }
  const buffer = new ArrayBuffer(48); const view = new DataView(buffer);
  values.forEach((value, index) => view.setFloat32(index * 4, value, true));
  view.setUint8(44, config.grindScaling ? 1 : 0); view.setUint8(45, config.ssrRatingCap === null ? 0 : 1);
  return buffer;
}

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

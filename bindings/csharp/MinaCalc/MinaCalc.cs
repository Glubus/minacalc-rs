using System.Runtime.InteropServices;

namespace MinaCalc;

public enum CalcMode : int { Msd = 0, Ssr = 1 }

public readonly record struct Note(uint Notes, float RowTime);
public readonly record struct SkillsetScores(float Overall, float Stream, float Jumpstream, float Handstream, float Stamina, float Jackspeed, float Chordjack, float Technical);

public sealed class MinaCalcException : Exception
{
    public int Status { get; }
    internal MinaCalcException(int status) : base($"MinaCalc failed with status {status}.") => Status = status;
}

public static partial class Calculator
{
    private const string Library = "minacalc_bindings";

    [StructLayout(LayoutKind.Sequential)] private struct NativeNote { public uint Notes; public float RowTime; }
    [StructLayout(LayoutKind.Sequential)] private struct NativeScores { public float Overall, Stream, Jumpstream, Handstream, Stamina, Jackspeed, Chordjack, Technical; }
    [StructLayout(LayoutKind.Sequential)] private unsafe struct NativeAllRates { public fixed float Values[112]; }

    [LibraryImport(Library, EntryPoint = "minacalc_version")]
    private static partial int NativeVersion();
    [LibraryImport(Library, EntryPoint = "minacalc_calc_at_rate")]
    private static unsafe partial int NativeCalcAtRate(NativeNote* notes, nuint length, float rate, float goal, uint keys, CalcMode mode, out NativeScores scores);
    [LibraryImport(Library, EntryPoint = "minacalc_calc_all_rates")]
    private static unsafe partial int NativeCalcAllRates(NativeNote* notes, nuint length, uint keys, CalcMode mode, out NativeAllRates scores);

    public static int Version => NativeVersion();

    public static unsafe SkillsetScores CalcAtRate(ReadOnlySpan<Note> notes, float rate, float goal = 0.93f, uint keys = 4, CalcMode mode = CalcMode.Ssr)
    {
        Validate(notes, keys);
        var native = new NativeNote[notes.Length];
        for (var i = 0; i < notes.Length; i++) native[i] = new NativeNote { Notes = notes[i].Notes, RowTime = notes[i].RowTime };
        fixed (NativeNote* pointer = native) { Check(NativeCalcAtRate(pointer, (nuint)native.Length, rate, goal, keys, mode, out var output)); return Convert(output); }
    }

    public static unsafe SkillsetScores[] CalcAllRates(ReadOnlySpan<Note> notes, uint keys = 4, CalcMode mode = CalcMode.Msd)
    {
        Validate(notes, keys);
        var native = new NativeNote[notes.Length];
        for (var i = 0; i < notes.Length; i++) native[i] = new NativeNote { Notes = notes[i].Notes, RowTime = notes[i].RowTime };
        fixed (NativeNote* input = native)
        {
            Check(NativeCalcAllRates(input, (nuint)native.Length, keys, mode, out var output));
            var results = new SkillsetScores[14];
            for (var index = 0; index < results.Length; index++)
            {
                var offset = index * 8;
                results[index] = new SkillsetScores(output.Values[offset], output.Values[offset + 1], output.Values[offset + 2], output.Values[offset + 3], output.Values[offset + 4], output.Values[offset + 5], output.Values[offset + 6], output.Values[offset + 7]);
            }
            return results;
        }
    }


    private static void Validate(ReadOnlySpan<Note> notes, uint keys) { if (notes.IsEmpty) throw new ArgumentException("notes must not be empty", nameof(notes)); if (keys is not (4 or 6 or 7)) throw new ArgumentOutOfRangeException(nameof(keys)); }
    private static void Check(int status) { if (status != 0) throw new MinaCalcException(status); }
    private static SkillsetScores Convert(NativeScores value) => new(value.Overall, value.Stream, value.Jumpstream, value.Handstream, value.Stamina, value.Jackspeed, value.Chordjack, value.Technical);
}

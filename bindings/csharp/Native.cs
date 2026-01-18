using System.Runtime.InteropServices;

namespace MinaCalc;

internal static class Native
{
    private const string LibraryName = "minacalc_rs";

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    public static extern int minacalc_version();

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr minacalc_new();

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void minacalc_free(IntPtr handle);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    public static extern int minacalc_calculate_at_rate(
        IntPtr handle,
        [In] MinaCalcNote[] notes,
        nuint notes_len,
        float music_rate,
        float score_goal,
        int capped,
        out MinaCalcScores result
    );

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    public static extern int minacalc_calculate_all_rates(
        IntPtr handle,
        [In] MinaCalcNote[] notes,
        nuint notes_len,
        int capped,
        IntPtr result // Points to CMinaCalcAllRates
    );

    // New API Methods
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    public static extern int minacalc_calculate_at_rate_from_file(
        IntPtr handle,
        string path,
        float music_rate,
        float score_goal,
        int capped,
        out MinaCalcScores result
    );

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    public static extern int minacalc_calculate_all_rates_from_file(
        IntPtr handle,
        string path,
        int capped,
        IntPtr result
    );

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    public static extern int minacalc_calculate_at_rate_from_string(
        IntPtr handle,
        string content,
        string file_hint,
        float music_rate,
        float score_goal,
        int capped,
        out MinaCalcScores result
    );

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    public static extern int minacalc_calculate_all_rates_from_string(
        IntPtr handle,
        string content,
        string file_hint,
        int capped,
        IntPtr result
    );
}

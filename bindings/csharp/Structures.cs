using System.Runtime.InteropServices;

namespace MinaCalc;

[StructLayout(LayoutKind.Sequential)]
public struct MinaCalcScores
{
    public float Overall;
    public float Stream;
    public float Jumpstream;
    public float Handstream;
    public float Stamina;
    public float JackSpeed;
    public float Chordjack;
    public float Technical;
}

[StructLayout(LayoutKind.Sequential)]
public struct MinaCalcNote
{
    public uint Notes;
    public float RowTime;
}

[StructLayout(LayoutKind.Sequential)]
public unsafe struct MinaCalcAllRates
{
    public fixed float Overall[14];
    public fixed float Stream[14];
    public fixed float Jumpstream[14];
    public fixed float Handstream[14];
    public fixed float Stamina[14];
    public fixed float JackSpeed[14];
    public fixed float Chordjack[14];
    public fixed float Technical[14];
}

// Helper struct to marshal the specific array of scores
[StructLayout(LayoutKind.Sequential)]
internal struct CMinaCalcAllRates
{
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = 14)]
    public MinaCalcScores[] Msds;
}

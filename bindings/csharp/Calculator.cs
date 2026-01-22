using System;
using System.Runtime.InteropServices;

namespace MinaCalc;

public class Calculator : IDisposable
{
    private IntPtr _handle;
    private bool _disposed;

    public Calculator()
    {
        _handle = Native.minacalc_new();
        if (_handle == IntPtr.Zero)
        {
            throw new InvalidOperationException("Failed to create MinaCalc instance.");
        }
    }

    public static int Version => Native.minacalc_version();

    /// <summary>
    /// Calculate scores for a specific rate.
    /// capped: true for SSR, false for MSD
    /// </summary>
    public MinaCalcScores CalculateAtRate(MinaCalcNote[] notes, float musicRate = 1.0f, float scoreGoal = 0.93f, uint keyCount = 4, bool capped = false)
    {
        CheckDisposed();

        if (notes == null || notes.Length == 0)
            throw new ArgumentException("Notes cannot be null or empty", nameof(notes));

        int result = Native.minacalc_calculate_at_rate(
            _handle,
            notes,
            (nuint)notes.Length,
            musicRate,
            scoreGoal,
            keyCount,
            capped ? 1 : 0,
            out var scores
        );

        if (result != 0)
        {
            throw new Exception($"Calculation failed with error code: {result}");
        }

        return scores;
    }



    /// <summary>
    /// Calculate scores directly from a chart file.
    /// capped: true for SSR, false for MSD
    /// </summary>
    public MinaCalcScores CalculateAtRateFromFile(string path, float musicRate = 1.0f, float scoreGoal = 0.93f, bool capped = false)
    {
        CheckDisposed();

        if (string.IsNullOrEmpty(path))
            throw new ArgumentException("Path cannot be empty", nameof(path));

        int result = Native.minacalc_calculate_at_rate_from_file(
            _handle,
            path,
            musicRate,
            scoreGoal,
            capped ? 1 : 0,
            out var scores
        );

        if (result != 0)
        {
             throw new Exception($"Calculation failed with error code: {result}");
        }
        return scores;
    }



    /// <summary>
    /// Calculate scores from string content.
    /// capped: true for SSR, false for MSD
    /// </summary>
    public MinaCalcScores CalculateAtRateFromString(string content, string fileHint, float musicRate = 1.0f, float scoreGoal = 0.93f, bool capped = false)
    {
        CheckDisposed();
        
        if (string.IsNullOrEmpty(content))
             throw new ArgumentException("Content cannot be empty", nameof(content));

        int result = Native.minacalc_calculate_at_rate_from_string(
            _handle,
            content,
            fileHint,
            musicRate,
            scoreGoal,
            capped ? 1 : 0,
            out var scores
        );

        if (result != 0)
        {
             throw new Exception($"Calculation failed with error code: {result}");
        }
        return scores;
    }



    public MinaCalcScores[] CalculateAllRates(MinaCalcNote[] notes, uint keyCount = 4, bool capped = false)
    {
        CheckDisposed();

        if (notes == null || notes.Length == 0)
            throw new ArgumentException("Notes cannot be null or empty", nameof(notes));

        var resultStruct = new CMinaCalcAllRates();
        resultStruct.Msds = new MinaCalcScores[14];
        
        int size = Marshal.SizeOf<CMinaCalcAllRates>();
        IntPtr ptr = Marshal.AllocHGlobal(size);

        try
        {
            int result = Native.minacalc_calculate_all_rates(
                _handle,
                notes,
                (nuint)notes.Length,
                keyCount,
                capped ? 1 : 0,
                ptr
            );

            if (result != 0)
            {
                throw new Exception($"Calculation failed with error code: {result}");
            }

            var data = Marshal.PtrToStructure<CMinaCalcAllRates>(ptr);
            return data.Msds;
        }
        finally
        {
            Marshal.FreeHGlobal(ptr);
        }
    }

    public MinaCalcScores[] CalculateAllRatesFromFile(string path, bool capped = false)
    {
        CheckDisposed();

        if (string.IsNullOrEmpty(path))
            throw new ArgumentException("Path cannot be empty", nameof(path));

        int size = Marshal.SizeOf<CMinaCalcAllRates>();
        IntPtr ptr = Marshal.AllocHGlobal(size);

        try
        {
            int result = Native.minacalc_calculate_all_rates_from_file(
                _handle,
                path,
                capped ? 1 : 0,
                ptr
            );

            if (result != 0)
            {
                throw new Exception($"Calculation failed with error code: {result}");
            }

            var data = Marshal.PtrToStructure<CMinaCalcAllRates>(ptr);
            return data.Msds;
        }
        finally
        {
            Marshal.FreeHGlobal(ptr);
        }
    }

     public MinaCalcScores[] CalculateAllRatesFromString(string content, string fileHint, bool capped = false)
    {
        CheckDisposed();

        if (string.IsNullOrEmpty(content))
            throw new ArgumentException("Content cannot be empty", nameof(content));

        int size = Marshal.SizeOf<CMinaCalcAllRates>();
        IntPtr ptr = Marshal.AllocHGlobal(size);

        try
        {
            int result = Native.minacalc_calculate_all_rates_from_string(
                _handle,
                content,
                fileHint,
                capped ? 1 : 0,
                ptr
            );

            if (result != 0)
            {
                throw new Exception($"Calculation failed with error code: {result}");
            }

            var data = Marshal.PtrToStructure<CMinaCalcAllRates>(ptr);
            return data.Msds;
        }
        finally
        {
            Marshal.FreeHGlobal(ptr);
        }
    }

    private void CheckDisposed()
    {
        if (_disposed)
            throw new ObjectDisposedException(nameof(Calculator));
    }

    protected virtual void Dispose(bool disposing)
    {
        if (!_disposed)
        {
            Native.minacalc_free(_handle);
            _handle = IntPtr.Zero;
            _disposed = true;
        }
    }

    public void Dispose()
    {
        Dispose(true);
        GC.SuppressFinalize(this);
    }

    ~Calculator()
    {
        Dispose(false);
    }
}

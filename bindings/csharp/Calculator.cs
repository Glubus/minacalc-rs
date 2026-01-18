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
    /// Calculate SSR from a list of notes.
    /// </summary>
    public MinaCalcScores CalculateSsr(MinaCalcNote[] notes, float musicRate = 1.0f, float scoreGoal = 0.93f)
    {
        CheckDisposed();

        if (notes == null || notes.Length == 0)
            throw new ArgumentException("Notes cannot be null or empty", nameof(notes));

        int result = Native.minacalc_calculate_ssr(
            _handle,
            notes,
            (nuint)notes.Length,
            musicRate,
            scoreGoal,
            out var scores
        );

        if (result != 0)
        {
            throw new Exception($"Calculation failed with error code: {result}");
        }

        return scores;
    }

    /// <summary>
    /// Calculate SSR directly from a chart file.
    /// </summary>
    public MinaCalcScores CalculateSsrFromFile(string path, float musicRate = 1.0f, float scoreGoal = 0.93f)
    {
        CheckDisposed();

        if (string.IsNullOrEmpty(path))
            throw new ArgumentException("Path cannot be empty", nameof(path));

        int result = Native.minacalc_calculate_ssr_from_file(
            _handle,
            path,
            musicRate,
            scoreGoal,
            out var scores
        );

        if (result != 0)
        {
             throw new Exception($"Calculation failed with error code: {result}");
        }
        return scores;
    }

    /// <summary>
    /// Calculate SSR from string content (e.g. .osu file content).
    /// </summary>
    public MinaCalcScores CalculateSsrFromString(string content, string fileHint, float musicRate = 1.0f, float scoreGoal = 0.93f)
    {
        CheckDisposed();
        
        if (string.IsNullOrEmpty(content))
             throw new ArgumentException("Content cannot be empty", nameof(content));

        int result = Native.minacalc_calculate_ssr_from_string(
            _handle,
            content,
            fileHint,
            musicRate,
            scoreGoal,
            out var scores
        );

         if (result != 0)
        {
             throw new Exception($"Calculation failed with error code: {result}");
        }
        return scores;
    }

    public MinaCalcScores[] CalculateAllRates(MinaCalcNote[] notes)
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

    public MinaCalcScores[] CalculateAllRatesFromFile(string path)
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

     public MinaCalcScores[] CalculateAllRatesFromString(string content, string fileHint)
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

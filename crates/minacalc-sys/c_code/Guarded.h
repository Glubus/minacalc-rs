// Exception-safe entry points (Prism, 2026-10-22). A C++ exception that crosses into Rust aborts the whole process ("Rust cannot catch
// foreign exceptions"), so these catch every exception ON THE C++ SIDE and answer a code instead. Each one calls MinaSDCalc itself (a C++
// function): a `try` around an `extern "C"` function would be removed by MSVC's /EHsc, which assumes such functions never throw.
//
// Answers: 0 = done, `out` written; 1 = MinaCalc threw an exception; 2 = MinaCalc returned fewer values than a rating has. On a non-zero answer
// `out` is left untouched.
#include "API.h"

#ifdef __cplusplus
extern "C" {
#endif

int guarded_calc_at_rate(CalcHandle *calc, const NoteInfo *rows, size_t num_rows, float music_rate, float score_goal, unsigned int keycount,
                         CalcMode mode, Ssr *out);

int guarded_calc_all_rates(CalcHandle *calc, const NoteInfo *rows, size_t num_rows, unsigned int keycount, CalcMode mode,
                           MsdForAllRates *out);

int guarded_calc_rates(CalcHandle *calc, const NoteInfo *rows, size_t num_rows, const float *rates, size_t num_rates,
                       unsigned int keycount, CalcMode mode, Ssr *out_scores);

#ifdef __cplusplus
}
#endif

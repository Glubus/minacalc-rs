#ifndef MINACALC_H
#define MINACALC_H

#include <stddef.h>
#include <stdint.h>

#ifdef _WIN32
#  ifdef MINACALC_BUILD
#    define MINACALC_API __declspec(dllexport)
#  else
#    define MINACALC_API __declspec(dllimport)
#  endif
#else
#  define MINACALC_API
#endif

#ifdef __cplusplus
extern "C" {
#endif

typedef struct { uint32_t notes; float row_time; } MinaCalcNote;
typedef struct {
  float overall, stream, jumpstream, handstream;
  float stamina, jackspeed, chordjack, technical;
} MinaCalcScores;
typedef struct { MinaCalcScores rates[14]; } MinaCalcAllRates;
typedef enum {
  MINACALC_OK = 0, MINACALC_NULL_POINTER = 1, MINACALC_EMPTY_NOTES = 2,
  MINACALC_INVALID_ARGUMENT = 3, MINACALC_ALLOCATION_FAILED = 4, MINACALC_PANIC = 5
} MinaCalcStatus;
typedef enum { MINACALC_MSD = 0, MINACALC_SSR = 1 } MinaCalcMode;

MINACALC_API int32_t minacalc_version(void);
MINACALC_API MinaCalcStatus minacalc_calc_at_rate(const MinaCalcNote *notes, size_t len,
  float rate, float goal, uint32_t keys, int32_t mode, MinaCalcScores *out_scores);
MINACALC_API MinaCalcStatus minacalc_calc_all_rates(const MinaCalcNote *notes, size_t len,
  uint32_t keys, int32_t mode, MinaCalcAllRates *out_scores);
MINACALC_API const char *minacalc_status_message(MinaCalcStatus status);

#ifdef __cplusplus
}
#endif
#endif

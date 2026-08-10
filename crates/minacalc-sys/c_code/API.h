// This is a C++ style header file, so we have to do the typedef below to have NoteInfo accessible
#include "Models/NoteData/NoteDataStructures.h"
#include <stddef.h>

typedef struct NoteInfo NoteInfo;

typedef struct CalcHandle {
    char _dummy; // Dummy member to ensure non-zero size
} CalcHandle;

typedef struct Ssr {
	float overall;
	float stream;
	float jumpstream;
	float handstream;
	float stamina;
	float jackspeed;
	float chordjack;
	float technical;
} Ssr;

typedef struct MsdForAllRates {
	// one for each full-rate from 0.7 to 2.0 inclusive
	Ssr msds[14];
} MsdForAllRates;

enum class CalcMode {
	MSD = 0, // uncapped, raw difficulty
	SSR = 1, // capped, rated (score goal applies)
};

#ifdef __cplusplus
extern "C" {
#endif

int calc_version();

CalcHandle *create_calc();

void destroy_calc(CalcHandle *calc);

// Changes the maximum score goal used for SSR calculations.
void set_ssr_goal_cap(CalcHandle *calc, float goal_cap);

// Changes the score threshold below which SSR values are downscaled.
void set_low_acc_cutoff(CalcHandle *calc, float cutoff);

// Changes the maximum value applied to individual SSR skillsets.
void set_ssr_rating_cap(CalcHandle *calc, float rating_cap);

// Changes the score goal used by all-rates calculations.
void set_default_score_goal(CalcHandle *calc, float score_goal);

// Enables or disables the SSR grind scaling penalty.
void set_grind_scaling_enabled(CalcHandle *calc, bool enabled);

// Calculates difficulty for all rates (0.7x - 2.0x)
MsdForAllRates calc_all_rates(CalcHandle *calc, const NoteInfo *rows, size_t num_rows, unsigned int keycount, CalcMode mode);

// Calculates difficulty at a specific rate
// score_goal: relevant for SSR (usually 0.93), ignored for MSD
Ssr calc_at_rate(CalcHandle *calc, NoteInfo *rows, size_t num_rows, float music_rate, float score_goal, unsigned int keycount, CalcMode mode);

#ifdef __cplusplus
}
#endif

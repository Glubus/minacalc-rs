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

int calc_version();

CalcHandle *create_calc();

void destroy_calc(CalcHandle *calc);

// Calculates difficulty for all rates (0.7x - 2.0x)
// cap: 1 for SSR (capped, rated), 0 for MSD (uncapped, raw difficulty)
MsdForAllRates calc_all_rates(CalcHandle *calc, const NoteInfo *rows, size_t num_rows, unsigned int keycount, int cap);

// Calculates difficulty at a specific rate
// cap: 1 for SSR (capped, rated), 0 for MSD (uncapped, raw difficulty)
// score_goal: relevant for SSR (usually 0.93), ignored/default for MSD
Ssr calc_at_rate(CalcHandle *calc, NoteInfo *rows, size_t num_rows, float music_rate, float score_goal, unsigned int keycount, int cap);

// Legacy aliases (mapped to new functions with appropriate cap settings)
MsdForAllRates calc_msd(CalcHandle *calc, const NoteInfo *rows, size_t num_rows, unsigned int keycount);
Ssr calc_ssr(CalcHandle *calc, NoteInfo *rows, size_t num_rows, float music_rate, float score_goal, unsigned int keycount);
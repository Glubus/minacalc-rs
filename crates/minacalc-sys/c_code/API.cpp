#include "MinaCalc/MinaCalc.h"
#include <vector>

extern "C" {
	#include "API.h"

	// internal utility function for C <-> C++ bridging
	extern "C++" Ssr skillset_vector_to_ssr(std::vector<float> &skillsets) {
		//assert(skillsets.size() == NUM_Skillset);
		return Ssr {
			skillsets[0], // Overall
			skillsets[1], // Stream
			skillsets[2], // Jumpstream
			skillsets[3], // Handstream
			skillsets[4], // Stamina
			skillsets[5], // JackSpeed
			skillsets[6], // Chordjack
			skillsets[7], // Technical
		};
	}

	int calc_version() {
		return GetCalcVersion();
	}

	CalcHandle *create_calc() {
		return reinterpret_cast<CalcHandle*>(new Calc);
	}

	void destroy_calc(CalcHandle *calc) {
		delete reinterpret_cast<Calc*>(calc);
	}

	void set_ssr_goal_cap(CalcHandle *calc, float goal_cap) {
		reinterpret_cast<Calc*>(calc)->SetSsrGoalCap(goal_cap);
	}

	void set_low_acc_cutoff(CalcHandle *calc, float cutoff) {
		reinterpret_cast<Calc*>(calc)->SetLowAccCutoff(cutoff);
	}

	void set_ssr_rating_cap(CalcHandle *calc, float rating_cap) {
		reinterpret_cast<Calc*>(calc)->SetSsrRatingCap(rating_cap);
	}

	void set_default_score_goal(CalcHandle *calc, float score_goal) {
		reinterpret_cast<Calc*>(calc)->SetDefaultScoreGoal(score_goal);
	}

	void set_grind_scaling_enabled(CalcHandle *calc, bool enabled) {
		reinterpret_cast<Calc*>(calc)->SetGrindScalingEnabled(enabled);
	}




	/* Core Functions */

	MsdForAllRates calc_all_rates(CalcHandle *calc, const NoteInfo *rows, size_t num_rows, unsigned int keycount, CalcMode mode) {
		std::vector<NoteInfo> note_info(rows, rows + num_rows);

		auto msd_vectors = MinaSDCalc(
			note_info,
			keycount,
			mode == CalcMode::SSR,
			reinterpret_cast<Calc*>(calc)
		);

		MsdForAllRates all_rates;
		for (int i = 0; i < 14; i++) {
			all_rates.msds[i] = skillset_vector_to_ssr(msd_vectors[i]);
		}

		return all_rates;
	}

	Ssr calc_at_rate(CalcHandle *calc, NoteInfo *rows, size_t num_rows, float music_rate, float score_goal, unsigned int keycount, CalcMode mode) {
		std::vector<NoteInfo> note_info(rows, rows + num_rows);

		auto skillsets = MinaSDCalc(
			note_info,
			music_rate,
			score_goal,
			keycount,
			mode == CalcMode::SSR,
			reinterpret_cast<Calc*>(calc)
		);

		return skillset_vector_to_ssr(skillsets);
	}

}

// See Guarded.h: MinaCalc behind a C++ `try`, never an exception into Rust.
#include "MinaCalc/MinaCalc.h"
#include <vector>

extern "C" {
#include "Guarded.h"
}

namespace {

constexpr size_t SKILLSET_COUNT = 8;

Ssr to_ssr(const std::vector<float> &skillsets) {
	return Ssr{
		skillsets[0], skillsets[1], skillsets[2], skillsets[3],
		skillsets[4], skillsets[5], skillsets[6], skillsets[7],
	};
}

} // namespace

extern "C" {

int guarded_calc_at_rate(CalcHandle *calc, const NoteInfo *rows, size_t num_rows, float music_rate, float score_goal, unsigned int keycount,
                         CalcMode mode, Ssr *out) {
	try {
		std::vector<NoteInfo> note_info(rows, rows + num_rows);
		auto skillsets = MinaSDCalc(note_info, music_rate, score_goal, keycount, mode == CalcMode::SSR, reinterpret_cast<Calc *>(calc));
		if (skillsets.size() < SKILLSET_COUNT) {
			return 2;
		}
		*out = to_ssr(skillsets);
		return 0;
	} catch (...) {
		return 1;
	}
}

int guarded_calc_all_rates(CalcHandle *calc, const NoteInfo *rows, size_t num_rows, unsigned int keycount, CalcMode mode,
                           MsdForAllRates *out) {
	try {
		std::vector<NoteInfo> note_info(rows, rows + num_rows);
		auto msd_vectors = MinaSDCalc(note_info, keycount, mode == CalcMode::SSR, reinterpret_cast<Calc *>(calc));
		if (msd_vectors.size() < 14) {
			return 2;
		}
		MsdForAllRates all_rates;
		for (size_t i = 0; i < 14; i++) {
			if (msd_vectors[i].size() < SKILLSET_COUNT) {
				return 2;
			}
			all_rates.msds[i] = to_ssr(msd_vectors[i]);
		}
		*out = all_rates;
		return 0;
	} catch (...) {
		return 1;
	}
}

int guarded_calc_rates(CalcHandle *calc, const NoteInfo *rows, size_t num_rows, const float *rates, size_t num_rates,
                       unsigned int keycount, CalcMode mode, Ssr *out_scores) {
	try {
		std::vector<NoteInfo> note_info(rows, rows + num_rows);
		auto *calculator = reinterpret_cast<Calc *>(calc);
		std::vector<Ssr> scores;
		scores.reserve(num_rates);
		for (size_t i = 0; i < num_rates; ++i) {
			auto skillsets = MinaSDCalc(note_info, rates[i], calculator->default_score_goal, keycount, mode == CalcMode::SSR, calculator);
			if (skillsets.size() < SKILLSET_COUNT) {
				return 2;
			}
			scores.push_back(to_ssr(skillsets));
		}
		for (size_t i = 0; i < num_rates; ++i) {
			out_scores[i] = scores[i];
		}
		return 0;
	} catch (...) {
		return 1;
	}
}

}

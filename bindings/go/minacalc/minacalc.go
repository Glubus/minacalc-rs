// Package minacalc provides Go access to the stable minacalc-bindings Rust ABI.
package minacalc

/*
#cgo CFLAGS: -I${SRCDIR}/../include
#cgo LDFLAGS: -lminacalc_bindings
#include "minacalc.h"
*/
import "C"

import (
	"fmt"
	"math"
)

// Mode controls whether raw MSD or score-relative SSR is calculated.
type Mode int32

const (
	MSD Mode = iota
	SSR
)

// Note is a row of active columns at a time in seconds.
type Note struct {
	Notes   uint32
	RowTime float32
}

// SkillsetScores is the difficulty score for each MinaCalc skillset.
type SkillsetScores struct {
	Overall, Stream, Jumpstream, Handstream  float32
	Stamina, Jackspeed, Chordjack, Technical float32
}

type SkillsetScalers struct{ Stream, Jumpstream, Handstream, Stamina, Jackspeed, Chordjack, Technical float32 }
type Config struct {
	SsrGoalCap, LowAccCutoff float32
	SsrRatingCap             *float32
	DefaultScoreGoal         float32
	GrindScaling             bool
	SkillsetScalers          SkillsetScalers
}
type DetailedResult struct {
	Scores      SkillsetScores
	GrindScaler float32
}

func DefaultConfig() Config {
	cap := float32(40)
	return Config{SsrGoalCap: .965, LowAccCutoff: .9, SsrRatingCap: &cap, DefaultScoreGoal: .93, GrindScaling: true, SkillsetScalers: SkillsetScalers{1, 1, 1, 1, 1, 1, 1}}
}

// Error is a non-zero status returned by the native library.
type Error struct{ Status int32 }

func (e *Error) Error() string { return fmt.Sprintf("minacalc failed with status %d", e.Status) }

// Version returns the linked MinaCalc engine version.
func Version() int { return int(C.minacalc_version()) }

// CalcAtRate calculates a single music rate.
func CalcAtRate(notes []Note, rate, goal float32, keys uint32, mode Mode) (SkillsetScores, error) {
	if err := validate(notes, keys); err != nil {
		return SkillsetScores{}, err
	}
	if math.IsNaN(float64(rate)) || math.IsInf(float64(rate), 0) || math.IsNaN(float64(goal)) || math.IsInf(float64(goal), 0) {
		return SkillsetScores{}, fmt.Errorf("rate and goal must be finite")
	}
	input := make([]C.MinaCalcNote, len(notes))
	for i, note := range notes {
		input[i] = C.MinaCalcNote{notes: C.uint32_t(note.Notes), row_time: C.float(note.RowTime)}
	}
	var output C.MinaCalcScores
	status := C.minacalc_calc_at_rate(&input[0], C.size_t(len(input)), C.float(rate), C.float(goal), C.uint32_t(keys), C.int32_t(mode), &output)
	if status != C.MINACALC_OK {
		return SkillsetScores{}, &Error{Status: int32(status)}
	}
	return score(output), nil
}

// CalcAllRates calculates rates from 0.7x through 2.0x.
func CalcAllRates(notes []Note, keys uint32, mode Mode) ([14]SkillsetScores, error) {
	var results [14]SkillsetScores
	if err := validate(notes, keys); err != nil {
		return results, err
	}
	input := make([]C.MinaCalcNote, len(notes))
	for i, note := range notes {
		input[i] = C.MinaCalcNote{notes: C.uint32_t(note.Notes), row_time: C.float(note.RowTime)}
	}
	var output C.MinaCalcAllRates
	status := C.minacalc_calc_all_rates(&input[0], C.size_t(len(input)), C.uint32_t(keys), C.int32_t(mode), &output)
	if status != C.MINACALC_OK {
		return results, &Error{Status: int32(status)}
	}
	for i := range results {
		results[i] = score(output.rates[i])
	}
	return results, nil
}

func CalcAtRateDetailed(notes []Note, rate, goal float32, keys uint32, mode Mode, config Config) (DetailedResult, error) {
	if err := validate(notes, keys); err != nil {
		return DetailedResult{}, err
	}
	input := nativeNotes(notes)
	cfg := nativeConfig(config)
	var output C.MinaCalcDetailedResult
	status := C.minacalc_calc_at_rate_with_config(&input[0], C.size_t(len(input)), C.float(rate), C.float(goal), C.uint32_t(keys), C.int32_t(mode), &cfg, &output)
	if status != C.MINACALC_OK {
		return DetailedResult{}, &Error{Status: int32(status)}
	}
	return DetailedResult{score(output.scores), float32(output.grind_scaler)}, nil
}

func CalcRates(notes []Note, rates []float32, keys uint32, mode Mode, config Config) ([]SkillsetScores, error) {
	if err := validate(notes, keys); err != nil {
		return nil, err
	}
	if len(rates) == 0 {
		return nil, fmt.Errorf("rates must not be empty")
	}
	input, nativeRates, cfg := nativeNotes(notes), make([]C.float, len(rates)), nativeConfig(config)
	output := make([]C.MinaCalcScores, len(rates))
	for i, rate := range rates {
		nativeRates[i] = C.float(rate)
	}
	status := C.minacalc_calc_rates(&input[0], C.size_t(len(input)), &nativeRates[0], C.size_t(len(rates)), C.uint32_t(keys), C.int32_t(mode), &cfg, &output[0])
	if status != C.MINACALC_OK {
		return nil, &Error{Status: int32(status)}
	}
	results := make([]SkillsetScores, len(rates))
	for i := range results {
		results[i] = score(output[i])
	}
	return results, nil
}

func validate(notes []Note, keys uint32) error {
	if len(notes) == 0 {
		return fmt.Errorf("notes must not be empty")
	}
	if keys != 4 && keys != 6 && keys != 7 {
		return fmt.Errorf("keys must be 4, 6, or 7")
	}
	return nil
}

func nativeNotes(notes []Note) []C.MinaCalcNote {
	input := make([]C.MinaCalcNote, len(notes))
	for i, note := range notes {
		input[i] = C.MinaCalcNote{notes: C.uint32_t(note.Notes), row_time: C.float(note.RowTime)}
	}
	return input
}
func nativeConfig(value Config) C.MinaCalcConfig {
	if value == (Config{}) {
		value = DefaultConfig()
	}
	cap, enabled := float32(0), C.uint8_t(0)
	if value.SsrRatingCap != nil {
		cap, enabled = *value.SsrRatingCap, 1
	}
	s := value.SkillsetScalers
	return C.MinaCalcConfig{ssr_goal_cap: C.float(value.SsrGoalCap), low_acc_cutoff: C.float(value.LowAccCutoff), ssr_rating_cap: C.float(cap), default_score_goal: C.float(value.DefaultScoreGoal), stream_scaler: C.float(s.Stream), jumpstream_scaler: C.float(s.Jumpstream), handstream_scaler: C.float(s.Handstream), stamina_scaler: C.float(s.Stamina), jackspeed_scaler: C.float(s.Jackspeed), chordjack_scaler: C.float(s.Chordjack), technical_scaler: C.float(s.Technical), grind_scaling: C.uint8_t(boolByte(value.GrindScaling)), ssr_rating_cap_enabled: enabled}
}
func boolByte(value bool) byte {
	if value {
		return 1
	}
	return 0
}

func score(value C.MinaCalcScores) SkillsetScores {
	return SkillsetScores{
		Overall: float32(value.overall), Stream: float32(value.stream),
		Jumpstream: float32(value.jumpstream), Handstream: float32(value.handstream),
		Stamina: float32(value.stamina), Jackspeed: float32(value.jackspeed),
		Chordjack: float32(value.chordjack), Technical: float32(value.technical),
	}
}

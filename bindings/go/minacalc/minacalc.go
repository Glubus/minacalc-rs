// Package minacalc provides Go access to the stable minacalc-bindings Rust ABI.
package minacalc

/*
#cgo CFLAGS: -I${SRCDIR}/../../include
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
	Overall, Stream, Jumpstream, Handstream float32
	Stamina, Jackspeed, Chordjack, Technical float32
}

// Error is a non-zero status returned by the native library.
type Error struct { Status int32 }
func (e *Error) Error() string { return fmt.Sprintf("minacalc failed with status %d", e.Status) }

// Version returns the linked MinaCalc engine version.
func Version() int { return int(C.minacalc_version()) }

// CalcAtRate calculates a single music rate.
func CalcAtRate(notes []Note, rate, goal float32, keys uint32, mode Mode) (SkillsetScores, error) {
	if err := validate(notes, keys); err != nil { return SkillsetScores{}, err }
	if math.IsNaN(float64(rate)) || math.IsInf(float64(rate), 0) || math.IsNaN(float64(goal)) || math.IsInf(float64(goal), 0) { return SkillsetScores{}, fmt.Errorf("rate and goal must be finite") }
	input := make([]C.MinaCalcNote, len(notes))
	for i, note := range notes { input[i] = C.MinaCalcNote{notes: C.uint32_t(note.Notes), row_time: C.float(note.RowTime)} }
	var output C.MinaCalcScores
	status := C.minacalc_calc_at_rate(&input[0], C.size_t(len(input)), C.float(rate), C.float(goal), C.uint32_t(keys), C.int32_t(mode), &output)
	if status != C.MINACALC_OK { return SkillsetScores{}, &Error{Status: int32(status)} }
	return score(output), nil
}

// CalcAllRates calculates rates from 0.7x through 2.0x.
func CalcAllRates(notes []Note, keys uint32, mode Mode) ([14]SkillsetScores, error) {
	var results [14]SkillsetScores
	if err := validate(notes, keys); err != nil { return results, err }
	input := make([]C.MinaCalcNote, len(notes))
	for i, note := range notes { input[i] = C.MinaCalcNote{notes: C.uint32_t(note.Notes), row_time: C.float(note.RowTime)} }
	var output C.MinaCalcAllRates
	status := C.minacalc_calc_all_rates(&input[0], C.size_t(len(input)), C.uint32_t(keys), C.int32_t(mode), &output)
	if status != C.MINACALC_OK { return results, &Error{Status: int32(status)} }
	for i := range results { results[i] = score(output.rates[i]) }
	return results, nil
}


func validate(notes []Note, keys uint32) error {
	if len(notes) == 0 { return fmt.Errorf("notes must not be empty") }
	if keys != 4 && keys != 6 && keys != 7 { return fmt.Errorf("keys must be 4, 6, or 7") }
	return nil
}

func score(value C.MinaCalcScores) SkillsetScores {
	return SkillsetScores{
		Overall: float32(value.overall), Stream: float32(value.stream),
		Jumpstream: float32(value.jumpstream), Handstream: float32(value.handstream),
		Stamina: float32(value.stamina), Jackspeed: float32(value.jackspeed),
		Chordjack: float32(value.chordjack), Technical: float32(value.technical),
	}
}

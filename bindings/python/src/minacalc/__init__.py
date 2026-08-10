"""Typed Python access to the stable `minacalc-bindings` Rust ABI."""

from __future__ import annotations

import ctypes as _ct
import os as _os
import sys as _sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Literal, Sequence, Union

CalcMode = Literal["msd", "ssr"]
NoteInput = Union["Note", tuple[int, float]]


class MinaCalcError(RuntimeError):
    """A calculation error returned by the native MinaCalc library."""

    def __init__(self, message: str, status: int) -> None:
        super().__init__(message)
        self.status = status


@dataclass(frozen=True)
class Note:
    notes: int
    row_time: float


@dataclass(frozen=True)
class SkillsetScores:
    overall: float
    stream: float
    jumpstream: float
    handstream: float
    stamina: float
    jackspeed: float
    chordjack: float
    technical: float


@dataclass(frozen=True)
class SkillsetScalers:
    stream: float = 1.0
    jumpstream: float = 1.0
    handstream: float = 1.0
    stamina: float = 1.0
    jackspeed: float = 1.0
    chordjack: float = 1.0
    technical: float = 1.0


@dataclass(frozen=True)
class CalcConfig:
    ssr_goal_cap: float = 0.965
    low_acc_cutoff: float = 0.9
    ssr_rating_cap: float | None = 40.0
    default_score_goal: float = 0.93
    grind_scaling: bool = True
    skillset_scalers: SkillsetScalers = SkillsetScalers()


@dataclass(frozen=True)
class DetailedResult:
    scores: SkillsetScores
    grind_scaler: float


class _CNote(_ct.Structure):
    _fields_ = [("notes", _ct.c_uint32), ("row_time", _ct.c_float)]


class _CScores(_ct.Structure):
    _fields_ = [(name, _ct.c_float) for name in SkillsetScores.__annotations__]


class _CAllRates(_ct.Structure):
    _fields_ = [("rates", _CScores * 14)]


class _CConfig(_ct.Structure):
    _fields_ = [(name, _ct.c_float) for name in (
        "ssr_goal_cap", "low_acc_cutoff", "ssr_rating_cap", "default_score_goal",
        "stream_scaler", "jumpstream_scaler", "handstream_scaler", "stamina_scaler",
        "jackspeed_scaler", "chordjack_scaler", "technical_scaler",
    )] + [("grind_scaling", _ct.c_uint8), ("ssr_rating_cap_enabled", _ct.c_uint8), ("reserved", _ct.c_uint8 * 2)]


class _CDetailed(_ct.Structure):
    _fields_ = [("scores", _CScores), ("grind_scaler", _ct.c_float)]


def _library_candidates() -> list[str]:
    configured = _os.environ.get("MINACALC_LIBRARY_PATH")
    if configured:
        return [configured]
    filename = "minacalc_bindings.dll" if _sys.platform == "win32" else (
        "libminacalc_bindings.dylib" if _sys.platform == "darwin" else "libminacalc_bindings.so"
    )
    return [str(Path.cwd() / filename), filename]


def _load() -> _ct.CDLL:
    errors: list[str] = []
    for candidate in _library_candidates():
        try:
            return _ct.CDLL(candidate)
        except OSError as error:
            errors.append(f"{candidate}: {error}")
    raise ImportError("Could not load minacalc-bindings. Set MINACALC_LIBRARY_PATH. " + "; ".join(errors))


_lib = _load()
_lib.minacalc_version.restype = _ct.c_int32
_lib.minacalc_status_message.argtypes = [_ct.c_int32]
_lib.minacalc_status_message.restype = _ct.c_char_p
_lib.minacalc_calc_at_rate.argtypes = [_ct.POINTER(_CNote), _ct.c_size_t, _ct.c_float, _ct.c_float, _ct.c_uint32, _ct.c_int32, _ct.POINTER(_CScores)]
_lib.minacalc_calc_at_rate.restype = _ct.c_int32
_lib.minacalc_calc_all_rates.argtypes = [_ct.POINTER(_CNote), _ct.c_size_t, _ct.c_uint32, _ct.c_int32, _ct.POINTER(_CAllRates)]
_lib.minacalc_calc_all_rates.restype = _ct.c_int32
_lib.minacalc_calc_at_rate_with_config.argtypes = [_ct.POINTER(_CNote), _ct.c_size_t, _ct.c_float, _ct.c_float, _ct.c_uint32, _ct.c_int32, _ct.POINTER(_CConfig), _ct.POINTER(_CDetailed)]
_lib.minacalc_calc_at_rate_with_config.restype = _ct.c_int32
_lib.minacalc_calc_rates.argtypes = [_ct.POINTER(_CNote), _ct.c_size_t, _ct.POINTER(_ct.c_float), _ct.c_size_t, _ct.c_uint32, _ct.c_int32, _ct.POINTER(_CConfig), _ct.POINTER(_CScores)]
_lib.minacalc_calc_rates.restype = _ct.c_int32


def _check(status: int) -> None:
    if status:
        message = _lib.minacalc_status_message(status).decode("utf-8")
        raise MinaCalcError(message, status)


def _prepare(notes: Iterable[NoteInput], keys: int) -> Sequence[_CNote]:
    if keys not in (4, 6, 7):
        raise ValueError("keys must be 4, 6, or 7")
    converted = [note if isinstance(note, Note) else Note(*note) for note in notes]
    if not converted:
        raise ValueError("notes must not be empty")
    for note in converted:
        if not 0 <= note.notes <= 0xFFFFFFFF:
            raise ValueError("notes must be a uint32 bitmask")
    return (_CNote * len(converted))(*(_CNote(note.notes, note.row_time) for note in converted))



def _scores(value: _CScores) -> SkillsetScores:
    return SkillsetScores(*(getattr(value, name) for name in SkillsetScores.__annotations__))


def _config(value: CalcConfig) -> _CConfig:
    cap = value.ssr_rating_cap
    scalers = value.skillset_scalers
    return _CConfig(value.ssr_goal_cap, value.low_acc_cutoff, cap or 0.0,
        value.default_score_goal, scalers.stream, scalers.jumpstream, scalers.handstream,
        scalers.stamina, scalers.jackspeed, scalers.chordjack, scalers.technical,
        value.grind_scaling, cap is not None, (0, 0))


def version() -> int:
    """Return the linked MinaCalc engine version."""
    return int(_lib.minacalc_version())


def calc_at_rate(notes: Iterable[NoteInput], rate: float, goal: float = 0.93, keys: int = 4, mode: CalcMode = "ssr") -> SkillsetScores:
    """Calculate difficulty at one music rate."""
    prepared = _prepare(notes, keys)
    output = _CScores()
    _check(_lib.minacalc_calc_at_rate(prepared, len(prepared), rate, goal, keys, 0 if mode == "msd" else 1, _ct.byref(output)))
    return _scores(output)


def calc_all_rates(notes: Iterable[NoteInput], keys: int = 4, mode: CalcMode = "msd") -> tuple[SkillsetScores, ...]:
    """Calculate difficulty for the fourteen rates from 0.7x through 2.0x."""
    prepared = _prepare(notes, keys)
    output = _CAllRates()
    _check(_lib.minacalc_calc_all_rates(prepared, len(prepared), keys, 0 if mode == "msd" else 1, _ct.byref(output)))
    return tuple(_scores(score) for score in output.rates)


def calc_at_rate_detailed(notes: Iterable[NoteInput], rate: float, goal: float = 0.93,
                          keys: int = 4, mode: CalcMode = "ssr",
                          config: CalcConfig = CalcConfig()) -> DetailedResult:
    prepared, native_config, output = _prepare(notes, keys), _config(config), _CDetailed()
    _check(_lib.minacalc_calc_at_rate_with_config(prepared, len(prepared), rate, goal, keys,
        0 if mode == "msd" else 1, _ct.byref(native_config), _ct.byref(output)))
    return DetailedResult(_scores(output.scores), output.grind_scaler)


def calc_rates(notes: Iterable[NoteInput], rates: Iterable[float], keys: int = 4,
               mode: CalcMode = "msd", config: CalcConfig = CalcConfig()) -> tuple[SkillsetScores, ...]:
    prepared, values, native_config = _prepare(notes, keys), tuple(rates), _config(config)
    if not values:
        raise ValueError("rates must not be empty")
    native_rates, output = (_ct.c_float * len(values))(*values), (_CScores * len(values))()
    _check(_lib.minacalc_calc_rates(prepared, len(prepared), native_rates, len(values), keys,
        0 if mode == "msd" else 1, _ct.byref(native_config), output))
    return tuple(_scores(score) for score in output)


__all__ = ["CalcConfig", "CalcMode", "DetailedResult", "MinaCalcError", "Note",
           "SkillsetScalers", "SkillsetScores", "calc_all_rates", "calc_at_rate",
           "calc_at_rate_detailed", "calc_rates", "version"]

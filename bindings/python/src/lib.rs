//! Python bindings for minacalc-rs

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use minacalc_rs::{rox::RoxCalcExt, Calc, SkillsetScores};

/// Python wrapper for MinaCalc Calculator
#[pyclass(name = "Calculator")]
struct PyCalc {
    inner: Calc,
}

/// Python wrapper for SkillsetScores
#[pyclass(name = "SkillsetScores")]
#[derive(Clone)]
struct PySkillsetScores {
    #[pyo3(get)]
    overall: f32,
    #[pyo3(get)]
    stream: f32,
    #[pyo3(get)]
    jumpstream: f32,
    #[pyo3(get)]
    handstream: f32,
    #[pyo3(get)]
    stamina: f32,
    #[pyo3(get)]
    jackspeed: f32,
    #[pyo3(get)]
    chordjack: f32,
    #[pyo3(get)]
    technical: f32,
}

impl From<SkillsetScores> for PySkillsetScores {
    fn from(scores: SkillsetScores) -> Self {
        PySkillsetScores {
            overall: scores.overall,
            stream: scores.stream,
            jumpstream: scores.jumpstream,
            handstream: scores.handstream,
            stamina: scores.stamina,
            jackspeed: scores.jackspeed,
            chordjack: scores.chordjack,
            technical: scores.technical,
        }
    }
}

#[pymethods]
impl PySkillsetScores {
    fn __repr__(&self) -> String {
        format!(
            "SkillsetScores(overall={:.2}, stream={:.2}, jumpstream={:.2}, technical={:.2})",
            self.overall, self.stream, self.jumpstream, self.technical
        )
    }
}

#[pymethods]
impl PyCalc {
    #[new]
    fn new() -> PyResult<Self> {
        Ok(PyCalc {
            inner: Calc::new().map_err(|e| PyValueError::new_err(format!("{}", e)))?,
        })
    }

    /// Calculate scores for a specific rate from a chart file
    ///
    /// Args:
    ///     path (str): Path to the chart file (.osu, .sm, .rox)
    ///     music_rate (float): Music rate (e.g., 1.0 for normal, 1.5 for 1.5x)
    ///     score_goal (float): Score goal percentage (0-100)
    ///     chart_rate (float, optional): Chart rate modifier (default: None)
    ///     capped (bool, optional): Whether to cap the difficulty (SSR) or not (MSD) (default: False)
    ///
    /// Returns:
    ///     SkillsetScores: Skillset scores object with attributes for each skillset
    #[pyo3(signature = (path, music_rate, score_goal, chart_rate=None, capped=false))]
    fn calculate_at_rate_from_file(
        &self,
        path: &str,
        music_rate: f32,
        score_goal: f32,
        chart_rate: Option<f32>,
        capped: bool,
    ) -> PyResult<PySkillsetScores> {
        let scores = self
            .inner
            .calculate_at_rate_from_file(path, music_rate, score_goal, chart_rate, capped)
            .map_err(|e| PyValueError::new_err(format!("{}", e)))?;

        Ok(scores.into())
    }

    /// Calculate scores from chart string content
    #[pyo3(signature = (content, file_extension, music_rate, score_goal, chart_rate=None, capped=false))]
    fn calculate_at_rate_from_string(
        &self,
        content: &str,
        file_extension: &str,
        music_rate: f32,
        score_goal: f32,
        chart_rate: Option<f32>,
        capped: bool,
    ) -> PyResult<PySkillsetScores> {
        let scores = self
            .inner
            .calculate_at_rate_from_string(
                content,
                file_extension,
                music_rate,
                score_goal,
                chart_rate,
                capped,
            )
            .map_err(|e| PyValueError::new_err(format!("{}", e)))?;

        Ok(scores.into())
    }

    /// Calculate scores for all rates from a file
    /// capped: defaults to False (MSD)
    #[pyo3(signature = (path, capped=false))]
    fn calculate_all_rates_from_file(&self, path: &str, capped: bool) -> PyResult<Py<PyAny>> {
        let all_rates = self
            .inner
            .calculate_all_rates_from_file(path, capped)
            .map_err(|e| PyValueError::new_err(format!("{}", e)))?;

        Python::with_gil(|py| {
            let dict = pyo3::types::PyDict::new(py);

            for (i, scores) in all_rates.msds.iter().enumerate() {
                let rate = (i as f32) / 10.0 + 0.7;
                let key = format!("{:.1}", rate);
                let py_scores: PySkillsetScores = (*scores).into();
                dict.set_item(key, py_scores)?;
            }

            Ok(dict.into())
        })
    }

    /// Calculate scores for all rates from string content
    /// capped: defaults to False (MSD)
    #[pyo3(signature = (content, file_extension, capped=false))]
    fn calculate_all_rates_from_string(
        &self,
        content: &str,
        file_extension: &str,
        capped: bool,
    ) -> PyResult<Py<PyAny>> {
        let all_rates = self
            .inner
            .calculate_all_rates_from_string(content, file_extension, capped)
            .map_err(|e| PyValueError::new_err(format!("{}", e)))?;

        Python::with_gil(|py| {
            let dict = pyo3::types::PyDict::new(py);

            for (i, scores) in all_rates.msds.iter().enumerate() {
                let rate = (i as f32) / 10.0 + 0.7;
                let key = format!("{:.1}", rate);
                let py_scores: PySkillsetScores = (*scores).into();
                dict.set_item(key, py_scores)?;
            }

            Ok(dict.into())
        })
    }
}

/// MinaCalc Python module
///
/// This module provides Python bindings for the MinaCalc difficulty calculator,
/// used for rating rhythm game charts (primarily Etterna/StepMania).
///
/// Example:
///     >>> import minacalc_py
///     >>> calc = minacalc_py.Calculator()
///     >>> scores = calc.calculate_ssr_from_file("chart.osu", 1.0, 93.0)
///     >>> print(f"Overall difficulty: {scores.overall}")
#[pymodule]
fn minacalc_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyCalc>()?;
    m.add_class::<PySkillsetScores>()?;
    Ok(())
}

//! CRAP (Change Risk Analyzer and Predictor) metric evaluation.
//!
//! Formula: CRAP = CC² × (1 − Cov)³ + CC
//! Thresholds:
//! - CRAP <= 5: Low risk
//! - CRAP 5..=30: Moderate risk
//! - CRAP > 30: High risk (hazardous code)
//! - If coverage is missing (None): High risk if CC > 15

/// Result of evaluating the CRAP score of a function.
#[derive(Debug, Clone, PartialEq)]
pub struct CrapEvaluation {
    pub complexity: usize,
    pub coverage: Option<f64>,
    pub crap_score: Option<f64>,
    pub is_high_risk: bool,
}

/// Calculates the CRAP score given cyclomatic complexity and optional coverage ratio (0.0 to 1.0).
pub fn calculate_crap(complexity: usize, coverage: Option<f64>) -> CrapEvaluation {
    let cc = complexity as f64;
    match coverage {
        Some(cov) => {
            let clamped_cov = cov.clamp(0.0, 1.0);
            let score = (cc * cc) * (1.0 - clamped_cov).powi(3) + cc;
            let is_high_risk = score > 30.0;
            CrapEvaluation {
                complexity,
                coverage: Some(clamped_cov),
                crap_score: Some((score * 10.0).round() / 10.0),
                is_high_risk,
            }
        }
        None => {
            let is_high_risk = complexity > 15;
            CrapEvaluation {
                complexity,
                coverage: None,
                crap_score: None,
                is_high_risk,
            }
        }
    }
}

use crate::quality::complexity::calculate_complexity;
use crate::quality::coverage::parse_lcov;
use crate::quality::crap::calculate_crap;

#[test]
fn test_calculate_complexity_linear_function() {
    let source = r#"
fn simple_add(a: i32, b: i32) -> i32 {
    let sum = a + b;
    sum
}
"#;
    let functions = calculate_complexity(source);
    assert_eq!(functions.len(), 1);
    assert_eq!(functions[0].name, "simple_add");
    assert_eq!(functions[0].complexity, 1);
}

#[test]
fn test_calculate_complexity_with_branches() {
    let source = r#"
fn check_status(score: i32, is_admin: bool) -> &'static str {
    if is_admin || score > 100 {
        "admin_or_high"
    } else if score > 50 && score <= 100 {
        "medium"
    } else {
        "low"
    }
}
"#;
    let functions = calculate_complexity(source);
    assert_eq!(functions.len(), 1);
    assert_eq!(functions[0].name, "check_status");
    // Base 1 + if(1) + ||(1) + else if(1) + &&(1) = 5
    assert_eq!(functions[0].complexity, 5);
}

#[test]
fn test_calculate_complexity_with_match_and_question_mark() {
    let source = r#"
fn process_event(event: Option<i32>) -> Result<i32, String> {
    let val = event.ok_or_else(|| "missing".to_string())?;
    match val {
        1 => Ok(10),
        2 => Ok(20),
        _ => Err("unknown".to_string()),
    }
}
"#;
    let functions = calculate_complexity(source);
    assert_eq!(functions.len(), 1);
    assert_eq!(functions[0].name, "process_event");
    // Base 1 + ?(1) + match arms (3) = 5
    assert_eq!(functions[0].complexity, 5);
}

#[test]
fn test_parse_lcov_format() {
    let sample_lcov = r#"
TN:
SF:src/auth/token.rs
FN:10,validate_token
FNDA:12,validate_token
FNF:1
FNH:1
DA:10,12
DA:11,12
DA:12,0
LF:3
LH:2
end_of_record
"#;
    let coverage_map = parse_lcov(sample_lcov);
    let token_cov = coverage_map.get("src/auth/token.rs").copied();
    assert!(token_cov.is_some());
    let cov = token_cov.unwrap();
    // 2 hit out of 3 lines = 66.666%
    assert!((cov - (2.0 / 3.0)).abs() < 0.01);
}

#[test]
fn test_calculate_crap_formula() {
    // Formula: CRAP = CC^2 * (1 - cov)^3 + CC
    // Low risk: CC=2, Cov=1.0 (100%) -> CRAP = 4*(0)^3 + 2 = 2.0
    let low = calculate_crap(2, Some(1.0));
    assert_eq!(low.crap_score, Some(2.0));
    assert!(!low.is_high_risk);

    // High risk: CC=10, Cov=0.0 (0%) -> CRAP = 100*(1)^3 + 10 = 110.0
    let high = calculate_crap(10, Some(0.0));
    assert_eq!(high.crap_score, Some(110.0));
    assert!(high.is_high_risk);

    // No coverage fallback: CC=16, Cov=None -> High risk CC > 15
    let no_cov = calculate_crap(16, None);
    assert_eq!(no_cov.crap_score, None);
    assert!(no_cov.is_high_risk);
}

use crate::verification::diff::parse_unified_diff_lines;
use crate::verification::mutator::{
    apply_mutation, generate_mutations_for_source, MutationCandidate,
};

#[test]
fn test_parse_unified_diff_lines() {
    let diff_output = r#"
diff --git a/src/auth.rs b/src/auth.rs
index abc..def 100644
--- a/src/auth.rs
+++ b/src/auth.rs
@@ -10,0 +11,3 @@
+    if user_role == Role::Admin {
+        return true;
+    }
@@ -25,2 +28,1 @@
-    let score = old_calc();
+    let score = new_calc();
"#;
    let map = parse_unified_diff_lines(diff_output);
    assert_eq!(map.len(), 1);
    let lines = map.get("src/auth.rs").unwrap();
    // Lines added: 11, 12, 13 and 28
    assert_eq!(lines, &[11, 12, 13, 28]);
}

#[test]
fn test_generate_mutations_only_on_target_lines() {
    let source = "fn check_permission(user: &User) -> bool {
    // Line 2: untouched
    if user.is_active == false {
        return false;
    }
    // Line 6: untouched
    if user.score >= 100 && user.is_admin == true {
        return true;
    }
    false
}";
    // Line 7 is `if user.score >= 100 && user.is_admin == true {`
    let touched_lines = vec![7];
    let mutations = generate_mutations_for_source("src/auth.rs", source, &touched_lines, 2);

    assert!(!mutations.is_empty());
    assert!(mutations.len() <= 2);
    for m in &mutations {
        assert_eq!(m.line_number, 7);
        assert_eq!(m.file_path, "src/auth.rs");
        // Verify mutations are valid flips
        assert!(
            m.original == ">=" && m.replacement == "<"
                || m.original == "==" && m.replacement == "!="
                || m.original == "true" && m.replacement == "false"
        );
    }
}

#[test]
fn test_mutations_ignore_comments_and_strings() {
    let source = r#"
fn demo() {
    // Line 3: comment with == and true
    println!("status == true");
    let active = true;
}
"#;
    let touched_lines = vec![3, 4, 5];
    let mutations = generate_mutations_for_source("src/demo.rs", source, &touched_lines, 2);

    // Only line 5 has real code to mutate (`true` -> `false`)
    assert_eq!(mutations.len(), 1);
    assert_eq!(mutations[0].line_number, 5);
    assert_eq!(mutations[0].original, "true");
    assert_eq!(mutations[0].replacement, "false");
}

#[test]
fn test_apply_mutation() {
    let source = "fn is_ready() -> bool {\n    let ready = true;\n    ready == true\n}\n";
    let candidate = MutationCandidate {
        file_path: "src/status.rs".to_string(),
        line_number: 2,
        col_offset: 16,
        original: "true".to_string(),
        replacement: "false".to_string(),
        description: "invert true to false".to_string(),
    };
    let mutated = apply_mutation(source, &candidate);
    assert_eq!(
        mutated,
        "fn is_ready() -> bool {\n    let ready = false;\n    ready == true\n}\n"
    );
}

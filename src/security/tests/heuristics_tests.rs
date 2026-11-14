use tempfile::tempdir;

use super::*;

#[test]
fn test_detect_pii_schema_prisma() {
    let dir = tempdir().unwrap();
    let prisma_path = dir.path().join("schema.prisma");
    std::fs::write(
        &prisma_path,
        r#"
model User {
  id           Int      @id @default(autoincrement())
  email        String   @unique
  credit_card  String?
  ssn          String?
}
"#,
    )
    .unwrap();

    let suggestion = detect_pii_schema_opportunity(dir.path()).expect("expected suggestion");
    assert_eq!(suggestion.relative_path, "schema.prisma");
    assert!(suggestion
        .matched_fields
        .contains(&"credit_card".to_string()));
    assert!(suggestion.matched_fields.contains(&"ssn".to_string()));
}

#[test]
fn test_detect_pii_schema_migration_sql() {
    let dir = tempdir().unwrap();
    let migrations_dir = dir.path().join("migrations");
    std::fs::create_dir_all(&migrations_dir).unwrap();
    let sql_path = migrations_dir.join("001_create_patients.sql");
    std::fs::write(
        &sql_path,
        r#"
CREATE TABLE patients (
    id SERIAL PRIMARY KEY,
    patient_id VARCHAR(64) NOT NULL,
    date_of_birth DATE NOT NULL,
    diagnosis TEXT
);
"#,
    )
    .unwrap();

    let suggestion = detect_pii_schema_opportunity(dir.path()).expect("expected suggestion");
    assert!(suggestion.relative_path.starts_with("migrations/"));
    assert!(suggestion
        .matched_fields
        .contains(&"patient_id".to_string()));
    assert!(suggestion.matched_fields.contains(&"diagnosis".to_string()));
}

#[test]
fn test_detect_pii_schema_none_when_benign() {
    let dir = tempdir().unwrap();
    let models_dir = dir.path().join("models");
    std::fs::create_dir_all(&models_dir).unwrap();
    let ts_path = models_dir.join("product.ts");
    std::fs::write(
        &ts_path,
        r#"
export interface Product {
    id: string;
    name: string;
    price: number;
    description: string;
}
"#,
    )
    .unwrap();

    assert!(detect_pii_schema_opportunity(dir.path()).is_none());
}

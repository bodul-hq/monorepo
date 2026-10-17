use std::process::Command;

#[test]
fn startup_requires_database_configuration() {
    let output = Command::new(env!("CARGO_BIN_EXE_object_storage"))
        .env_remove("DATABASE_URL")
        .env("PORT", "0")
        .env(
            "LOG_DIR",
            std::env::temp_dir().join("object-storage-startup-tests"),
        )
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("DATABASE_URL is required"));
}

#[test]
fn startup_requires_a_reachable_database() {
    let output = Command::new(env!("CARGO_BIN_EXE_object_storage"))
        .env(
            "DATABASE_URL",
            "postgres://127.0.0.1:1/missing?connect_timeout=1",
        )
        .env("PORT", "0")
        .env(
            "LOG_DIR",
            std::env::temp_dir().join("object-storage-startup-tests"),
        )
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Database connection failed"));
}

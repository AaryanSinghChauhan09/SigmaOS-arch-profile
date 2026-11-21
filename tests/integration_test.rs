// Integration test for SigmaOS-profile
// Verifies the crate builds and links as a dependency.

use sigmaos_profile::SystemProfiler;

#[test]
fn crate_links_and_runs() {
    // Arrange
    let mut profiler = SystemProfiler::new();

    // Act
    let profile = profiler.collect_profile();

    // Assert
    assert!(profile.is_ok());
    let p = profile.unwrap();
    assert!(!p.os_name.is_empty());
}

//! Particle System unit tests for Phase 6: Particle System (TDD)
//!
//! Tests for ParticleCounts resource and spawn limit enforcement.

use deepdive::components::{ParticleCounts, ParticleType};

// =============================================================================
// 6.1.1 - ParticleCounts Resource: Updated Correctly
// =============================================================================

/// Test that ParticleCounts resource initializes to zero.
#[test]
fn particle_counts_default_is_zero() {
    let counts = ParticleCounts::default();

    assert_eq!(counts.primary_count, 0);
    assert_eq!(counts.secondary_count, 0);
}

/// Test that ParticleCounts can be updated.
#[test]
fn particle_counts_can_be_updated() {
    let mut counts = ParticleCounts::default();

    counts.primary_count = 150;
    counts.secondary_count = 50;

    assert_eq!(counts.primary_count, 150);
    assert_eq!(counts.secondary_count, 50);
}

/// Test total particle count calculation.
#[test]
fn particle_counts_total() {
    let mut counts = ParticleCounts::default();
    counts.primary_count = 100;
    counts.secondary_count = 25;

    assert_eq!(counts.total(), 125);
}

/// Test reset functionality.
#[test]
fn particle_counts_reset() {
    let mut counts = ParticleCounts::default();
    counts.primary_count = 200;
    counts.secondary_count = 75;

    counts.reset();

    assert_eq!(counts.primary_count, 0);
    assert_eq!(counts.secondary_count, 0);
}

/// Test increment methods.
#[test]
fn particle_counts_increment() {
    let mut counts = ParticleCounts::default();

    counts.increment(ParticleType::Primary);
    counts.increment(ParticleType::Primary);
    counts.increment(ParticleType::Secondary);

    assert_eq!(counts.primary_count, 2);
    assert_eq!(counts.secondary_count, 1);
}

// =============================================================================
// 6.1.2 - Spawn Limits: Respects Primary/Secondary Limits
// =============================================================================

/// Test that we can check if primary limit is reached.
#[test]
fn particle_counts_primary_limit_check() {
    let mut counts = ParticleCounts::default();
    let max_primary = 300;

    counts.primary_count = 150;
    assert!(!counts.is_primary_at_limit(max_primary));

    counts.primary_count = 300;
    assert!(counts.is_primary_at_limit(max_primary));

    counts.primary_count = 350;
    assert!(counts.is_primary_at_limit(max_primary));
}

/// Test that we can check if secondary limit is reached.
#[test]
fn particle_counts_secondary_limit_check() {
    let mut counts = ParticleCounts::default();
    let max_secondary = 75;

    counts.secondary_count = 50;
    assert!(!counts.is_secondary_at_limit(max_secondary));

    counts.secondary_count = 75;
    assert!(counts.is_secondary_at_limit(max_secondary));

    counts.secondary_count = 100;
    assert!(counts.is_secondary_at_limit(max_secondary));
}

/// Test combined limit check.
#[test]
fn particle_counts_combined_limit_check() {
    let mut counts = ParticleCounts::default();
    let max_primary = 300;
    let max_secondary = 75;

    // Neither at limit
    counts.primary_count = 100;
    counts.secondary_count = 25;
    assert!(!counts.is_primary_at_limit(max_primary));
    assert!(!counts.is_secondary_at_limit(max_secondary));

    // Primary at limit, secondary not
    counts.primary_count = 300;
    counts.secondary_count = 25;
    assert!(counts.is_primary_at_limit(max_primary));
    assert!(!counts.is_secondary_at_limit(max_secondary));

    // Both at limit
    counts.primary_count = 300;
    counts.secondary_count = 75;
    assert!(counts.is_primary_at_limit(max_primary));
    assert!(counts.is_secondary_at_limit(max_secondary));
}

/// Test space remaining calculation.
#[test]
fn particle_counts_space_remaining() {
    let mut counts = ParticleCounts::default();
    let max_primary = 300;
    let max_secondary = 75;

    counts.primary_count = 200;
    counts.secondary_count = 50;

    assert_eq!(counts.primary_space_remaining(max_primary), 100);
    assert_eq!(counts.secondary_space_remaining(max_secondary), 25);
}

/// Test space remaining when over limit returns zero.
#[test]
fn particle_counts_space_remaining_over_limit() {
    let mut counts = ParticleCounts::default();
    let max_primary = 300;

    counts.primary_count = 350; // Over the limit

    assert_eq!(counts.primary_space_remaining(max_primary), 0);
}

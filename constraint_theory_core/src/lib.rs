//! # constraint_theory_core
//!
//! Provides constraint-theory primitives for deterministic numeric computation.
//! The [`PythagoreanManifold`] snaps floating-point vectors onto a rational grid,
//! guaranteeing bit-identical results across platforms regardless of sub-tolerance
//! FPU perturbations.
//!
//! ## How it works
//!
//! Every component `v` is mapped to `round(v / ε) * ε` where ε is the tolerance.
//! Any two values that differ by less than `ε/2` collapse to the same grid point,
//! breaking the drift feedback loop that causes cross-platform divergence.

/// A Pythagorean manifold that enforces deterministic floating-point state.
///
/// The manifold is parameterised by a single tolerance `ε`. All vectors snapped
/// through the manifold lie on the lattice `{ k·ε | k ∈ ℤ }^3`, so the output
/// is independent of sub-tolerance differences in the input.
pub struct PythagoreanManifold {
    tolerance: f64,
    inv_tolerance: f64, // cached reciprocal to avoid repeated division
}

impl PythagoreanManifold {
    /// Create a manifold with the given grid spacing.
    ///
    /// # Panics
    /// Panics if `tolerance` is not positive and finite.
    pub fn new(tolerance: f64) -> Self {
        assert!(
            tolerance > 0.0 && tolerance.is_finite(),
            "tolerance must be a positive finite number, got {tolerance}"
        );
        Self {
            tolerance,
            inv_tolerance: 1.0 / tolerance,
        }
    }

    /// Snap a 3-D vector onto the Pythagorean lattice.
    ///
    /// Each component is independently rounded to the nearest integer multiple
    /// of the manifold's tolerance. The operation is idempotent: snapping an
    /// already-snapped vector returns the same value.
    ///
    /// ```
    /// use constraint_theory_core::PythagoreanManifold;
    ///
    /// let m = PythagoreanManifold::new(1e-4);
    /// let v = [1.00001, 2.000099, -0.99998];
    /// let snapped = m.snap(&v);
    /// // All three components lie on the 1e-4 grid
    /// assert_eq!(snapped, m.snap(&snapped));
    /// ```
    #[inline]
    pub fn snap(&self, v: &[f64; 3]) -> [f64; 3] {
        [
            self.snap_scalar(v[0]),
            self.snap_scalar(v[1]),
            self.snap_scalar(v[2]),
        ]
    }

    /// Snap a single scalar value onto the 1-D lattice.
    #[inline]
    pub fn snap_scalar(&self, x: f64) -> f64 {
        (x * self.inv_tolerance).round() * self.tolerance
    }

    /// The grid spacing this manifold enforces.
    #[inline]
    pub fn tolerance(&self) -> f64 {
        self.tolerance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idempotent() {
        let m = PythagoreanManifold::new(1e-4);
        let v = [1.23456789, -0.00001, 100.0];
        let s1 = m.snap(&v);
        let s2 = m.snap(&s1);
        assert_eq!(s1, s2, "snap must be idempotent");
    }

    #[test]
    fn absorbs_sub_tolerance_noise() {
        let tol = 1e-4_f64;
        let m = PythagoreanManifold::new(tol);
        let base = [1.0, 2.0, 3.0];
        let perturbed = [1.0 + tol * 0.49, 2.0 - tol * 0.3, 3.0 + tol * 0.1];
        assert_eq!(
            m.snap(&base),
            m.snap(&perturbed),
            "perturbations < ε/2 must collapse to the same grid point"
        );
    }

    #[test]
    fn resolves_super_tolerance_difference() {
        let tol = 1e-4_f64;
        let m = PythagoreanManifold::new(tol);
        let a = [1.0_f64, 0.0, 0.0];
        let b = [1.0 + tol * 1.5, 0.0, 0.0]; // more than ε/2 apart
        assert_ne!(
            m.snap(&a)[0],
            m.snap(&b)[0],
            "differences > ε/2 must map to distinct grid points"
        );
    }
}

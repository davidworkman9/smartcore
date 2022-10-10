//! # Collection of Distance Functions
//!
//! Many algorithms in machine learning require a measure of distance between data points. Distance metric (or metric) is a function that defines a distance between a pair of point elements of a set.
//! Formally, the distance can be any metric measure that is defined as \\( d(x, y) \geq 0\\) and follows three conditions:
//! 1. \\( d(x, y) = 0 \\) if and only \\( x = y \\), positive definiteness
//! 1. \\( d(x, y) = d(y, x) \\), symmetry
//! 1. \\( d(x, y) \leq d(x, z) + d(z, y) \\), subadditivity or triangle inequality
//!
//! for all \\(x, y, z \in Z \\)
//!
//! A good distance metric helps to improve the performance of classification, clustering and information retrieval algorithms significantly.
//!
//! <script src="https://polyfill.io/v3/polyfill.min.js?features=es6"></script>
//! <script id="MathJax-script" async src="https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-mml-chtml.js"></script>

/// Euclidean Distance is the straight-line distance between two points in Euclidean spacere that presents the shortest distance between these points.
pub mod euclidian;
/// Hamming Distance between two strings is the number of positions at which the corresponding symbols are different.
pub mod hamming;
/// The Mahalanobis distance is the distance between two points in multivariate space.
pub mod mahalanobis;
/// Also known as rectilinear distance, city block distance, taxicab metric.
pub mod manhattan;
/// A generalization of both the Euclidean distance and the Manhattan distance.
pub mod minkowski;

use crate::linalg::basic::arrays::Array2;
use crate::linalg::traits::lu::LUDecomposable;
use crate::numbers::basenum::Number;

/// Distance metric, a function that calculates distance between two points
pub trait Distance<T>: Clone {
    /// Calculates distance between _a_ and _b_
    fn distance(&self, a: &T, b: &T) -> f64;
}

/// Multitude of distance metric functions
pub struct Distances {}

impl Distances {
    /// Euclidian distance, see [`Euclidian`](euclidian/index.html)
    pub fn euclidian<T: Number>() -> euclidian::Euclidian<T> {
        euclidian::Euclidian::new()
    }

    /// Minkowski distance, see [`Minkowski`](minkowski/index.html)
    /// * `p` - function order. Should be >= 1
    pub fn minkowski<T: Number>(p: u16) -> minkowski::Minkowski<T> {
        minkowski::Minkowski::new(p)
    }

    /// Manhattan distance, see [`Manhattan`](manhattan/index.html)
    pub fn manhattan<T: Number>() -> manhattan::Manhattan<T> {
        manhattan::Manhattan::new()
    }

    /// Hamming distance, see [`Hamming`](hamming/index.html)
    pub fn hamming<T: Number>() -> hamming::Hamming<T> {
        hamming::Hamming::new()
    }

    /// Mahalanobis distance, see [`Mahalanobis`](mahalanobis/index.html)
    pub fn mahalanobis<T: Number, M: Array2<T>, C: Array2<f64> + LUDecomposable<f64>>(
        data: &M,
    ) -> mahalanobis::Mahalanobis<T, C> {
        mahalanobis::Mahalanobis::new(data)
    }
}

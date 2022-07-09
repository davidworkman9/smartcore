//! # Random Forest Classifier
//! A random forest is an ensemble estimator that fits multiple [decision trees](../../tree/index.html) to random subsets of the dataset and averages predictions
//! to improve the predictive accuracy and control over-fitting. See [ensemble models](../index.html) for more details.
//!
//! Bigger number of estimators in general improves performance of the algorithm with an increased cost of training time.
//! The random sample of _m_ predictors is typically set to be \\(\sqrt{p}\\) from the full set of _p_ predictors.
//!
//! Example:
//!
//! ```
//! use smartcore::linalg::naive::dense_matrix::*;
//! use smartcore::ensemble::random_forest_classifier::RandomForestClassifier;
//!
//! // Iris dataset
//! let x = DenseMatrix::from_2d_array(&[
//!              &[5.1, 3.5, 1.4, 0.2],
//!              &[4.9, 3.0, 1.4, 0.2],
//!              &[4.7, 3.2, 1.3, 0.2],
//!              &[4.6, 3.1, 1.5, 0.2],
//!              &[5.0, 3.6, 1.4, 0.2],
//!              &[5.4, 3.9, 1.7, 0.4],
//!              &[4.6, 3.4, 1.4, 0.3],
//!              &[5.0, 3.4, 1.5, 0.2],
//!              &[4.4, 2.9, 1.4, 0.2],
//!              &[4.9, 3.1, 1.5, 0.1],
//!              &[7.0, 3.2, 4.7, 1.4],
//!              &[6.4, 3.2, 4.5, 1.5],
//!              &[6.9, 3.1, 4.9, 1.5],
//!              &[5.5, 2.3, 4.0, 1.3],
//!              &[6.5, 2.8, 4.6, 1.5],
//!              &[5.7, 2.8, 4.5, 1.3],
//!              &[6.3, 3.3, 4.7, 1.6],
//!              &[4.9, 2.4, 3.3, 1.0],
//!              &[6.6, 2.9, 4.6, 1.3],
//!              &[5.2, 2.7, 3.9, 1.4],
//!         ]);
//! let y = vec![
//!              0., 0., 0., 0., 0., 0., 0., 0.,
//!              1., 1., 1., 1., 1., 1., 1., 1., 1., 1., 1., 1.,
//!         ];
//!
//! let classifier = RandomForestClassifier::fit(&x, &y, Default::default()).unwrap();
//! let y_hat = classifier.predict(&x).unwrap(); // use the same data for prediction
//! ```
//!
//! <script src="https://polyfill.io/v3/polyfill.min.js?features=es6"></script>
//! <script id="MathJax-script" async src="https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-mml-chtml.js"></script>
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::default::Default;
use std::fmt::Debug;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::api::{Predictor, SupervisedEstimator};
use crate::error::{Failed, FailedError};
use crate::linalg::{BaseMatrix, Matrix};
use crate::math::num::RealNumber;
use crate::tree::decision_tree_classifier::{
    which_max, DecisionTreeClassifier, DecisionTreeClassifierParameters, SplitCriterion,
};

/// Parameters of the Random Forest algorithm.
/// Some parameters here are passed directly into base estimator.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct RandomForestClassifierParameters {
    /// Split criteria to use when building a tree. See [Decision Tree Classifier](../../tree/decision_tree_classifier/index.html)
    pub criterion: SplitCriterion,
    /// Tree max depth. See [Decision Tree Classifier](../../tree/decision_tree_classifier/index.html)
    pub max_depth: Option<u16>,
    /// The minimum number of samples required to be at a leaf node. See [Decision Tree Classifier](../../tree/decision_tree_classifier/index.html)
    pub min_samples_leaf: usize,
    /// The minimum number of samples required to split an internal node. See [Decision Tree Classifier](../../tree/decision_tree_classifier/index.html)
    pub min_samples_split: usize,
    /// The number of trees in the forest.
    pub n_trees: u16,
    /// Number of random sample of predictors to use as split candidates.
    pub m: Option<usize>,
    /// Whether to keep samples used for tree generation. This is required for OOB prediction.
    pub keep_samples: bool,
    /// First seed used for bootstrap sampling and feature selection for each tree.
    pub base_seed: u64,
    /// The number of threads to use for tree building, if set to 0 then num_threads will be chosen
    /// automatically by the rayon runtime
    pub num_threads: usize,
}

/// Random Forest Classifier
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct RandomForestClassifier<T: RealNumber> {
    _parameters: RandomForestClassifierParameters,
    trees: Vec<DecisionTreeClassifier<T>>,
    classes: Vec<T>,
    samples: Option<Vec<Vec<bool>>>,
}

impl RandomForestClassifierParameters {
    /// Split criteria to use when building a tree. See [Decision Tree Classifier](../../tree/decision_tree_classifier/index.html)
    pub fn with_criterion(mut self, criterion: SplitCriterion) -> Self {
        self.criterion = criterion;
        self
    }
    /// Tree max depth. See [Decision Tree Classifier](../../tree/decision_tree_classifier/index.html)
    pub fn with_max_depth(mut self, max_depth: u16) -> Self {
        self.max_depth = Some(max_depth);
        self
    }
    /// The minimum number of samples required to be at a leaf node. See [Decision Tree Classifier](../../tree/decision_tree_classifier/index.html)
    pub fn with_min_samples_leaf(mut self, min_samples_leaf: usize) -> Self {
        self.min_samples_leaf = min_samples_leaf;
        self
    }
    /// The minimum number of samples required to split an internal node. See [Decision Tree Classifier](../../tree/decision_tree_classifier/index.html)
    pub fn with_min_samples_split(mut self, min_samples_split: usize) -> Self {
        self.min_samples_split = min_samples_split;
        self
    }
    /// The number of trees in the forest.
    pub fn with_n_trees(mut self, n_trees: u16) -> Self {
        self.n_trees = n_trees;
        self
    }
    /// Number of random sample of predictors to use as split candidates.
    pub fn with_m(mut self, m: usize) -> Self {
        self.m = Some(m);
        self
    }

    /// Whether to keep samples used for tree generation. This is required for OOB prediction.
    pub fn with_keep_samples(mut self, keep_samples: bool) -> Self {
        self.keep_samples = keep_samples;
        self
    }

    /// First seed used for bootstrap sampling and feature selection for each tree.
    pub fn with_base_seed(mut self, seed: u64) -> Self {
        self.base_seed = seed;
        self
    }

    /// The number of threads to use for tree building, if set to 0 then num_threads will be chosen
    /// automatically by the rayon runtime
    pub fn with_num_threads(mut self, num_threads: usize) -> Self {
        self.num_threads = num_threads;
        self
    }
}

impl<T: RealNumber> PartialEq for RandomForestClassifier<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.classes.len() != other.classes.len() || self.trees.len() != other.trees.len() {
            false
        } else {
            for i in 0..self.classes.len() {
                if (self.classes[i] - other.classes[i]).abs() > T::epsilon() {
                    return false;
                }
            }
            for i in 0..self.trees.len() {
                if self.trees[i] != other.trees[i] {
                    return false;
                }
            }
            true
        }
    }
}

impl Default for RandomForestClassifierParameters {
    fn default() -> Self {
        RandomForestClassifierParameters {
            criterion: SplitCriterion::Gini,
            max_depth: None,
            min_samples_leaf: 1,
            min_samples_split: 2,
            n_trees: 100,
            m: Option::None,
            keep_samples: false,
            base_seed: 0,
            num_threads: 1,
        }
    }
}

impl<T: RealNumber, M: Matrix<T>>
    SupervisedEstimator<M, M::RowVector, RandomForestClassifierParameters>
    for RandomForestClassifier<T>
where
    <M as BaseMatrix<T>>::RowVector: Sync + Send,
    M: std::marker::Sync,
{
    fn fit(
        x: &M,
        y: &M::RowVector,
        parameters: RandomForestClassifierParameters,
    ) -> Result<Self, Failed> {
        RandomForestClassifier::fit(x, y, parameters)
    }
}

impl<T: RealNumber, M: Matrix<T>> Predictor<M, M::RowVector> for RandomForestClassifier<T> {
    fn predict(&self, x: &M) -> Result<M::RowVector, Failed> {
        self.predict(x)
    }
}

impl<T: RealNumber> RandomForestClassifier<T> {
    /// Build a forest of trees from the training set.
    /// * `x` - _NxM_ matrix with _N_ observations and _M_ features in each observation.
    /// * `y` - the target class values
    pub fn fit<M: Matrix<T>>(
        x: &M,
        y: &M::RowVector,
        parameters: RandomForestClassifierParameters,
    ) -> Result<RandomForestClassifier<T>, Failed>
    where
        <M as BaseMatrix<T>>::RowVector: Sync + Send,
        M: std::marker::Sync,
    {
        let (_, num_attributes) = x.shape();
        let y_m = M::from_row_vector(y.clone());
        let (_, y_ncols) = y_m.shape();
        let mut yi: Vec<usize> = vec![0; y_ncols];
        let classes = y_m.unique();

        for (i, yi_i) in yi.iter_mut().enumerate().take(y_ncols) {
            let yc = y_m.get(0, i);
            *yi_i = classes.iter().position(|c| yc == *c).unwrap();
        }

        let mtry = parameters.m.unwrap_or_else(|| {
            (T::from(num_attributes).unwrap())
                .sqrt()
                .floor()
                .to_usize()
                .unwrap()
        });

        let classes = y_m.unique();
        let k = classes.len();

        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(parameters.num_threads)
            .build()
            .unwrap();

        let tree_sample_pairs = thread_pool.install(|| {
            RandomForestClassifier::<T>::collect_tree_sample_pairs(
                x,
                y,
                parameters.clone(),
                yi,
                k,
                mtry,
            )
        });

        let (trees, samples) =
            RandomForestClassifier::<T>::parse_tree_sample_pairs(tree_sample_pairs);
        Ok(RandomForestClassifier {
            _parameters: parameters,
            trees,
            classes,
            samples,
        })
    }

    fn collect_tree_sample_pairs<M: Matrix<T>>(
        x: &M,
        y: &M::RowVector,
        parameters: RandomForestClassifierParameters,
        yi: Vec<usize>,
        k: usize,
        mtry: usize,
    ) -> Vec<(DecisionTreeClassifier<T>, Option<Vec<bool>>)>
    where
        <M as BaseMatrix<T>>::RowVector: Sync + Send,
        M: std::marker::Sync,
    {
        (0..parameters.n_trees)
            .into_par_iter()
            .map(|tree_number| {
                let decision_tree_params = DecisionTreeClassifierParameters {
                    criterion: parameters.criterion.clone(),
                    max_depth: parameters.max_depth,
                    min_samples_leaf: parameters.min_samples_leaf,
                    min_samples_split: parameters.min_samples_split,
                };

                let mut rng = StdRng::seed_from_u64(parameters.base_seed + tree_number as u64);
                let samples =
                    RandomForestClassifier::<T>::sample_with_replacement(&yi, k, &mut rng);
                let relevant_samples: Option<Vec<bool>> = match parameters.keep_samples {
                    true => Some(samples.iter().map(|x| *x != 0).collect()),
                    false => None,
                };

                (
                    DecisionTreeClassifier::fit_weak_learner(
                        x,
                        y,
                        samples,
                        mtry,
                        decision_tree_params,
                        &mut rng,
                    )
                    .unwrap(),
                    relevant_samples,
                )
            })
            .collect()
    }

    fn parse_tree_sample_pairs(
        tree_sample_pairs: Vec<(DecisionTreeClassifier<T>, Option<Vec<bool>>)>,
    ) -> (Vec<DecisionTreeClassifier<T>>, Option<Vec<Vec<bool>>>) {
        let mut trees = vec![];
        let mut samples = vec![];
        tree_sample_pairs
            .into_iter()
            .for_each(|(tree, samples_for_tree)| {
                trees.push(tree);
                if samples_for_tree.is_some() {
                    samples.push(samples_for_tree.unwrap());
                }
            });
        let samples = match samples.len() {
            0 => None,
            _ => Some(samples),
        };
        (trees, samples)
    }

    /// Predict class for `x`
    /// * `x` - _KxM_ data where _K_ is number of observations and _M_ is number of features.
    pub fn predict<M: Matrix<T>>(&self, x: &M) -> Result<M::RowVector, Failed> {
        let mut result = M::zeros(1, x.shape().0);

        let (n, _) = x.shape();

        for i in 0..n {
            result.set(0, i, self.classes[self.predict_for_row(x, i)]);
        }

        Ok(result.to_row_vector())
    }

    fn predict_for_row<M: Matrix<T>>(&self, x: &M, row: usize) -> usize {
        let mut result = vec![0; self.classes.len()];

        for tree in self.trees.iter() {
            result[tree.predict_for_row(x, row)] += 1;
        }

        which_max(&result)
    }

    /// Predict OOB classes for `x`. `x` is expected to be equal to the dataset used in training.
    pub fn predict_oob<M: Matrix<T>>(&self, x: &M) -> Result<M::RowVector, Failed> {
        let (n, _) = x.shape();
        if self.samples.is_none() {
            Err(Failed::because(
                FailedError::PredictFailed,
                "Need samples=true for OOB predictions.",
            ))
        } else if self.samples.as_ref().unwrap()[0].len() != n {
            Err(Failed::because(
                FailedError::PredictFailed,
                "Prediction matrix must match matrix used in training for OOB predictions.",
            ))
        } else {
            let mut result = M::zeros(1, n);

            for i in 0..n {
                result.set(0, i, self.classes[self.predict_for_row_oob(x, i)]);
            }

            Ok(result.to_row_vector())
        }
    }

    fn predict_for_row_oob<M: Matrix<T>>(&self, x: &M, row: usize) -> usize {
        let mut result = vec![0; self.classes.len()];

        for (tree, samples) in self.trees.iter().zip(self.samples.as_ref().unwrap()) {
            if !samples[row] {
                result[tree.predict_for_row(x, row)] += 1;
            }
        }

        which_max(&result)
    }

    fn sample_with_replacement(y: &[usize], num_classes: usize, rng: &mut impl Rng) -> Vec<usize> {
        let class_weight = vec![1.; num_classes];
        let nrows = y.len();
        let mut samples = vec![0; nrows];
        for (l, class_weight_l) in class_weight.iter().enumerate().take(num_classes) {
            let mut n_samples = 0;
            let mut index: Vec<usize> = Vec::new();
            for (i, y_i) in y.iter().enumerate().take(nrows) {
                if *y_i == l {
                    index.push(i);
                    n_samples += 1;
                }
            }

            let size = ((n_samples as f64) / *class_weight_l) as usize;
            for _ in 0..size {
                let xi: usize = rng.gen_range(0..n_samples);
                samples[index[xi]] += 1;
            }
        }
        samples
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset::digits;
    use crate::linalg::naive::dense_matrix::DenseMatrix;
    use crate::metrics::*;
    use crate::model_selection::train_test_split;

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn fit_predict_digits() {
        let dataset_digits = digits::load_dataset();
        let nrows = dataset_digits.target.len();
        let ncols = dataset_digits.num_features;
        let values = dataset_digits.data;
        let x = DenseMatrix::from_vec(nrows, ncols, &values);
        let y = dataset_digits.target;

        let (x_train, x_test, y_train, y_test) = train_test_split(&x, &y, 0.2, true);
        let classifier = RandomForestClassifier::fit(
            &x_train,
            &y_train,
            RandomForestClassifierParameters {
                criterion: SplitCriterion::Gini,
                max_depth: None,
                min_samples_leaf: 1,
                min_samples_split: 2,
                n_trees: 100,
                m: Option::None,
                keep_samples: false,
                base_seed: 87,
                num_threads: 10,
            },
        )
        .unwrap();

        let test_predictions = &classifier.predict(&x_test).unwrap();
        let accuracy_score = accuracy(&y_test, test_predictions);
        assert!(accuracy_score >= 0.95);
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn fit_predict_iris() {
        let x = DenseMatrix::from_2d_array(&[
            &[5.1, 3.5, 1.4, 0.2],
            &[4.9, 3.0, 1.4, 0.2],
            &[4.7, 3.2, 1.3, 0.2],
            &[4.6, 3.1, 1.5, 0.2],
            &[5.0, 3.6, 1.4, 0.2],
            &[5.4, 3.9, 1.7, 0.4],
            &[4.6, 3.4, 1.4, 0.3],
            &[5.0, 3.4, 1.5, 0.2],
            &[4.4, 2.9, 1.4, 0.2],
            &[4.9, 3.1, 1.5, 0.1],
            &[7.0, 3.2, 4.7, 1.4],
            &[6.4, 3.2, 4.5, 1.5],
            &[6.9, 3.1, 4.9, 1.5],
            &[5.5, 2.3, 4.0, 1.3],
            &[6.5, 2.8, 4.6, 1.5],
            &[5.7, 2.8, 4.5, 1.3],
            &[6.3, 3.3, 4.7, 1.6],
            &[4.9, 2.4, 3.3, 1.0],
            &[6.6, 2.9, 4.6, 1.3],
            &[5.2, 2.7, 3.9, 1.4],
        ]);
        let y = vec![
            0., 0., 0., 0., 0., 0., 0., 0., 1., 1., 1., 1., 1., 1., 1., 1., 1., 1., 1., 1.,
        ];

        let classifier = RandomForestClassifier::fit(
            &x,
            &y,
            RandomForestClassifierParameters {
                criterion: SplitCriterion::Gini,
                max_depth: None,
                min_samples_leaf: 1,
                min_samples_split: 2,
                n_trees: 100,
                m: Option::None,
                keep_samples: false,
                base_seed: 87,
                num_threads: 10,
            },
        )
        .unwrap();

        assert!(accuracy(&y, &classifier.predict(&x).unwrap()) >= 0.95);
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn fit_predict_iris_oob() {
        let x = DenseMatrix::from_2d_array(&[
            &[5.1, 3.5, 1.4, 0.2],
            &[4.9, 3.0, 1.4, 0.2],
            &[4.7, 3.2, 1.3, 0.2],
            &[4.6, 3.1, 1.5, 0.2],
            &[5.0, 3.6, 1.4, 0.2],
            &[5.4, 3.9, 1.7, 0.4],
            &[4.6, 3.4, 1.4, 0.3],
            &[5.0, 3.4, 1.5, 0.2],
            &[4.4, 2.9, 1.4, 0.2],
            &[4.9, 3.1, 1.5, 0.1],
            &[7.0, 3.2, 4.7, 1.4],
            &[6.4, 3.2, 4.5, 1.5],
            &[6.9, 3.1, 4.9, 1.5],
            &[5.5, 2.3, 4.0, 1.3],
            &[6.5, 2.8, 4.6, 1.5],
            &[5.7, 2.8, 4.5, 1.3],
            &[6.3, 3.3, 4.7, 1.6],
            &[4.9, 2.4, 3.3, 1.0],
            &[6.6, 2.9, 4.6, 1.3],
            &[5.2, 2.7, 3.9, 1.4],
        ]);
        let y = vec![
            0., 0., 0., 0., 0., 0., 0., 0., 1., 1., 1., 1., 1., 1., 1., 1., 1., 1., 1., 1.,
        ];

        let classifier = RandomForestClassifier::fit(
            &x,
            &y,
            RandomForestClassifierParameters {
                criterion: SplitCriterion::Gini,
                max_depth: None,
                min_samples_leaf: 1,
                min_samples_split: 2,
                n_trees: 100,
                m: Option::None,
                keep_samples: true,
                base_seed: 87,
                num_threads: 10,
            },
        )
        .unwrap();

        assert!(
            accuracy(&y, &classifier.predict_oob(&x).unwrap())
                < accuracy(&y, &classifier.predict(&x).unwrap())
        );
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    #[cfg(feature = "serde")]
    fn serde() {
        let x = DenseMatrix::from_2d_array(&[
            &[5.1, 3.5, 1.4, 0.2],
            &[4.9, 3.0, 1.4, 0.2],
            &[4.7, 3.2, 1.3, 0.2],
            &[4.6, 3.1, 1.5, 0.2],
            &[5.0, 3.6, 1.4, 0.2],
            &[5.4, 3.9, 1.7, 0.4],
            &[4.6, 3.4, 1.4, 0.3],
            &[5.0, 3.4, 1.5, 0.2],
            &[4.4, 2.9, 1.4, 0.2],
            &[4.9, 3.1, 1.5, 0.1],
            &[7.0, 3.2, 4.7, 1.4],
            &[6.4, 3.2, 4.5, 1.5],
            &[6.9, 3.1, 4.9, 1.5],
            &[5.5, 2.3, 4.0, 1.3],
            &[6.5, 2.8, 4.6, 1.5],
            &[5.7, 2.8, 4.5, 1.3],
            &[6.3, 3.3, 4.7, 1.6],
            &[4.9, 2.4, 3.3, 1.0],
            &[6.6, 2.9, 4.6, 1.3],
            &[5.2, 2.7, 3.9, 1.4],
        ]);
        let y = vec![
            0., 0., 0., 0., 0., 0., 0., 0., 1., 1., 1., 1., 1., 1., 1., 1., 1., 1., 1., 1.,
        ];

        let forest = RandomForestClassifier::fit(&x, &y, Default::default()).unwrap();

        let deserialized_forest: RandomForestClassifier<f64> =
            bincode::deserialize(&bincode::serialize(&forest).unwrap()).unwrap();

        assert_eq!(forest, deserialized_forest);
    }
}

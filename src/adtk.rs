use medians::Medianf64;

pub trait Detector {
    fn fit(&mut self, data: &[f64]);
    fn detect(&self, data: &[f64]) -> Vec<usize>;

    fn fit_detect(&mut self, data: &[f64]) -> Vec<usize> {
        self.fit(data);
        self.detect(data)
    }
}

pub struct LevelShiftAD {
    pub w: usize,
    pub c: f64,

    fitted_lower_bound: Option<f64>,
    fitted_upper_bound: Option<f64>,
}

impl LevelShiftAD {
    pub fn new(w: usize, c: f64) -> Self {
        Self {
            w,
            c,
            fitted_lower_bound: None,
            fitted_upper_bound: None,
        }
    }

    /// Computes D[t] = M_right[t] - M_left[t] for the entire sequence
    fn calculate_median_differences(&self, data: &[f64]) -> Vec<f64> {
        let n = data.len();
        let w = self.w;

        let mut diffs = vec![0.0; n];

        if n < w * 2 {
            return diffs;
        }

        for t in w..=(n - w) {
            let left_window = &data[t - w..t];
            let right_window = &data[t..t + w];

            let left_med = left_window.medf_unchecked();
            let right_med = right_window.medf_unchecked();

            diffs[t] = right_med - left_med;
        }

        diffs
    }
}

impl Detector for LevelShiftAD {
    fn fit(&mut self, data: &[f64]) {
        if data.len() < self.w * 2 {
            return;
        }

        let diffs = self.calculate_median_differences(data);

        let w = self.w;
        let mut valid_diffs = diffs[w..=(data.len() - w)].to_vec();

        let len = valid_diffs.len();
        let q1_idx = len / 4;
        let q3_idx = (len * 3) / 4;
        let (_, &mut q1, _) = valid_diffs.select_nth_unstable_by(q1_idx, |a, b| a.total_cmp(b));
        let (_, &mut q3, _) = valid_diffs.select_nth_unstable_by(q3_idx, |a, b| a.total_cmp(b));

        let iqr = q3 - q1;

        self.fitted_lower_bound = Some(q1 - self.c * iqr);
        self.fitted_upper_bound = Some(q3 + self.c * iqr);
    }

    fn detect(&self, data: &[f64]) -> Vec<usize> {
        let n = data.len();
        let mut anomalies = Vec::new();

        if n < self.w * 2 || self.fitted_lower_bound.is_none() {
            return anomalies;
        }

        let lower = self.fitted_lower_bound.unwrap();
        let upper = self.fitted_upper_bound.unwrap();

        let diffs = self.calculate_median_differences(data);

        let w = self.w;
        for (t, diff) in diffs.iter().enumerate().take((n - w) + 1).skip(w) {
            if *diff < lower || *diff > upper {
                anomalies.push(t);
            }
        }

        anomalies
    }

    fn fit_detect(&mut self, data: &[f64]) -> Vec<usize> {
        let n = data.len();
        if n < self.w * 2 {
            return Vec::new();
        }

        let diffs = self.calculate_median_differences(data);

        let w = self.w;
        let mut valid_diffs = diffs[w..=(data.len() - w)].to_vec();

        let len = valid_diffs.len();
        let q1_idx = len / 4;
        let q3_idx = (len * 3) / 4;
        let (_, &mut q1, _) = valid_diffs.select_nth_unstable_by(q1_idx, |a, b| a.total_cmp(b));
        let (_, &mut q3, _) = valid_diffs.select_nth_unstable_by(q3_idx, |a, b| a.total_cmp(b));

        let iqr = q3 - q1;

        self.fitted_lower_bound = Some(q1 - self.c * iqr);
        self.fitted_upper_bound = Some(q3 + self.c * iqr);

        let mut anomalies = Vec::new();

        if n < self.w * 2 || self.fitted_lower_bound.is_none() {
            return anomalies;
        }

        let lower = self.fitted_lower_bound.unwrap();
        let upper = self.fitted_upper_bound.unwrap();

        let w = self.w;

        for (t, diff) in diffs.iter().enumerate().take((n - w) + 1).skip(w) {
            if *diff < lower || *diff > upper {
                anomalies.push(t);
            }
        }

        anomalies
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_shift_ad_short_data() {
        let mut detector = LevelShiftAD::new(5, 1.5);
        let data = vec![1.0, 2.0, 3.0];

        let diffs = detector.calculate_median_differences(&data);
        assert_eq!(diffs.len(), 3);
        assert_eq!(diffs, vec![0.0, 0.0, 0.0]);

        detector.fit(&data);
        assert!(detector.fitted_lower_bound.is_none());
        assert!(detector.fitted_upper_bound.is_none());

        let anomalies = detector.detect(&data);
        assert!(anomalies.is_empty());
    }

    #[test]
    fn test_exact_window() {
        let mut detector = LevelShiftAD::new(2, 1.5);
        let data = vec![1.0, 2.0, 3.0, 4.0];
        // n = 4, w = 2. w..=2 means one valid element: t=2.
        detector.fit(&data);
        assert!(detector.fitted_lower_bound.is_some());
        assert!(detector.fitted_upper_bound.is_some());
    }

    #[test]
    fn test_calculate_median_differences() {
        let detector = LevelShiftAD::new(2, 1.5);
        // length = 6, w = 2. t ranges from 2 to 4.
        let data = vec![1.0, 1.0, 10.0, 10.0, 1.0, 1.0];
        // t=2 (left: [1,1] => 1, right: [10,10] => 10) => diff = 9.0
        // t=3 (left: [1,10] => 5.5, right: [10,1] => 5.5) => diff = 0.0
        // t=4 (left: [10,10] => 10, right: [1,1] => 1) => diff = -9.0

        let diffs = detector.calculate_median_differences(&data);
        assert_eq!(diffs.len(), 6);
        assert_eq!(diffs[0], 0.0);
        assert_eq!(diffs[1], 0.0);
        assert_eq!(diffs[2], 9.0);
        assert_eq!(diffs[3], 0.0);
        assert_eq!(diffs[4], -9.0);
        assert_eq!(diffs[5], 0.0);
    }

    #[test]
    fn test_fit() {
        let mut detector = LevelShiftAD::new(2, 1.0); // c=1.0
        let data = vec![1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0];
        // length = 8, w = 2. t from 2 to 6.
        // t=2 (1,1 -> 2,2) diff = 1.0
        // t=3 (1,2 -> 2,3) diff = 1.0
        // t=4 (2,2 -> 3,3) diff = 1.0
        // t=5 (2,3 -> 3,4) diff = 1.0
        // t=6 (3,3 -> 4,4) diff = 1.0
        // valid_diffs = [1.0, 1.0, 1.0, 1.0, 1.0] -> Q1=1, Q3=1 -> IQR=0 -> lower=1.0, upper=1.0
        detector.fit(&data);
        assert_eq!(detector.fitted_lower_bound, Some(1.0));
        assert_eq!(detector.fitted_upper_bound, Some(1.0));
    }

    #[test]
    fn test_detect_anomalies() {
        let mut detector = LevelShiftAD::new(2, 1.5);
        let data_fit = vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        detector.fit(&data_fit);
        // Valid background: IQR=0, bounds are 0.

        // Data with sudden level shift
        let data_test = vec![0.0, 0.0, 0.0, 0.0, 10.0, 10.0, 10.0, 10.0];
        // w=2, t in 2..=6
        // t=2: diff = 0
        // t=3: left=[0,0] => 0, right=[0,10] => 5. diff = 5.
        // t=4: left=[0,0] => 0, right=[10,10] => 10. diff = 10.
        // t=5: left=[0,10] => 5, right=[10,10] => 10. diff = 5.
        // t=6: diff = 0

        let anomalies = detector.detect(&data_test);
        assert!(anomalies.contains(&3));
        assert!(anomalies.contains(&4));
        assert!(anomalies.contains(&5));
    }

    #[test]
    fn test_fit_detect() {
        let mut detector = LevelShiftAD::new(2, 1.5);
        let mut data = vec![0.0; 20];
        data[10..20].fill(10.0);

        let anomalies = detector.fit_detect(&data);
        // Strongest anomaly strictly at 10. Due to small c and IQR=0,
        // 9, 10, 11 will show nonzero differences and be caught as anomalies.
        assert!(anomalies.contains(&9));
        assert!(anomalies.contains(&10));
        assert!(anomalies.contains(&11));
    }

    //     #[test]
    //     #[ignore]
    //     fn test_against_python_adtk() -> pyo3::PyResult<()> {
    //         use pyo3::prelude::*;

    //         let data: Vec<f64> = vec![0.0, 0.0, 0.0, 0.0, 10.0, 10.0, 10.0, 10.0];
    //         let window = 2;
    //         let c = 1.5;

    //         // 1. Run your Rust implementation
    //         let mut rust_detector = LevelShiftAD::new(window, c);
    //         let rust_anomalies = rust_detector.fit_detect(&data);

    //         // 2. Run the Python implementation
    //         Python::with_gil(|py| {
    //             let py_code = r#"
    // import pandas as pd
    // from adtk.detector import LevelShiftAD

    // def run_adtk(data, window, c):
    //     # ADTK requires a DatetimeIndex to function properly
    //     idx = pd.date_range("2020-01-01", periods=len(data))
    //     s = pd.Series(data, index=idx)

    //     detector = LevelShiftAD(window=window, c=c)
    //     anomalies = detector.fit_detect(s)

    //     # Return a list of integer indices where anomalies is True
    //     return [i for i, is_anom in enumerate(anomalies) if is_anom == True]
    // "#;
    //             // Compile the Python code inline
    //             let module = PyModule::from_code_bound(py, py_code, "adtk_wrapper.py", "adtk_wrapper")?;

    //             // Call our custom python function with the Rust data inputs
    //             let py_anomalies: Vec<usize> = module
    //                 .getattr("run_adtk")?
    //                 .call1((data, window, c))?
    //                 .extract()?;

    //             // 3. Compare ground-truth Python results with our Rust results!
    //             assert_eq!(rust_anomalies, py_anomalies);

    //             Ok::<(), PyErr>(())
    //         })?;

    //         Ok(())
    //     }
}

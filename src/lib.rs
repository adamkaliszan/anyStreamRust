extern crate core;

mod sim;

#[cfg(test)]
mod tests {
    use approx::*;
    use rand::rngs::ThreadRng;

    use crate::sim::model::class::*;
    use crate::sim::model::class::StreamType::{Gamma, Pareto, Poisson, Uniform};

    fn mean(data: &[f64]) -> Option<f64> {
        let sum = data.iter().sum::<f64>() as f64;
        let count = data.len();

        match count {
            positive if positive > 0 => Some(sum / count as f64),
            _ => None,
        }
    }

    fn std_deviation(data: &[f64]) -> Option<f64> {
        match (mean(data), data.len()) {
            (Some(data_mean), count) if count > 0 => {
                let variance = data.iter().map(|value| {
                    let diff = data_mean - (*value as f64);

                    diff * diff
                }).sum::<f64>() / count as f64;

                Some(variance.sqrt())
            },
            _ => None
        }
    }

    fn test_tr_class(arrival_type: StreamType, arrival_intensity: f64, arrival_e2d2: f64) {
        let serv_intensity = 1.0;
        let tr_class = Class::new(arrival_type, Poisson, arrival_intensity, arrival_e2d2, serv_intensity, 1.0).unwrap();
        let mut rng: ThreadRng = ThreadRng::default();

        let len = 10_000_000;
        let samples:Vec<f64> = (0..len).enumerate().map(|_| tr_class.get_time_new_call(&mut rng)).collect();

        let exp_mean_time = serv_intensity / tr_class.get_a();
        let exp_std_dev_time = (exp_mean_time * exp_mean_time /tr_class.get_new_e2d2()).sqrt();

        let mean_time = mean(&samples);
        match mean_time {
            Some(cur_mean_time) => assert_relative_eq!(cur_mean_time, exp_mean_time, max_relative=0.01),
            None => assert!(false)
        }

        let std_dev_time = std_deviation(&samples);
        match std_dev_time {
            Some(cur_std_dev_time) => assert_relative_eq!(cur_std_dev_time, exp_std_dev_time, max_relative=0.1),
            None => assert!(false, "variance fail")
        }
    }

    #[test]
    fn test_poison_intensity1() {
        test_tr_class(Poisson, 1.0, 1.0);
    }

    #[test]
    #[should_panic]
    fn test_poison_intensity1_wronge_2d2() {
        test_tr_class(Poisson, 1.0, 3.0);
    }

    #[test]
    fn test_poison_intensity10() {
        test_tr_class(Poisson, 10.0, 1.0);
    }

    #[test]
    fn test_gamma_intensity1() {
        test_tr_class(Gamma, 1.0, 1.0);
    }

    #[test]
    fn test_gamma_intensity1_e2d2_3() {
        test_tr_class(Gamma, 1.0, 3.0);
    }

    #[test]
    #[should_panic]
    fn test_uniform_intensity1() {
        test_tr_class(Uniform, 1.0, 1.0);
    }

    #[test]
    fn test_uniform_intensity1_e2d2_3() {
        test_tr_class(Uniform, 1.0, 3.0);
    }

    #[test]
    fn test_uniform_intensity1_e2d2_5() {
        test_tr_class(Uniform, 1.0, 5.0);
    }

    #[test]
    fn test_pareto_intensity1_e2d2_10() {
        test_tr_class(Pareto, 1.0, 10.0);
    }

    #[test]
    fn test_pareto_intensity1_e2d2_3() {
        test_tr_class(Pareto, 1.0, 3.0);
    }

}
use assert_approx_eq::assert_approx_eq;

pub fn uniform_gen_min_max(intensity: f64, e2_d2: f64) -> (f64, f64) {
    let e_val = 1f64/intensity;
    let d2_val = e_val * e_val / e2_d2;

    let delta_t = f64::sqrt(12f64 * d2_val);
    let t_min = e_val-0.5*delta_t;
    let t_max = e_val+0.5*delta_t;

    (t_min, t_max)
}

/// Returns Mean (Expected value) E and Variance D
///
/// # Arguments
/// * `intensity` - (1/E), where E is expected Value (Mean)
/// * `E²/σ²` - relation between power of meand and variance D=σ², where σ is standard deviation
pub fn get_e_d(intensity: f64, e2_d2: f64) -> (f64, f64) {
    let par_e = 1f64/intensity;
    let par_d = par_e * par_e / e2_d2;

    (par_e, par_d)
}

pub fn gamma_get_scale_shape(intensity: f64, e2_d2: f64) -> (f64, f64) {
    let (par_e, par_d) = get_e_d(intensity, e2_d2);

    let shape = par_d / par_e;
    let scale = par_e / shape;

    (scale, shape)
}

pub fn pareto_get_scale_shape(intensity: f64, e2_d2: f64) -> (f64, f64) {
    let (par_e, par_d) = get_e_d(intensity, e2_d2);

    //https://en.wikipedia.org/wiki/Pareto_distribution
    //shape = \alpha
    //scale = x_m
    //pdf(x) = $\frac{\alpha \cdot x \x_m^{\aplha}}{x^{\alpha+1}}$

    let tmp = f64::sqrt(1f64 + par_e * par_e / par_d);
    let alpha = 1f64 + tmp;
    let x_m = par_e * (alpha - 1f64)/alpha;

    let chck_e = alpha * x_m  / (alpha - 1f64);
    let chck_d = x_m*x_m*alpha/((alpha-1f64)*(alpha-1f64)*(alpha-2f64));

    assert_approx_eq!(chck_e, par_e, 0.001f64);
    assert_approx_eq!(chck_d, par_d, 0.001f64);

    assert!(alpha > 2f64);
    assert!(x_m > 0f64);

    (x_m, alpha)
}
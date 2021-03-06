pub fn uniform_gen_min_max(intensity: f64, e2_d2: f64) -> (f64, f64) {
    let e_val = 1f64/intensity;
    let d2_val = e_val * e_val / e2_d2;

    let delta_t = f64::sqrt(12f64 * d2_val);
    let t_min = e_val-0.5*delta_t;
    let t_max = e_val+0.5*delta_t;

    (t_min, t_max)
}

pub fn gamma_get_scale_shape(intensity: f64, e2_d2: f64) -> (f64, f64) {
    let par_e = 1f64/intensity;
    let par_d = par_e * par_e / e2_d2;

    let shape = par_d / par_e;
    let scale = par_e / shape;

    (scale, shape)
}

pub fn pareto_get_scale_shape(intensity: f64, e2_d2: f64) -> (f64, f64) {
    let par_e = 1f64/intensity;
    let par_d = par_e * par_e / e2_d2;

    let tmp = f64::sqrt(1f64 + par_e * par_e / par_d);
    let shape = 1f64 + tmp;
    let scale = par_e * (shape - 1f64) * shape;

    (scale, shape)
}
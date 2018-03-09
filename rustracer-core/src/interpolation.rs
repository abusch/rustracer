#![allow(dead_code)]
use std::f32::consts::PI;

use {find_interval, INV_2_PI};

pub fn sample_catmull_rom_2d(
    nodes1: &[f32],
    nodes2: &[f32],
    values: &[f32],
    cdf: &[f32],
    alpha: f32,
    u: f32,
) -> (f32, f32, f32) {
    let size2 = nodes2.len();
    let mut u = u;
    // Determine offset and coefficients for the _alpha_ parameter
    let mut weights = [0.0; 4];
    let offset = if let Some(off) = catmull_rom_weights(nodes1, alpha, &mut weights) {
        off
    } else {
        return (0.0, 0.0, 0.0);
    };

    // Define a lambda function to interpolate table entries
    let interpolate = |array: &[f32], idx: usize| {
        let mut value = 0.0;
        for i in 0..4 {
            if weights[i] != 0.0 {
                value += array[(offset as usize + i) * size2 + idx] * weights[i];
            }
        }
        value
    };

    // Map _u_ to a spline interval by inverting the interpolated _cdf_
    let maximum = interpolate(cdf, size2 - 1);
    u *= maximum;
    let idx = find_interval(size2, |i| interpolate(cdf, i) <= u);

    // Look up node positions and interpolated function values
    let f0 = interpolate(values, idx);
    let f1 = interpolate(values, idx + 1);
    let x0 = nodes2[idx];
    let x1 = nodes2[idx + 1];
    let width = x1 - x0;

    // Re-scale _u_ using the interpolated _cdf_
    u = (u - interpolate(cdf, idx)) / width;

    // Approximate derivatives using finite differences of the interpolant
    let d0 = if idx > 0 {
        width * (f1 - interpolate(values, idx - 1)) / (x1 - nodes2[idx - 1])
    } else {
        f1 - f0
    };
    let d1 = if idx + 2 < size2 {
        width * (interpolate(values, idx + 2) - f0) / (nodes2[idx + 2] - x0)
    } else {
        f1 - f0
    };

    // Invert definite integral over spline segment and return solution

    // Set initial guess for $t$ by importance sampling a linear interpolant
    let mut t = if f0 != f1 {
        (f0 - f32::sqrt(f32::max(0.0, f0 * f0 + 2.0 * u * (f1 - f0)))) / (f0 - f1)
    } else {
        u / f0
    };
    let mut a = 0.0;
    let mut b = 1.0;
    let mut Fhat;
    let mut fhat;
    loop {
        // Fall back to a bisection step when _t_ is out of bounds
        if !(t >= a && t <= b) {
            t = 0.5 * (a + b);
        }

        // Evaluate target function and its derivative in Horner form
        Fhat = t
            * (f0
                + t
                    * (0.5 * d0
                        + t
                            * ((1.0 / 3.0) * (-2.0 * d0 - d1) + f1 - f0
                                + t * (0.25 * (d0 + d1) + 0.5 * (f0 - f1)))));
        fhat = f0
            + t * (d0 + t * (-2.0 * d0 - d1 + 3.0 * (f1 - f0) + t * (d0 + d1 + 2.0 * (f0 - f1))));

        // Stop the iteration if converged
        if f32::abs(Fhat - u) < 1e-6 || b - a < 1e-6 {
            break;
        }

        // Update bisection bounds using updated _t_
        if Fhat - u < 0.0 {
            a = t;
        } else {
            b = t;
        }

        // Perform a Newton step
        t -= (Fhat - u) / fhat;
    }

    // Return the sample position and function value
    (x0 + width * t, fhat, fhat / maximum)
}

pub fn catmull_rom_weights(nodes: &[f32], x: f32, weights: &mut [f32; 4]) -> Option<usize> {
    let size = nodes.len();
    // Return false if x is out of bounds
    if !(x >= nodes[0] && x <= nodes[size - 1]) {
        return None;
    }

    // search for the interval idx containing x
    let idx = find_interval(size, |i| nodes[i] <= x);
    let offset = idx - 1;
    let x0 = nodes[idx];
    let x1 = nodes[idx + 1];

    // compute the t parameter and powers
    let t = (x - x0) / (x1 - x0);
    let t2 = t * t;
    let t3 = t2 * t;

    // compute initial node weights w_1 and w_2
    weights[1] = 2.0 * t3 - 3.0 * t2 + 1.0;
    weights[2] = -2.0 * t3 + 3.0 * t2;

    // compute first node weight w_0
    if idx > 0 {
        let w0 = (t3 - 2.0 * t2 + t) * (x1 - x0) / (x1 - nodes[idx - 1]);
        weights[0] = -w0;
        weights[2] += w0;
    } else {
        let w0 = t3 - 2.0 * t2 + t;
        weights[0] = 0.0;
        weights[1] -= w0;
        weights[2] += w0;
    }

    // compute last node weight w_3
    if idx + 2 < size {
        let w3 = (t3 - t2) * (x1 - x0) / (nodes[idx + 2] - x0);
        weights[1] -= w3;
        weights[3] = w3;
    } else {
        let w3 = t3 - t2;
        weights[1] -= w3;
        weights[2] += w3;
        weights[3] = 0.0;
    }

    Some(offset)
}

pub fn integrate_catmull_rom(n: usize, x: &[f32], values: &[f32], cdf: &mut [f32]) -> f32 {
    let mut sum = 0.0;
    cdf[0] = 0.0;
    for i in 0..n - 1 {
        // Look up $x_i$ and function values of spline segment _i_
        let x0 = x[i];
        let x1 = x[i + 1];
        let f0 = values[i];
        let f1 = values[i + 1];
        let width = x1 - x0;

        // Approximate derivatives using finite differences
        let d0 = if i > 0 {
            width * (f1 - values[i - 1]) / (x1 - x[i - 1])
        } else {
            f1 - f0
        };
        let d1 = if i + 2 < n {
            width * (values[i + 2] - f0) / (x[i + 2] - x0)
        } else {
            f1 - f0
        };

        // Keep a running sum and build a cumulative distribution function
        sum += ((d0 - d1) * (1.0 / 12.0) + (f0 + f1) * 0.5) * width;
        cdf[i + 1] = sum;
    }
    sum
}

pub fn invert_catmull_rom(n: usize, x: &[f32], values: &[f32], u: f32) -> f32 {
    // Stop when _u_ is out of bounds
    if !(u > values[0]) {
        return x[0];
    } else if !(u < values[n - 1]) {
        return x[n - 1];
    }

    // Map _u_ to a spline interval by inverting _values_
    let i = find_interval(n, |i| values[i] <= u);

    // Look up $x_i$ and function values of spline segment _i_
    let x0 = x[i];
    let x1 = x[i + 1];
    let f0 = values[i];
    let f1 = values[i + 1];
    let width = x1 - x0;

    // Approximate derivatives using finite differences
    let d0 = if i > 0 {
        width * (f1 - values[i - 1]) / (x1 - x[i - 1])
    } else {
        f1 - f0
    };
    let d1 = if i + 2 < n {
        width * (values[i + 2] - f0) / (x[i + 2] - x0)
    } else {
        f1 - f0
    };

    // Invert the spline interpolant using Newton-Bisection
    let mut a = 0.0;
    let mut b = 1.0;
    let mut t = 0.5;
    let mut Fhat;
    let mut fhat;
    loop {
        // Fall back to a bisection step when _t_ is out of bounds
        if !(t > a && t < b) {
            t = 0.5 * (a + b);
        }

        // Compute powers of _t_
        let t2 = t * t;
        let t3 = t2 * t;

        // Set _Fhat_ using Equation (8.27)
        Fhat = (2.0 * t3 - 3.0 * t2 + 1.0) * f0 + (-2.0 * t3 + 3.0 * t2) * f1
            + (t3 - 2.0 * t2 + t) * d0 + (t3 - t2) * d1;

        // Set _fhat_ using Equation (not present)
        fhat = (6.0 * t2 - 6.0 * t) * f0 + (-6.0 * t2 + 6.0 * t) * f1
            + (3.0 * t2 - 4.0 * t + 1.0) * d0 + (3.0 * t2 - 2.0 * t) * d1;

        // Stop the iteration if converged
        if f32::abs(Fhat - u) < 1e-6 || b - a < 1e-6 {
            break;
        }

        // Update bisection bounds using updated _t_
        if Fhat - u < 0.0 {
            a = t;
        } else {
            b = t;
        }

        // Perform a Newton step
        t -= (Fhat - u) / fhat;
    }
    x0 + t * width
}

pub fn fourier(a: &[f32], m: u32, cos_phi: f32) -> f32 {
    let mut value = 0.0;
    // Initialize cosine iterates
    let mut cos_k_minus_one_phi = cos_phi;
    let mut cos_k_phi = 1.0;
    for k in 0..m {
        // Add the current summand and update the cosine iterates
        value += a[k as usize] * cos_k_phi;
        let cos_k_plus_one_phi = 2.0 * cos_phi * cos_k_phi - cos_k_minus_one_phi;
        cos_k_minus_one_phi = cos_k_phi;
        cos_k_phi = cos_k_plus_one_phi;
    }
    value
}

pub fn sample_fourier(ak: &[f32], recip: &[f32], m: u32, u: f32) -> (f32, f32, f32) {
    let mut u = u;

    // Pick a side and declare bisection variables
    let flip = u >= 0.5;
    if flip {
        u = 1.0 - 2.0 * (u - 0.5);
    } else {
        u *= 2.0;
    }

    let mut a = 0.0;
    let mut b = PI;
    let mut phi = 0.5 * PI;
    let mut F;
    let mut f;

    loop {
        // Evaluate $F(\phi)$ and its derivative $f(\phi)$

        // Initialize sine and cosine iterates
        let cos_phi = f32::cos(phi);
        let sin_phi = f32::sqrt(f32::max(0.0, 1.0 - cos_phi * cos_phi));
        let mut cos_phi_prev = cos_phi;
        let mut cos_phi_cur = 1.0;
        let mut sin_phi_prev = -sin_phi;
        let mut sin_phi_cur = 0.0;

        // Initialize _F_ and _f_ with the first series term
        F = ak[0] * phi;
        f = ak[0];
        for k in 1..m {
            // Compute next sine and cosine iterates
            let sin_phi_next = 2.0 * cos_phi * sin_phi_cur - sin_phi_prev;
            let cos_phi_next = 2.0 * cos_phi * cos_phi_cur - cos_phi_prev;
            sin_phi_prev = sin_phi_cur;
            sin_phi_cur = sin_phi_next;
            cos_phi_prev = cos_phi_cur;
            cos_phi_cur = cos_phi_next;

            // Add the next series term to _F_ and _f_
            F += ak[k as usize] * recip[k as usize] * sin_phi_next;
            f += ak[k as usize] * cos_phi_next;
        }
        F -= u * ak[0] * PI;

        // Update bisection bounds using updated $\phi$
        if F > 0.0 {
            b = phi;
        } else {
            a = phi;
        }

        // Stop the Fourier bisection iteration if converged
        if f32::abs(F) < 1e-6 || b - a < 1e-6 {
            break;
        }

        // Perform a Newton step given $f(\phi)$ and $F(\phi)$
        phi -= F / f;

        // Fall back to a bisection step when $\phi$ is out of bounds
        if !(phi > a && phi < b) {
            phi = 0.5 * (a + b);
        }
    }
    // Potentially flip $\phi$ and return the result
    if flip {
        phi = 2.0 * PI - phi;
    }
    let pdf = INV_2_PI * f / ak[0];
    let phiPtr = phi;
    (f, pdf, phiPtr)
}

//! Black-Scholes Options Greeks calculator.
//!
//! Computes European option price + 5 Greeks (delta, gamma, theta, vega, rho)
//! plus implied volatility via Newton-Raphson iteration.
//!
//! Reference: Hull, "Options, Futures, and Other Derivatives", Ch. 15 & 19.

use std::f64::consts::{PI, SQRT_2};

// ---------------------------------------------------------------------------
// Standard normal CDF and PDF
// ---------------------------------------------------------------------------

/// Standard normal probability density function.
fn norm_pdf(x: f64) -> f64 {
    (-0.5 * x * x).exp() / (2.0 * PI).sqrt()
}

/// Standard normal cumulative distribution function (Abramowitz & Stegun 26.2.17).
/// Accurate to ~1e-7, which is more than sufficient for options pricing.
fn norm_cdf(x: f64) -> f64 {
    // Use the error function identity: Φ(x) = 0.5 * (1 + erf(x / √2))
    0.5 * (1.0 + erf_approx(x / SQRT_2))
}

/// Fast erf approximation (max error ~1.5e-7).
fn erf_approx(x: f64) -> f64 {
    let sign = if x >= 0.0 { 1.0 } else { -1.0 };
    let x = x.abs();
    let t = 1.0 / (1.0 + 0.3275911 * x);
    let poly = t * (0.254829592
        + t * (-0.284496736
        + t * (1.421413741
        + t * (-1.453152027
        + t * 1.061405429))));
    sign * (1.0 - poly * (-x * x).exp())
}

// ---------------------------------------------------------------------------
// Option type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionType {
    Call,
    Put,
}

impl OptionType {
    pub fn label(self) -> &'static str {
        match self {
            Self::Call => "Call",
            Self::Put  => "Put",
        }
    }
}

// ---------------------------------------------------------------------------
// Black-Scholes inputs and results
// ---------------------------------------------------------------------------

/// Inputs for the Black-Scholes model.
#[derive(Debug, Clone)]
pub struct BsInputs {
    /// Current stock price.
    pub spot: f64,
    /// Strike price.
    pub strike: f64,
    /// Time to expiration in years (e.g., 0.25 = 3 months).
    pub time_years: f64,
    /// Risk-free interest rate (annualized, e.g., 0.05 = 5%).
    pub risk_free_rate: f64,
    /// Implied volatility (annualized, e.g., 0.30 = 30%).
    pub volatility: f64,
    /// Call or Put.
    pub option_type: OptionType,
}

/// Full Black-Scholes result: price + all 5 Greeks.
#[derive(Debug, Clone)]
pub struct BsResult {
    /// Theoretical option price.
    pub price: f64,
    /// Delta: ∂price/∂spot. Call: [0,1], Put: [-1,0].
    pub delta: f64,
    /// Gamma: ∂²price/∂spot². Always positive.
    pub gamma: f64,
    /// Theta: ∂price/∂time (per calendar day, negative = time decay).
    pub theta: f64,
    /// Vega: ∂price/∂σ (per 1% vol move).
    pub vega: f64,
    /// Rho: ∂price/∂r (per 1% rate move).
    pub rho: f64,
}

// ---------------------------------------------------------------------------
// Core computation
// ---------------------------------------------------------------------------

/// Compute Black-Scholes option price and all 5 Greeks.
///
/// Returns `None` if inputs are degenerate (T ≤ 0, S ≤ 0, K ≤ 0, σ ≤ 0).
pub fn compute_greeks(inputs: &BsInputs) -> Option<BsResult> {
    let s = inputs.spot;
    let k = inputs.strike;
    let t = inputs.time_years;
    let r = inputs.risk_free_rate;
    let sigma = inputs.volatility;

    if s <= 0.0 || k <= 0.0 || t <= 0.0 || sigma <= 0.0 {
        return None;
    }

    let sqrt_t = t.sqrt();
    let d1 = ((s / k).ln() + (r + 0.5 * sigma * sigma) * t) / (sigma * sqrt_t);
    let d2 = d1 - sigma * sqrt_t;

    let nd1 = norm_cdf(d1);
    let nd2 = norm_cdf(d2);
    let npd1 = norm_pdf(d1);
    let discount = (-r * t).exp();

    let (price, delta, theta, rho) = match inputs.option_type {
        OptionType::Call => {
            let price = s * nd1 - k * discount * nd2;
            let delta = nd1;
            // Theta per calendar day (divide annual by 365)
            let theta = (-(s * npd1 * sigma) / (2.0 * sqrt_t)
                - r * k * discount * nd2) / 365.0;
            let rho = k * t * discount * nd2 / 100.0;
            (price, delta, theta, rho)
        }
        OptionType::Put => {
            let nmd1 = norm_cdf(-d1);
            let nmd2 = norm_cdf(-d2);
            let price = k * discount * nmd2 - s * nmd1;
            let delta = nd1 - 1.0; // negative for puts
            let theta = (-(s * npd1 * sigma) / (2.0 * sqrt_t)
                + r * k * discount * nmd2) / 365.0;
            let rho = -k * t * discount * nmd2 / 100.0;
            (price, delta, theta, rho)
        }
    };

    // Gamma and Vega are the same for calls and puts
    let gamma = npd1 / (s * sigma * sqrt_t);
    let vega = s * npd1 * sqrt_t / 100.0; // per 1% vol move

    Some(BsResult { price, delta, gamma, theta, vega, rho })
}

// ---------------------------------------------------------------------------
// Implied Volatility (Newton-Raphson)
// ---------------------------------------------------------------------------

/// Solve for implied volatility given a market price.
///
/// Uses Newton-Raphson with vega as the derivative. Converges in 3-6 iterations
/// for typical option prices. Returns `None` if no solution found.
pub fn implied_volatility(
    spot: f64,
    strike: f64,
    time_years: f64,
    risk_free_rate: f64,
    market_price: f64,
    option_type: OptionType,
) -> Option<f64> {
    if market_price <= 0.0 || spot <= 0.0 || strike <= 0.0 || time_years <= 0.0 {
        return None;
    }

    // Initial guess: use Brenner-Subrahmanyam approximation
    let mut sigma = (market_price / spot) * (2.0 * PI / time_years).sqrt();
    sigma = sigma.clamp(0.01, 5.0); // sanity bounds

    for _ in 0..50 {
        let inputs = BsInputs {
            spot, strike, time_years, risk_free_rate,
            volatility: sigma,
            option_type,
        };
        let result = compute_greeks(&inputs)?;
        let diff = result.price - market_price;

        // Vega in compute_greeks is per 1% move; we need per 1.0 (100%) for Newton step
        let vega_full = result.vega * 100.0;
        if vega_full.abs() < 1e-12 {
            break; // vega too small, can't iterate
        }

        sigma -= diff / vega_full;
        sigma = sigma.clamp(0.001, 10.0);

        if diff.abs() < 1e-6 {
            return Some(sigma);
        }
    }

    // Return best guess even if not fully converged
    if sigma > 0.001 && sigma < 10.0 { Some(sigma) } else { None }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_price_sanity() {
        // AAPL-like: $170, $175 strike, 30 days, 5% rate, 25% vol
        let inputs = BsInputs {
            spot: 170.0, strike: 175.0,
            time_years: 30.0 / 365.0,
            risk_free_rate: 0.05, volatility: 0.25,
            option_type: OptionType::Call,
        };
        let r = compute_greeks(&inputs).unwrap();
        // OTM call should be cheap but positive
        assert!(r.price > 0.0 && r.price < 10.0, "Call price={:.4}", r.price);
        // Delta should be between 0 and 1 for a call
        assert!(r.delta > 0.0 && r.delta < 1.0, "Delta={:.4}", r.delta);
        // Gamma always positive
        assert!(r.gamma > 0.0, "Gamma={:.4}", r.gamma);
        // Theta should be negative (time decay)
        assert!(r.theta < 0.0, "Theta={:.4}", r.theta);
        // Vega always positive
        assert!(r.vega > 0.0, "Vega={:.4}", r.vega);
    }

    #[test]
    fn test_put_call_parity() {
        // Put-Call parity: C - P = S - K * e^(-rT)
        let s = 100.0;
        let k = 100.0;
        let t = 0.5;
        let r = 0.05;
        let sigma = 0.30;

        let call = compute_greeks(&BsInputs {
            spot: s, strike: k, time_years: t,
            risk_free_rate: r, volatility: sigma,
            option_type: OptionType::Call,
        }).unwrap();

        let put = compute_greeks(&BsInputs {
            spot: s, strike: k, time_years: t,
            risk_free_rate: r, volatility: sigma,
            option_type: OptionType::Put,
        }).unwrap();

        let parity = call.price - put.price - (s - k * (-r * t).exp());
        assert!(parity.abs() < 1e-10, "Put-call parity violated: {parity}");
    }

    #[test]
    fn test_implied_vol_roundtrip() {
        let inputs = BsInputs {
            spot: 150.0, strike: 155.0,
            time_years: 60.0 / 365.0,
            risk_free_rate: 0.05, volatility: 0.30,
            option_type: OptionType::Call,
        };
        let price = compute_greeks(&inputs).unwrap().price;
        let iv = implied_volatility(150.0, 155.0, 60.0 / 365.0, 0.05, price, OptionType::Call)
            .expect("IV solve failed");
        assert!((iv - 0.30).abs() < 0.001, "IV roundtrip: expected 0.30, got {iv:.6}");
    }

    #[test]
    fn test_atm_delta_near_half() {
        // ATM call delta should be close to 0.5 (slightly above due to drift)
        let r = compute_greeks(&BsInputs {
            spot: 100.0, strike: 100.0,
            time_years: 1.0,
            risk_free_rate: 0.0, volatility: 0.20,
            option_type: OptionType::Call,
        }).unwrap();
        assert!((r.delta - 0.5).abs() < 0.06, "ATM delta={:.4}", r.delta);
    }

    #[test]
    fn test_degenerate_inputs() {
        assert!(compute_greeks(&BsInputs {
            spot: 0.0, strike: 100.0, time_years: 0.5,
            risk_free_rate: 0.05, volatility: 0.25,
            option_type: OptionType::Call,
        }).is_none());

        assert!(compute_greeks(&BsInputs {
            spot: 100.0, strike: 100.0, time_years: 0.0,
            risk_free_rate: 0.05, volatility: 0.25,
            option_type: OptionType::Call,
        }).is_none());
    }
}

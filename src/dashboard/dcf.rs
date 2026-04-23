//! Discounted Cash Flow (DCF) intrinsic value calculator.
//!
//! Computes a fair value per share using a two-stage DCF model:
//!   Stage 1: FCF grows at `growth_rate` for `growth_years` (typically 5-10 years)
//!   Stage 2: Terminal value at perpetuity growth rate (typically 2-3%)
//!
//! Discount rate = WACC (user-supplied, default 10%).
//!
//! This is Principle #7 (Margin of Safety): the headline is the margin of
//! safety percentage, not the raw intrinsic value.

/// User-configurable DCF inputs.
#[derive(Debug, Clone)]
pub struct DcfInputs {
    /// Free cash flow (latest TTM), in dollars.
    pub fcf: f64,
    /// Annual FCF growth rate for Stage 1 (e.g., 0.15 = 15%).
    pub growth_rate: f64,
    /// Number of years for Stage 1 growth (typically 5-10).
    pub growth_years: u32,
    /// Perpetuity growth rate for terminal value (e.g., 0.025 = 2.5%).
    pub terminal_growth: f64,
    /// Weighted average cost of capital / discount rate (e.g., 0.10 = 10%).
    pub discount_rate: f64,
    /// Total shares outstanding.
    pub shares_outstanding: f64,
    /// Current market price per share.
    pub current_price: f64,
}

/// DCF computation results.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Detailed breakdown fields used by future year-by-year table
pub struct DcfResult {
    /// Present value of Stage 1 FCFs.
    pub pv_stage1: f64,
    /// Present value of terminal value.
    pub pv_terminal: f64,
    /// Total enterprise value (pv_stage1 + pv_terminal).
    pub enterprise_value: f64,
    /// Intrinsic value per share (enterprise_value / shares).
    pub intrinsic_per_share: f64,
    /// Margin of safety: (intrinsic - price) / intrinsic * 100.
    /// Positive = undervalued, negative = overvalued.
    pub margin_of_safety_pct: f64,
    /// Year-by-year projected FCFs for display.
    pub yearly_fcfs: Vec<(u32, f64, f64)>, // (year, projected_fcf, pv_of_fcf)
}

/// Compute the two-stage DCF model.
pub fn compute_dcf(inputs: &DcfInputs) -> DcfResult {
    let r = inputs.discount_rate;
    let g = inputs.growth_rate;
    let mut yearly_fcfs = Vec::with_capacity(inputs.growth_years as usize);

    // Stage 1: project FCFs and discount them
    let mut pv_stage1 = 0.0;
    let mut fcf = inputs.fcf;

    for year in 1..=inputs.growth_years {
        fcf *= 1.0 + g;
        let discount_factor = (1.0 + r).powi(year as i32);
        let pv = fcf / discount_factor;
        pv_stage1 += pv;
        yearly_fcfs.push((year, fcf, pv));
    }

    // Stage 2: terminal value (Gordon Growth Model)
    // TV = FCF_final * (1 + g_terminal) / (r - g_terminal)
    let terminal_fcf = fcf * (1.0 + inputs.terminal_growth);
    let terminal_value = if r > inputs.terminal_growth {
        terminal_fcf / (r - inputs.terminal_growth)
    } else {
        // Edge case: discount rate <= terminal growth makes Gordon model undefined.
        // Fall back to a simple 20x multiple on final FCF.
        fcf * 20.0
    };
    let pv_terminal = terminal_value / (1.0 + r).powi(inputs.growth_years as i32);

    let enterprise_value = pv_stage1 + pv_terminal;
    let intrinsic_per_share = if inputs.shares_outstanding > 0.0 {
        enterprise_value / inputs.shares_outstanding
    } else {
        0.0
    };

    let margin_of_safety_pct = if intrinsic_per_share > 0.0 {
        (intrinsic_per_share - inputs.current_price) / intrinsic_per_share * 100.0
    } else {
        0.0
    };

    DcfResult {
        pv_stage1,
        pv_terminal,
        enterprise_value,
        intrinsic_per_share,
        margin_of_safety_pct,
        yearly_fcfs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dcf_basic() {
        let inputs = DcfInputs {
            fcf: 100_000_000_000.0, // $100B FCF (like AAPL)
            growth_rate: 0.08,      // 8% growth
            growth_years: 5,
            terminal_growth: 0.025,
            discount_rate: 0.10,
            shares_outstanding: 15_000_000_000.0, // 15B shares
            current_price: 170.0,
        };
        let result = compute_dcf(&inputs);
        assert!(result.intrinsic_per_share > 0.0);
        assert_eq!(result.yearly_fcfs.len(), 5);
        // With these inputs, AAPL-like should be roughly fair valued
        assert!(result.intrinsic_per_share > 50.0 && result.intrinsic_per_share < 500.0);
    }

    #[test]
    fn test_dcf_margin_of_safety() {
        let inputs = DcfInputs {
            fcf: 10_000_000.0,
            growth_rate: 0.10,
            growth_years: 5,
            terminal_growth: 0.02,
            discount_rate: 0.10,
            shares_outstanding: 1_000_000.0,
            current_price: 50.0,
        };
        let result = compute_dcf(&inputs);
        // Positive margin = undervalued, negative = overvalued
        if result.intrinsic_per_share > 50.0 {
            assert!(result.margin_of_safety_pct > 0.0);
        } else {
            assert!(result.margin_of_safety_pct < 0.0);
        }
    }
}

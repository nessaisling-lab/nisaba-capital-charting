-- v2.0.5: Add concordance column to lagrange_history
-- Concordance measures whether astrology and financials agree or diverge.
-- Values: 'Strong Confirm', 'Mild Confirm', 'Divergence', 'Mild Deny', 'Strong Deny'

ALTER TABLE lagrange_history ADD COLUMN IF NOT EXISTS concordance TEXT;

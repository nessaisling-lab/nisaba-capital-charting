-- Migration 002: Seed full watchlist tickers
INSERT INTO tickers (ticker, name, sector) VALUES
    ('MSFT',  'Microsoft Corp.',       'Technology'),
    ('GOOGL', 'Alphabet Inc.',         'Technology'),
    ('AMZN',  'Amazon.com Inc.',       'Consumer Discretionary'),
    ('NVDA',  'Nvidia Corp.',          'Technology'),
    ('META',  'Meta Platforms Inc.',   'Technology'),
    ('TSLA',  'Tesla Inc.',            'Consumer Discretionary'),
    ('JPM',   'JPMorgan Chase & Co.',  'Financials'),
    ('V',     'Visa Inc.',             'Financials'),
    ('UNH',   'UnitedHealth Group',    'Healthcare')
ON CONFLICT (ticker) DO NOTHING;

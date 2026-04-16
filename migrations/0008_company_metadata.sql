-- Migration 0008: Company metadata for astrological birth charts
-- IPO date + time + exchange coordinates serve as the "birth moment" for each company.
-- All times are market open (09:30 EST = 14:30 UTC).
-- NYSE coordinates: 40.7069° N, 74.0089° W (11 Wall Street)
-- NASDAQ coordinates: 40.7589° N, 73.9851° W (Rockefeller Center, NYC)

CREATE TABLE IF NOT EXISTS company_metadata (
    ticker        TEXT PRIMARY KEY,
    company_name  TEXT NOT NULL,
    ipo_date      DATE NOT NULL,
    ipo_time      TIME NOT NULL DEFAULT '09:30:00',
    exchange      TEXT NOT NULL DEFAULT 'NYSE',
    latitude      DOUBLE PRECISION NOT NULL,
    longitude     DOUBLE PRECISION NOT NULL,
    founding_date DATE,
    notes         TEXT
);

INSERT INTO company_metadata
    (ticker, company_name, ipo_date, ipo_time, exchange, latitude, longitude, founding_date, notes)
VALUES
    ('MSFT',  'Microsoft Corporation',      '1986-03-13', '09:30:00', 'NASDAQ', 40.7589, -73.9851,
     '1975-04-04', 'Listed on NASDAQ at market open'),

    ('GOOGL', 'Alphabet Inc.',              '2004-08-19', '09:30:00', 'NASDAQ', 40.7589, -73.9851,
     '1998-09-04', 'Dutch auction IPO, opened at $100.01'),

    ('AMZN',  'Amazon.com Inc.',            '1997-05-15', '09:30:00', 'NASDAQ', 40.7589, -73.9851,
     '1994-07-05', 'IPO price $18, Bezos rang the opening bell from Seattle'),

    ('NVDA',  'NVIDIA Corporation',         '1999-01-22', '09:30:00', 'NASDAQ', 40.7589, -73.9851,
     '1993-04-05', 'IPO price $12, raised $42M'),

    ('META',  'Meta Platforms Inc.',        '2012-05-18', '09:30:00', 'NASDAQ', 40.7589, -73.9851,
     '2004-02-04', 'Largest tech IPO at the time, $38/share open'),

    ('TSLA',  'Tesla Inc.',                 '2010-06-29', '09:30:00', 'NASDAQ', 40.7589, -73.9851,
     '2003-07-01', 'IPO price $17, first US automaker IPO since Ford in 1956'),

    ('JPM',   'JPMorgan Chase & Co.',       '2000-12-31', '09:30:00', 'NYSE',   40.7069, -74.0089,
     '1799-09-01', 'Current entity formed by Chase Manhattan + J.P. Morgan merger Dec 2000'),

    ('V',     'Visa Inc.',                  '2008-03-19', '09:30:00', 'NYSE',   40.7069, -74.0089,
     '1958-09-18', 'Largest US IPO at the time, raised $17.9B'),

    ('UNH',   'UnitedHealth Group Inc.',    '1984-10-17', '09:30:00', 'NYSE',   40.7069, -74.0089,
     '1977-01-01', 'Formerly United HealthCare Corporation')

ON CONFLICT (ticker) DO NOTHING;

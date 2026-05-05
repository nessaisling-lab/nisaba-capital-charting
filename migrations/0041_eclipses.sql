-- Wave 6.B4 — eclipse calendar.
-- Hardcoded entries cover 2025-2028 from NASA Five-Millennium Catalog.
-- Per-ticker activations detected at runtime when a natal planet lies
-- within 6° of an eclipse's ecliptic longitude.

CREATE TABLE IF NOT EXISTS eclipses (
    eclipse_date  DATE NOT NULL PRIMARY KEY,
    eclipse_type  TEXT NOT NULL CHECK (eclipse_type IN (
        'solar_total','solar_partial','solar_annular','solar_hybrid',
        'lunar_total','lunar_partial','lunar_penumbral'
    )),
    longitude     NUMERIC(6, 3) NOT NULL,  -- ecliptic longitude at maximum
    magnitude     NUMERIC(5, 3),            -- 0.0-1.0 for partial, 1.0+ for total
    saros_series  INTEGER,                  -- Saros family number
    notes         TEXT
);

CREATE INDEX IF NOT EXISTS idx_eclipses_upcoming
    ON eclipses(eclipse_date) WHERE eclipse_date >= CURRENT_DATE;

-- Seed data: 2025-2028 eclipses from NASA's Five-Millennium Catalog.
-- Longitudes are approximate ecliptic positions of the Sun (solar) or
-- Moon (lunar) at the moment of greatest eclipse.

INSERT INTO eclipses (eclipse_date, eclipse_type, longitude, magnitude, saros_series, notes) VALUES
    ('2025-03-14', 'lunar_total',   173.95, 1.180, 123, 'Total lunar — Worm/Pi Day blood moon'),
    ('2025-03-29', 'solar_partial',   8.87, 0.938, 149, 'Partial solar over Atlantic + N. Europe'),
    ('2025-09-07', 'lunar_total',  165.04, 1.367, 128, 'Total lunar over Asia + Africa'),
    ('2025-09-21', 'solar_partial', 178.85, 0.855, 154, 'Partial solar over S. Pacific + Antarctica'),
    ('2026-02-17', 'solar_annular', 328.72, 0.963, 121, 'Annular solar — Antarctica'),
    ('2026-03-03', 'lunar_total',  142.65, 1.151, 133, 'Total lunar over Pacific'),
    ('2026-08-12', 'solar_total',  140.05, 1.039, 126, 'Total solar — Greenland/Iceland/Spain'),
    ('2026-08-28', 'lunar_partial', 335.20, 0.930, 138, 'Partial lunar over Americas'),
    ('2027-02-06', 'solar_annular', 317.18, 0.928, 131, 'Annular solar over S. America/Africa'),
    ('2027-02-21', 'lunar_penumbral', 332.48, 0.927, 143, 'Penumbral lunar'),
    ('2027-07-18', 'lunar_penumbral', 295.62, 0.038, 110, 'Faint penumbral'),
    ('2027-08-02', 'solar_total',  129.87, 1.079, 136, 'Great American Eclipse 2 — Spain/Egypt path'),
    ('2027-08-17', 'lunar_penumbral', 324.87, 0.140, 148, 'Penumbral lunar'),
    ('2028-01-12', 'solar_annular', 291.58, 0.920, 141, 'Annular solar over N. America'),
    ('2028-01-26', 'lunar_total',  126.10, 1.082, 153, 'Total lunar'),
    ('2028-07-22', 'solar_total',  119.62, 1.056, 146, 'Total solar — Australia'),
    ('2028-08-06', 'lunar_partial', 314.19, 0.392, 158, 'Partial lunar')
ON CONFLICT (eclipse_date) DO NOTHING;

-- Add hidden column to symbol_conversions table
ALTER TABLE symbol_conversions ADD COLUMN hidden BOOLEAN NOT NULL DEFAULT 0;

-- Create index for filtering
CREATE INDEX idx_symbol_conversions_hidden ON symbol_conversions(hidden);

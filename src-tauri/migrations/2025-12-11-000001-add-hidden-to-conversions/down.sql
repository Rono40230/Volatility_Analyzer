-- Remove hidden column from symbol_conversions table
DROP INDEX IF EXISTS idx_symbol_conversions_hidden;
ALTER TABLE symbol_conversions DROP COLUMN hidden;

#!/bin/bash
# Test manuel d'un INSERT dans pairs.db avec les paramètres identiques au code Rust

DB_PATH="$HOME/.local/share/volatility-analyzer/pairs.db"

echo "=== Test INSERT pairs.db ==="
echo "Database: $DB_PATH"
echo ""

# Clean slate - supprimer les données de test
echo "Nettoyage des données de test précédentes..."
sqlite3 "$DB_PATH" "DELETE FROM candle_data WHERE symbol='TESTPAIR'; DELETE FROM pair_metadata WHERE symbol='TESTPAIR';"

# Insérer 3 bougies de test
echo "Insertion de 3 bougies de test..."
sqlite3 "$DB_PATH" <<EOF
BEGIN TRANSACTION;

INSERT INTO candle_data (symbol, timeframe, time, open, high, low, close, volume, imported_at, source_file)
VALUES ('TESTPAIR', 'M1', '2024-01-01T00:00:00Z', 1.0000, 1.0100, 0.9900, 1.0050, 1000.0, '2025-11-10T14:00:00Z', 'test.csv');

INSERT INTO candle_data (symbol, timeframe, time, open, high, low, close, volume, imported_at, source_file)
VALUES ('TESTPAIR', 'M1', '2024-01-01T00:01:00Z', 1.0050, 1.0150, 0.9950, 1.0100, 1100.0, '2025-11-10T14:00:00Z', 'test.csv');

INSERT INTO candle_data (symbol, timeframe, time, open, high, low, close, volume, imported_at, source_file)
VALUES ('TESTPAIR', 'M1', '2024-01-01T00:02:00Z', 1.0100, 1.0200, 1.0000, 1.0150, 1200.0, '2025-11-10T14:00:00Z', 'test.csv');

INSERT INTO pair_metadata (symbol, timeframe, row_count, last_updated, last_imported_file)
VALUES ('TESTPAIR', 'M1', 3, '2025-11-10T14:00:00Z', 'test.csv')
ON CONFLICT(symbol, timeframe) DO UPDATE SET
    row_count = row_count + excluded.row_count,
    last_updated = excluded.last_updated,
    last_imported_file = excluded.last_imported_file;

INSERT INTO import_log (filename, symbol, timeframe, row_count, expected_row_count, status, imported_at)
VALUES ('test.csv', 'TESTPAIR', 'M1', 3, 3, 'success', '2025-11-10T14:00:00Z');

COMMIT;
EOF

echo ""
echo "=== Résultats ===" 
echo ""
echo "Candles TESTPAIR:"
sqlite3 "$DB_PATH" "SELECT COUNT(*) as count FROM candle_data WHERE symbol='TESTPAIR';"

echo ""
echo "Métadonnées TESTPAIR:"
sqlite3 "$DB_PATH" "SELECT * FROM pair_metadata WHERE symbol='TESTPAIR';"

echo ""
echo "Import log entries for test.csv:"
sqlite3 "$DB_PATH" "SELECT * FROM import_log WHERE filename='test.csv';"

echo ""
echo "✅ Test complété!"

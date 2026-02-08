# Migrations Diesel CLI — Archives

> **Ces migrations NE SONT PAS exécutées au runtime.**

L'application utilise `src/db/migrations.rs` avec des `CREATE TABLE IF NOT EXISTS`
comme unique système de migration. Ce choix est motivé par :

1. L'app utilise **2 bases SQLite** (calendrier + paires) — Diesel CLI gère une seule base.
2. Les migrations `IF NOT EXISTS` sont **idempotentes** : aucun risque de conflit au lancement.
3. Certaines migrations CLI ici sont **historiquement cassées** (double CREATE de `calendar_events`).

## Source de vérité

Le schéma de chaque table est défini dans `src/db/migrations.rs` :
- `ensure_calendar_table()` → `calendar_events`
- `ensure_calendar_imports_table()` → `calendar_imports`
- `ensure_pair_tables()` → `candle_data`, `pair_metadata`, `import_log`
- `ensure_archives_table()` → `archives`

Ces fonctions sont appelées au démarrage dans `lib.rs`.

## Pour ajouter une colonne

1. Ajouter la colonne dans le `CREATE TABLE IF NOT EXISTS` correspondant.
2. Ajouter un `ALTER TABLE ADD COLUMN` (avec `let _ =`) pour les bases existantes.
3. Mettre à jour `schema.rs` si la table y est déclarée.

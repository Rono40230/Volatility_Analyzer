
# Vibe - Checklist d'audit exécutable

Ce fichier liste les commandes et vérifications minimales à exécuter avant toute modification automatique ou mise en production.

1) Isolation & debug

```bash
# Lancer la sentinelle en mode debug (pas d'auto-fix)
./vibe --debug

# Ou créer le lock pour s'assurer qu'auto-fix est désactivé
touch debug.lock
```

2) Tests Rust (unit + clippy)

```bash
cd src-tauri
cargo test --all
cargo clippy --all -- -D warnings
```

3) Audit dépendances

```bash
# Rust
cd src-tauri
cargo install cargo-audit || true
cargo audit

# Node
cd ../.. # racine du projet
npm install --package-lock-only || true
npm audit --audit-level=moderate || true
```

4) Tests frontend

```bash
# Si Vue
npm ci
npm run lint -- --max-warnings=0
npm test
```

5) Tests d'intégration / pipeline complet

```bash
# Exécuter le test d'intégration et le test end-to-end localement
cd src-tauri
cargo test --tests
# Si existant
npm run e2e || true
```

6) Vérification sécurité (SAST simple)

```bash
# Installer semgrep ou cargo-audit selon besoin
semgrep --config=auto src/ || true
```

7) Génération de patchs (workflow Vibe)

```bash
# Lancer la sentinelle (mode normal) et récupérer les patchs dans .vibe/patches/
./vibe
ls .vibe/patches

# Appliquer manuellement un patch et créer une PR plutôt que laisser auto-commit
git apply .vibe/patches/name.patch
git add -A && git commit -m "Apply Vibe patch: ..."
```

8) Validation out-of-sample (recommandé pour modèles)

```bash
# Script custom: exécuter un walk-forward ou split 70/30 et comparer métriques
# Exemple minimal (à adapter selon projet)
python scripts/walk_forward_validate.py --pair EURUSD --years 2018-2023
```

9) Post-audit / monitoring

```bash
# Envoyer metrics ou logs vers endpoint ou sauvegarder
cat .vibe/metrics.json
```

---

Conserver cette checklist à jour et l'exécuter avant d'accepter des patchs automatiques.

# 🎯 SYSTEM PROMPT - À LIRE AU DÉBUT DE CHAQUE SESSION

## RÈGLES OBLIGATOIRES

Tu DOIS respecter :
1. `.clinerules` (racine) - Règles frontend/Vue.js
2. `src-tauri/.clinerules` - Règles Rust/Tauri
3. **RÈGLE 16** : Valider compréhension AVANT de coder
4. **RÈGLE 17** : Phase 1 (accumule) + Phase 2 (valide)

## DOCUMENTS À CONNAÎTRE

Lis en priorité :
- `QUICK_START_5MIN.md` (5 minutes)
- `SYSTEM_COMPLETE_VIBE_PROOF.md` (vue complète)
- `scripts/impact-detection/README.md` (technique)

## WORKFLOW OBLIGATOIRE

### Phase 1 : Accumulation (TU CODES)
1. **Reformuler** la demande de l'utilisateur (RÈGLE 16)
2. Proposer un plan d'action
3. Attendre approbation
4. Coder + tester
5. Accumuler les changements (PAS de commit)

### Phase 2 : Validation (TU VALIDES)
Exécuter AUTOMATIQUEMENT quand l'utilisateur dit "valide tout" :
```bash
./scripts/impact-detection/validate-phase2.sh
```

Attendre le rapport :
- ✅ APPROUVÉ → Commit
- ❌ BLOQUÉ → Signaler le problème

## GARANTIES À MAINTENIR

- ✅ Zéro régression (tests doivent passer)
- ✅ Zéro surprise (rapport complet avant commit)
- ✅ Zéro effort (tu gères les détails tech)
- ✅ Zéro code brisé (impact détecté)

## STRUCTURE DU PROJET

```
Analyses-historiques/        ← Frontend (Vue.js)
├── .clinerules              ← Règles frontend
├── QUICK_START_5MIN.md
├── SYSTEM_COMPLETE_VIBE_PROOF.md
├── INSTALLATION.md
├── scripts/
│   └── impact-detection/    ← Scripts de validation
│       ├── init-impact-system.sh
│       ├── validate-phase2.sh
│       └── README.md
│
└── src-tauri/               ← Backend (Rust)
    ├── .clinerules          ← Règles Rust
    ├── src/
    └── Cargo.toml
```

## RAPPELS IMPORTANTS

🔴 **NE JAMAIS** commit directement (Phase 1 seulement)
🔴 **NE JAMAIS** ignorer les tests
🔴 **NE JAMAIS** modifier .clinerules sans discussion
✅ **TOUJOURS** reformuler avant de coder (RÈGLE 16)
✅ **TOUJOURS** attendre approbation
✅ **TOUJOURS** exécuter Phase 2 à la demande

## COMMANDES CLÉS

```bash
# Initialisation (une seule fois)
./scripts/impact-detection/init-impact-system.sh

# Validation Phase 2 (quand l'utilisateur dit "valide")
./scripts/impact-detection/validate-phase2.sh

# Menu d'aide
./scripts/impact-detection/help.sh
```

---

**Ce fichier doit être lu au début de CHAQUE nouvelle session avec l'IA.**

Si l'IA ne le respecte pas → demande-lui de relire ce fichier.

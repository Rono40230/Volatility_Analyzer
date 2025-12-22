# 📅 Nouvel Onglet : Planning & Feuille de Route (Roadmap)

Cet onglet servira de pont entre l'analyse historique (Passé) et l'exécution du trading (Futur). Il permet de projeter les statistiques historiques sur le calendrier économique à venir.

## 🏗️ Phase 1 : Architecture & Interface (Frontend)

- [x] **Création de la Vue `PlanningView.vue`**
    - Ajouter l'onglet "Planning" dans la barre de navigation principale (`App.vue`), positionné après "Backtest".
    - Structure de base : Sélecteur de semaine (Date Picker) + Zone d'affichage du calendrier.

- [x] **Composant Calendrier Hebdomadaire**
    - Affichage chronologique (Lundi -> Vendredi).
    - Pour chaque jour : Liste des événements économiques prévus.
    - Design "Carte d'Action" pour chaque événement :
        - Heure & Nom de l'événement.
        - Indicateurs visuels (Impact prévu).
        - **Zone Paramètres** (Offset, TP, SL) pré-remplie mais modifiable.

## 🧠 Phase 2 : Moteur de Projection (Backend)

- [x] **Synchronisation Calendrier (Forex Factory)**
    - Import automatique des événements de la semaine (`sync_forex_factory_week`).
    - Gestion des mises à jour et doublons.
    - Parsing robuste (gestion Rate Limit, formats CSV).

- [x] **Service de "Matching" (Projection)**
    - Créer une commande `project_stats_on_calendar(start_date, end_date)`.
    - **Logique de jointure** :
        1. Récupérer les événements du calendrier pour la plage donnée.
        2. Pour chaque événement, chercher dans la base historique (Archives ou Stats Volatilité) les métriques correspondantes (clé : `Nom` + `Devise`).
        3. Calculer les paramètres Straddle suggérés (Offset P95, SL, TP) basés sur cet historique.

- [ ] **Gestion des "Manquants"**
    - Si aucun historique n'est trouvé pour un événement futur, afficher un état "Pas de données" ou permettre une configuration manuelle.

## 📝 Phase 3 : Interactivité & Export

- [ ] **Édition Manuelle**
    - Permettre à l'utilisateur de modifier les paramètres suggérés (ex: ajuster l'Offset manuellement).
    - Sauvegarder ces modifications localement (State/LocalStorage) pour ne pas les perdre en changeant de vue.

- [ ] **Export de la Feuille de Route**
    - Bouton "Exporter le Planning" (PDF).
    - Générer un document propre "Prêt à imprimer" avec la liste chronologique des trades à prendre et leurs paramètres validés.

- [ ] **Exports Fiches Paramètres Bidi (PDF)**
    - [x] **Fiche Paire/Période (Volatilité Brute)** :
        - Source : Archives "Volatilité brute".
        - Contenu : Paramètres pour Trading de Session (Plage horaire fixe).
        - Stratégies : Straddle Directionnel + Straddle Simultané (Données distinctes).
    - [x] **Fiche Paire/Événements (Corrélation)** :
        - Source : Archives "Corrélation de la volatilité".
        - Contenu : Paramètres pour News Trading (Straddle sur événement).
        - Stratégies : Straddle Directionnel + Straddle Simultané.
    - **Contrainte** : Zéro recalcul, utilisation exclusive des données JSON archivées.

## 📊 Phase 4 : Méta-Analyse des Archives (Tableau de Bord)

Création d'un outil d'analyse globale pour identifier les tendances lourdes et les divergences à travers toutes les archives sauvegardées.

- [x] **Bouton & Modale**
    - Ajouter un bouton "📊 Méta-Analyse" dans l'en-tête de la vue `ArchivesView.vue`.
    - Créer le composant `MetaAnalysisModal.vue`.

- [x] **Pilier 1 : Graphique de Divergence (Scatter Plot)**
    - Axe X : Volatilité Moyenne (Puissance).
    - Axe Y : Score de Directionnalité (Propreté).
    - Identification visuelle des zones : Pépites (Haut-Droit), Danger/Whipsaw (Bas-Droit), Bruit (Bas-Gauche).

- [x] **Pilier 2 : Matrice de Rentabilité (Heatmap)**
    - Lignes : Types d'Événements.
    - Colonnes : Paires de devises.
    - Valeur : Score de Confiance Moyen.

- [x] **Pilier 3 : Leaderboard des Événements**
    - Tableau classant les événements par "Straddle-abilité" (Fréquence des recommandations positives).
    - Métriques : Moyenne P95, Ratio de Bruit Moyen.

- [x] **Pilier 4 : Optimiseur de Paramètres**
    - Statistiques agrégées sur les paramètres techniques (Offset moyen, SL moyen) par type d'événement.
    - Aide à la définition de "règles par défaut".

module.exports = {
  root: true,
  env: {
    browser: true,
    es2021: true,
    node: true,
  },
  extends: [
    'eslint:recommended',
    'plugin:@typescript-eslint/recommended',
    'plugin:vue/vue3-recommended',
  ],
  parser: 'vue-eslint-parser',
  parserOptions: {
    ecmaVersion: 'latest',
    parser: '@typescript-eslint/parser',
    sourceType: 'module',
  },
  plugins: [
    '@typescript-eslint',
    'vue',
    'unused-imports',
  ],
  rules: {
    // ═══════════════════════════════════════════════════════════
    // RÈGLES CRITIQUES (ERREURS)
    // ═══════════════════════════════════════════════════════════

    // Imports inutilisés (RÈGLE 13 - Zéro code mort)
    'no-unused-vars': 'off', // Désactivé au profit de @typescript-eslint
    '@typescript-eslint/no-unused-vars': ['error', {
      argsIgnorePattern: '^_',
      varsIgnorePattern: '^_',
      caughtErrorsIgnorePattern: '^_',
    }],
    'unused-imports/no-unused-imports': 'error',
    'unused-imports/no-unused-vars': ['error', {
      vars: 'all',
      varsIgnorePattern: '^_',
      args: 'after-used',
      argsIgnorePattern: '^_',
    }],

    // Console.log en production (RÈGLE 10.5)
    'no-console': 'warn',  // Temporairement 'warn' pour voir toutes les violations

    // Debugger en production (RÈGLE 10.5)
    'no-debugger': 'error',

    // Alert en production (RÈGLE 10.5)
    'no-alert': 'warn',  // Temporairement 'warn' pour voir les violations

    // Types any (RÈGLE 10.5)
    '@typescript-eslint/no-explicit-any': 'warn', // Warning car parfois nécessaire

    // ═══════════════════════════════════════════════════════════
    // RÈGLES DE QUALITÉ (WARNINGS)
    // ═══════════════════════════════════════════════════════════

    // Code commenté
    'no-commented-out-code': 'off', // Pas de règle native, géré manuellement

    // Variables non utilisées dans Vue
    'vue/no-unused-components': 'error',
    'vue/no-unused-vars': 'error',

    // Props non utilisées
    'vue/no-unused-properties': ['warn', {
      groups: ['props', 'data', 'computed', 'methods', 'setup'],
    }],

    // ═══════════════════════════════════════════════════════════
    // RÈGLES DE STYLE (DÉSACTIVÉES - pas prioritaires)
    // ═══════════════════════════════════════════════════════════

    'vue/multi-word-component-names': 'off',
    '@typescript-eslint/no-non-null-assertion': 'off',
    'vue/require-default-prop': 'off',

    // ═══════════════════════════════════════════════════════════
    // RÈGLES SPÉCIFIQUES PROJET
    // ═══════════════════════════════════════════════════════════

    // Pas de fonctions vides (sauf constructeurs)
    'no-empty-function': ['error', {
      allow: ['constructors'],
    }],

    // Pas de variables redéclarées
    'no-redeclare': 'error',

    // Pas de code inaccessible
    'no-unreachable': 'error',
  },
  overrides: [
    {
      // Règles spécifiques pour les fichiers de test
      files: ['**/*.spec.ts', '**/*.test.ts', '**/__tests__/**'],
      rules: {
        'no-console': 'off',
        '@typescript-eslint/no-explicit-any': 'off',
      },
    },
  ],
  ignorePatterns: [
    'node_modules/',
    'dist/',
    'target/',
    'src-tauri/target/',
    '*.config.js',
    '*.config.ts',
  ],
}

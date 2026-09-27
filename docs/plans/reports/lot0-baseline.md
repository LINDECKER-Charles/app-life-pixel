# Lot 0 — baseline (état de départ avant la refonte Rose Atelier)

Date : 2026-09-27. Branche : `feat/rose-atelier-redesign` (créée depuis `4995eaa`, HEAD de
`feat/v1-integration`, arbre propre hors `docs/plans/`).

## Captures de référence

Produites par `LP_ENGINE_PREBUILT=1 npm run e2e:visual --prefix frontend` (5 tests, 5 passés,
17 s) contre le serveur 4260, moteur réel. Dossier ignoré par Git :
`frontend/e2e/visual/output/`.

- `screens/<langue>-<thème>/<écran>-<largeur>.png` : 4 combinaisons (`en-light`, `en-dark`,
  `fr-light`, `fr-dark`) × 5 écrans × 4 largeurs = **80 PNG**.
  - Écrans : `settings`, `not-found` (route inconnue `/visual-review-missing-page`, accessible
    sans pile hébergée), `editor-new-animation` (la modale imposée au premier accès),
    `editor` (animation 32×32, diagonale noire + barre rouge sur l'image 1, point vert sur
    l'image 2, image 1 active), `export` (dialogue d'export, tailles calculées).
  - Largeurs × hauteurs : 1440×900, 1024×768, 768×1024, 390×844.
  - `screens/<combo>/report.json` : liste des captures, langue et thème réellement appliqués
    (`applied.language`, `applied.dark` — vérifiés : `fr`/`en`, `true`/`false`), erreurs.
- `measurements.json` : mesures ci-dessous.
- Pilotage : langue par la `locale` Playwright (le web ne persiste aucune préférence, D37 ;
  `initializeI18n` choisit d'après `navigator.languages`), thème par `colorScheme` émulé (la
  préférence par défaut `system` suit `prefers-color-scheme` dans `_tokens.scss`).
- L'étape de création passe par la modale actuelle et bascule seule sur un bouton
  « Create animation » / « Créer une animation » s'il apparaît (capture `editor-welcome` en plus).
- Erreurs : aucune `pageerror`. Console : `401 Unauthorized` sur `/api/v1/auth/session` à chaque
  chargement (pas de serveur derrière `npm start` ; attendu, non bloquant).

## Mesures du canevas (animation 32×32, zoom 8 = 256 px)

| Fenêtre | Scène (région « Canvas ») | Surface de dessin | Part de la hauteur | Œuvre entière visible |
|---|---|---|---|---|
| 1366×768 | 374 px | 256 × 374 px | **0,487** | oui (tout juste en largeur) |
| 1024×768 | 374 px | 256 × 374 px | **0,487** | oui (tout juste en largeur) |

Critère §5.5 (scène ≥ moitié de la hauteur) : **non atteint** aujourd'hui (48,7 %). La colonne du
canevas ne fait que 256 px de large à 1024 comme à 1440 : les colonnes Outils et Palette prennent
le reste ; la timeline (`auto`) occupe ~225 px sous le canevas.

À 390 px (capture `editor-390`), l'éditeur empilé ne défile pas : la palette et la timeline se
chevauchent et sont coupées par le pied de page (même cause que ci-dessous).

## Défilement des pages sans `ion-content` à 390×640 — hypothèse §2.3 **confirmée**

Méthode (`measureScroll`) : ouverture de la route, molette (12 × 500 px) au milieu de la page,
puis comparaison du bas du contenu routé au bas de l'outlet. Deux passes : contenu tel quel, puis
contenu allongé d'un espaceur de 2000 px (le contenu réel de ces pages tient dans 640 px, donc
seule la passe allongée dit si le conteneur défile).

| Route (finale) | `ion-content` | Tel quel : déborde / atteint le bas | Allongé : atteint le bas / reste caché |
|---|---|---|---|
| `/sign-in` | non | non / oui | **non / 1865 px** |
| `/sign-up` | non | non / oui (tient juste) | **non / 2000 px** |
| `/reset-password` | non | non / oui | **non / 1721 px** |
| `/verify-email` | non | non / oui | **non / 1649 px** |
| `/library` → `/sign-in` (visiteur) | non | non / oui | **non / 1865 px** |
| `/settings` (témoin) | oui | non / oui | oui / 0 |
| `/legal/terms` (témoin) | oui | oui / oui | oui / 0 |
| `/visual-review-missing-page` (404, témoin) | oui | non / oui | oui / 0 |

Verdict : une page sans `ion-content` **ne défile pas du tout** ; tout contenu qui dépasse l'outlet
est inaccessible. Les pages avec `ion-content` défilent. Nuance sur la cause : l'hôte `.ion-page`
n'est pas en `overflow: hidden` (calculé `visible`) ; il est en `position: absolute` avec
`contain: size layout style` et une hauteur fixe, dans `ion-router-outlet` et `ion-app` de même
nature, sous un `body` en `position: fixed; overflow: hidden` — aucun ancêtre ne défile. Aucune
page actuelle n'est assez longue en visiteur pour le montrer sans l'espaceur (`/sign-up` tient à
quelques pixels près) ; une bibliothèque longue (`library-page.ts`, `project-page.ts`) ou les
pages jetons/compte le seront. Le gabarit C6 (`ion-content` partout) est donc nécessaire.

## Environnement

| Élément | État |
|---|---|
| Node | hôte `v22.23.2` alors que `.nvmrc` demande `24.21.0` (écart signalé ; tout a tourné) |
| `npm ci --prefix frontend` | OK (520 paquets, 26 s). npm a bloqué 4 scripts d'installation non couverts par `allowScripts` (`@parcel/watcher`, `esbuild`, `lmdb`, `msgpackr-extract`) ; sans effet observé sur serve, lint, tests ni e2e |
| Moteur | `wasm-bindgen-cli` absent au départ : installé en 0.2.129 (version de `Cargo.lock`, prérequis documenté de `docs/v1/README.md`) par `cargo binstall`. Puis `node frontend/tools/prebuild.mjs` → `cargo xtask build-editor` OK (16 s). Les trois sorties existent : `generated/editor_engine.js`, `generated/editor_engine.d.ts`, `public/engine/editor_engine_bg.wasm` (1 072 295 o). rustup a installé la chaîne 1.98 manquante à la volée |
| Chromium Playwright | déjà présent (`chromium-1243`, `chromium_headless_shell-1243` pour Playwright 1.63.0) ; `playwright install chromium` n'a rien téléchargé |
| Docker | 29.8.0 ; pile `life-pixel` déjà en marche (postgres, objectstore, mail, server, admin : healthy) — non utilisée ni modifiée par le lot 0 |
| Serveur 4260 | `LP_ENGINE_PREBUILT=1 npm start --prefix frontend`, lancé détaché (nohup) depuis la racine, laissé en marche ; journal : `docs/plans/reports/dev-server-4260.log` |

## Vérifications

| Commande | Résultat |
|---|---|
| `npm run lint --prefix frontend` | OK (app, shared, admin ; Prettier) |
| `npm run test:tools --prefix frontend` | OK, 28 tests (dont 7 pour `merge-i18n-fragments`) |
| `npx tsc -p tsconfig.e2e.json` (frontend) | OK |
| `npm run i18n:check --prefix frontend` | OK (code 0 ; liste informative de clés non référencées, préexistante) |
| `node design-system/scripts/check.mjs` | PASS |
| `cmp CLAUDE.md AGENTS.md` | identiques |
| `npm run e2e:visual --prefix frontend` | 5/5 passés, 80 captures + `measurements.json` |
| `npm run e2e --prefix frontend` (état de départ) | **6/6 passés** (journey, keyboard, 4 × accessibility) : rien n'échouait avant la refonte |
| `node frontend/tools/merge-i18n-fragments.mjs` sur le dépôt | « no fragment », rien écrit ; fragment témoin sans effet → catalogues identiques octet pour octet |

Non lancées (hors mission du lot 0) : `test:ci`, `build`, `test:engine`, `i18n:check-bundle`,
`e2e:hosted`.

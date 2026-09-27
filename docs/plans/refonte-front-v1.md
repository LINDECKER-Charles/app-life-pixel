# Refonte du front V1 — adoption du design system « Rose Atelier »

Plan de réflexion préalable à une tâche d'agents. Aucun code n'est écrit ici. Les chemins sont
relatifs à la racine du dépôt `F:\Git\app-life-pixel`. Branche de départ : `dev` (la branche de
travail actuelle est `feat/v1-integration`, propre ; commit de tête
`4995eaa feat(design-system): add the Rose Atelier design system`).

## 1. Contexte et objectif

Pendant la V1, l'orchestrateur a découpé le front en tâches parallèles (U1 à U6, H7 à H17, A3, T2,
T3 : voir `docs/v1/editor.md` et `docs/implementation-plan.md`) sans référence visuelle, sans
composants communs et sans moyen de voir le rendu. Résultat : chaque composant a son propre style,
les régions de l'éditeur sont empilées sans hiérarchie, et plusieurs fonctions existent dans le
store mais sont invisibles dans l'interface. Le mainteneur juge l'outil « impossible à utiliser ».

Un design system minimaliste a été ajouté à la racine : `design-system/` (Rose Atelier 0.1.0,
statut « proposition »). Il fixe les tokens (`design-system/tokens/tokens.css`), les assets
originaux (`design-system/assets/` : `mark.svg`, `pip.svg`, `sprout.svg`, police Nunito + OFL),
une référence interactive (`design-system/preview/`, servie par
`node design-system/scripts/serve.mjs` sur `http://127.0.0.1:4265/design-system/preview/`) et neuf
documents de spécification (`design-system/docs/` : vision, foundations, components, journeys,
patterns, content, accessibility, adoption, verification).

**Objectif.** Repenser toute l'expérience utilisateur de l'application (`frontend/projects/app`)
en appliquant ce design system, pour qu'une personne puisse, sans compte : comprendre où elle est,
créer une animation, dessiner, animer, prévisualiser, enregistrer ou exporter, et se remettre
d'une erreur — en français comme en anglais, au clavier, en thème clair et sombre, sur grand
écran comme sur un écran étroit. Secondairement : donner aux agents les consignes et les outils
(captures d'écran de la vraie application) qui leur manquaient.

**Hors périmètre.** Aucun changement du moteur Rust, de `editor-wasm`, du format, de l'API ni des
règles de domaine. Pas de nouvelle fonctionnalité produit inventée (pas de galerie, vignettes de
bibliothèque, autosave, tri, abonnement, application Android). Les propositions marquées comme
telles dans `design-system/docs/journeys.md` restent hors plan sauf mention explicite ci-dessous.

### Ce que dit le design system (à retenir, résumé du README de `design-system/`)

- Shell chaleureux (crème, rose pâle, action framboise `#a93663`), **zone de dessin neutre**
  (fond `--lp-canvas-bg`, damier `#ffffff`/`#ececee` dans les deux thèmes) : le thème ne recolore
  jamais l'œuvre.
- Utiliser des tokens sémantiques, jamais de couleurs copiées en dur.
- Pip (le lapin chibi) uniquement dans l'accueil, les états vides et le succès d'export ; jamais
  dans les erreurs, suppressions, quotas, authentification ou admin.
- Aucune nouvelle dépendance npm n'est requise ; police hébergée localement.
- La référence interactive est un spécimen de mise en page : elle ne dessine, ne compile, ne
  persiste et n'exporte rien. Ces opérations restent celles de l'application.
- Vérification : `node design-system/scripts/check.mjs` (contrastes, parité des catalogues,
  assets) et `node design-system/scripts/browser-check.mjs` (Playwright du workspace frontend,
  serveur lancé à part ; captures dans `design-system/review/`, ignoré par Git).
- Version 0.1.0 = proposition ; le mainteneur valide. Toute modification de token ou
  d'interaction met à jour ensemble tokens, contrats, référence, les deux catalogues et la preuve
  de vérification (`design-system/docs/verification.md`).

## 2. État des lieux

### 2.1 Pile et contraintes du workspace

- Angular 22 standalone + Ionic 9 (`frontend/package.json`), Transloco (catalogues à plat dans
  `i18n/en.json` et `i18n/fr.json`, copiés vers `frontend/projects/app/public/i18n/` par
  `frontend/tools/copy-i18n.mjs`, car Angular refuse les assets hors workspace).
- Budgets de production (`frontend/angular.json`) : style par composant 4 kB en avertissement,
  **8 kB en erreur** ; bundle initial app 750 kB / 1 MB. Conséquence : les styles communs doivent
  être globaux, pas recopiés dans chaque composant.
- Règles du dépôt (`AGENTS.md` = `CLAUDE.md`, identiques octet pour octet, vérifié par
  `cmp CLAUDE.md AGENTS.md`) : fichiers ≤ 300/400 lignes, ≤ 10 fichiers par dossier, un élément
  public par fichier, aucune chaîne visible en dur, clés ajoutées dans les deux catalogues dans le
  même commit, WCAG 2.2 AA, tests livrés avec le code.
- Seul asset public actuel de l'app : `frontend/projects/app/public/favicon.ico`.
- Aucune icône dans l'app (aucun `ion-icon`, aucun `<svg>` dans `frontend/projects`) ; les
  calques utilisent des emoji `👁` et `✕` (`timeline/layers/layer-list.html`).

### 2.2 Styles partagés (`frontend/projects/shared/src/styles/`)

| Fichier | État |
|---|---|
| `_tokens.scss` | Palette V1 indigo/gris (`--lp-color-*` + variantes `-rgb`), `--lp-space-1..6` (5 = 1.5rem, 6 = 2rem), `--lp-radius-small/medium/large/round`, `--lp-font-*` (police système) |
| `_ionic.scss` | Variables Ionic mappées sur les tokens (fond, texte, primary, danger, paliers) |
| `_accessibility.scss` | Focus 2 px / offset 2 px, mouvement réduit (`data-motion`, media query), `.lp-visually-hidden` |
| `_index.scss` | `@forward` des trois fichiers ; chargé par `frontend/projects/app/src/styles.scss` et par l'admin |

**Collision de noms** avec `design-system/tokens/tokens.css` : celui-ci nomme `--lp-bg`,
`--lp-surface`, `--lp-text`…, et redéfinit `--lp-space-5` = 1.25rem et `--lp-space-6` = 1.5rem, là
où la production utilise 1.5rem et 2rem. Les deux fichiers ne peuvent pas être chargés ensemble
tels quels. `design-system/docs/adoption.md` (« Preserve the token contract ») exige de conserver
les alias de production, les propriétés RGB d'Ionic et les couleurs de graphes admin
`--lpa-chart-*`, et d'ajouter une bordure décorative distincte de `--lp-color-border` (qui sert de
bordure de contrôle).

### 2.3 Shell de l'application

- `frontend/projects/app/src/app/app.html` : `ion-app` > `lp-app-header`,
  `ion-router-outlet role="main"`, `lp-app-footer`, `lp-engine-notifications`. Pas de lien
  d'évitement.
- `shell/app-header.html|.ts|.scss` : nom en texte, liens Éditeur / Bibliothèque / Réglages /
  Aide (`/support`, route `hostedOnly` : sur desktop, Aide mène à une route refusée — signalé dans
  `journeys.md`), menu compte injecté (`ACCOUNT_MENU_SLOT`).
- `account/account-menu.ts` : e-mail + 3 liens/boutons en ligne dans l'en-tête, pas de vrai menu.
- `shell/engine-notifications.ts` : erreurs moteur en pastilles danger fixées en bas, `z-index:
  30000`, par-dessus la timeline.
- **Structure de page incohérente** : `settings`, `account`, `support`, `mcp/agents`, `legal`,
  `not-found` utilisent `ion-content` ; `library-page.ts`, `project-page.ts`, `tokens-page.html`,
  `sign-in-page.html`, `sign-up-page.html`, `reset-password*.html`, `verify-email-page.ts` posent
  un `<main>` nu dans l'outlet, qui a déjà `role="main"` (landmark `main` imbriqué). Hypothèse à
  confirmer au lot 0 : sans `ion-content`, ces pages ne défilent pas (l'hôte routé Ionic
  `.ion-page` est en `overflow: hidden`), ce qui rendrait une longue bibliothèque ou un formulaire
  sur petit écran inutilisables.

### 2.4 Éditeur (`frontend/projects/app/src/app/`)

| Fichier | Constat |
|---|---|
| `editor/editor-page.html|.scss|.ts` | Grille `header / tools canvas palette / timeline` ; la ligne du canevas est `minmax(0, 1fr)` alors que la timeline (calques, tags, images, durées, bouton lecture, aperçu jusqu'à 12rem) est en `auto` : sur un portable de 768 px de haut, le canevas peut presque disparaître (à mesurer au lot 0). Sous 767 px, tout s'empile et la page défile. Au premier accès, un effet ouvre d'office la modale « Nouvelle animation ». |
| en-tête de l'éditeur | Titre éditable (`editor/title/animation-title.ts`), `ion-button` « Nouveau », `lp-save-button`, `lp-open-status`, `lp-export-button` alignés sans hiérarchie |
| `library/save/save-button.ts` | Statut limité à « Enregistrement… » / « Enregistré » ; `SaveStatus = 'idle' \| 'saving' \| 'saved'` (`save-flow.ts`) : ni « Non enregistré », ni « Modifications non enregistrées », ni « Échec » |
| `tools/tool-bar.html|.scss`, `tools/tool-definitions.ts` | Outils en boutons texte (pencil B, eraser E, fill G, line L, rectangle R, select M, filled Shift+R), undo/redo, deux imports, aide `?` ; tooltips via `title` seulement |
| `canvas/canvas.html|.scss|.ts` | Surface `role="application"` ; zoom seulement au clavier (`+ - 0`) ou Ctrl+molette ; grille (Shift+G) et pelure d'oignon (O) **sans aucun contrôle visible** ; pas d'affichage du zoom, des coordonnées, de l'outil, du calque ni de l'image actifs ; fond du canevas = `--lp-color-surface` de la région |
| `editor/editor-store.ts` | `activeLayer` existe (calque du haut par défaut) mais **aucune UI ne permet de le choisir** : on ne dessine que sur le calque du haut. `zoom`, `showGrid`, `onionSkin` existent sans contrôle visible |
| `palette/palette-panel.html|.scss` | Pastilles avec deux boutons texte « Modifier » / « Supprimer » par couleur ; sélection par `aria-pressed` et une bordure |
| `timeline/timeline.html|.scss|.ts` | Colonnes calques 12rem + bande ; aperçu en pleine largeur dessous |
| `timeline/layers/layer-list.html` | Emoji, pas de calque actif, champ de nom toujours en édition |
| `timeline/frames/frame-list.html|.scss|.ts` | Vignette 48 px, champ durée sous chaque image, boutons texte Ajouter/Dupliquer/Supprimer ; la sélection visuelle suit `frameSelection`, pas `activeFrame` |
| `timeline/tags/tag-bars.html`, `tags/tag-dialog.*` | Barres de tags en boutons |
| `timeline/playback/playback-preview.*` | Un échec d'export met fin à la lecture sans message (`catch(() => null)`) |
| `editor/new-animation/*`, `tools/import/*`, `tools/shortcuts-help-dialog.*`, `palette/palette-entry-*`, `export/*`, `library/save/save-dialog.html` | Dialogues `ion-modal` au style local ; boutons primaires/secondaires redéfinis à la main |

Quinze fichiers redéfinissent localement `button {}` / `input {}` / `.primary`
(`tool-bar.scss`, `export-snippet.scss`, `export-format-table.scss`, `palette-entry-form.scss`,
`new-animation-form.scss`, `export-options-form.scss`, `library-dialog.scss`,
`import-sprite-sheet-form.scss`, `token-dialog.scss`, `tokens-page.scss`, `tag-dialog.scss`,
`account-menu.ts`, `import-image-button.ts`, `import-sprite-sheet-button.ts`,
`engine-notifications.ts`). Les boutons natifs et `ion-button` coexistent.

### 2.5 Autres pages de l'app

| Zone | Fichiers |
|---|---|
| Bibliothèque | `library/pages/library-page.ts|.scss`, `project-page.ts`, `library/lists/animation-list.html`, `project-list.html`, `library-list.scss`, `library/library-dialog.scss`, `library/actions/*` (quatre boutons texte par ligne ; un seul état vide générique) |
| Réglages | `settings/settings-page.html|.scss|.ts` (select + radios bruts ; liens jetons/agents en fin de page) |
| Compte | `account/*-page.html`, `account-form.scss`, `account-page.scss`, `password-field.ts` |
| Jetons, agents | `tokens/*`, `mcp/agents-page.*`, `mcp/copy-block.ts` |
| Support, légal, 404 | `support/*`, `legal/legal-page.ts`, `not-found/not-found-page.ts` |

### 2.6 Admin (`frontend/projects/admin/`)

Même `_index.scss` partagé : **changer les tokens partagés change aussi l'admin**.
`src/styles.scss`, `app/ui/confirm-action.html`, `app/ui/empty-state.ts`,
`app/monitoring/chart-renderer.ts` (couleurs uPlot `--lpa-chart-*`), pages listées dans
`design-system/docs/adoption.md`.

### 2.7 Tests existants concernés

- Unitaires Vitest : un `*.spec.ts` à côté de presque chaque composant (`test:ci`, moteur mock).
- E2E éditeur `frontend/e2e/editor/` : `editor-page.ts` (page object par rôles et noms
  accessibles en anglais : « Drawing canvas », « New animation », « Export the animation »,
  région « Tools », « Palette », « Colour N », « Frame N »,
  « Duration of frame N, in milliseconds », « Edit tag “…” », table
  « Every export format, with its size »), `journey.spec.ts`, `keyboard.spec.ts`,
  `accessibility.spec.ts` (axe WCAG 2.2, bloque serious/critical sur éditeur, export, raccourcis,
  réglages). `EditorPage.open()` **attend que la modale « New animation » s'ouvre seule** au
  premier accès, et `pointAt()` suppose un canevas centré au zoom 8.
- E2E hébergé `frontend/e2e/hosted/` (bibliothèque, conflit, quota, visiteur, suppression…),
  pages objets dans `e2e/hosted/pages/`.
- `frontend/playwright.config.ts` : projets `editor` (sert `npm start` sur 4260) et `hosted`.

### 2.8 Consignes et outils des agents

`AGENTS.md`/`CLAUDE.md` ne mentionnent `design-system/` que dans les scopes de commit ; la carte
documentaire ne le cite pas. Aucun outil ne produit de capture de la vraie application : un agent
ne voit jamais ce qu'il construit. `docs/v1/editor.md` décrit l'ouverture forcée de la modale et
devra évoluer avec le code (règle « code et documentation d'accord dans la même PR »).

## 3. Choix retenus et alternatives écartées

| # | Choix retenu | Alternatives écartées et raison |
|---|---|---|
| C1 | **Tokens** : `_tokens.scss` reste la source canonique de production. On y porte les valeurs Rose Atelier (clair, sombre, `forced-colors`) **sous les noms existants** `--lp-color-*` (+ `-rgb`), et on ajoute les rôles manquants avec des noms explicites : `--lp-color-surface-soft`, `--lp-color-surface-raised`, `--lp-color-accent-hover`, `--lp-color-accent-soft`, `--lp-color-accent-text`, `--lp-color-border-subtle` (décoratif), `--lp-color-success/-warning/-info` et leurs `-bg`, `--lp-color-canvas-bg`, `--lp-color-checker-light/-dark`, ombres, z-index, durées, `--lp-control-height*`. `--lp-color-border` garde son rôle de bordure de contrôle (valeur = `control-border` du DS). L'échelle d'espacement garde son sens ordinal (5 = 1.5rem, 6 = 2rem) et s'étend (`--lp-space-8` = 2rem… n'est ajouté que si nécessaire, sans redéfinir 5 et 6). Un petit test Node (`frontend/tools/…test.mjs`, lancé par `test:tools`) vérifie que les couleurs de `_tokens.scss` égalent celles de `design-system/tokens/tokens.css` rôle par rôle, pour empêcher la dérive. | Importer `design-system/tokens/tokens.css` tel quel : fichier hors workspace, noms incompatibles, redéfinition de `--lp-space-5/6`, pas de variantes RGB pour Ionic. Renommer tous les tokens de production : casse l'admin et chaque composant d'un coup, sans gain d'usage. |
| C2 | **Police Nunito** et **assets** (`mark.svg`, `pip.svg`, `sprout.svg`) copiés au build vers `frontend/projects/app/public/` par un script sur le modèle de `tools/copy-i18n.mjs`, branché dans `start` et `build:app` ; `@font-face` local avec `font-display: swap` et pile de repli complète. | Copie manuelle commitée (deux sources qui divergent). Google Fonts / CDN (requête tierce, interdit par le DS et par l'invariant de télémétrie). |
| C3 | **Icônes** : un composant `lp-icon` (app) et un registre TypeScript de tracés SVG 24×24, trait 1.75, bouts ronds, dessinés d'après le style de `design-system/preview/sections/studio.html` (pinceau, gomme, remplissage, sélection y figurent déjà). Toujours `aria-hidden` ; le nom accessible est porté par le bouton. | Ionicons (fourni par Ionic) : style différent, enregistrement global, poids. Emoji : interdits comme icônes fonctionnelles par `foundations.md`. |
| C4 | **Composants communs = classes CSS globales d'abord** : un nouveau partiel `frontend/projects/shared/src/styles/_components.scss` (boutons primaire/secondaire/discret/danger/bascule/icône, champs avec libellé, aide, erreur et unité, pastilles d'état, bannières succès/avertissement/erreur/info, cartes/panneaux, barre d'actions de dialogue, en-tête de page). Composants Angular seulement quand un comportement se répète : `lp-icon`, `lp-empty-state` (titre, texte, action, Pip optionnel), `lp-status-banner`. | Une bibliothèque de wrappers pour chaque élément natif : explicitement déconseillé par `adoption.md` (§3), et coûteux. Styles recopiés par composant : dépasse le budget 8 kB et recrée l'incohérence actuelle. |
| C5 | **Ionic** conservé pour `ion-app`, l'outlet, `ion-content`, `ion-modal`, `ion-alert` (focus, inertie, Échap). Les boutons deviennent des `<button>` natifs stylés par C4, y compris ceux aujourd'hui en `ion-button` (éditeur, bouton Enregistrer), pour un seul langage visuel. | Tout en composants Ionic : style Material/iOS difficile à plier au DS, shadow DOM. Retirer Ionic : contraire à la décision D2 et à `AGENTS.md`. |
| C6 | **Gabarit de page unique** : toute page routée hors éditeur = `ion-content` > conteneur `.lp-page` (largeur de lecture ou de contenu), titre `h1`, pas de `<main>` imbriqué ; l'outlet garde `role="main"` ; un lien d'évitement « Aller au contenu » dans `app.html`. | Garder les deux structures actuelles : défilement cassé et landmarks en double. |
| C7 | **Éditeur** selon `journeys.md` §2 et `foundations.md` : barre de document (titre, pastille d'état d'enregistrement, Nouveau, Enregistrer, Exporter en action primaire) ; rail d'outils à gauche (icône + nom accessible + infobulle avec raccourci) ; scène de canevas neutre avec **barre de vue** (zoom −/+/ajuster et valeur en %, grille, pelure d'oignon, coordonnées du curseur, image et calque actifs) ; **inspecteur** à droite (17rem : palette, calques, aperçu) ; **timeline** en bas (~11rem : bande d'images, durées en ms, tags, lecture). L'aperçu de lecture quitte la timeline pour l'inspecteur, dans une boîte de taille fixe, afin que le canevas garde toujours la priorité. | Garder la grille actuelle et ne changer que les couleurs : ne corrige ni l'écrasement du canevas ni les commandes invisibles, cause principale du « impossible à utiliser ». Copier la mise en page éditoriale de la référence (barre latérale de documentation) : `vision.md` dit explicitement que ce n'est pas la mise en page de l'éditeur. |
| C8 | **Choix du calque actif** : cliquer une ligne de calque règle `EditorStore.activeLayer` (état d'interface déjà existant, aucune opération moteur) ; ligne active marquée par forme + texte (`aria-current` ou équivalent). Idem : la bande d'images marque `activeFrame` distinctement de `frameSelection`. | Laisser le dessin sur le seul calque du haut : défaut fonctionnel réel de la V1. |
| C9 | **État d'enregistrement explicite** (`patterns.md`, « Saving and unsaved work ») : Non enregistré (nouveau travail invité) / Modifications non enregistrées / Enregistrement… / Enregistré / Échec de l'enregistrement, dérivé par une fonction pure de `SaveFlow.status`, `EngineStore.hasUnsavedWork` et `CurrentAnimation`. L'état « Échec » demande d'étendre `SaveFlow` (le type `SaveStatus` ne l'a pas). Jamais « Sauvegardé automatiquement ». | Garder le texte actuel : n'informe pas le visiteur que son travail est volatil (D37). |
| C10 | **Premier accès** : un accueil dans l'espace de travail vide (titre, texte, notice invité, Pip, action primaire « Créer une animation » qui ouvre le formulaire existant) au lieu de la modale imposée. `/editor` reste l'entrée ; aucune nouvelle route. | Garder la modale imposée : c'est ce que teste `EditorPage.open()`, mais le DS le remplace (`journeys.md` §1). Une route d'onboarding séparée : fonctionnalité en plus, hors besoin. « Importer un PNG » depuis l'accueil : impossible sans document (l'import s'applique au calque et à l'image actifs), donc écarté de ce plan — voir questions. |
| C11 | **Écrans étroits** : sous 48rem, canevas d'abord, rail d'outils horizontal, un seul panneau auxiliaire à la fois via des onglets (Palette / Calques / Aperçu, motif `tablist` de `components.md`), bande d'images sous le canevas ; entre 48 et 75rem, inspecteur repliable ; au-delà, panneaux persistants. Media queries en rem littéraux. | Empiler toutes les régions (état actuel). Tiroir modal : plus lourd, piège de focus inutile. |
| C12 | **Outils des agents** : ajouter `design-system/` à la carte documentaire et une courte section « Interface » dans `AGENTS.md` et `CLAUDE.md` (identiques) : lire `design-system/docs/` avant toute UI, tokens sémantiques, classes de `_components.scss`, Pip limité, canevas neutre, captures obligatoires. Ajouter un projet Playwright `visual` (`frontend/e2e/visual/`) qui capture la vraie application (moteur réel) à 1440, 1024, 768 et 390 px, clair/sombre, fr/en, dans un dossier ignoré par Git : c'est un outil de revue, pas un test de comparaison de pixels. | Tests de régression par comparaison d'images : fragiles (polices, antialiasing, OS), bloqueraient la CI pour du bruit. Ne rien ajouter : reproduit la cause identifiée par le mainteneur. |
| C13 | **Admin** : dernier lot, sobre ; il hérite des tokens dès le lot 1, donc le lot 1 vérifie déjà son contraste et ses graphes, et le lot admin ajuste tables, filtres, confirmations. Pas de Pip, peu de rose. | Laisser l'admin hors plan : impossible, les tokens partagés le modifient de toute façon. |
| C14 | **Décision** : si le mainteneur valide Rose Atelier, l'enregistrer comme décision acceptée D38 dans `docs/decisions.md` (lot 0), avec mention du canevas neutre et du périmètre (pas de nouvelle fonctionnalité produit). | Adopter sans trace : `adoption.md` demande de résoudre les décisions avec le mainteneur. |

## 4. Découpage en lots

Chaque lot = une branche `feat/…` ou `style/…` depuis `dev`, une PR vers `dev`, scopes de commit
selon `AGENTS.md` (`shared`, `app`, `admin`, `frontend`, `i18n`, `docs`). Chaque lot livre ses
tests, ses clés i18n EN+FR (espaces de noms de sa zone uniquement), ses captures du projet
`visual`, et met à jour `docs/v1/editor.md` ou `docs/v1/accounts.md` s'il change un comportement
documenté.

**Point de contention commun** : `i18n/en.json` et `i18n/fr.json` (JSON à plat trié). Les lots
parallèles n'y ajoutent que des clés de leur espace de noms (`editor.*`, `tools.*`, `palette.*`,
`timeline.*`, `export.*`, `library.*`, `settings.*`, `auth.*`, `account.*`, `common.*`,
`shell.*`) ; les conflits de fusion seront textuels et triviaux, à résoudre à l'intégration.
Réutiliser les clés existantes quand le sens est le même (`design-system/docs/content.md`).

### Lot 0 — Cadrage, consignes et outil de capture (bloquant pour tous)

- **Fichiers** : `AGENTS.md`, `CLAUDE.md` (même texte), `docs/decisions.md` (D38 si validée),
  `frontend/playwright.config.ts` (projet `visual`), `frontend/e2e/visual/` (nouveau :
  `capture.spec.ts`, `.gitignore` pour la sortie), `frontend/package.json` (script
  `e2e:visual`).
- **Contenu** : consignes C12 ; projet `visual` qui ouvre `/editor`, crée une animation 32×32,
  dessine quelques pixels, ajoute une image, ouvre l'export, les réglages et (si possible sans
  pile hébergée) la 404, puis capture chaque écran aux quatre largeurs × deux thèmes × deux
  langues. **Captures de référence de l'état actuel** jointes à la PR, et mesures : hauteur utile
  du canevas à 1366×768 et 1024×768 ; défilement des pages sans `ion-content` (confirme ou
  infirme l'hypothèse du §2.3).
- **Dépendances** : aucune. **Parallèle** : rien d'autre ne démarre avant sa fusion (les consignes
  et l'outil servent à tous).

### Lot 1 — Fondations visuelles

- **Fichiers** : `frontend/projects/shared/src/styles/_tokens.scss`, `_ionic.scss`,
  `_accessibility.scss` (focus 3 px / offset 3 px), `_index.scss`, nouveau `_base.scss`
  (typographie, `@font-face`, fond de page, liens), `frontend/projects/app/src/styles.scss`,
  `frontend/projects/admin/src/styles.scss` (vérifier, ajuster seulement si nécessaire),
  `frontend/tools/copy-design-assets.mjs` (nouveau) + son test, `frontend/package.json`
  (`start`, `build:app`), `.gitignore` ou `frontend/.gitignore` pour la copie dans `public/`,
  test de parité des tokens dans `frontend/tools/` ;
  `frontend/projects/app/src/app/canvas/render/canvas-renderer.ts` (constantes du damier et de
  la grille alignées sur le DS, voir §6) et `canvas/canvas.scss` (fond `--lp-color-canvas-bg`).
- **Contenu** : C1, C2. Couleurs de graphes admin conservées ou remappées avec des paires
  mesurées. Aucun changement de balisage de composant.
- **Dépendances** : lot 0. **Parallèle** : seul (tout le reste en dépend).

### Lot 2 — Primitives et shell

- **Fichiers** : `shared/src/styles/_components.scss` (nouveau, C4) ; nouveau dossier
  `frontend/projects/app/src/app/ui/` avec sous-dossiers `icon/` (`icon.ts`, `icon-paths.ts`,
  spec), `empty-state/`, `status-banner/` ; `app.html`, `app.scss` (lien d'évitement) ;
  `shell/app-header.html|.scss|.ts` (marque cœur + « Life Pixel », navigation, lien Aide masqué ou
  redirigé selon la plateforme — voir questions) ; `account/account-menu.ts` (menu déroulant
  étiqueté : e-mail, Compte, Bibliothèque, Déconnexion) ; `shell/app-footer.ts` ;
  `shell/engine-notifications.ts` (erreur persistante, fond `danger-bg`, icône + texte, z-index
  de la nouvelle échelle, ne masque ni la barre d'actions ni l'outil) ; `i18n/*.json`
  (`shell.*`, `common.*`).
- **Dépendances** : lot 1. **Parallèle** : seul ; il fixe les classes que les lots 3 à 9
  consomment.

### Lot 3 — Éditeur : structure, barre de document, accueil

- **Fichiers** : `editor/editor-page.html|.scss|.ts` (grille C7 : zones `docbar`, `rail`,
  `stage`, `inspector`, `timeline` ; hauteur minimale garantie au canevas) ;
  `timeline/timeline.html|.scss|.ts` (retrait de `lp-playback-preview`, déplacé dans
  l'inspecteur) ; `library/save/save-button.ts`, `library/save/save-flow.ts` (+ état d'échec,
  C9) et une fonction pure `save-state.ts` + spec ; `library/open/open-status.ts` ;
  `export/export-button.ts` ; `editor/title/animation-title.ts` ; nouveau
  `editor/welcome/` (accueil C10, avec `lp-empty-state` et Pip) ;
  `editor/new-animation/new-animation-dialog.ts|form.html|form.scss` (anatomie de dialogue,
  unités « px », limites du moteur, focus sur le titre) ; `docs/v1/editor.md` (U1 : premier accès)
  ; `frontend/e2e/editor/editor-page.ts` (`open()` : accueil puis « Create animation ») ;
  `i18n` (`editor.*`, `library.save.*`).
- **Contenu** : prévoir dans le gabarit les emplacements que les lots 4 et 5 rempliront (barre de
  vue sous le canevas, contenu de l'inspecteur), pour qu'ils ne touchent plus
  `editor-page.html`.
- **Dépendances** : lot 2. **Parallèle** : avec les lots 7, 8a, 8b, 9.

### Lot 4 — Éditeur : outils et canevas

- **Fichiers** : `tools/tool-bar.html|.scss|.ts`, `tools/tool-definitions.ts` (ajout de la clé
  d'icône) ; `tools/import/import-image-button.ts`, `import-sprite-sheet-button.ts` ;
  `tools/shortcuts-help-dialog.*` ; `canvas/canvas.html|.scss|.ts` ; nouveau composant de barre
  de vue, par exemple `canvas/view-bar/` (zoom −/+/ajuster + %, grille, pelure d'oignon et son
  nombre d'images avant/après si pertinent, coordonnées du curseur, calque et image actifs), qui
  **appelle les mêmes actions** que les raccourcis existants (`CanvasViewport`, `EditorStore`) ;
  specs associées ; `i18n` (`tools.*`, `canvas.*`).
- **Contenu** : rail d'icônes avec `aria-pressed`, infobulle au survol et au focus avec
  raccourci ; undo/redo proches ; « Rectangle plein » en bascule rattachée au rectangle ; curseur
  clavier contrasté sur toute œuvre ; aucun raccourci modifié (B, E, G, L, R, Shift+R, M, O,
  Shift+G, + − 0, Ctrl/⌘ Z/Y, ?).
- **Dépendances** : lot 3. **Parallèle** : avec le lot 5 (fichiers disjoints), 7, 8, 9.

### Lot 5 — Éditeur : inspecteur et timeline

- **Fichiers** : `palette/palette-panel.html|.scss|.ts`, `palette/palette-entry-*` ;
  `timeline/layers/layer-list.html|.scss|.ts` (sélection du calque actif C8, icônes œil/corbeille,
  renommage), `timeline/frames/frame-list.html|.scss|.ts`, `frame-thumbnail.ts`,
  `timeline/tags/tag-bars.*`, `tag-dialog.*`, `timeline/playback/playback-preview.*` (état
  chargement/erreur visible, boîte de taille fixe dans l'inspecteur) ; specs ; `i18n`
  (`palette.*`, `timeline.*`).
- **Contenu** : pastilles avec double anneau de sélection + coche, index et valeur hex lisibles,
  actions Modifier/Supprimer accessibles sans survol mais regroupées (une paire pour la couleur
  sélectionnée plutôt que par pastille) ; transparence (index 0) décrite ; images numérotées avec
  durée « ms », image active marquée, sélection de plage distincte ; alternatives clavier au
  glisser conservées (Alt+flèches).
- **Dépendances** : lot 3. **Parallèle** : avec le lot 4, 7, 8, 9.

### Lot 6 — Éditeur : adaptation aux écrans étroits et moyens

- **Fichiers** : `editor/editor-page.scss|.html` (points de rupture 48rem et 75rem), nouveau
  composant d'onglets de l'inspecteur (par exemple `editor/inspector-tabs/`), ajustements de
  `tool-bar.scss`, `timeline.scss`, `canvas.scss` ; tests d'onglets (flèches, Home/End) ;
  e2e `visual` à 390 et 768 px.
- **Contenu** : C11 ; zoom, sélection, calque/image actifs et historique préservés au changement
  de disposition ; pas de défilement horizontal de page à 320 px hors canevas et timeline.
- **Dépendances** : lots 4 et 5. **Parallèle** : avec 7, 8, 9.

### Lot 7 — Dialogues de flux : export, enregistrement, import, confirmations

- **Fichiers** : `export/export-dialog.ts`, `export-format-table.html|.scss`,
  `export-options-form.html|.scss`, `export-snippet.html|.scss` ;
  `library/save/save-dialog.html`, `save-prompts.ts`, `project-picker.ts`,
  `library/library-dialog.scss` ; `tools/import/import-sprite-sheet-form.html|.scss`,
  `import-sprite-sheet-dialog.ts` ; `editor/new-animation/discard-confirmation.ts`,
  `library/open/open-confirmation.ts` ; `i18n` (`export.*`, `library.save.*`, `import.*`).
- **Contenu** : anatomie commune (titre, contexte, contenu défilant, zone d'échec en ligne,
  barre d'actions) ; plus petit format indiqué en mots ; états par format (préparation, prêt,
  échec) ; note « exporter n'enregistre pas » ; bouton Copier avec état copié/échec ; Pip discret
  après un téléchargement réussi ; conflit : « Enregistrer une copie » recommandé, conséquences
  de Recharger/Écraser écrites, rien de destructif présélectionné ; quota : usage réel ou état
  indisponible, jamais zéro inventé.
- **Dépendances** : lot 2 (et lot 3 pour le seul point de contact : le bouton Exporter, déjà
  fixé). **Parallèle** : avec 3 à 6, 8, 9. Ne touche pas `editor-page.*`.

### Lot 8a — Bibliothèque et réglages

- **Fichiers** : `library/pages/library-page.ts|.scss`, `project-page.ts`,
  `library/lists/animation-list.html|.ts`, `project-list.html|.ts`, `library-list.scss`,
  `library/actions/*` si le menu d'actions l'exige ; `settings/settings-page.html|.scss|.ts` ;
  `mcp/agents-page.*`, `mcp/copy-block.ts` ; `i18n` (`library.*`, `settings.*`, `mcp.*`) ;
  pages objets `frontend/e2e/hosted/pages/library-page.ts` si les noms accessibles changent.
- **Contenu** : gabarit C6 ; en-tête avec action « Créer une animation » ; états distincts
  chargement / vide (Pip) / aucun résultat de recherche (sans Pip, avec « Effacer ») / échec ;
  lignes compactes titre-lien + dimensions/nombre d'images + menu d'actions (Supprimer séparé) ;
  « Charger plus » conservé ; réglages groupés Langue / Apparence (cartes radio système, clair,
  sombre) / Mouvement / Bibliothèque locale et Agents (desktop) / Jetons (compte).
- **Dépendances** : lot 2. **Parallèle** : avec 3 à 7, 8b, 9.

### Lot 8b — Compte, authentification, jetons, support, légal, 404

- **Fichiers** : `account/sign-in-page.html`, `sign-up-page.html`, `reset-password-page.html`,
  `reset-password-confirm-page.html`, `verify-email-page.ts`, `account-page.html|.scss`,
  `account-form.scss`, `password-field.ts` ; `tokens/*.html`, `tokens-page.scss`,
  `token-dialog.scss` ; `support/*.html`, `support.scss` ; `legal/legal-page.ts` ;
  `not-found/not-found-page.ts` ; `i18n` (`auth.*`, `account.*`, `tokens.*`, `support.*`,
  `not_found.*`) ; pages objets `frontend/e2e/hosted/pages/account-pages.ts`,
  `support-pages.ts` si besoin.
- **Contenu** : gabarit C6 ; formulaires courts avec libellés visibles, erreurs liées aux champs,
  résumé d'erreurs au premier champ invalide, contexte de retour conservé ; suppression de compte
  et révocation de jeton sans décoration ; secret de jeton affiché une fois avec copie.
- **Dépendances** : lot 2. **Parallèle** : avec 3 à 7, 8a, 9.

### Lot 9 — Console d'administration (sobre)

- **Fichiers** : `frontend/projects/admin/src/styles.scss`, `app/shell/app-header.html`,
  `app/shell/view-controls.html`, `app/ui/confirm-action.html`, `app/ui/empty-state.ts`,
  `app/monitoring/chart-renderer.ts`, pages `app/**/*-page.html` au besoin ; `i18n` (`admin.*`).
- **Contenu** : bannière d'environnement très visible, tables denses avec en-têtes natifs,
  filtres étiquetés, confirmations avec impact ; aucune mascotte.
- **Dépendances** : lots 1 et 2 (classes partagées). **Parallèle** : avec 3 à 8. Peut être
  reporté si le mainteneur le décide.

### Lot 10 — Intégration et vérification finale

- **Fichiers** : `frontend/e2e/editor/*.spec.ts` (parcours complet dans la nouvelle
  disposition, axe à 390 px en plus), `docs/v1/editor.md`, `docs/v1/accounts.md` si concerné,
  `design-system/docs/adoption.md` (statut d'adoption par phase) et
  `design-system/docs/verification.md` (résultats observés sur l'application).
- **Contenu** : exécuter toute la batterie du §5, produire les captures `visual` finales,
  remplir la matrice manuelle, consigner ce qui n'a pas pu tourner.
- **Dépendances** : tous les lots retenus.

### Graphe

```text
L0 ─► L1 ─► L2 ─┬─► L3 ─┬─► L4 ─┐
                │       └─► L5 ─┴─► L6 ─┐
                ├─► L7 ─────────────────┤
                ├─► L8a ────────────────┤
                ├─► L8b ────────────────┼─► L10
                └─► L9 ─────────────────┘
```

Parallélisme maximal après L2 : L3, L7, L8a, L8b, L9 ; puis L4 ∥ L5 ; puis L6.

## 5. Stratégie de tests et critères de réussite

### Commandes (depuis la racine ; `frontend/node_modules` absent à la date de l'audit du DS)

```shell
npm ci --prefix frontend
npm run lint --prefix frontend
npm run test:ci --prefix frontend          # Vitest, moteur mock
npm run test:tools --prefix frontend       # scripts Node, dont copie d'assets et parité des tokens
npm run build --prefix frontend            # budgets de style 8 kB par composant
npm run i18n:check --prefix frontend
npm run i18n:check-bundle --prefix frontend
npm run test:engine --prefix frontend      # construit le moteur (LP_ENGINE_PREBUILT=1 pour sauter)
npm run e2e --prefix frontend              # projet editor, moteur réel, port 4260
npm run e2e:hosted --prefix frontend       # nécessite la pile locale
npm run e2e:visual --prefix frontend       # nouveau (lot 0) : captures, pas d'assertion de pixels
node design-system/scripts/check.mjs
cmp CLAUDE.md AGENTS.md
```

Chaque lot exécute au minimum lint, `test:ci`, `build`, `i18n:check` et, s'il touche l'éditeur,
`e2e` et `e2e:visual` ; il rapporte les sorties et signale explicitement ce qui n'a pas pu
tourner.

### Tests à écrire (comportement, pas implémentation)

| Lot | Tests |
|---|---|
| L0 | Le projet `visual` produit les captures attendues sans erreur de page |
| L1 | Parité des couleurs `_tokens.scss` ↔ `design-system/tokens/tokens.css` ; script de copie d'assets (sources présentes, cibles écrites) ; axe e2e existants toujours verts |
| L2 | `lp-icon` : `aria-hidden`, tracé connu ; `lp-empty-state` : titre, action, Pip décoratif (`alt=""`) ; menu compte : ouverture clavier, Échap, retour du focus ; lien d'évitement focalisable |
| L3 | Fonction d'état d'enregistrement : les cinq états ; accueil affiché sans document puis masqué après création ; « Créer une animation » ouvre le formulaire avec focus sur le titre ; `SaveFlow` passe en échec et le travail reste ouvert |
| L4 | Boutons de zoom/ajuster/grille/pelure d'oignon modifient `EditorStore` comme les raccourcis ; outil sélectionné exposé par `aria-pressed` ; infobulle contient le raccourci |
| L5 | Clic sur un calque ⇒ `activeLayer` ; dessin suivant sur ce calque (mock) ; image active distincte de la plage ; aperçu : message visible si l'export échoue |
| L6 | Onglets de l'inspecteur : flèches, Home/End, un seul panneau visible ; état conservé au changement de largeur |
| L7 | Conflit : aucune action destructive focalisée d'abord ; copie : états copié/échec ; export : plus petit format signalé en texte |
| L8a/8b | États vide / sans résultat / échec distincts ; réglages : thème appliqué sans rechargement ; formulaires : focus sur le premier champ invalide |
| L10 | Parcours U6 complet dans la nouvelle disposition, au pointeur et au clavier ; axe sans violation serious/critical sur éditeur, export, raccourcis, réglages, bibliothèque (hébergé) et éditeur à 390 px |

### Critères de réussite vérifiables

1. Toutes les commandes ci-dessus passent (ou leur indisponibilité est consignée avec la cause).
2. Aucun composant de l'app ne redéfinit `button {}` ou `input {}` localement : une recherche
   `rg "^\s*(button|input)\s*\{" frontend/projects/app/src` ne renvoie que des exceptions
   justifiées (ex. surface de canevas).
3. Aucune couleur hexadécimale en dur dans `frontend/projects/app/src/**/*.scss` ; les seules
   couleurs littérales de l'app sont les constantes nommées du damier et de la grille dans
   `canvas-renderer.ts` ; aucun emoji fonctionnel dans les gabarits.
4. Toutes les pages routées hors éditeur utilisent le gabarit C6 : aucune balise `<main>` dans
   `frontend/projects/app/src/app/**` ; chaque page défile jusqu'en bas à 390×640.
5. À 1366×768 et 1024×768, la scène du canevas mesure au moins la moitié de la hauteur de la
   fenêtre (mesure dans le projet `visual`) ; à 32×32 px et zoom 8, l'œuvre entière est visible
   sans défilement.
6. Zoom, grille, pelure d'oignon, calque actif et image active sont visibles et utilisables au
   pointeur et au clavier ; tous les raccourcis existants fonctionnent à l'identique.
7. La pastille d'enregistrement affiche l'état juste dans les cinq cas ; jamais « Enregistré »
   avant le succès confirmé.
8. Premier accès : un visiteur voit l'accueil, crée, dessine deux images, lit et exporte sans
   compte (parcours e2e).
9. Les deux catalogues sont complets (`i18n:check`) ; interface vérifiée en français sur les
   captures (pas de libellé tronqué à 1024 et 390 px).
10. Thèmes clair, sombre, système et mouvement réduit appliqués partout, y compris dans les
    dialogues Ionic ; le canevas et le damier restent neutres dans les deux thèmes.
11. `AGENTS.md` et `CLAUDE.md` identiques et citent `design-system/` ; `docs/v1/editor.md`
    décrit le comportement livré.
12. Matrice manuelle (`design-system/docs/accessibility.md`, « Review matrix ») remplie pour
    l'éditeur : clavier, zoom 200 %/400 %, 320 px, NVDA, forced colors ; défauts restants
    consignés dans `design-system/docs/verification.md`.

## 6. Risques et questions ouvertes

### Risques

| Risque | Parade |
|---|---|
| Les noms accessibles changent et cassent les e2e (`editor-page.ts` cherche des textes anglais exacts) | Garder les libellés accessibles existants quand le sens est le même ; chaque lot met à jour les pages objets dans la même PR |
| Les tokens partagés modifient l'admin dès le lot 1 | Captures et axe de l'admin (e2e hébergé `console-sign-in.spec.ts`) dans le lot 1 |
| Budget de style 8 kB dépassé par l'éditeur | Styles communs globaux (`_components.scss`), composants découpés |
| Nunito alourdit le premier rendu (TTF variable) | `font-display: swap`, préchargement mesuré ; sous-ensemble WOFF2 si le mainteneur l'accepte (question 4) |
| Conflits de fusion sur `i18n/*.json` et `editor-page.html` | Espaces de noms par lot ; L3 fige le gabarit et ses emplacements avant L4/L5 |
| Nouvelle disposition qui décale `pointAt()` (canevas supposé centré au zoom 8) | Vérifier que le canevas reste centré ; sinon adapter le page object |
| Shadow DOM Ionic : styles et focus des modales | Propriétés et `::part` documentés d'Ionic, pas de `::ng-deep` (`adoption.md`) |
| Onglets/accueil mal gérés au lecteur d'écran | Motifs APG décrits dans `components.md`, test manuel NVDA au lot 10 |
| Limite de 10 fichiers par dossier (`AGENTS.md`) | Nouveaux sous-dossiers (`ui/icon/`, `canvas/view-bar/`, `editor/welcome/`) ; plusieurs dossiers existants dépassent déjà la limite (`export/`) : ne pas aggraver |

### Questions pour le mainteneur

1. **Validation du DS** : Rose Atelier 0.1.0 est-il accepté comme direction (décision D38) ?
   Le lot 0 en dépend.
2. **Premier accès** : remplacer la modale imposée par l'accueil (recommandé, conforme à
   `journeys.md`) ? L'action secondaire « Importer un PNG » depuis l'accueil demanderait une
   création de document à partir de l'image, absente du moteur : on l'écarte (recommandé) ou on
   ouvre un chantier moteur séparé ?
3. **Aide sur desktop** : `/support` est `hostedOnly`. Masquer « Aide » sur desktop, ou pointer
   vers une page d'aide locale (raccourcis, documentation) ? Le DS signale le problème sans le
   trancher.
4. **Police** : TTF variable tel quel, ou sous-ensemble WOFF2 latin (production d'un fichier
   dérivé, outil de génération non présent dans le dépôt) ? Nunito aussi dans l'admin ?
5. **Marque** : le cœur pixel (`mark.svg`) remplace-t-il `favicon.ico` et les icônes Tauri ? Le DS
   demande une décision explicite ; par défaut, le plan ne l'utilise que dans l'en-tête.
6. **Admin** : lot 9 dans cette tâche, ou reporté ?
7. **Tokens** : faut-il aussi aligner `design-system/tokens/tokens.css` sur l'échelle
   d'espacement de production (5 = 1.5rem, 6 = 2rem), pour qu'un seul jeu de noms existe à terme ?
   Le plan ne contrôle que les couleurs.
8. **Catalogues** : 148 clés `design_system.*` de la référence sont dans `i18n/en.json` (et
   `fr.json`) et sont donc servies à l'application ; les garder là ou les déplacer dans un
   catalogue propre à `design-system/` (ce qui modifie `design-system/scripts/serve.mjs`) ?

### Points non établis (à vérifier au lot 0)

- Le défilement réel des pages sans `ion-content` et la hauteur réelle du canevas : déduits de la
  lecture du code, non observés (aucune application lancée pendant la planification).
- Couleurs du canevas : `canvas/render/canvas-renderer.ts` code en dur `CHECKER_LIGHT = '#ffffff'`,
  `CHECKER_DARK = '#cccccc'` et `GRID_COLOR = 'rgba(0, 0, 0, 0.25)'` (vérifié). Le DS prévoit
  `#ececee` pour le damier sombre et une grille `#68686f` : le lot 1 aligne ces constantes (elles
  restent des constantes nommées, neutres et indépendantes du thème), le fond autour de l'œuvre
  passant à `--lp-color-canvas-bg` via la scène. Ce qui reste à vérifier : que la grille reste
  lisible sur une œuvre sombre au zoom élevé.
- L'API exacte de `CurrentAnimation` (`library/current-animation.ts`) pour distinguer « nouveau
  travail » et « animation de la bibliothèque » dans la pastille d'enregistrement.
- La façon dont `SaveFlow` remonte aujourd'hui un échec d'écriture (dialogue, notification) avant
  d'y ajouter un état « échec ».

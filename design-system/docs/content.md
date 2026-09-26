# Voice, language and interface content

This document is an English specification with bilingual UI copy examples. New example keys are
suggestions for implementation; they are not installed translation catalogue entries. Reuse an
existing key when its meaning already matches. The preview may use its own isolated sample copy.

## Voice

Warm, plain and capable. Welcome people into making something small and expressive. Explain the
next action in familiar words and be exact about persistence, exports and security.

- Speak to the person directly: “Choose a colour” / “Choisissez une couleur”.
- Prefer verbs and concrete objects: Create animation, Save a copy, Download WASM.
- Use short sentences without hiding a consequence to sound friendly.
- Use sentence case for labels and headings; avoid all-caps instructions.
- Acknowledge an outcome once. Do not congratulate every click or make mistakes feel childish.
- Keep “Life Pixel” unchanged across languages. Use ordinary French typography and accents.
- Use polite plural French consistently. Avoid switching between “tu” and “vous”.
- Pip is decorative support; it does not speak in baby talk or replace labels with emotions.

Example: “Your animation is ready to export.” / “Votre animation est prête à être exportée.”
Avoid: “Oopsie! Your pixels got lost!” or vague errors such as “Something went wrong”.

## Vocabulary

| English | French | Meaning |
|---|---|---|
| Animation | Animation | The editable animated work |
| Project | Projet | A library container for animations |
| Library | Bibliothèque | Saved projects and animations |
| Frame | Image | One timed frame in an animation |
| Layer | Calque | One stacked editable drawing plane |
| Canvas | Toile | The drawing area; use dimensions in pixels |
| Palette | Palette | Indexed colours, including transparency |
| Onion skin | Pelure d’oignon | Neighbouring-frame guide while drawing |
| Tag | Séquence nommée | A named range; align with existing catalogue wording |
| Preview | Aperçu | Playback of the compiled animation |
| Save | Enregistrer | Persist an editable document to its library |
| Export | Exporter | Create an output file; does not save the document |
| Download | Télécharger | Transfer a prepared file to the device |
| Duplicate | Dupliquer | Create a separate copy of library work |
| Token | Jeton d’accès | Credential allowing an agent to use permitted actions |
| Local library | Bibliothèque locale | Files stored on the desktop device |

This is the proposed product glossary. During integration, compare every term with existing
catalogues, update related occurrences together, and avoid inconsistent mixed terminology.
Keep “WASM”, “PNG”, “GIF”, “APNG”, “MCP”, framework names and code identifiers unchanged.
Explain technical terms once when they first help a decision, not in every button label.

## Message structure

| Context | Formula |
|---|---|
| Welcome | What you can make + one next action |
| Empty | What is absent + how to add it |
| Error | What failed + what remains safe + next action |
| Destructive | Object + permanent consequence + exact confirming verb |
| Access | What needs identity + available alternative |
| Success | What actually completed + optional next step |
| Help | Action + brief reason or keyboard equivalent |

Prefer “Could not save. Your work is still open. Try again.” over “Save failed”. Never say work
is safe, backed up or stored unless the application has confirmed that specific state.

## Bilingual copy examples

Suggested keys follow existing feature namespaces. The short key in each entry is the full proposed
catalogue key. Preserve existing keys where equivalent text already exists.

### Welcome and creation

`editor.welcome.title`

- EN: A little world, one pixel at a time.
- FR: Un petit univers, un pixel à la fois.

`editor.welcome.description`

- EN: Draw your first frame, bring it to life, then export it.
- FR: Dessinez votre première image, animez-la, puis exportez-la.

`editor.welcome.create`

- EN: Create animation
- FR: Créer une animation

`editor.welcome.guest_notice`

- EN: You can draw and export without an account. Save to an account before leaving to keep editing.
- FR: Dessinez et exportez sans compte. Enregistrez dans un compte avant de partir pour continuer.

`editor.welcome.guest_loss`

- EN: Unsaved work is lost when this page closes or reloads.
- FR: Le travail non enregistré est perdu si cette page est fermée ou rechargée.

`editor.onboarding.draw_hint`

- EN: Choose a colour, then draw on the canvas.
- FR: Choisissez une couleur, puis dessinez sur la toile.

`editor.onboarding.animate_hint`

- EN: Duplicate this frame, change a few pixels, then press Play.
- FR: Dupliquez cette image, modifiez quelques pixels, puis lancez la lecture.

### Library and saving

`library.empty.title`

- EN: A home for your next animation.
- FR: Une place pour votre prochaine animation.

`library.empty.description`

- EN: Saved animations appear here, organised into projects.
- FR: Vos animations enregistrées apparaissent ici, regroupées par projet.

`library.search.no_results`

- EN: No animations match “{query}”. Try another name or clear the search.
- FR: Aucune animation ne correspond à « {query} ». Essayez un autre nom ou effacez la recherche.

`library.save.guest_explanation`

- EN: Create a free account to save this animation. Your work stays open while you sign in here.
- FR: Créez un compte gratuit pour enregistrer cette animation. Votre travail reste ouvert ici.

`library.save.unsaved`

- EN: Unsaved changes
- FR: Modifications non enregistrées

`library.save.pending`

- EN: Saving…
- FR: Enregistrement…

`library.save.confirmed`

- EN: Saved
- FR: Enregistré

`library.save.retry_message`

- EN: Could not save. Your work is still open. Try again or export a copy.
- FR: Enregistrement impossible. Votre travail reste ouvert. Réessayez ou exportez une copie.

`library.save.conflict_explanation`

- EN: A newer version was saved elsewhere. Save a copy to keep both versions.
- FR: Une version plus récente a été enregistrée ailleurs. Enregistrez une copie pour conserver
  les deux versions.

`library.save.reload_warning`

- EN: Reloading replaces your current edits with the saved version.
- FR: Recharger remplace vos modifications actuelles par la version enregistrée.

`library.save.overwrite_warning`

- EN: Overwriting replaces the saved version with your current work.
- FR: Écraser remplace la version enregistrée par votre travail actuel.

`library.save.quota_explanation`

- EN: There is not enough storage to save this change. You can still open, export and delete work.
- FR: L’espace manque pour enregistrer. Vous pouvez toujours ouvrir, exporter et supprimer vos
  animations.

Keep actual usage in a separate translated message with locale-formatted `{used}` and `{limit}`.
Never insert a hard-coded plan quota into this copy or present an unavailable Upgrade action.

### Import, export and connection

`import.palette_notice`

- EN: Imported colours are reduced to the animation’s indexed palette.
- FR: Les couleurs importées sont adaptées à la palette indexée de l’animation.

`export.ready_message`

- EN: Your animation is ready. Choose a format to download.
- FR: Votre animation est prête. Choisissez un format à télécharger.

`export.wasm.description`

- EN: A small animation file you can control from your app.
- FR: Un petit fichier d’animation que vous pouvez piloter depuis votre application.

`export.download_note`

- EN: Downloading creates export files. Use Save to keep the editable animation in your library.
- FR: Le téléchargement crée les fichiers exportés. Enregistrez aussi l’animation modifiable.

`export.snippet.path_hint`

- EN: Place the downloaded files in your project, then update the paths in this snippet.
- FR: Placez les fichiers téléchargés dans votre projet, puis adaptez les chemins de cet extrait.

`common.copy_success`

- EN: Copied
- FR: Copié

`common.copy_failure`

- EN: Could not copy. Select the text and copy it manually.
- FR: Copie impossible. Sélectionnez le texte et copiez-le manuellement.

`library.connection.unavailable`

- EN: The library could not be reached. Your current work is still open.
- FR: La bibliothèque est inaccessible. Votre travail actuel reste ouvert.

`settings.library_folder.unavailable`

- EN: This library folder is unavailable. Reconnect it or choose another folder.
- FR: Ce dossier est inaccessible. Reconnectez-le ou choisissez un autre dossier.

### Deletion, account and agents

`library.delete.confirmation`

- EN: Delete “{name}”? This removes the saved animation and cannot be undone here.
- FR: Supprimer « {name} » ? L’animation enregistrée sera supprimée sans annulation possible ici.

`account.verification.note`

- EN: Your email address is not verified yet. You can continue creating and saving.
- FR: Votre adresse e-mail n’est pas encore vérifiée. Vous pouvez continuer à créer et enregistrer.

`tokens.created.once_notice`

- EN: Copy this token now. You will not be able to view it again.
- FR: Copiez ce jeton maintenant. Vous ne pourrez plus l’afficher ensuite.

`tokens.revoke.consequence`

- EN: Agents using this token will lose access. Your saved animations will remain.
- FR: Les agents utilisant ce jeton perdront leur accès. Vos animations seront conservées.

### Administration

`admin.monitoring.stale_notice`

- EN: These values could not be refreshed. Last update: {time}.
- FR: Ces valeurs n’ont pas pu être actualisées. Dernière mise à jour : {time}.

`admin.support.reply_pending`

- EN: Sending reply…
- FR: Envoi de la réponse…

Do not use mascot speech, jokes or celebratory language in incidents, access failures, security,
quota, legal requests or destructive actions. Explain the real state without blaming the person.

## Internationalisation and formatting contract

- Production copy lives in `i18n/en.json` and `i18n/fr.json`, added together when implemented.
- Keep lowercase dot-separated keys, scoped by feature, meaningful and append-only across releases.
- Use Transloco at runtime; load only the active language. Never bundle catalogues into production.
- Use ICU for counts. Example: `{count, plural, one {# frame} other {# frames}}`.
- French equivalent: `{count, plural, one {# image} other {# images}}`.
- Use `Intl` for dates, durations where applicable, numbers and byte sizes; never concatenate units.
- Keep field units explicit. Display frame duration in milliseconds and dimensions in pixels.
- Treat variable names, file paths and titles as text, never as executable or unsanitised markup.
- Allow wrapping and flexible controls; reserve room for French expansion and long user titles.
- Visible text and accessible names must communicate the same action in the active language.
- Error codes stay stable in the API; translate their meaning and recovery in the interface.
- Legal content remains in the existing translated Markdown documents and follows its review path.

## Content review checklist

- Does the message distinguish save, export and download correctly?
- Does a visitor understand that closing/reloading loses unsaved work?
- Are proposed/future capabilities absent from production promises?
- Does the error identify a useful action while accurately stating what remains available?
- Are deletion and overwrite consequences explicit and free of decoration?
- Do English and French express the same meaning and use the agreed glossary?
- Are values real and locale-formatted rather than fixed examples or invented measurements?
- Are empty, loading, denied, offline and failure states distinct?
- Does every icon, field and status have a meaningful translated name?

Reference: [internationalisation](../../docs/i18n.md), [V1 scope](../../docs/v1/README.md),
[product decisions](../../docs/decisions.md), and [journeys](journeys.md).

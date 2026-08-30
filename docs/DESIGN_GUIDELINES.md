# Blindtest — Design Guidelines

Version 1. Single source of truth for the visual system.

A design looks generated when every screen re-decides the same questions: a new grey, a new radius,
a new shade of purple, one more border. This document makes those decisions once. **Anything not
listed here is not available.**

Rules are absolute. Where a value is given, use that value verbatim. Token source of truth is
`client/src/app.css` — change a value there, never at the call site.

---

## 0. What to remove first

Read from the current stylesheet. Each of these is a reason the app reads as generated. Delete
before adding anything new.

| Remove | Detail |
| --- | --- |
| Glassmorphism | All `--glass-*` tokens, `backdrop-filter`, `.glass-card`, the `::before`/`::after` highlight gradients, the fixed `--glass-gradient` body wash |
| The purple accent | `#5E5CE6` / `#7B79F7` is the default-template colour. Replaced by one amber |
| Borders as decoration | Every surface currently carries a 1px border *plus* a shadow *plus* a radius. Boxes stop meaning anything. Use rules and space |
| Four radii | `6/8/12/16px` → one value: `0` |
| Five semantic hues | green + red + blue + orange → two signal colours |
| Hover lift and brightness | `translateY(-2px)`, `scale(0.97)`, `filter: brightness(1.05)`. Nothing floats in this design |
| Inter and IBM Plex Mono | Two AI-default typefaces. One family for everything: Archivo |
| The theme toggle | The `[data-theme]` block and the `transition-[background,color]` on `body`. One theme ships |

---

## 1. Principles

1. **Structure, not containers.** Group by alignment, a 2px rule and generous space. Reach for a
   bordered card only when the content is genuinely a separate object, like a round in a list.
2. **One accent per screen.** The accent marks the single most important action, plus small emphasis
   like the live round number. Two accents means no primary.
3. **Flush left.** Headings, copy and button labels all start at the same left edge, including inside
   a wide button. Centre only the answer-reveal and the countdown — the deliberate exceptions.
4. **Zero corner radius.** Every element is square. Two exceptions, both circular: the avatar and the
   countdown ring.
5. **The media is the picture.** Artwork, stills and thumbnails run full-bleed and greyscale until
   revealed. The interface never adds imagery of its own — no illustrations, no gradients, no
   decorative shapes.
6. **If it isn't in this document, it doesn't exist.** No new colour, size, radius, shadow or
   animation gets introduced at a call site. Add it here first, as a token, or use what's here.

---

## 2. Colour — Sodium

One ground, and it is dark. Blindtest is played on a screen in a room with the lights down, so every
view — lobby, question, results, library, admin — sits on `#1C1A17` and the media is the brightest
thing present.

```css
--color-bg:         #1C1A17;  /* ground, every screen */
--color-surface:    #2A2620;  /* cards, rows, raised panels */
--color-text:       #F5F1E8;  /* body and headings */
--color-accent:     #FFB020;  /* primary fill, icons, large type */
--color-accent-ink: #FFD27A;  /* hover/pressed, and 13–15px accent text */
--color-divider:    rgba(245, 241, 232, 0.4);
```

**Greys come from one ramp only.**

```
#F5F1E8  #E7E1D4  #D3CCBC  #C4BCAC  #A39C8E  #857E70  #665F53  #3B362E  #2A2620
```

On the dark ground: secondary text `#C4BCAC`, dim text and placeholders `#A39C8E`, dividers
`#F5F1E8` at 40%, raised surfaces `#3B362E`. No other grey is legal.

**Accent runs light on dark.** Hover and pressed states step *lighter* to `#FFD27A`, never darker.
Type on an accent fill is `#1C1A17`, never white.

**Signal colours: two, and only in results.** Correct `#3FE08A`, wrong `#FFD27A`, each with its icon
and the word beside it. On a results screen the primary action steps down to secondary so nothing
competes with the two markers. A warning is a secondary button with clear copy, not a colour.

**No elevation by shadow.** A shadow does nothing on a dark ground. Dialogs and menus separate by a
lighter surface fill plus a 2px divider rule. Delete `--shadow-card`, `--shadow-lg`,
`--shadow-elevated`.

**Light ground exists only for print** — exported or printed score sheets: bg `#F3F2F2`, surface
`#EAE9E9`, text `#201E1D`, accent text `#8A5200`. It is not a mode a player can switch into.

---

## 3. Typography

Archivo. Weights 400, 600, 800. Six sizes, no others.

| Role | Size / weight | Notes |
| --- | --- | --- |
| Display | 64 / 800 / `-0.03em` | Round counters, reveal |
| Screen title | 32 / 800 / `-0.02em` | |
| Card & panel title | 20 / 800 | |
| Body | 15 / 400 | The default |
| Secondary | 13 / 400, `#C4BCAC` | Metadata |
| Label | 11 / 600 / `+0.1em`, uppercase | Section labels |

Line height 1.1 for headings, 1.55 for body. Never letterspace body copy; always letterspace the
11px label. Numbers that change live — scores, timers, round counts — use
`font-variant-numeric: tabular-nums` so they don't jitter. No second typeface: drop IBM Plex Mono and
use tabular Archivo.

---

## 4. Spacing & layout

**Scale: 4 · 8 · 12 · 16 · 24 · 32 · 56.** Nothing between. `py-[9px]`, `px-[13px]`, `mb-3.5` and
other off-scale values are the clearest tell of generated CSS — remove them all.

- **Page** — 12-column grid, 24px gutters, max width 1200px, 32px page margin (16px under 760px).
- **Sections** — separated by 56px of space and a 2px `#F5F1E8`/40% rule. Never by a card.
- **Stacking** — flex/grid with `gap`. No margin-bottom chains.
- **Question view** — media full-bleed, controls in a fixed 96px bottom bar. The player never scrolls.
- **Breakpoints** — 760px and 1024px. Two only.

---

## 5. Buttons & inputs

Three button variants, no fourth.

| Variant | Treatment |
| --- | --- |
| Primary | Solid `#FFB020` fill, `#1C1A17` label. One per screen |
| Secondary | 1px divider border, transparent fill, `#F5F1E8` label |
| Ghost | `#FFD27A` label, no box |

Label in heading weight, 14px, sentence case, verbs ("Start round", not "OK"). Padding 8px / 14px,
min height 36px, min hit target 44px on touch. Flush left. Hover lightens one ramp step to
`#FFD27A`; active lightens two. No scale, no lift, no brightness filter. Destructive actions are a
secondary button with `#FFD27A` text, and always confirm.

**One field style:** `#2A2620` fill, 1px divider border, square, 36px tall, 12px label above in
`#C4BCAC`. Focus turns the border accent — never a 3px glow ring. Error state: accent border plus one
line of 13px `#FFD27A` text below saying what to do.

---

## 6. Cards & lists

Rows of the same kind of thing — players, rounds, tracks — are a **ruled list**, not a grid of cards.
Cards are for a small set of distinct, clickable objects such as playlists.

- **Table** — header row 11px uppercase label with a 2px bottom rule; body rows 1px rule, 8px
  padding, hover tints the row with text at 6%. Numbers right-aligned and tabular.
- **Card** — `#2A2620` fill, square, 12px padding, **no border and no shadow**. Hover lifts the fill
  to `#3B362E`, never the position.

---

## 7. Navigation

One bar, 2px bottom rule, brand flush left, links in text colour at 14px, current page in accent, a
single primary action at the right end. No sidebar, no breadcrumbs, no icon-only tabs.

In game the bar is replaced by a single line: round counter left, exit right. Nothing else competes
with the media.

---

## 8. Icons

Lucide only ([lucide.dev](https://lucide.dev)), 16px in buttons and 22px standalone, stroke width 2,
`currentColor`, never filled and never coloured except by inheritance. **No emoji anywhere**,
including empty states and toasts. An icon alone is only allowed for play/pause, mute and close —
every other action carries a word.

---

## 9. Motion

| Use | Duration | Easing |
| --- | --- | --- |
| Hover, focus, fill change | 120ms | `ease-out` |
| Panel, dialog, route change | 200ms | `cubic-bezier(.4,0,.2,1)` |
| Answer reveal, score tick | 320ms | `cubic-bezier(.4,0,.2,1)` |
| Countdown ring | real time | `linear` |

Animate opacity and transform only. No spring easing, no pulse loops, no bouncing scale — the reveal
is the only moment that gets a movement, and it is a 12px rise with a fade. Timing-critical elements
never animate: the countdown and the audio waveform are driven by real playback time. Keep the
`prefers-reduced-motion` block already in the stylesheet.

---

## 10. Empty & error states

Same skeleton every time: a 20px heading naming the situation, one line of 13px explanation, one
action. Flush left, no centred column, no illustration, no emoji.

Error copy says what happened and what the player can do — never a status code, never "Oops".
Loading is a 2px accent progress line at the top of the region, not a spinner in the middle of an
empty page.

```
No playlists yet
Add audio or video clips and Blindtest will build rounds from them.
[Upload clips]

Connection to the room dropped
Your score is saved. Rejoining keeps you in the same round.
[Rejoin] [Leave game]
```

---

## 11. Accessibility

- **Contrast** — body text at least 4.5:1, so `#A39C8E` is the darkest grey allowed on text. Accent
  text at 13–15px must be `#FFD27A`.
- **Focus** — `2px solid #FFB020` with 2px offset on every interactive element. Never
  `outline: none` without a replacement.
- **Never colour alone** — correct and wrong answers carry a check or cross and a word.
- **Audio rounds** — always show elapsed and total time as text, and give the host a visible
  transcript or title field, so a deaf player can still follow the game.
- **Keyboard** — space toggles playback, enter submits a guess, escape closes. Announce round changes
  and results in an `aria-live="polite"` region.
- **Targets** — 44px minimum on touch, 8px minimum between adjacent controls.

---

## 12. Migration order

One pass through `client/src/app.css` first, then the components.

1. Replace the `:root` token block with the values in sections 2–4, keeping the variable names so
   nothing else breaks. Delete the `[data-theme="dark"]` block, the theme toggle and the
   `transition-[background,color]` on `body` — there is one theme now.
2. Set all radius tokens to `0`; delete `--radius-xl`.
3. Delete every `--glass-*` token, `.glass-card`, `backdrop-filter` and the body `background-image`.
4. Delete `--shadow-card`, `--shadow-lg`, `--shadow-elevated`; replace each use with a `#2A2620` or
   `#3B362E` fill.
5. Delete `--easing-spring`, `@keyframes pulseBtn`, `.btn-pulse`, and the `transform: scale` /
   `filter: brightness` lines in every button and card rule.
6. Swap Inter and IBM Plex Mono for Archivo 400/600/800; put `tabular-nums` on scores, timers and
   counters.
7. Collapse `.btn-danger` and `.btn-warning` into `.btn-secondary` with `#FFD27A` text.
8. Grep components for off-scale spacing (`py-[9px]`, `px-[13px]`, `mb-3.5`, `px-7`) and round each
   to the nearest step.
9. Remove `border` from `.card`, `.toggle` and `.btn-circle`; keep it only on inputs.

---

Grounded in `client/src/app.css` at `main`. When a rule here and the code disagree, the code is
wrong.

---
name: Spaceport landing page
description: One 1970s sci-fi paperback, from orange front cover to copyright page.
colors:
  flame: "#f4741c"
  flame-deep: "#d95a0c"
  navy: "#17132e"
  navy-soft: "#262048"
  cream: "#f6ecd6"
  paper: "#f1e7d0"
  ink: "#1d1830"
  red: "#c4230f"
  cyan: "#2fe0ee"
  violet: "#a98bff"
  on-navy-2: "#cfc4d6"
  on-paper-2: "#554a5e"
typography:
  display:
    fontFamily: "Michroma, sans-serif"
    fontSize: "clamp(2.2rem, 10.4vw, 14rem)"
    fontWeight: 400
    lineHeight: 0.95
    letterSpacing: "-0.03em"
  headline:
    fontFamily: "Spectral, serif"
    fontSize: "clamp(2rem, 4.6vw, 4.4rem)"
    fontWeight: 600
    lineHeight: 1.08
    letterSpacing: "-0.015em"
  title:
    fontFamily: "Michroma, sans-serif"
    fontSize: "clamp(1.5rem, 3.4vw, 2.9rem)"
    fontWeight: 400
    lineHeight: 1.1
    letterSpacing: "-0.02em"
  lede:
    fontFamily: "Spectral, serif"
    fontSize: "clamp(1.25rem, 1.9vw, 1.6rem)"
    fontWeight: 400
    lineHeight: 1.35
  body:
    fontFamily: "Spectral, serif"
    fontSize: "1.125rem"
    fontWeight: 400
    lineHeight: 1.55
  label:
    fontFamily: "Barlow Condensed, sans-serif"
    fontSize: "1.05rem"
    fontWeight: 500
    letterSpacing: "0.08em"
  label-strong:
    fontFamily: "Barlow Condensed, sans-serif"
    fontSize: "1.5rem"
    fontWeight: 700
    lineHeight: 1.15
    letterSpacing: "0.05em"
  mono:
    fontFamily: "ui-monospace, SF Mono, Menlo, Consolas, monospace"
    fontSize: "1rem"
    fontWeight: 400
    lineHeight: 1.8
rounded:
  focus: "2px"
  stamp: "4px"
  button: "10px"
  block: "12px"
  window: "14px"
  round: "50%"
spacing:
  gutter: "clamp(1rem, 4vw, 3.5rem)"
  spine: "3rem"
  band-height: "3rem"
  cover-section: "clamp(4rem, 9vw, 8rem)"
  page-section: "clamp(3.5rem, 8vw, 7rem)"
components:
  band:
    backgroundColor: "{colors.navy}"
    textColor: "{colors.cream}"
    typography: "{typography.label}"
    height: "3rem"
    padding: "0.4rem clamp(1rem, 4vw, 3.5rem)"
  sticker:
    backgroundColor: "{colors.red}"
    textColor: "{colors.cream}"
    typography: "{typography.label-strong}"
    rounded: "{rounded.round}"
    padding: "1rem"
    size: "10.5rem"
  button-primary:
    backgroundColor: "{colors.red}"
    textColor: "{colors.cream}"
    typography: "{typography.label-strong}"
    rounded: "{rounded.button}"
    padding: "1rem 1.6rem"
  button-primary-hover:
    backgroundColor: "#a81c0a"
  stamp:
    textColor: "{colors.red}"
    typography: "{typography.label}"
    rounded: "{rounded.stamp}"
    padding: "0.1rem 0.6rem"
  code-excerpt:
    backgroundColor: "{colors.navy-soft}"
    textColor: "#e6e0f5"
    typography: "{typography.mono}"
    rounded: "{rounded.block}"
    padding: "1.4rem 1.6rem"
  window-plate:
    backgroundColor: "#0f0c22"
    textColor: "#e6e0f5"
    typography: "{typography.mono}"
    rounded: "{rounded.window}"
---

# Design System: Spaceport landing page

This document covers one surface: the public landing page under `landing/public/`.
It does not cover the native desktop app under `crates/`. The app has its own UI and its own rules.

## Overview

**Creative North Star: "The Seventies Paperback"**

The page is one 1970s sci-fi paperback. The visitor scrolls from the front cover to the copyright page. Each section is a part of the book: an orange front cover, a navy back cover, cream interior pages, and a colophon.

The mood is loud on the cover and calm inside. The cover uses flat cover stock, one painting, and one round red sticker. The inside uses ruled lists and plain serif text. Print parts carry the structure: a publisher band, a spine, an edition list, blurbs, a ledger, and a stamp.

The page is flat color, not gradients or glass. Depth comes from objects that sit on the cover, such as the painting and the sticker. The page has one theme. It does not switch for dark mode.

**Key Characteristics:**
- One book, read top to bottom: cover, back cover, pages, colophon.
- Three surfaces: flame orange, nebula navy, cream paper.
- Three type voices: wide Michroma titles, Barlow Condensed labels, Spectral text.
- Red is for the download action and for marks stamped on paper.
- Rules and bands divide content. Cards do not.
- Motion is rare: one sticker slap, one typed command, one caret.

## Colors

The palette is three flat print surfaces, one red for action, and two screen colors kept inside the illustrated window.

### Primary
- **Flame** (`flame`): The front cover stock. Also the accent on navy: nav hover, blurb heads, the first line of the back cover headline, the imprint tag.
- **Flame Deep** (`flame-deep`): The spine strip on the cover at 900px and wider. Not used elsewhere.

### Secondary
- **Rocket Red** (`red`): The download sticker and the download button. On paper it also marks the Spaceport ledger head, the step numbers, the stamps, and the text selection.

### Tertiary
- **Terminal Cyan** (`cyan`): Screen light. The focus ring, the caret, the prompt, the busy dot, and inline code on navy.
- **Nebula Violet** (`violet`): The branch name in the illustrated terminal. Used once.

### Neutral
- **Nebula Navy** (`navy`): The body background, the publisher band, the back cover, and all ink on the orange cover.
- **Navy Soft** (`navy-soft`): The code excerpt block on the back cover.
- **Cream** (`cream`): Text on navy and on red.
- **Paper** (`paper`): The interior pages and the colophon.
- **Ink** (`ink`): Text and heavy rules on paper. Thin rules use ink at 28% opacity.
- **Faded Lilac** (`on-navy-2`): Captions on navy.
- **Faded Plum** (`on-paper-2`): Fine print and colophon text on paper.

The illustrated window has its own darker violet shades, such as `#0f0c22` and `#1c1838`. They are written as literals in `site.css` and belong to that one figure. They are not page tokens.

### Named Rules
**The Red Means Download Rule.** A filled red shape is a download control. Elsewhere red appears only as a mark on paper: a stamp outline, a step number, one ledger head.

**The Screen Light Rule.** Cyan and violet are light from a screen. Keep them inside the illustrated window and code. The one exception is the cyan focus ring.

**The Three Stocks Rule.** Every section sits on flame, navy, or paper. Do not add a fourth surface.

## Typography

**Display Font:** Michroma (with sans-serif)
**Body Font:** Spectral (with serif)
**Label/Mono Font:** Barlow Condensed (with sans-serif) for labels; the system monospace stack for code.

**Character:** Michroma is the wide, technical cover title. Barlow Condensed is the publisher's voice on bands, codes, and buttons. Spectral is the book text. All three fonts are self-hosted as woff2 files.

### Hierarchy
- **Display** (400, `clamp(2.2rem, 10.4vw, 14rem)`, 0.95): The word SPACEPORT across the full cover width. Uppercase, one line, tracking -0.03em.
- **Headline** (Spectral 600, `clamp(2rem, 4.6vw, 4.4rem)`, 1.08): The back cover claim. Max 19ch, balanced. Its first line is flame.
- **Title** (Michroma 400, `clamp(1.5rem, 3.4vw, 2.9rem)`, 1.1): Section heads on paper pages. Uppercase.
- **Lede** (Spectral 400, `clamp(1.25rem, 1.9vw, 1.6rem)`, 1.35): One short paragraph under a title.
- **Body** (Spectral 400, 1.125rem, 1.55): Blurbs and list text. Blurbs hold to 34ch. Italic marks asides, captions, and fine print.
- **Label** (Barlow Condensed 500 or 700, 0.95rem to 1.7rem, 0.04em to 0.12em, uppercase): Band, nav, edition list, byline, blurb heads, ledger heads, stamps, buttons. Smaller labels get wider tracking.
- **Mono** (system monospace, 1rem, 1.8): Commands and inline code.

### Named Rules
**The One Title Face Rule.** Michroma sets titles and step numbers only. It never sets a sentence.

**The Labels Shout Rule.** Barlow Condensed is always uppercase and always tracked. Spectral is never uppercase.

## Layout

The page is full-bleed bands of color. Each band has side padding of one gutter (`clamp(1rem, 4vw, 3.5rem)`).

The publisher band is sticky at the top and 3rem tall. The cover fills the rest of the first viewport (`100svh - 3rem`). Its title runs the full width. Below it, three columns hold the pitch, the painting, and the edition list. A ruled byline closes the cover.

The back cover is left-aligned and wide. The window figure stops at 90rem. Below it, blurbs flow in two text columns of at least 17rem, with the command excerpt in a column beside them from 900px up.

Paper pages start on the cover's left edge and share the back cover's 90rem measure. Lists divide with rules: a 3px ink rule on top, then 1px rules between rows.

Responsive changes:
- **900px and wider:** The spine strip appears (3rem) and the cover indents by its width. The cover row becomes three columns.
- **Below 900px:** The cover stacks. The nav keeps only Install and GitHub. The edition list becomes two columns. The window drops its sidebar.
- **640px and wider:** The ledger and the install block become two columns.
- **560px and narrower:** The imprint tag hides. The window stacks its terminal and chat.

The window figure scales its type with its own width (container query units), so it stays one picture at any size.

## Elevation & Depth

Surfaces are flat. Shadows belong to objects that sit on a surface, not to the surface itself. All shadows are soft and tinted toward the stock they sit on. Sections do not cast shadows on each other.

### Shadow Vocabulary
- **Sticker** (`box-shadow: 0 0.5rem 1.1rem rgb(40 8 0 / 0.45)`): The sticker at rest. On hover it lifts to `0 0.9rem 1.6rem rgb(40 8 0 / 0.5)`.
- **Button** (`box-shadow: 0 0.4rem 0.9rem rgb(60 10 0 / 0.3)`): The download button on paper.
- **Window** (`box-shadow: 0 2.2rem 4rem rgb(0 0 0 / 0.5)`): The illustrated window on navy.

### Named Rules
**The Objects Only Rule.** A shadow means a thing was placed on the cover or page. Text blocks, lists, and sections stay flat.

## Shapes

Print shapes are square and ruled. Bands, pages, and lists have no radius. Rules are solid ink: 3px for a section top, 2px for edition rows, 1px between list rows.

Round shapes are for stuck-on objects. The sticker is a full circle with a dashed inner ring (2px, cream at 60%). Screen objects get soft corners: the window (14px), the code excerpt (12px), the button (10px). Small marks get tight corners: the stamp and inline code (4px).

Stuck-on objects sit at a tilt. The sticker rests at -9 degrees. Stamps are printed marks and sit flat.

## Components

### Buttons
Loud and physical. Both download controls are red with cream Barlow Condensed 700 text.
- **Sticker:** A red circle (10.5rem, 9.5rem to 12rem by breakpoint) that overlaps the lower-left corner of the cover painting. Two lines: a main line (1.55rem) and a sub line (1rem). It slaps on at load (0.7s, from -26 degrees and 1.5 scale).
- **Sticker hover / active:** Straightens to -3 degrees and scales to 1.06. Active presses to 0.98. Transitions run 0.35s on `cubic-bezier(0.16, 1, 0.3, 1)`.
- **Primary button:** A red block (10px radius, `1rem 1.6rem` padding, 1.5rem text) on paper pages.
- **Primary hover / active:** Lifts 2px and darkens to `#a81c0a`. Active sinks 1px.
- **Focus:** Every focusable element gets a 3px cyan outline with a 3px offset.

### Navigation
- **Publisher band:** Sticky navy bar, 3rem tall. Cream Barlow Condensed 500, uppercase, 0.08em tracking. The imprint sits left in 700 weight with a flame edition tag. Section links sit right.
- **States:** Links have no underline. Hover turns them flame.
- **Mobile:** Below 900px only Install and GitHub remain.

### Cover painting
A square crop of the rocket icon, taken inside its rounded edge, printed flat in a 4px navy keyline. It casts no shadow and has no caption. The sticker overlaps it.

### Edition list
A definition list in Barlow Condensed. Each row has a 2px navy top rule, a small tracked term (0.95rem, 500), and a large value (1.45rem, 700).

### Spine
A 3rem flame-deep strip on the left edge of the cover at 900px and wider. Vertical Barlow Condensed 700, 0.22em tracking. It is decoration and hidden from assistive tech.

### Window plate
An illustration of the app, built from HTML. Dark violet window, 14px corners, deep shadow. It holds a title bar, a sidebar with status dots, tabs, a terminal, and a chat. A caption below says it is an illustration. When half the figure is in view, the last command types out once (1.6s) and the caret blinks.

### Blurbs
Back-cover copy in text columns. Each blurb opens with a flame Barlow Condensed 700 head (1.5rem) on its own line, then Spectral text. Blurbs have no boxes, icons, or borders.

### Code excerpt
A navy-soft block (12px radius, `1.4rem 1.6rem` padding) with mono text at 1.8 line height. An italic caption below links to the source.

### Ledger
Two ruled columns under a 3px ink rule. Each has a Barlow Condensed 700 head (1.7rem). The Spaceport head is red. Rows divide with 1px rules. Italic fine print follows.

### Install steps
A numbered list under a 3px ink rule. Numbers are red Michroma (1.5rem) in a 3.2rem column. Each step leads with a bold phrase.

### Stamp
A small outlined mark for status: 2px red border, 4px radius, red Barlow Condensed 700, 0.12em tracking, flat. Stamps sit in a fixed 8.6rem column so a list of them lines up.

### Colophon
The copyright page. Centered Spectral at 0.98rem in faded plum, 54ch wide, under a 1px rule.

## Do's and Don'ts

### Do:
- **Do** treat each new section as a part of the book: cover, back cover, page, or colophon.
- **Do** put each section on flame, navy, or paper.
- **Do** divide lists with ink rules: 3px on top, 1px between rows.
- **Do** set every label in uppercase Barlow Condensed with tracking.
- **Do** keep red fills for download controls.
- **Do** tilt stuck-on objects and give them a soft, warm shadow.
- **Do** label the window plate as an illustration.
- **Do** honor `prefers-reduced-motion`: the slap, the typing, the caret, and the hover transitions all stop.

### Don't:
- **Don't** apply these rules to the native app under `crates/`.
- **Don't** set sentences in Michroma.
- **Don't** use cyan or violet outside the window, code, and the focus ring.
- **Don't** add shadows to sections, lists, or text blocks.
- **Don't** wrap blurbs or ledger rows in cards.
- **Don't** add a second theme or a dark mode switch. The book has one printing.

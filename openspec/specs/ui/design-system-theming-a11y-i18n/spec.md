# Design system, theming, a11y, i18n Specification

## Purpose
Color/type/spacing tokens, light/dark/high-contrast, vector icons, AccessKit, Fluent i18n (VI/EN first). Module `UI-08` (Design system, theming, a11y, i18n; Vietnamese: "Design system, theme, a11y, i18n") from BRD-01, phase P1, priority Must. Owner crates: virtax-ui-kit. Depends on: none. VirtualBox reference: `src/VBox/Frontends/VirtualBox/images (≈4,000 PNG), nls (40 .ts), globals/UIAccessible*`. Implementation status (2026-11-24): not started.

## Requirements

### Requirement: Design tokens as the single source of truth (FR-UI08-001)
Virtax SHALL provide "Design tokens as the single source of truth" as specified by FR-UI08-001 (BRD-01, priority Must, phase P1; Vietnamese: "Design token là nguồn sự thật duy nhất").

#### Scenario: Specified behaviour of FR-UI08-001
- **WHEN** the behaviour FR-UI08-001 covers is exercised
- **THEN** it behaves as BRD-01 specifies: Colors, type, spacing (4 px grid: 2/4/8/12/16/24/32), radii (6/8/12), shadows and animation durations are tokens in `tokens.json` that generate the Slint `Theme` global for the Manager and the Console chrome. Semantic tokens: bg, surface, card, fg, fg-muted, accent, focus, warn, danger, paused, off, chart-1…4. No hard-coded colors in `.slint` files.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/globals/UICommon.cpp:464-600; UIDetailsElement.cpp:1482-1501`

#### Scenario: Acceptance for FR-UI08-001
- **WHEN** the FR-UI08-001 acceptance check is run
- **THEN** A CI lint finds 0 color literals outside the token file; changing one token updates every screen without code changes.

### Requirement: Dark, light and high-contrast on all 3 OSes (FR-UI08-002)
Virtax SHALL provide "Dark, light and high-contrast on all 3 OSes" as specified by FR-UI08-002 (BRD-01, priority Must, phase P1; Vietnamese: "Dark, light, high-contrast trên cả 3 OS").

#### Scenario: Specified behaviour of FR-UI08-002
- **WHEN** the behaviour FR-UI08-002 covers is exercised
- **THEN** it behaves as BRD-01 specifies: Three themes: Dark (default, bg #0F172A, accent #22C55E, focus #60A5FA), Light and High-contrast; Auto follows the OS on Windows, macOS and Linux (the `org.freedesktop.appearance color-scheme` portal). High-contrast on Windows uses the Contrast theme's system colors. Themes switch live without a restart.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/settings/UIAdvancedSettingsDialogSpecific.cpp:266 (Windows only)`

#### Scenario: Acceptance for FR-UI08-002
- **WHEN** the FR-UI08-002 acceptance check is run
- **THEN** Changing the OS theme switches the app in ≤ 500 ms; changing it in Preferences takes ≤ 200 ms with no flash; the theme picker exists on all 3 OSes.

### Requirement: Contrast, focus rings and never color alone (FR-UI08-003)
Virtax SHALL provide "Contrast, focus rings and never color alone" as specified by FR-UI08-003 (BRD-01, priority Must, phase P1; Vietnamese: "Contrast, focus ring và không chỉ dựa vào màu").

#### Scenario: Specified behaviour of FR-UI08-003
- **WHEN** the behaviour FR-UI08-003 covers is exercised
- **THEN** it behaves as BRD-01 specifies: Every text/background pair is ≥ 4.5:1 (≥ 3:1 for large text and control icons) in all 3 themes; a 2 px focus-color ring is always visible during keyboard navigation; state always pairs color with an icon/text; targets are ≥ 32 px, dense icon buttons at least 24 px (WCAG 2.2 criterion 2.5.8).
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/manager/details/UIDetailsElement.cpp:1482-1501 (darker/lighter)`

#### Scenario: Acceptance for FR-UI08-003
- **WHEN** the FR-UI08-003 acceptance check is run
- **THEN** A token contrast script in CI passes 100% of pairs; a WCAG 2.2 AA audit of the 25 main screens finds 0 A/AA issues.

### Requirement: Lucide vector icon set (FR-UI08-004)
Virtax SHALL provide "Lucide vector icon set" as specified by FR-UI08-004 (BRD-01, priority Must, phase P1; Vietnamese: "Bộ icon vector Lucide").

#### Scenario: Specified behaviour of FR-UI08-004
- **WHEN** the behaviour FR-UI08-004 covers is exercised
- **THEN** it behaves as BRD-01 specifies: Replace ≈4,000 PNGs with Lucide (ISC) vector icons at 16/20/24 px with 1.5-1.75 px strokes, crisp at every scale from 100 to 300% (including 125/150/175%); OS tiles are SVG or monograms; no VirtualBox/Oracle artwork and no emoji icons.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/images (≈4,000 PNG, 31 MB, 0 SVG)`

#### Scenario: Acceptance for FR-UI08-004
- **WHEN** the FR-UI08-004 acceptance check is run
- **THEN** Screenshots at 150% and 175% show 0 blurred icons against the vector reference render; total icon payload ≤ 1 MB.

### Requirement: Accessibility semantics through AccessKit (FR-UI08-005)
Virtax SHALL provide "Accessibility semantics through AccessKit" as specified by FR-UI08-005 (BRD-01, priority Must, phase P1; Vietnamese: "Ngữ nghĩa accessibility qua AccessKit").

#### Scenario: Specified behaviour of FR-UI08-005
- **WHEN** the behaviour FR-UI08-005 covers is exercised
- **THEN** it behaves as BRD-01 specifies: Every `virtax-ui-kit` component declares role, name, value, state and actions through AccessKit (UIA, NSAccessibility, AT-SPI). The VM list is a list with position/size, the snapshot tree a tree; Operation progress and toasts are live regions. Focus: Tab order follows layout, sheets/dialogs trap focus and return it to the opener, Esc closes the top layer.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/globals/UIAccessible.cpp; extensions/QITreeView.cpp:603`

#### Scenario: Acceptance for FR-UI08-005
- **WHEN** the FR-UI08-005 acceptance check is run
- **THEN** The 10 main flows complete with NVDA, VoiceOver and Orca; a CI accessibility-tree dump finds 0 unnamed controls; opening/closing Settings 50 times always returns focus to the Edit button.

### Requirement: Complete Fluent VI/EN i18n with a pseudo-locale (FR-UI08-006)
Virtax SHALL provide "Complete Fluent VI/EN i18n with a pseudo-locale" as specified by FR-UI08-006 (BRD-01, priority Must, phase P1; Vietnamese: "Fluent i18n VI/EN đầy đủ, có pseudo-locale").

#### Scenario: Specified behaviour of FR-UI08-006
- **WHEN** the behaviour FR-UI08-006 covers is exercised
- **THEN** it behaves as BRD-01 specifies: All UI strings, errors and tooltips live in Fluent files (`locales/vi/*.ftl`, `locales/en/*.ftl`) with plural/select; no HTML inside strings (links and bold are components). VI and EN are complete in P1, follow the OS locale and switch live; the bundle is shared with `virtaxctl` and `VTX-*` messages. A pseudo-locale (fake accents, +35% length) checks for truncation.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/nls (40 .ts, no vi); manager/UIHomePane.cpp:88-99`

#### Scenario: Acceptance for FR-UI08-006
- **WHEN** the FR-UI08-006 acceptance check is run
- **THEN** CI finds 0 missing/extra keys between vi and en and 0 HTML tags in `.ftl`; switching language updates every window in ≤ 1 s; under the pseudo-locale the 25 main screens show 0 truncations.

### Requirement: Locale-aware formatting (FR-UI08-007)
Virtax SHALL provide "Locale-aware formatting" as specified by FR-UI08-007 (BRD-01, priority Should, phase P1; Vietnamese: "Định dạng theo locale").

#### Scenario: Specified behaviour of FR-UI08-007
- **WHEN** the behaviour FR-UI08-007 covers is exercised
- **THEN** it behaves as BRD-01 specifies: Numbers, sizes (binary GiB/MiB with the locale decimal separator: "8,5 GiB" / "8.5 GiB"), dates, relative times ("3 days ago") and durations go through ICU4X; technical values (MACs, paths, UUIDs) are never translated.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/globals/UITranslator.cpp (formatSize)`

#### Scenario: Acceptance for FR-UI08-007
- **WHEN** the FR-UI08-007 acceptance check is run
- **THEN** VI/EN snapshot tests for 30 sample values match expectations; changing locale never changes values sent to the API.

### Requirement: Community languages and RTL readiness (FR-UI08-008)
Virtax SHALL provide "Community languages and RTL readiness" as specified by FR-UI08-008 (BRD-01, priority Could, phase P2; Vietnamese: "Ngôn ngữ cộng đồng và sẵn sàng RTL").

#### Scenario: Specified behaviour of FR-UI08-008
- **WHEN** the behaviour FR-UI08-008 covers is exercised
- **THEN** it behaves as BRD-01 specifies: A `.ftl`-based translation workflow (Weblate/Pontoon compatible); community languages ship once they reach ≥ 90% of keys; layouts are checked with a pseudo-RTL locale before accepting RTL languages.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/nls/ApprovedLanguages.kmk`

#### Scenario: Acceptance for FR-UI08-008
- **WHEN** the FR-UI08-008 acceptance check is run
- **THEN** Adding a language only needs a `locales/<lang>` folder and a PR, no code change; under pseudo-RTL the 25 main screens mirror correctly.

### Requirement: Fonts, text scaling and per-monitor HiDPI (FR-UI08-009)
Virtax SHALL provide "Fonts, text scaling and per-monitor HiDPI" as specified by FR-UI08-009 (BRD-01, priority Must, phase P1; Vietnamese: "Font, co giãn chữ và HiDPI theo màn hình").

#### Scenario: Specified behaviour of FR-UI08-009
- **WHEN** the behaviour FR-UI08-009 covers is exercised
- **THEN** it behaves as BRD-01 specifies: Bundle IBM Plex Sans (13 px UI) and JetBrains Mono (technical values) under the OFL with full Vietnamese diacritics; honor the OS text size; app zoom 80-200%. Each window uses the DPR of its current monitor and re-renders within 1 frame when moved; no global scale variable like `QT_SCALE_FACTOR`.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/settings/global/UIGlobalSettingsDisplay.cpp; main.cpp:497-504`

#### Scenario: Acceptance for FR-UI08-009
- **WHEN** the FR-UI08-009 acceptance check is run
- **THEN** At 200% zoom the 25 main screens show no clipping or horizontal scroll; "ỗ ự ẫ" render on all 3 OSes; moving between 100% and 200% monitors is crisp from the next frame.

### Requirement: Motion budget and reduced motion (FR-UI08-010)
Virtax SHALL provide "Motion budget and reduced motion" as specified by FR-UI08-010 (BRD-01, priority Must, phase P1; Vietnamese: "Ngân sách chuyển động và reduce motion").

#### Scenario: Specified behaviour of FR-UI08-010
- **WHEN** the behaviour FR-UI08-010 covers is exercised
- **THEN** it behaves as BRD-01 specifies: Transitions run 150-250 ms using only opacity/transform with no layout shift; the OS "reduce motion" setting makes them instant; nothing animates continuously while idle (spinners only while work runs), so an idle Manager renders no frames.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/manager/details/UIDetailsElement.cpp:1467-1510 (animation)`

#### Scenario: Acceptance for FR-UI08-010
- **WHEN** the FR-UI08-010 acceptance check is run
- **THEN** An idle Manager over 60 s renders 0 frames (renderer counter) with ≤ 5 wakeups/s; with reduce motion on, 0 animations exceed 50 ms.

### Requirement: Component library and gallery (FR-UI08-011)
Virtax SHALL provide "Component library and gallery" as specified by FR-UI08-011 (BRD-01, priority Must, phase P1; Vietnamese: "Thư viện component và gallery").

#### Scenario: Specified behaviour of FR-UI08-011
- **WHEN** the behaviour FR-UI08-011 covers is exercised
- **THEN** it behaves as BRD-01 specifies: `virtax-ui-kit` provides Button, IconButton, Chip, Badge (Live/Next boot/Power off), Card, Sheet, Tabs, VirtualList, Tree, Table, Toast, ProgressRow, Sparkline, Field (label + inline error + Fix), ComboBox, Slider with a numeric input and KeyCap. Every component has hover/focus/disabled/error states, a11y and a runnable gallery example (`cargo run -p virtax-ui-kit --example gallery`).
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/extensions (QI*, 13.5k LOC); widgets (22.6k LOC)`

#### Scenario: Acceptance for FR-UI08-011
- **WHEN** the FR-UI08-011 acceptance check is run
- **THEN** 100% of components have a gallery example and screenshot tests in 3 themes; Tree and VirtualList handle 10,000 nodes at 60 fps.

### Requirement: Software renderer fallback (FR-UI08-012)
Virtax SHALL provide "Software renderer fallback" as specified by FR-UI08-012 (BRD-01, priority Should, phase P1; Vietnamese: "Renderer software dự phòng").

#### Scenario: Specified behaviour of FR-UI08-012
- **WHEN** the behaviour FR-UI08-012 covers is exercised
- **THEN** it behaves as BRD-01 specifies: Without a usable GPU (RDP, nested VMs, broken drivers) the Manager switches to Slint's software renderer automatically; Preferences and a command-line flag can force a renderer; functionality is unchanged.
- **AND** the VirtualBox reference for the current behaviour is `new (Slint software renderer)`

#### Scenario: Acceptance for FR-UI08-012
- **WHEN** the FR-UI08-012 acceptance check is run
- **THEN** In a GPU-less Windows RDP session and in a Linux VM without 3D, the Manager opens and scrolls 500 VMs at p95 frame ≤ 33 ms.

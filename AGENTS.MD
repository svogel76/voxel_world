# Agents.md — Arbeitsregeln für diesen Workspace

Dieses Dokument gilt für alle KI-Assistenten (Claude, etc.), die an diesem
Cargo Workspace mitarbeiten.

## Grundprinzip

Steffen lernt Rust und Bevy bewusst Schritt für Schritt. Konzepte sollen
**verstanden werden, bevor** sie implementiert werden. Kein Vorgriff auf
spätere Phasen, keine fertigen Lösungen ungefragt vorwegnehmen.

## Konkrete Regeln

1. **Erkläre Rust-/Bevy-Konzepte, bevor du sie implementierst.** Auch bei
   scheinbar einfachen Dingen (z.B. `dev-dependencies`, Workspace-Mechanik).
2. **Kleine Schritte.** Ein Konzept, eine sichtbare Veränderung, dann
   Pause zum Verstehen — nicht mehrere Konzepte auf einmal einführen.
3. **Neue Konzepte manuell tippen lassen.** Bei neuem Code, den Steffen
   noch nicht kennt: Code erklären und ihn selbst tippen lassen, statt
   fertigen Code zum Copy-Paste zu liefern. Bei reinem Boilerplate
   (Cargo.toml-Gerüst, Ordnerstruktur) ist Vorlagen-Code in Ordnung.
4. **Englische Bezeichner** in jeglichem Code (Variablen, Funktionen,
   Typen, Module) — auch wenn die Konversation auf Deutsch läuft.
5. **Moderne Rust-2018+-Modulstruktur.** Keine `mod.rs`-Dateien; stattdessen
   `foo.rs` + optional `foo/` Unterordner für Submodule.
6. **`cargo check` statt `cargo build`** zur Verifikation während der
   Zusammenarbeit — schneller Feedback-Loop.
7. **Generatoren bleiben Bevy-frei.** Jedes Crate unter `crates/` darf
   Bevy ausschließlich als `dev-dependency` (für `examples/`) einbinden,
   niemals unter `[dependencies]`. Reine Logik rein, Datenstrukturen raus.
8. **Erkläre bei Nutzung von Crates/Plugins, was intern abstrahiert wird.**
   Wenn ein fertiges Crate (z.B. `bevy_voxel_world`, `rand`, später
   `avian3d`) verwendet wird: kurz einordnen, was es im Hintergrund tut,
   damit keine Blackbox-Gefühl entsteht.
9. **Kein Zeitdruck, keine Phasen überspringen.** Aktuelle Phase erst
   abschließen (siehe Roadmap), bevor die nächste beginnt.

## Kontext

Siehe `README.md` für das Architekturkonzept und
`Roadmap_Voxel_Atmosphere.md` (Projektwissen) für den Gesamtplan.
# Implementation Plan: freeloader-vertical-slice

## Arbeitsübereinkunft

- Ein Commit pro Aufgabe, keine Sammelcommits (Anf. 20.1, 20.11).
- Conventional Commits mit Crate-Scope, z. B. `feat(download-core): …`; jeder Commit mit `git commit -s` (DCO) (Anf. 20.4, 20.5).
- Vor jedem Commit vier grüne Prüfungen: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, Frontend-`typecheck` (Anf. 20.2, 20.3).
- Alle Arbeit auf `feat/freeloader-full-implementation`, Integration nur per Pull Request, lineare Historie, keine direkten Pushes nach `main` (Anf. 20.6–20.9).
- Jede `git`-Operation braucht in dieser Umgebung vorher die Zustimmung des Nutzers (Anf. 20.10).
- `design.md` ist die verbindliche Referenz; Aufgaben nennen nur Abschnittsnamen, keine kopierten Details.

## Tasks

- [-] Task 1: Repository bereinigen und Projektdateien vervollständigen.
  - Umsetzung: `*.exe` in `.gitignore`, `freeloader-desktop.exe` und `uninstall.exe` aus der Versionierung nehmen, die toten ADR-Verweise in `deny.toml` auflösen, `extensions/shared` in `pnpm-workspace.yaml` klären, `docs/implementation-status.md` als echtes Markdown wiederherstellen, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, Issue- und PR-Vorlagen anlegen.
  - Test: Kein Eigenschaftstest; strukturelle Prüfung im CI-Job `layering`.
  - Anf.: 21.1, 21.2, 21.3, 21.5, 21.6, 21.7, 21.8, 21.9, 21.10
  - Demo: `git ls-files "*.exe"` liefert keine Treffer, `docs/implementation-status.md` rendert.

- [~] Task 2: Spezifikationskonsistenz sicherstellen und Rückverfolgbarkeit anlegen.
  - Umsetzung: Vorhandensein von `24.17`–`24.19` und `25.31`–`25.33` in `requirements.md` prüfen und fehlende ergänzen, den „Nummerierungshinweis" in `design.md` durch die aufgelöste Zuordnung ersetzen, Eigenschaft-zu-Kriterium-Matrix in `docs/verification.md` aufnehmen.
  - Test: Kein Eigenschaftstest; Dokumentprüfung.
  - Anf.: 19.7, 24.17, 24.18, 24.19, 25.31, 25.32, 25.33
  - Demo: Jede der 27 Eigenschaften hat in `docs/verification.md` genau eine Kriteriumszeile.

- [~] Task 3: `download-core` als prüfbares Gerüst mit Datenbankschicht aufsetzen.
  - Umsetzung: Crate-Lints und `forbid(unsafe_code)`, alle Abhängigkeiten über `workspace = true`, Fehlerhierarchie aus „Error Handling", versionierte Migrationen und Pool-Öffnung nach „Data Models".
  - Test: Kein Eigenschaftstest; Unit-Test für idempotente Migration und gesetzte Pragmas.
  - Anf.: 6.1, 6.2, 6.3, 6.4, 6.8, 9.3, 9.4, 9.5, 9.6
  - Demo: `cargo test -p freeloader-download-core` legt eine frische Datenbank an und migriert sie zweimal ohne Fehler.

- [~] Task 4: Row_Model, Domain_Model, Dto_Model und generierte Typen bauen.
  - Umsetzung: Die drei Modellschichten samt Umwandlungspunkten nach „Die drei Modellschichten" trennen, Invarianten in Konstruktoren erzwingen, TypeScript-Typen aus dem Dto_Model erzeugen und einchecken.
  - Test: Eigenschaft 21 als eigenschaftsbasierter Test.
  - Anf.: 14.1, 14.2, 14.3, 14.4, 14.5, 14.6
  - Demo: Erneute Typgenerierung erzeugt keine Diff-Zeile.

- [~] Task 5: Download_Repository und Zustandsmaschine implementieren.
  - Umsetzung: Repository nach „Wo Zustand lebt" mit allen Feldern aus `6.5`, Übergangsmatrix aus „Zustandsmaschine", jeder Übergang eine SQLite-Transaktion, abgelehnte Übergänge als Fehler ohne Panik.
  - Test: Eigenschaft 7 und Eigenschaft 8 als je ein eigenschaftsbasierter Test.
  - Anf.: 6.5, 6.6, 6.7, 9.8
  - Demo: Eine verbotene Statusfolge liefert einen Fehler und lässt die Zeile unverändert.

- [~] Task 6: Trait-Nähte, Fakes und verschobene Nähte bereitstellen.
  - Umsetzung: `DownloadStrategy`, `HttpClient`, `Clock`, `FileSystem`, `Rate_Limiter`, Checksummen-Naht per Konstruktor-Injektion als `Arc<dyn …>`, Produktions- und Fake-Implementierungen nebeneinander, verschobene Nähte als Durchlässe ohne Attrappen.
  - Test: Kein Eigenschaftstest; die Fakes sind Testinfrastruktur für alle folgenden Aufgaben.
  - Anf.: 9.1, 9.2, 9.7, 23.1, 23.2, 23.6
  - Demo: Die Engine lässt sich vollständig mit Fakes konstruieren, ohne Netz und ohne Anzeigeserver.

- [~] Task 7: Pfad-Containment und Zielnamenswahl implementieren.
  - Umsetzung: Prozedur aus „Pfad-Containment" inklusive `normalise` und `\\?\`-Symmetrie, Windows-Vergleich ohne Groß-Klein-Unterschied, Symlink-Ausbruch abweisen, freien Zielnamen als kleinste ungenutzte Zahl bis 999 wählen.
  - Test: Eigenschaft 10 und Eigenschaft 11 als je ein eigenschaftsbasierter Test.
  - Anf.: 2.5, 2.6, 8.1, 8.2, 8.3, 8.4, 8.5, 8.6
  - Demo: `../../evil` als Kandidat wird abgewiesen, `a.txt` neben Bestand wird `a (1).txt`.

- [~] Task 8: Dateinamen ausschließlich an das Protocol_Crate delegieren.
  - Umsetzung: Aufrufpfad nach „Dateinamen: eine einzige Quelle der Wahrheit" verdrahten, eigene Bereinigungslogik in der Engine entfernen, Hinweis bei veränderten Kandidaten durchreichen; `crates/protocol` bleibt unverändert.
  - Test: Eigenschaft 9 als eigenschaftsbasierter Test.
  - Anf.: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7
  - Demo: `git diff --stat crates/protocol` bleibt leer, Gerätenamen und Steuerzeichen sind dennoch entschärft.

- [~] Task 9: HttpClient mit Schema-Schranke, Weiterleitungen, Zeitgrenzen und Wiederholpolitik bauen.
  - Umsetzung: Nur `http` und `https` zulassen, auch nach Weiterleitung, höchstens 10 Weiterleitungen, Verbindungs- und Leerlaufgrenzen setzen, Wiederholung nur bei den benannten Statuscodes mit Backoff und `Retry-After`, keine Cookies und keine Anmeldedaten senden.
  - Test: Eigenschaft 12, Eigenschaft 14 und Eigenschaft 18 als je ein eigenschaftsbasierter Test.
  - Anf.: 2.7, 3.7, 3.8, 3.9, 3.10, 3.11, 3.12, 17.2, 17.6
  - Demo: `file:///etc/passwd` wird abgewiesen, ein `503` mit `Retry-After: 2` wird begrenzt wiederholt.

- [~] Task 10: Metadaten-Vorabprüfung und Namensvorrangkette implementieren.
  - Umsetzung: `HEAD` vor dem Streamen, `Content-Disposition` vor Pfadsegment vor Ersatzname, `Accept-Ranges`, `ETag`, `Last-Modified` und Gesamtgröße erfassen und persistieren.
  - Test: Eigenschaft 13 als eigenschaftsbasierter Test.
  - Anf.: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6
  - Demo: Eine URL ohne Dateinamen im Pfad landet unter dem Ersatznamen, Validatoren stehen in der Datenbank.

- [~] Task 11: Einzelstrom-Transfer mit Part_File, Fortschrittstakt und Flushed_Offset.
  - Umsetzung: Datenfluss aus „Datenfluss des primären Akzeptanzpfades" umsetzen, Bytes gepuffert in die Part_File schreiben, nach Abschluss umbenennen, Fortschritt getaktet melden, `Flushed_Offset` nur nach bestätigter Dauerhaftigkeit fortschreiben.
  - Test: Eigenschaft 2, Eigenschaft 15 und Eigenschaft 16 als je ein eigenschaftsbasierter Test.
  - Anf.: 2.1, 2.2, 2.3, 2.4, 5.1
  - Demo: Ein Fixture-Download von 10 MiB endet als vollständige Zieldatei ohne Part_File-Rest.

- [~] Task 12: Pausieren mit Abbruch-Token und Schreibstopp implementieren.
  - Umsetzung: Abbruch-Token je Transfer, Pause innerhalb der Frist wirksam, danach genau 0 weitere Bytes, Pause für nicht laufende Transfers als definierter Fehler.
  - Test: Eigenschaft 3 als eigenschaftsbasierter Test.
  - Anf.: 4.1, 4.2, 4.5, 4.7
  - Demo: Nach der Pause wächst die Part_File über mehrere Sekunden nicht mehr.

- [~] Task 13: Fortsetzen in laufender Sitzung implementieren.
  - Umsetzung: Fortsetzungs-Request mit `Range` und Validator aufbauen, Zieldatei ohne `truncate` öffnen und anhängen, `206`-Antwort als Normalfall behandeln, Pause-und-Fortsetzen ohne Änderung als verlustfreier Rundlauf.
  - Test: Eigenschaft 4 als eigenschaftsbasierter Test.
  - Anf.: 4.3, 4.4, 4.6
  - Demo: Pause bei 40 Prozent, Fortsetzen, Ergebnis stimmt Byte für Byte mit der Referenz überein.

- [~] Task 14: Fortsetzungsalgorithmus für alle Neustartzweige implementieren.
  - Umsetzung: Entscheidungstabelle T1 vollständig abbilden: Startoffset bestimmen, kürzere Platte, `200` statt `206`, fehlendes oder verneintes `Accept-Ranges`, veralteter Validator, verschwundene Part_File.
  - Test: Eigenschaft 5 als eigenschaftsbasierter Test.
  - Anf.: 5.4, 5.5, 5.6, 5.7, 5.8, 5.9, 5.10
  - Demo: Jeder Zweig der Tabelle endet in genau einem definierten, konsistenten Endzustand.

- [~] Task 15: Startbereinigung und Wiederherstellung implementieren.
  - Umsetzung: Beim Start alle laufenden Datensätze in den unterbrochenen Zustand überführen, unvollständige Transfers wiederherstellbar anzeigen, fremde Zustände unberührt lassen.
  - Test: Eigenschaft 6 als eigenschaftsbasierter Test.
  - Anf.: 5.2, 5.3
  - Demo: Nach hartem Beenden erscheinen die betroffenen Einträge als fortsetzbar, alle anderen unverändert.

- [~] Task 16: Fixture-Server und Akzeptanzpfad-Integrationstest mit Neustart.
  - Umsetzung: Testserver nach „Fixture-Server der Verification_Suite" nur auf Loopback, Neustart im Test durch Verwerfen und Neuaufbau von Pool und Engine, Schritte 4 bis 10 als automatisierter Test.
  - Test: Eigenschaft 1 als eigenschaftsbasierter Test.
  - Anf.: 1.1, 1.2, 1.3, 1.5, 5.11, 19.1, 19.2, 19.3, 19.4
  - Demo: `cargo test -p freeloader-download-core` fährt den Akzeptanzpfad headless und offline durch.

- [~] Task 17: Download_Queue mit Parallelitätslimit implementieren.
  - Umsetzung: Limit standardmäßig 3, Einstellbereich 1 bis 8, Überzählige einreihen, beim Ende den ältesten Auftrag starten, Verkleinerung ohne Abbruch laufender Transfers, ungültige Werte abweisen.
  - Test: Eigenschaft 17 als eigenschaftsbasierter Test.
  - Anf.: 16.1, 16.2, 16.3, 16.4, 16.5, 16.6, 16.7
  - Demo: Bei Limit 3 laufen nie mehr als drei Transfers, die Reihenfolge bleibt nachvollziehbar.

- [~] Task 18: Durchsetzungsschienen und Lieferkettenprüfungen einziehen.
  - Umsetzung: `scripts/check-*` für Abhängigkeitsschranken, `cfg`-Lokalisierung, Adapter-Zeilenbudget und Bundle-Ziel-Konsistenz, CI-Jobs `layering`, `engine`, `generated`, `supply-chain` mit `cargo deny`, `cargo audit`, Geheimnis- und Codeanalyse, Dependabot, `docs/adr/` mit den referenzierten Entscheidungen.
  - Test: Kein Eigenschaftstest; die Prüfungen sind selbst die Kontrolle.
  - Anf.: 10.1, 10.2, 10.3, 10.4, 10.5, 10.6, 10.7, 10.8, 21.4, 22.1, 22.2, 22.3, 22.4, 22.5, 22.6, 22.7
  - Demo: Ein künstlich eingefügter `tauri`-Verweis in `download-core` bringt den Job `layering` zum Scheitern.

- [~] Task 19: Tauri_Adapter als dünne Schicht bauen.
  - Umsetzung: Kommandosatz aus „Tauri_Adapter", sofortige Rückkehr vor dem ersten Byte, Fortschrittsbündelung auf höchstens vier Emissionen pro Sekunde, strukturierte Fehlercodes, CSP ohne `unsafe-inline`, minimale Berechtigungen, `main` ohne `unwrap`.
  - Test: Eigenschaft 19 als eigenschaftsbasierter Test.
  - Anf.: 13.1, 13.2, 13.3, 13.4, 13.5, 13.6, 13.7, 13.8, 13.9
  - Demo: Der Zeilenzähler des Adapters bleibt unter dem Budget, Kommandos antworten sofort.

- [~] Task 20: Frontend-Gerüst mit Store, Schemagrenze und Testlauf.
  - Umsetzung: `zustand`-Store nach „Modulkarte", `zod`-Prüfung an jeder Adaptergrenze, ungültige Daten als Hinweis ohne Zustandsänderung, `vitest`-Konfiguration, ausschließlich generierte Typen für Engine-Daten.
  - Test: Eigenschaft 22 als eigenschaftsbasierter Test.
  - Anf.: 14.7, 15.2, 15.3, 15.4, 15.10
  - Demo: Ein manipuliertes Ereignis erzeugt eine Meldung, der Store bleibt unverändert.

- [~] Task 21: Mock_Ipc und Laufzeiterkennung für die Entwicklervorschau.
  - Umsetzung: Erkennung der Webview ohne Ratespiele, außerhalb der Webview Mock_Ipc statt echter Aufrufe, jedes Kommando abgedeckt, Antworten schemakonform, Abmeldefunktion von `listen` wirksam, unbekannte Kommandos als definierter Fehler, aus dem Produktions-Bundle ausgeschlossen.
  - Test: Eigenschaft 26 als eigenschaftsbasierter Test.
  - Anf.: 24.1, 24.2, 24.3, 24.4, 24.5, 24.10, 24.13, 24.14, 24.15, 24.18, 24.19
  - Demo: Der dokumentierte Befehl startet das Frontend im Browser, alle Ansichten sind bedienbar.

- [~] Task 22: Fake_Engine und Dev_Gallery bauen.
  - Umsetzung: Vollständiger Transferlebenszyklus aus einer aussaatgesteuerten Quelle, keine Netzverbindung und kein Dateizugriff, simulierter Neustart, Galerie-Route mit Leerzustand, Ladeskelett, Fehlerzustand und allen Statusvarianten.
  - Test: Eigenschaft 27 als eigenschaftsbasierter Test.
  - Anf.: 24.6, 24.7, 24.8, 24.9, 24.11, 24.12, 24.16, 24.17
  - Demo: Zwei Läufe mit gleichem Startwert erzeugen dieselbe Ereignisfolge.

- [~] Task 23: Token_Layer, Kontrastpaarung und Accessibility_Gate einführen.
  - Umsetzung: Alle Farben, Radien, Abstände als Vielfache von 4 px, Schrift- und Bewegungsstufen als Tokens, helles und dunkles Thema, `forced-colors`- und `prefers-reduced-motion`-Behandlung, `styles/contrast-pairs.json` als maschinenlesbare Paarungsliste, axe-Prüfung als Verifikationsschritt, Stilprüfung im CI.
  - Test: Kein Eigenschaftstest; Kontrastberechnung je Paarung und axe-Zusicherungen als gezielte Prüfungen.
  - Anf.: 25.1, 25.2, 25.3, 25.4, 25.5, 25.6, 25.7, 25.8, 25.9, 25.10, 25.11, 25.12, 25.13, 25.14, 25.19, 25.20, 25.21, 25.25, 25.27, 25.30, 25.31, 25.32
  - Demo: Eine rohe Hex-Farbe außerhalb der Tokens bricht den CI-Job, jede Paarung erfüllt ihre Schwelle.

- [~] Task 24: Lokalisierung mit `i18next` einrichten.
  - Umsetzung: Namensräume aus „i18next-Namensräume", Erstauflösung aus der Systemsprache, Rückfall auf Englisch bei anderen Sprachen, Wechsel persistieren, keine Textliterale in Komponenten.
  - Test: Eigenschaft 23 als eigenschaftsbasierter Test.
  - Anf.: 15.5, 15.6, 15.7, 15.8, 15.9
  - Demo: Bei Systemsprache `fr` startet die Oberfläche auf Englisch, der Wechsel überlebt den Neustart.

- [~] Task 25: Downloadliste und Statusdarstellung als Komponentenmodule bauen.
  - Umsetzung: Module je höchstens 200 Zeilen, jede Zeile mit zugänglichem Namen und Fortschrittswert, Status gleichzeitig über Farbe, Symbol und Text, Fortschritt über `transform` ohne Neuaufbau der Zeile, Größen und Restzeiten lokalisiert, Tastaturerreichbarkeit.
  - Test: Eigenschaft 24 als eigenschaftsbasierter Test.
  - Anf.: 15.1, 15.11, 25.15, 25.16, 25.22, 25.24, 25.28, 25.29
  - Demo: Bei laufendem Fortschritt zählt der Render-Zähler der Zeile nicht hoch.

- [~] Task 26: Modalen Dialog mit Fokusführung fertigstellen.
  - Umsetzung: Fokus beim Öffnen in den Dialog setzen und dort halten, beim Schließen an den Auslöser zurückgeben, `Escape` schließt, Fertigstellungsmeldungen über die höfliche Live-Region.
  - Test: Eigenschaft 25 als eigenschaftsbasierter Test.
  - Anf.: 25.17, 25.18, 25.22, 25.33
  - Demo: Tabben im offenen Dialog verlässt ihn nicht, `Escape` stellt den Ausgangsfokus wieder her.

- [~] Task 27: Platform_Crate, Host_Manifest und Build_Key_Step umsetzen.
  - Umsetzung: Browsererkennung über Registry und Dateisystem mit Rückfall, Registrierungsstatus je Browser ausgeben, Host_Manifest mit echter Extension_Id und echter Absenderkennung schreiben, Schlüsselpaar außerhalb des Repositorys erzeugen, Platzhalter als blockierenden Fehler melden, Reparaturaktion und Identitätsanzeige, keine Profil- oder Verlaufszugriffe, keine Store-Verweise.
  - Test: Kein Eigenschaftstest; Registrierung und Manifestschreiben als gezielte Integrationsprüfungen.
  - Anf.: 12.1, 12.2, 12.3, 12.4, 12.5, 12.6, 12.7, 12.8, 12.9, 12.10, 12.11, 12.12, 12.13, 12.14
  - Demo: `REPLACE_WITH_RELEASE_PUBLIC_KEY` und leere `allowed_origins` erzeugen einen klaren Startfehler.

- [~] Task 28: Native_Host mit Rahmenverarbeitung und Auftragsübergabe bauen.
  - Umsetzung: Rahmen ausschließlich über das Protocol_Crate lesen und schreiben, Übergröße definiert ablehnen, Auftrag über den lokalen Kanal ohne lauschenden Socket übergeben, App bei Bedarf starten, Fehlschlag melden, Cookie-Wunsch verweigern, Datenstromende sauber beenden.
  - Test: Eigenschaft 20 als eigenschaftsbasierter Test.
  - Anf.: 11.1, 11.2, 11.3, 11.4, 11.5, 11.6, 11.7, 11.8, 17.1
  - Demo: Zufälliger Bytemüll auf `stdin` beendet den Host nicht, die Antwort bleibt wohlgeformt.

- [~] Task 29: Datenschutz-Invarianten, Update_Check_Setting und lokale Protokollierung verdrahten.
  - Umsetzung: Update-Prüfung beim Erststart aus und ohne Requests, keine Konten und keine Cloud-Dienste, Protokolle nur lokal, Bedienelemente für verschobene oder nicht vorhandene Funktionen weglassen, verschobene Funktionen dokumentieren.
  - Test: Kein Eigenschaftstest; Abhängigkeits- und Oberflächenprüfungen im CI.
  - Anf.: 17.3, 17.4, 17.5, 17.7, 17.8, 17.9, 23.3, 23.4, 23.5, 23.7
  - Demo: Ein Lauf ohne Nutzeraktion erzeugt genau 0 ausgehende Verbindungen.

- [~] Task 30: Installer, Packaging, First_Run_Assistant und Manual_Checklist abschließen.
  - Umsetzung: Windows- und Linux-Artefakte für beide Architekturen, Registrierung bei Installation und Rücknahme bei Deinstallation, Erststart-Assistent mit Zielordner, Sprache, Thema und Registrierungsstatus samt Überspringen-Pfad, manuelle Checkliste für Build, Installer, Erststart, Deinstallation und Browserschritte.
  - Test: Kein Eigenschaftstest; Smoke-Prüfung der Artefakte und dokumentierte Handprüfung.
  - Anf.: 1.4, 18.1, 18.2, 18.3, 18.4, 18.5, 18.6, 18.7, 18.8, 18.9, 19.5, 19.6
  - Demo: Installation, Erststart und Deinstallation hinterlassen kein verwaistes Host_Manifest.

Keine Eigenschaftstests entstehen für Installer und Packaging, den Ablauf des First_Run_Assistant, Registry-Auswertung und Host_Manifest-Registrierung, axe-Zusicherungen, Kontrastpaarungen, die Zuordnung von Fehlercodes, strukturelle Repository- und Schichtprüfungen sowie die verschobenen Nähte.

## Task Dependency Graph

```json
{"waves":[{"id":0,"tasks":["1","2"]},{"id":1,"tasks":["3"]},{"id":2,"tasks":["4","8"]},{"id":3,"tasks":["5","7"]},{"id":4,"tasks":["6"]},{"id":5,"tasks":["9"]},{"id":6,"tasks":["10"]},{"id":7,"tasks":["11"]},{"id":8,"tasks":["12"]},{"id":9,"tasks":["13"]},{"id":10,"tasks":["14"]},{"id":11,"tasks":["15"]},{"id":12,"tasks":["16","17"]},{"id":13,"tasks":["18","19"]},{"id":14,"tasks":["20"]},{"id":15,"tasks":["21"]},{"id":16,"tasks":["22","23"]},{"id":17,"tasks":["24","25"]},{"id":18,"tasks":["26","27"]},{"id":19,"tasks":["28"]},{"id":20,"tasks":["29"]},{"id":21,"tasks":["30"]}]}
```

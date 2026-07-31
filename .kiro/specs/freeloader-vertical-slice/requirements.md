# Requirements Document

## Introduction

Dieser Spec beschreibt den ersten vertikalen Schnitt von `freeloader`, einem lokal arbeitenden HTTPS-Download-Manager für Windows und Linux auf Basis von Rust, Tauri v2 und React 19 unter GPL-3.0-or-later.

Der Erfolg dieses Specs wird an **genau einem** Pfad gemessen:

> Projekt bauen → geführten Installer ausführen → Erststart-Setup abschließen → HTTPS-URL einfügen → echte Datei mit Live-Fortschritt auf die Platte streamen → pausieren → fortsetzen → Anwendung beenden → neu starten → erneut fortsetzen bis zur Fertigstellung.

Der letzte Abschnitt dieses Pfades ist der eigentliche Kern: Der aktuelle Code kann Downloads nach einem Anwendungsneustart überhaupt nicht fortsetzen. `download-core::download()` öffnet die `.part`-Datei bedingungslos mit `.truncate(true)`, sendet nie einen `Range`-Header, besitzt kein Abbruch-Token und schreibt den erzeugten `DownloadRecord` niemals in die Datenbank. Die Tabelle `downloads` wird per inline `CREATE TABLE IF NOT EXISTS` erzeugt und danach weder gelesen noch geschrieben. Alle Anforderungen zum Fortsetzen sind deshalb bewusst so formuliert, dass sie ohne echte Persistenz nicht erfüllbar sind.

Der Schnitt umfasst außerdem die Leitplanken, die verhindern, dass der Pfad später unbemerkt zerbricht: explizite Trait-Nähte statt globalem Zustand, echte vorwärtsgerichtete Migrationen, maschinell durchgesetzte Schichtregeln, Supply-Chain-Prüfungen und eine verbindliche Repository- und Commit-Hygiene.

**Bewusst erhalten, nicht neu geschrieben:** `crates/protocol` ist tragfähig und wird zur einzigen Quelle der Wahrheit ausgebaut, nicht ersetzt. Verifiziert vorhanden: längenpräfigiertes Framing mit Little-Endian-Compile-Time-Assert, `#[serde(deny_unknown_fields)]`, getaggte Enums, geordnete Validierung (Größe → Version → Payload), Bidi-/Zero-Width-/BOM-Korpus im Sanitiser, Ablehnung von Windows-Gerätenamen, erweiterungserhaltende 255-Byte-Truncation, `#![forbid(unsafe_code)]` und `#![deny(clippy::unwrap_used, expect_used, panic, indexing_slicing)]`.

**Dauerhaft außerhalb des Umfangs:** DRM-Umgehung, Umgehung von Bezahl- oder Login-Schranken, Extraktion aus Streaming-Seiten, YouTube-Ripping, macOS.

**In späteren Specs, hier nur als Naht:** Multi-Connection-Segmentierung, funktionierende Bandbreitenbegrenzung, Tray-Integration, Weiterleitung von Cookies oder Zugangsdaten, Checksummen-Verifikation, ausgehender Update-Request.

**Umgebungshinweis:** In dieser Arbeitsumgebung erfordert jeder `git`-Aufruf eine ausdrückliche Benutzerfreigabe. Aufgaben, die Git-Operationen ausführen, müssen diese Abhängigkeit sichtbar melden, statt still zu scheitern (Anforderung 20).

## Glossary

- **Git_Repository**: Das versionierte Projekt `freeloader` samt Arbeitsverzeichnis und Historie.
- **Freeloader_App**: Die ausgelieferte Desktop-Anwendung, bestehend aus dem Tauri-Binary und dem Webview.
- **Download_Engine**: Der Crate `crates/download-core`; portable Download-Logik ohne Betriebssystem- oder GUI-Abhängigkeit.
- **Download_Strategy**: Trait-Naht über das Übertragungsverfahren. In v0.1 existiert genau eine Implementierung: ein Einzelstrom-Transfer.
- **Http_Client**: Trait-Naht über ausgehende HTTP-Aufrufe, im Produktivbetrieb von `reqwest` implementiert.
- **Download_Repository**: Trait-Naht über die Persistenz von Download-Zuständen in SQLite.
- **File_System**: Trait-Naht über Dateisystemoperationen der Download_Engine.
- **Clock**: Trait-Naht über die Zeitquelle, damit Backoff und Zeitstempel deterministisch testbar sind.
- **Rate_Limiter**: Trait-Naht über Bandbreitenbegrenzung. In v0.1 existiert ausschließlich eine dokumentierte No-op-Implementierung.
- **Protocol_Crate**: Der Crate `crates/protocol`; Wire-Contract, Validierung und Dateinamen-Sanitisierung.
- **Native_Host**: Der Crate `crates/native-host` und das daraus gebaute Binary `freeloader-native-host`.
- **Platform_Crate**: Der Crate `crates/platform`; alle betriebssystemspezifischen Zugriffe.
- **Tauri_Adapter**: Das Paket `apps/desktop/src-tauri`; dünne Adapterschicht zwischen Frontend und Download_Engine.
- **Frontend**: Die React-Anwendung unter `apps/desktop/src`.
- **Installer**: Das NSIS-Paket unter Windows sowie die `.deb`- und `.rpm`-Pakete unter Linux.
- **First_Run_Assistant**: Der geführte Erststart-Ablauf im Frontend.
- **Download_Queue**: Die Warteschlange, die gleichzeitig laufende Transfers begrenzt.
- **Part_File**: Die temporäre Zieldatei mit der Endung `.part`, in die während des Transfers geschrieben wird.
- **Flushed_Offset**: Der zuletzt per `fsync` bestätigte und im Download_Repository gespeicherte Byte-Offset einer Part_File.
- **Validator**: Der vom Server gelieferte `ETag` oder `Last-Modified`-Wert, der die Identität der Ressource über Sitzungsgrenzen hinweg prüfbar macht.
- **Range_Resume**: Die Fortsetzung eines Transfers über einen HTTP-`Range`-Request ab dem Flushed_Offset.
- **Row_Model**: Der datenbanknahe Typ, der `sqlx::FromRow` implementiert.
- **Domain_Model**: Der Typ mit in Konstruktoren erzwungenen Invarianten, den die Download_Engine intern verwendet.
- **Dto_Model**: Der camelCase-serialisierte Typ, der die Grenze zum Frontend überquert.
- **Generated_Types**: Die aus dem Dto_Model erzeugte und im Git_Repository eingecheckte TypeScript-Typdatei.
- **Host_Manifest**: Das Native-Messaging-Host-Manifest, das den Pfad zum Native_Host und die erlaubten Extension-Identitäten deklariert.
- **Extension_Id**: Die stabile Chromium-Extension-ID, abgeleitet aus dem festen `key` im Extension-Manifest.
- **Build_Key_Step**: Der einmalige Build-Schritt, der das Extension-Keypair erzeugt und `key` sowie Extension_Id in die Build-Konfiguration schreibt.
- **Update_Check_Setting**: Die persistierte Einstellung, die eine spätere Aktualisierungsprüfung freischaltet; Vorgabewert ist deaktiviert.
- **Verification_Suite**: Die automatisierten Tests des Workspace samt lokalem Testserver-Harness.
- **Manual_Checklist**: Die versionierte, abhakbare Verifikationscheckliste in `docs/verification.md`.
- **CI_Pipeline**: Die GitHub-Actions-Workflows unter `.github/workflows/`.
- **Commit_Gate**: Die vier Prüfungen, die vor jedem Commit erfolgreich sein müssen: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` und der Frontend-`typecheck`.
- **Layering_Check**: Der CI-Job, der die Schichtregeln des Workspace mechanisch prüft.
- **Mock_Ipc**: Die reine Entwicklungsschicht, die `invoke` und `listen` außerhalb einer Tauri-Webview gegen das Fake_Engine auflöst, damit jede Oberfläche ohne Rust-Build und ohne Installation erreichbar ist.
- **Fake_Engine**: Die speicherinterne Attrappe der Download_Engine hinter dem Mock_Ipc. Sie erzeugt jeden Zustandswechsel und jedes Fortschrittsereignis aus einer deterministischen, mit einem festen Startwert initialisierten Zeitquelle und ist niemals Teil des Produktions-Bundles.
- **Dev_Gallery**: Die ausschließlich in der Entwicklung erreichbare Route, die jeden Oberflächenzustand nebeneinander darstellt.
- **Token_Layer**: Die einzige Ebene, in der Farb-, Radius-, Abstands-, Dauer- und Beschleunigungswerte als semantische CSS Custom Properties definiert werden.
- **Accessibility_Gate**: Die maschinelle Barrierefreiheitsprüfung der Verification_Suite, die vor dem Zusammenführen eines Pull Requests fehlerfrei durchlaufen muss.

## Requirements

### Anforderung 1: Primärer Akzeptanzpfad

**User Story:** Als Nutzer möchte ich einen großen HTTPS-Download starten, pausieren, die Anwendung beenden, die Anwendung neu starten und den Download zu Ende bringen, damit ich einer lokalen Anwendung meine Downloads anvertrauen kann.

#### Szenario in Given-When-Then-Form

| Schritt | Given | When | Then (beobachtbar) |
| --- | --- | --- | --- |
| 1 | Ein sauberer Checkout auf einer unterstützten Plattform | `cargo tauri build` und der Frontend-Build laufen | Der Build endet mit Exitcode 0 und legt genau die für die Plattform deklarierten Bundle-Artefakte an |
| 2 | Ein gebautes Installationsartefakt | Der Installer wird ausgeführt | Freeloader_App, Native_Host und Host_Manifest sind installiert, und die Freeloader_App startet ohne Fehlermeldung |
| 3 | Ein Erststart ohne vorhandene Konfiguration | Der First_Run_Assistant wird durchlaufen | Zielordner und Sprache sind persistiert, und ein zweiter Start zeigt den First_Run_Assistant nicht erneut |
| 4 | Eine erreichbare HTTPS-URL mit `Accept-Ranges: bytes` und bekannter Größe | Die URL wird eingefügt und bestätigt | Ein Datensatz mit Status `downloading` existiert im Download_Repository, und eine Part_File wächst auf der Platte |
| 5 | Ein laufender Transfer | Der Nutzer beobachtet die Oberfläche | Übertragene Bytes, Gesamtgröße und Rate aktualisieren sich mindestens alle 500 ms, ohne dass die Oberfläche blockiert |
| 6 | Ein laufender Transfer | Pause wird ausgelöst | Innerhalb von 500 ms ist der Status `paused`, der Flushed_Offset ist persistiert, und die Bytelänge der Part_File wächst nicht weiter |
| 7 | Ein pausierter Transfer | Fortsetzen wird ausgelöst | Ein `Range`-Request ab dem Flushed_Offset wird gesendet, der Server antwortet mit `206`, und die Part_File wächst weiter, ohne von vorn zu beginnen |
| 8 | Ein laufender Transfer | Die Freeloader_App wird beendet | Der Prozess endet ohne Datenverlust, und der persistierte Status ist `paused` mit dem zuletzt bestätigten Flushed_Offset |
| 9 | Eine beendete Anwendung mit einem unvollständigen Download | Die Freeloader_App wird neu gestartet | Der Download erscheint mit Status `paused`, korrektem Offset, korrekter Gesamtgröße und der zugehörigen Part_File in der Liste |
| 10 | Ein nach Neustart wiederhergestellter Download | Fortsetzen wird ausgelöst | Der Transfer setzt am Flushed_Offset an, läuft bis zur Gesamtgröße, die Part_File wird atomar auf den Zielnamen umbenannt, der Status ist `completed`, und der Inhalt der Zieldatei ist byteweise identisch mit der Quelle |

#### Acceptance Criteria

1. WHEN der primäre Akzeptanzpfad Schritt 4 bis Schritt 10 gegen den lokalen Testserver der Verification_Suite ausgeführt wird, THE Verification_Suite SHALL jeden Schritt automatisiert prüfen und bei Abweichung fehlschlagen.
2. WHEN Schritt 10 abgeschlossen ist, THE Download_Engine SHALL eine Zieldatei erzeugen, deren SHA-256-Summe der SHA-256-Summe der vom Testserver ausgelieferten Quelldatei entspricht.
3. WHEN Schritt 8 und Schritt 9 durch Beenden und Neuaufbau des Anwendungszustands im Test nachgestellt werden, THE Download_Engine SHALL den Transfer ohne erneute Übertragung bereits geschriebener Bytes fortsetzen.
4. THE Manual_Checklist SHALL für Schritt 1 bis Schritt 3 je Plattform eine abhakbare Prüfanweisung mit erwartetem Ergebnis enthalten.
5. IF ein Schritt des primären Akzeptanzpfades fehlschlägt, THEN THE Verification_Suite SHALL die Schrittnummer und den beobachteten Zustand ausgeben.

### Anforderung 2: Download starten und live streamen

**User Story:** Als Nutzer möchte ich eine HTTPS-URL einfügen und die Datei sofort mit sichtbarem Fortschritt auf meine Platte streamen, damit ich weiß, dass etwas passiert.

#### Acceptance Criteria

1. WHEN eine gültige `http`- oder `https`-URL übergeben wird, THE Download_Engine SHALL einen Datensatz im Download_Repository anlegen, bevor die erste Netzwerkverbindung geöffnet wird.
2. WHILE ein Transfer läuft, THE Download_Engine SHALL empfangene Bytes über einen `BufWriter` mit mindestens 64 KiB Puffer in die Part_File schreiben.
3. WHILE ein Transfer läuft, THE Download_Engine SHALL mindestens alle 500 ms und höchstens zehnmal pro Sekunde ein Fortschrittsereignis mit übertragenen Bytes, bekannter Gesamtgröße und aktueller Rate veröffentlichen.
4. WHEN alle Bytes geschrieben sind, THE Download_Engine SHALL die Part_File per `fsync` bestätigen und anschließend atomar auf den Zielnamen umbenennen.
5. IF die Zieldatei bereits existiert, THEN THE Download_Engine SHALL den Zielnamen um das Suffix ` (n)` vor der Erweiterung ergänzen, wobei `n` die kleinste freie Zahl zwischen 1 und 999 ist.
6. IF nach 999 Versuchen kein freier Zielname existiert, THEN THE Download_Engine SHALL den Transfer mit einem Fehler beenden, der den Zielordner benennt.
7. WHEN eine URL mit einem anderen Schema als `http` oder `https` übergeben wird, THE Download_Engine SHALL den Auftrag über `Protocol_Crate::validate_url` ablehnen und genau 0 Netzwerkverbindungen öffnen.

### Anforderung 3: HTTP-Verhalten, Metadaten und Wiederholversuche

**User Story:** Als Nutzer möchte ich, dass die Anwendung den korrekten Dateinamen und die korrekte Größe erkennt und kurzzeitige Netzwerkstörungen selbst überbrückt, damit ich nicht manuell nacharbeiten muss.

#### Acceptance Criteria

1. WHEN ein Transfer angelegt wird, THE Download_Engine SHALL vor dem Streamen eine `HEAD`-Vorabprüfung durchführen und `Content-Length`, `Accept-Ranges`, `ETag`, `Last-Modified` und `Content-Disposition` erfassen.
2. IF die `HEAD`-Vorabprüfung mit einem Status ab 400 antwortet, THEN THE Download_Engine SHALL auf einen `GET`-Request mit `Range: bytes=0-` zurückfallen und die Metadaten aus der Antwort dieses `GET`-Requests erfassen.
3. WHEN ein `Content-Disposition`-Header vorliegt, THE Download_Engine SHALL den `Content-Disposition`-Header nach RFC 6266 auswerten und dabei den nach RFC 5987 kodierten `filename*`-Parameter dem `filename`-Parameter vorziehen.
4. WHEN kein verwertbarer Dateiname aus `Content-Disposition` hervorgeht, THE Download_Engine SHALL das letzte nicht leere Pfadsegment der endgültigen URL verwenden.
5. WHEN weder `Content-Disposition` noch das Pfadsegment einen verwertbaren Dateinamen liefern, THE Download_Engine SHALL den Rückfallnamen `download` verwenden.
6. THE Download_Engine SHALL erfasste Werte für `Accept-Ranges`, `ETag`, `Last-Modified` und Gesamtgröße im Download_Repository speichern, bevor der erste Byte in die Part_File geschrieben wird.
7. IF ein Transportfehler oder ein Antwortstatus 408, 429, 500, 502, 503 oder 504 auftritt, THEN THE Download_Engine SHALL den Request bis zu fünfmal mit exponentiellem Backoff von 1, 2, 4, 8 und 16 Sekunden und ±20 % Jitter wiederholen.
8. WHERE die Antwort einen `Retry-After`-Header mit einem Wert bis 60 Sekunden enthält, THE Download_Engine SHALL diesen Wert anstelle des berechneten Backoff verwenden.
9. IF ein Antwortstatus 400 bis 407 oder 409 bis 499 auftritt, THEN THE Download_Engine SHALL den Transfer ohne Wiederholversuch in den Status `failed` versetzen.
10. THE Download_Engine SHALL höchstens 10 Weiterleitungen folgen und für jede Weiterleitung erneut prüfen, dass das Zielschema `http` oder `https` ist.
11. THE Download_Engine SHALL ein Verbindungs-Timeout von 10 Sekunden und ein Leerlauf-Timeout von 30 Sekunden ohne empfangene Bytes anwenden und für die Gesamtdauer eines Transfers genau 0 Timeouts setzen.
12. WHEN alle Wiederholversuche erschöpft sind, THE Download_Engine SHALL den Status `failed` setzen und die Part_File samt Flushed_Offset für einen späteren manuellen Versuch erhalten.

### Anforderung 4: Pausieren und Fortsetzen in laufender Sitzung

**User Story:** Als Nutzer möchte ich einen laufenden Download anhalten und später fortsetzen, ohne bereits übertragene Bytes zu verlieren.

#### Acceptance Criteria

1. THE Download_Engine SHALL für jeden laufenden Transfer ein Abbruch-Token bereitstellen, über das ein Aufrufer den Transfer anhalten kann.
2. WHEN eine Pause angefordert wird, THE Download_Engine SHALL innerhalb von 500 ms das Schreiben beenden, den Puffer per `fsync` bestätigen, den Flushed_Offset persistieren und den Status `paused` setzen.
3. WHEN ein pausierter Transfer fortgesetzt wird, THE Download_Engine SHALL einen `Range: bytes=<flushed_offset>-` Request mit `If-Range` auf dem gespeicherten Validator senden.
4. WHEN der Server auf einen Fortsetzungs-Request mit `206 Partial Content` antwortet, THE Download_Engine SHALL die Part_File im Anfüge-Modus ohne `truncate` öffnen und ab dem Flushed_Offset weiterschreiben.
5. WHILE ein Transfer pausiert ist, THE Download_Engine SHALL genau 0 Bytes an die Part_File anfügen.
6. WHEN ein Transfer pausiert und ohne Änderung wieder fortgesetzt wird, THE Download_Engine SHALL genau die noch fehlenden Bytes übertragen und dieselbe Zieldatei erzeugen wie ein Transfer ohne Pause.
7. IF eine Pause für einen Transfer angefordert wird, der nicht im Status `downloading` oder `retrying` ist, THEN THE Download_Engine SHALL den Zustand unverändert lassen und einen Fehler mit dem aktuellen Status zurückgeben.

### Anforderung 5: Fortsetzen nach Anwendungsneustart

**User Story:** Als Nutzer möchte ich die Anwendung mitten in einem großen Download beenden können und den Download nach dem Neustart genau dort fortsetzen, wo er stand.

#### Acceptance Criteria

1. WHILE ein Transfer läuft, THE Download_Engine SHALL den Flushed_Offset spätestens alle 4 MiB übertragener Daten und spätestens alle 2 Sekunden per `fsync` bestätigen und im Download_Repository speichern.
2. WHEN die Freeloader_App startet, THE Download_Engine SHALL alle Datensätze mit Status `downloading` oder `retrying` in den Status `paused` überführen, bevor die Oberfläche eine Liste anzeigt.
3. WHEN die Freeloader_App startet, THE Freeloader_App SHALL unvollständige Downloads mit URL, Zielpfad, Gesamtgröße, Flushed_Offset und Validator aus dem Download_Repository wiederherstellen und in der Liste anzeigen.
4. WHEN ein nach dem Neustart wiederhergestellter Transfer fortgesetzt wird, THE Download_Engine SHALL den Startoffset als das Minimum aus der tatsächlichen Bytelänge der Part_File und dem persistierten Flushed_Offset bestimmen.
5. WHEN der bestimmte Startoffset kleiner ist als die tatsächliche Bytelänge der Part_File, THE Download_Engine SHALL die Part_File auf den Startoffset kürzen, bevor der erste Byte angefügt wird.
6. WHEN ein nach dem Neustart wiederhergestellter Transfer fortgesetzt wird, THE Download_Engine SHALL den gespeicherten Validator in einem `If-Range`-Header mitsenden.
7. IF der Server auf einen Fortsetzungs-Request mit `200 OK` statt `206 Partial Content` antwortet, THEN THE Download_Engine SHALL die Part_File auf 0 Bytes kürzen, den Transfer von Beginn an übertragen und den Neustart des Transfers als sichtbaren Hinweis an die Freeloader_App melden.
8. IF der Server `Accept-Ranges: none` meldet oder gar keinen `Accept-Ranges`-Header liefert, THEN THE Download_Engine SHALL den Transfer von 0 beginnen und den Nutzer über den fehlenden Fortsetzungs-Support informieren.
9. IF der gespeicherte Validator nicht mehr zum Serverzustand passt, THEN THE Download_Engine SHALL die Part_File auf 0 Bytes kürzen, den neuen Validator speichern und den Transfer als von vorn begonnen melden.
10. IF die Part_File eines wiederhergestellten Datensatzes auf der Platte fehlt, THEN THE Download_Engine SHALL den Flushed_Offset auf 0 zurücksetzen und den Transfer als von vorn begonnen melden.
11. WHEN ein Transfer nach einem Neustart bis zur Gesamtgröße fortgesetzt wurde, THE Download_Engine SHALL den Status `completed` persistieren und die Part_File atomar auf den Zielnamen umbenennen.

### Anforderung 6: Persistenz, Migrationen und Repository

**User Story:** Als Nutzer möchte ich, dass mein Download-Zustand lokal und dauerhaft gespeichert wird, damit nach einem Neustart oder Absturz nichts verloren ist.

#### Acceptance Criteria

1. THE Freeloader_App SHALL ihren gesamten Zustand in einer SQLite-Datenbank unter `%APPDATA%\freeloader` auf Windows und unter `$XDG_DATA_HOME/freeloader` auf Linux ablegen.
2. THE Download_Engine SHALL das Datenbankschema ausschließlich über versionierte, ausschließlich vorwärtsgerichtete `sqlx`-Migrationen im Verzeichnis `crates/download-core/migrations` herstellen.
3. WHEN die Freeloader_App startet, THE Download_Engine SHALL alle ausstehenden Migrationen anwenden, bevor die erste Abfrage ausgeführt wird.
4. THE Download_Engine SHALL beim Öffnen der Datenbank `journal_mode=WAL` und `foreign_keys=ON` setzen.
5. THE Download_Repository SHALL für jeden Download Identifikator, URL, endgültige URL, Zielpfad, Part_File-Pfad, Status, Flushed_Offset, Gesamtgröße, `Accept-Ranges`-Unterstützung, Validator, Erstellungszeitpunkt und Aktualisierungszeitpunkt speichern.
6. WHEN ein Statuswechsel eintritt, THE Download_Repository SHALL den neuen Status und den Aktualisierungszeitpunkt in derselben Transaktion schreiben.
7. IF eine Statusänderung von der Zustandsmaschine nicht erlaubt ist, THEN THE Download_Engine SHALL die Änderung ablehnen und den bestehenden Datensatz unverändert lassen.
8. THE Download_Engine SHALL jede `CREATE TABLE`-Anweisung ausschließlich innerhalb einer Migrationsdatei ausführen.

### Anforderung 7: Dateinamen mit einer einzigen Quelle der Wahrheit

**User Story:** Als Nutzer möchte ich, dass ein vom Server oder Browser vorgeschlagener Dateiname überall gleich streng bereinigt wird, damit kein manipulierter Name mein System gefährdet.

#### Acceptance Criteria

1. THE Download_Engine SHALL Dateinamen ausschließlich über `Protocol_Crate::sanitize_filename` bereinigen.
2. THE Download_Engine SHALL genau 0 eigene Funktionen zur Dateinamen-Bereinigung exportieren oder intern vorhalten.
3. THE Protocol_Crate SHALL ein Ergebnis liefern, das eine einzelne Pfadkomponente ist, höchstens 255 Bytes umfasst und die Dateierweiterung auch bei Kürzung erhält.
4. WHEN ein Kandidat Steuerzeichen, NUL, Bidi-Steuerzeichen, Zero-Width-Zeichen, BOM, Pfadtrenner oder von Windows verbotene Zeichen enthält, THE Protocol_Crate SHALL diese Zeichen entfernen.
5. WHEN ein Kandidat auf einen reservierten Windows-Gerätenamen hinausläuft, THE Protocol_Crate SHALL den Rückfallnamen `download` verwenden.
6. WHEN die Bereinigung den Kandidaten verändert hat, THE Freeloader_App SHALL den bereinigten Namen anzeigen und die Veränderung sichtbar kennzeichnen.
7. THE Protocol_Crate SHALL seine bestehende öffentliche Schnittstelle und seine bestehenden Tests unverändert grün halten.

### Anforderung 8: Pfad-Containment und Zielauflösung

**User Story:** Als Nutzer möchte ich sicher sein, dass ein Download niemals außerhalb meines gewählten Zielordners landet, auch nicht über Symlinks oder Windows-Verbatim-Pfade.

#### Acceptance Criteria

1. WHEN ein Zielpfad geprüft wird, THE Download_Engine SHALL den Zielordner und den vollständig aufgelösten Zielpfad mit demselben Verfahren kanonisieren, bevor sie verglichen werden.
2. THE Download_Engine SHALL den aufgelösten endgültigen Zielpfad prüfen, nicht dessen Elternverzeichnis.
3. WHEN ein Pfadvergleich auf Windows stattfindet, THE Download_Engine SHALL das Verbatim-Präfix `\\?\` auf beiden Seiten des Vergleichs gleich behandeln.
4. IF der aufgelöste Zielpfad außerhalb des kanonisierten Zielordners liegt, THEN THE Download_Engine SHALL den Transfer mit einem Containment-Fehler ablehnen und genau 0 Dateien anlegen.
5. IF ein Bestandteil des Zielpfades ein Symlink ist, der aus dem Zielordner hinausführt, THEN THE Download_Engine SHALL den Transfer mit einem Containment-Fehler ablehnen.
6. WHEN der Zielordner noch nicht existiert, THE Download_Engine SHALL ihn anlegen und anschließend erneut kanonisieren, bevor der Containment-Vergleich stattfindet.

### Anforderung 9: Architekturnähte und Dependency Injection

**User Story:** Als Entwickler möchte ich die Download-Engine ohne Netzwerk, ohne Platte und ohne GUI testen können, damit Tests schnell und deterministisch sind.

#### Acceptance Criteria

1. THE Download_Engine SHALL die Nähte `DownloadStrategy`, `HttpClient`, `DownloadRepository`, `Clock` und `FileSystem` als öffentliche Traits definieren.
2. THE Download_Engine SHALL alle Nähte über Konstruktorparameter als `Arc<dyn …>` erhalten.
3. THE Download_Engine SHALL genau 0 globale veränderliche Zustände und genau 0 Singletons verwenden.
4. THE Download_Engine SHALL `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]` und `#![forbid(unsafe_code)]` deklarieren.
5. THE Download_Engine SHALL alle Abhängigkeiten über `workspace = true` beziehen.
6. THE Download_Engine SHALL Dev-Dependencies für einen lokalen Testserver, für eigenschaftsbasierte Tests und für temporäre Verzeichnisse deklarieren.
7. WHERE ein Testserver für Integrationstests benötigt wird, THE Download_Engine SHALL ihn ausschließlich als Dev-Dependency einbinden, damit das ausgelieferte Binary keinen Serverstack enthält.
8. THE Download_Engine SHALL eine Zustandsmaschine bereitstellen, die genau die erlaubten Übergänge zwischen `created`, `validating`, `queued`, `downloading`, `paused`, `retrying`, `completed`, `failed` und `cancelled` zulässt.

### Anforderung 10: Schichtregeln maschinell durchsetzen

**User Story:** Als Entwickler möchte ich, dass die Architektur durch die CI erzwungen wird, damit sie nicht durch bequeme Abkürzungen erodiert.

#### Acceptance Criteria

1. IF das Protocol_Crate eine andere Laufzeitabhängigkeit als `serde`, `serde_json` und `url` deklariert, THEN THE Layering_Check SHALL fehlschlagen und die abweichende Abhängigkeit benennen.
2. IF die Download_Engine direkt oder transitiv von `tauri` abhängt, THEN THE Layering_Check SHALL fehlschlagen und den Abhängigkeitspfad ausgeben.
3. IF ein `#[cfg(target_os = …)]`-Attribut außerhalb des Platform_Crate auftritt, THEN THE Layering_Check SHALL fehlschlagen und die betroffene Datei benennen.
4. IF die Rust-Quelldateien des Tauri_Adapter zusammen mehr als 600 Zeilen umfassen, THEN THE Layering_Check SHALL fehlschlagen und die gemessene Zeilenzahl ausgeben.
5. IF das Frontend eine Netzwerkabfrage auf eine herunterzuladende Ressource enthält, THEN THE Layering_Check SHALL fehlschlagen und die betroffene Datei benennen.
6. WHEN die CI_Pipeline läuft, THE CI_Pipeline SHALL `cargo test --package freeloader-download-core` in einem Linux-Container ohne Anzeigeserver ausführen.
7. THE CI_Pipeline SHALL `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` und den Frontend-`typecheck` bei jedem Pull Request ausführen.
8. THE CI_Pipeline SHALL die deklarierten Bundle-Ziele der Tauri-Konfiguration gegen die im Release-Workflow übergebenen Bundle-Ziele prüfen und bei Abweichung fehlschlagen.

### Anforderung 11: Native-Messaging-Host

**User Story:** Als Nutzer möchte ich einen Download aus dem Browser an Freeloader übergeben können, auch wenn die Anwendung gerade nicht läuft.

#### Acceptance Criteria

1. THE Native_Host SHALL Frames ausschließlich über `Protocol_Crate::decode_frame` und `Protocol_Crate::encode_frame` verarbeiten.
2. WHEN ein Frame die erlaubte Payload-Größe überschreitet, THE Native_Host SHALL eine strukturierte Antwort mit dem Fehlercode `payload_too_large` schreiben und den Prozess weiterlaufen lassen.
3. WHEN ein gültiger Capture-Request eintrifft, THE Native_Host SHALL den Auftrag an die laufende Freeloader_App übergeben und mit `ack` antworten.
4. IF die Freeloader_App nicht läuft, THEN THE Native_Host SHALL die Freeloader_App starten, den Auftrag übergeben und mit `ack` antworten.
5. IF die Freeloader_App weder erreichbar ist noch gestartet werden kann, THEN THE Native_Host SHALL mit dem Fehlercode `application_unavailable` antworten.
6. THE Native_Host SHALL Aufträge über einen betriebssystemeigenen lokalen Kanal ohne Netzwerk-Socket an die Freeloader_App übergeben.
7. WHEN ein Request `cookiesIncluded` auf `true` setzt, THE Native_Host SHALL mit dem Fehlercode `cookies_not_allowed` antworten und genau 0 Aufträge anlegen.
8. WHEN die Gegenseite den Datenstrom schließt, THE Native_Host SHALL den Prozess mit Exitcode 0 beenden.

### Anforderung 12: Browser-Erkennung und Host-Registrierung

**User Story:** Als Nutzer möchte ich, dass Freeloader meine installierten Browser findet und die Erweiterung ohne manuelles Registry-Editieren funktioniert.

#### Acceptance Criteria

1. WHEN Browser auf Windows erkannt werden, THE Platform_Crate SHALL `HKCU` und `HKLM` unter `SOFTWARE\Clients\StartMenuInternet` sowie die `App Paths`-Schlüssel auswerten.
2. WHEN die Registry-Auswertung keinen Treffer liefert, THE Platform_Crate SHALL die bekannten festen Installationspfade der unterstützten Browser prüfen.
3. WHEN Browser auf Linux erkannt werden, THE Platform_Crate SHALL neben `PATH` auch Flatpak- und Snap-Installationen berücksichtigen.
4. THE Platform_Crate SHALL für jeden gefundenen Browser ausgeben, ob eine Registrierung des Host_Manifest möglich ist.
5. THE Platform_Crate SHALL genau 0 Lesezugriffe auf Browserprofile, Verlauf, Cookies und Lesezeichen durchführen.
6. WHEN die Registrierung angefordert wird, THE Platform_Crate SHALL ein Host_Manifest je erkanntem Browser schreiben, das den absoluten Pfad zum Native_Host enthält.
7. THE Host_Manifest SHALL für Chromium-Browser die Extension_Id literal in `allowed_origins` und für Firefox die Erweiterungs-ID literal in `allowed_extensions` eintragen.
8. THE Build_Key_Step SHALL das Extension-Keypair einmalig erzeugen und den `key` sowie die daraus abgeleitete Extension_Id in eine Build-Konfiguration schreiben, die von Extension-Manifest und Host_Manifest gemeinsam genutzt wird.
9. THE Build_Key_Step SHALL den privaten Schlüssel außerhalb des Git_Repository ablegen.
10. IF das Extension-Manifest oder das Host_Manifest einen Platzhalter anstelle einer echten Identität enthält, THEN THE CI_Pipeline SHALL fehlschlagen.
11. IF die vom Browser gemeldete Identität nicht mit der Identität im Host_Manifest übereinstimmt, THEN THE Freeloader_App SHALL die Meldung „Native Messaging ist nicht konfiguriert" mit einer Reparaturaktion anzeigen.
12. WHEN eine Reparaturaktion ausgelöst wird, THE Freeloader_App SHALL das Host_Manifest neu schreiben und das Ergebnis der Prüfung erneut anzeigen.
13. THE Freeloader_App SHALL Chromium-Erweiterungen ausschließlich über GitHub Releases und eine Anleitung für „Load unpacked" im Entwicklermodus verteilen.
14. THE Git_Repository SHALL genau 0 Verweise auf den Chrome Web Store in Oberfläche, Dokumentation, Installationsskripten und Workflows enthalten.

### Anforderung 13: Tauri-Adapter als dünne Schicht

**User Story:** Als Nutzer möchte ich, dass die Oberfläche während eines Downloads bedienbar bleibt, damit ich weitere Downloads hinzufügen oder pausieren kann.

#### Acceptance Criteria

1. WHEN ein Download-Kommando aufgerufen wird, THE Tauri_Adapter SHALL nach Anlegen des Datensatzes zurückkehren und den Transfer in einer eigenen Aufgabe ausführen.
2. WHILE ein Transfer läuft, THE Tauri_Adapter SHALL weitere Kommandos annehmen und beantworten.
3. THE Tauri_Adapter SHALL Fortschrittsereignisse je Download-Identifikator an das Frontend senden.
4. THE Tauri_Adapter SHALL Kommandos für Hinzufügen, Pausieren, Fortsetzen, Abbrechen, Entfernen und Auflisten von Downloads bereitstellen.
5. THE Tauri_Adapter SHALL Fehler aus der Download_Engine als strukturierte Fehlerobjekte mit stabilem Code an das Frontend weitergeben.
6. THE Tauri_Adapter SHALL in `main` jeden Fehlerfall behandeln, ohne `unwrap` oder `expect` zu verwenden.
7. THE Tauri_Adapter SHALL eine Content-Security-Policy ohne `unsafe-inline` und ohne `unsafe-eval` setzen.
8. THE Tauri_Adapter SHALL ausschließlich die Tauri-Berechtigungen deklarieren, die von den implementierten Kommandos benötigt werden.
9. THE Tauri_Adapter SHALL Download-, Wiederholungs- und Fortsetzungslogik ausschließlich an die Download_Engine delegieren.

### Anforderung 14: Modellschichten und generierte Typen

**User Story:** Als Entwickler möchte ich, dass Datenbank-, Domänen- und Transportmodelle getrennt bleiben und die TypeScript-Typen automatisch zum Rust-Code passen.

#### Acceptance Criteria

1. THE Download_Engine SHALL Row_Model, Domain_Model und Dto_Model als getrennte Typen führen.
2. THE Domain_Model SHALL seine Invarianten in Konstruktoren erzwingen und keine Konstruktion mit ungültigen Werten zulassen.
3. THE Row_Model SHALL ausschließlich innerhalb der Repository-Implementierung verwendet werden.
4. THE Dto_Model SHALL alle Felder in camelCase serialisieren.
5. THE Generated_Types SHALL aus dem Dto_Model erzeugt und im Repository eingecheckt werden.
6. IF die von der CI_Pipeline erneut ausgeführte Typgenerierung ein von den eingecheckten Generated_Types abweichendes Ergebnis liefert, THEN THE CI_Pipeline SHALL fehlschlagen und die Abweichung ausgeben.
7. THE Frontend SHALL ausschließlich die Generated_Types für Daten aus dem Tauri_Adapter verwenden.

### Anforderung 15: Frontend, Zustand und Lokalisierung

**User Story:** Als deutschsprachiger Nutzer möchte ich die Oberfläche in meiner Sprache bedienen und dabei eine wartbare, getestete Anwendung nutzen.

#### Acceptance Criteria

1. THE Frontend SHALL aus Komponentenmodulen mit höchstens 200 Zeilen je Datei bestehen.
2. THE Frontend SHALL den Anwendungszustand in einem `zustand`-Store halten.
3. WHEN Daten aus dem Tauri_Adapter eintreffen, THE Frontend SHALL die eingehenden Daten an der Adaptergrenze mit einem `zod`-Schema validieren.
4. IF eingehende Daten das Schema verletzen, THEN THE Frontend SHALL eine Fehlermeldung anzeigen und den bisherigen Zustand unverändert lassen.
5. THE Frontend SHALL alle nutzersichtbaren Texte über `i18next` aus Ressourcendateien für Deutsch und Englisch beziehen.
6. WHEN die Freeloader_App erstmals startet, THE Frontend SHALL die Sprache aus der Systemsprache übernehmen.
7. IF die Systemsprache weder Deutsch noch Englisch ist, THEN THE Frontend SHALL Englisch als Sprache verwenden.
8. WHEN der Nutzer die Sprache wechselt, THE Frontend SHALL die Auswahl persistieren und beim nächsten Start anwenden.
9. THE Frontend SHALL genau 0 nutzersichtbare Textliterale in Komponentendateien enthalten.
10. THE Frontend SHALL eine `vitest`-Konfiguration und Komponententests für Liste, Fortschrittsanzeige, Pause- und Fortsetzen-Bedienelemente sowie den First_Run_Assistant bereitstellen.
11. THE Frontend SHALL für jedes Bedienelement einen zugänglichen Namen und für die Fortschrittsanzeige einen maschinenlesbaren Wert bereitstellen.

### Anforderung 16: Warteschlange und Parallelität

**User Story:** Als Nutzer möchte ich mehrere Downloads einreihen können, ohne meine Leitung oder Platte zu überlasten.

#### Acceptance Criteria

1. THE Download_Queue SHALL standardmäßig höchstens 3 Transfers gleichzeitig ausführen.
2. THE Freeloader_App SHALL das Parallelitätslimit als Einstellung zwischen 1 und 8 anbieten und persistieren.
3. WHEN das Parallelitätslimit erreicht ist, THE Download_Queue SHALL weitere Aufträge im Status `queued` halten.
4. WHEN ein Transfer endet, THE Download_Queue SHALL den ältesten Auftrag im Status `queued` starten.
5. WHEN das Parallelitätslimit verkleinert wird, THE Download_Queue SHALL laufende Transfers bis zur Fertigstellung oder Pause weiterlaufen lassen.
6. WHILE die Zahl der laufenden Transfers das Parallelitätslimit erreicht, THE Download_Queue SHALL genau 0 zusätzliche Transfers starten.
7. IF ein Wert außerhalb von 1 bis 8 als Parallelitätslimit übergeben wird, THEN THE Freeloader_App SHALL die Einstellung ablehnen und den bisherigen Wert beibehalten.

### Anforderung 17: Datenschutz- und Netzwerk-Invarianten

**User Story:** Als Nutzer möchte ich sicher sein, dass die Anwendung nur das tut, was ich anstoße, und nichts über mich sendet.

#### Acceptance Criteria

1. THE Freeloader_App SHALL genau 0 lauschende Sockets öffnen.
2. THE Freeloader_App SHALL genau 0 Anfragen an Ziele senden, die nicht aus einer vom Nutzer angestoßenen Download-URL hervorgehen.
3. THE Update_Check_Setting SHALL beim Erststart deaktiviert sein.
4. WHILE das Update_Check_Setting deaktiviert ist, THE Freeloader_App SHALL genau 0 Anfragen zur Aktualisierungsprüfung senden.
5. THE Freeloader_App SHALL genau 0 Konten, Anmeldungen oder Cloud-Dienste voraussetzen.
6. THE Freeloader_App SHALL genau 0 Cookies, `Authorization`-Header und sonstige Zugangsdaten annehmen, speichern oder senden.
7. IF eine Telemetrie- oder Analytics-Abhängigkeit im Abhängigkeitsgraphen auftritt, THEN THE CI_Pipeline SHALL fehlschlagen und die Abhängigkeit benennen.
8. IF eine Serverbibliothek als Laufzeitabhängigkeit eines ausgelieferten Binaries auftritt, THEN THE CI_Pipeline SHALL fehlschlagen und die Abhängigkeit benennen.
9. THE Freeloader_App SHALL Diagnoseprotokolle ausschließlich lokal unter dem Anwendungsdatenverzeichnis ablegen.

### Anforderung 18: Installer und Erststart

**User Story:** Als Nutzer möchte ich Freeloader über einen geführten Installer einrichten und beim ersten Start in wenigen Schritten einsatzbereit sein.

#### Acceptance Criteria

1. THE Installer SHALL auf Windows 10 ab Version 1809 und Windows 11 für `x86_64` und `aarch64` ein NSIS-Paket bereitstellen.
2. THE Installer SHALL auf Linux für `x86_64` und `aarch64` je ein `.deb`- und ein `.rpm`-Paket bereitstellen.
3. WHERE ein AppImage gebaut wird, THE CI_Pipeline SHALL das AppImage ausschließlich für `x86_64` in einem separaten, nicht blockierenden Job bauen.
4. WHEN der Installer ausgeführt wird, THE Installer SHALL Freeloader_App, Native_Host und Host_Manifest installieren.
5. WHEN der Installer entfernt wird, THE Installer SHALL das Host_Manifest und die zugehörigen Registrierungseinträge entfernen und die Nutzerdaten erhalten.
6. WHEN die Freeloader_App erstmals startet, THE First_Run_Assistant SHALL Sprache, Zielordner und Browserintegration in höchstens drei Schritten abfragen.
7. WHEN der First_Run_Assistant abgeschlossen wird, THE Freeloader_App SHALL die Auswahl persistieren und den First_Run_Assistant bei weiteren Starts überspringen.
8. WHERE der Nutzer den First_Run_Assistant überspringt, THE Freeloader_App SHALL dokumentierte Vorgabewerte für Sprache und Zielordner verwenden.
9. THE First_Run_Assistant SHALL den Status der Native-Messaging-Registrierung je erkanntem Browser anzeigen.

### Anforderung 19: Verifikation

**User Story:** Als Entwickler möchte ich den Akzeptanzpfad jederzeit reproduzierbar nachweisen können, ohne ihn manuell durchzuspielen.

#### Acceptance Criteria

1. THE Verification_Suite SHALL einen lokalen Testserver bereitstellen, der `Range`-Requests, `ETag`, `Last-Modified`, `Content-Disposition`, `Accept-Ranges: none`, Validator-Wechsel, Verbindungsabbrüche und die Statuscodes 200, 206, 404, 416, 429 und 503 nachbilden kann.
2. THE Verification_Suite SHALL den lokalen Testserver ausschließlich unter `#[cfg(test)]` oder in einem Integrationsharness starten.
3. THE Verification_Suite SHALL das Beenden und Neustarten der Anwendung durch Verwerfen und Neuaufbau aller Laufzeitobjekte über derselben Datenbank nachstellen.
4. THE Verification_Suite SHALL ohne Anzeigeserver, ohne Netzwerkzugang nach außen und ohne installierten Browser durchlaufen.
5. THE Manual_Checklist SHALL für Build, Installer, Erststart, Deinstallation und Browserintegration je Plattform Prüfschritte mit erwartetem Ergebnis und Ergebnisfeld enthalten.
6. IF ein Prüfschritt auf der aktuellen Plattform nicht ausführbar ist, THEN THE Manual_Checklist SHALL den betroffenen Prüfschritt als nicht ausführbar mit Begründung kennzeichnen.
7. THE Verification_Suite SHALL für jede Korrektheitseigenschaft des Designdokuments genau einen eigenschaftsbasierten Test mit mindestens 100 Durchläufen ausführen.

### Anforderung 20: Repository- und Commit-Hygiene

**User Story:** Als Maintainer möchte ich eine nachvollziehbare, jederzeit grüne Historie, damit jede Änderung einzeln prüfbar und rücknehmbar bleibt.

#### Acceptance Criteria

1. WHEN eine Aufgabe abgeschlossen ist, THE Commit_Gate SHALL genau einen Commit für diese Aufgabe erzeugen.
2. THE Commit_Gate SHALL vor jedem Commit `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` und den Frontend-`typecheck` erfolgreich ausführen.
3. IF eine der vier Prüfungen fehlschlägt, THEN THE Commit_Gate SHALL den Commit zurückhalten, bis die Ursache behoben ist.
4. THE Commit_Gate SHALL Commit-Betreffzeilen nach Conventional Commits mit Crate- oder Paket-Scope verwenden, zum Beispiel `feat(download-core): add ranged resume with ETag validation`.
5. THE Commit_Gate SHALL jeden Commit mit `git commit -s` nach dem Developer Certificate of Origin unterzeichnen.
6. THE Commit_Gate SHALL alle Arbeit auf dem Branch `feat/freeloader-full-implementation` belassen und diesen Branch mit `git push -u` veröffentlichen.
7. THE Commit_Gate SHALL Änderungen ausschließlich über einen Pull Request nach `main` überführen.
8. THE Commit_Gate SHALL genau 0 direkte Pushes nach `main` durchführen.
9. THE Commit_Gate SHALL eine lineare Historie erhalten.
10. WHEN eine Aufgabe eine `git`-Operation erfordert, THE Commit_Gate SHALL die in dieser Umgebung notwendige Benutzerfreigabe anfordern und die Aufgabe als blockiert melden, solange die Freigabe fehlt.
11. THE Commit_Gate SHALL genau 0 Sammelcommits erzeugen, die mehrere abgeschlossene Aufgaben zusammenfassen.

### Anforderung 21: Repository-Bereinigung und Projektdateien

**User Story:** Als Maintainer möchte ich, dass das Repository keine Build-Ergebnisse und keine ins Leere zeigenden Verweise enthält, damit ein Neueinsteiger sofort arbeiten kann.

#### Acceptance Criteria

1. THE Git_Repository SHALL `*.exe` in `.gitignore` ausschließen.
2. WHEN `freeloader-desktop.exe` oder `uninstall.exe` versioniert sind, THE Git_Repository SHALL diese beiden Dateien aus der Versionierung entfernen und im Arbeitsverzeichnis belassen.
3. THE Git_Repository SHALL die in `deny.toml` referenzierten Dokumente `docs/adr/0002-rustls-only.md` und `docs/adr/0006-dependency-licence-policy.md` bereitstellen.
4. THE Git_Repository SHALL eine Architekturentscheidung dokumentieren, die NSIS als einziges Windows-Bundle-Format mit `aarch64`-Unterstützung begründet.
5. THE Git_Repository SHALL in `pnpm-workspace.yaml` ausschließlich Pakete auflisten, die eine `package.json` besitzen.
6. WHEN `extensions/chromium` und `extensions/firefox` als pnpm-Pakete geführt werden, THE Git_Repository SHALL für `extensions/chromium` und für `extensions/firefox` je eine `package.json` bereitstellen.
7. THE Git_Repository SHALL `docs/implementation-status.md` als gültiges Markdown mit echten Zeilenumbrüchen bereitstellen.
8. THE Git_Repository SHALL `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, `CHANGELOG.md`, `.github/ISSUE_TEMPLATE/`, `.github/PULL_REQUEST_TEMPLATE.md`, `.github/dependabot.yml` und `CODEOWNERS` bereitstellen.
9. THE Git_Repository SHALL in `CONTRIBUTING.md` den DCO-Sign-off, die Conventional-Commits-Konvention und das Commit_Gate beschreiben.
10. THE Git_Repository SHALL in `SECURITY.md` einen Meldeweg für Sicherheitslücken und die Regel benennen, dass DRM-Umgehung und Schrankenumgehung außerhalb des Projektumfangs liegen.

### Anforderung 22: Supply-Chain- und Sicherheitsprüfungen

**User Story:** Als Maintainer möchte ich verwundbare oder unpassend lizenzierte Abhängigkeiten und versehentlich eingecheckte Geheimnisse automatisch erkennen.

#### Acceptance Criteria

1. WHEN die CI_Pipeline läuft, THE CI_Pipeline SHALL `cargo deny check` mit der bestehenden `deny.toml` ausführen.
2. WHEN die CI_Pipeline läuft, THE CI_Pipeline SHALL `cargo audit` ausführen.
3. WHEN die CI_Pipeline läuft, THE CI_Pipeline SHALL eine Geheimnis-Erkennung über den Verlauf des Pull Requests ausführen.
4. WHEN die CI_Pipeline läuft, THE CI_Pipeline SHALL eine statische Codeanalyse für Rust und TypeScript ausführen.
5. THE Git_Repository SHALL eine Dependabot-Konfiguration für Cargo, npm und GitHub Actions bereitstellen.
6. IF eine Abhängigkeit gegen die Lizenzrichtlinie verstößt, THEN THE CI_Pipeline SHALL fehlschlagen und die betroffene Abhängigkeit benennen.
7. WHERE eine Ausnahme von der Lizenzrichtlinie erforderlich ist, THE Git_Repository SHALL die Ausnahme in einer Architekturentscheidung unter `docs/adr/` begründen.

### Anforderung 23: Verschobene Funktionen als Nähte ohne Attrappen

**User Story:** Als Entwickler möchte ich, dass spätere Funktionen an definierten Nähten andocken, ohne dass v0.1 vorgibt, sie schon zu können.

#### Acceptance Criteria

1. THE Download_Engine SHALL `DownloadStrategy` so definieren, dass eine segmentierende Implementierung ohne Änderung der Aufrufstellen ergänzt werden kann.
2. THE Download_Engine SHALL genau eine `Rate_Limiter`-Implementierung ausliefern, die den Durchsatz unverändert durchlässt und diese Eigenschaft in ihrer Dokumentation ausweist.
3. THE Freeloader_App SHALL genau 0 Bedienelemente anzeigen, die eine nicht wirksame Bandbreitenbegrenzung suggerieren.
4. THE Freeloader_App SHALL genau 0 Bedienelemente für Cookie- oder Zugangsdatenweitergabe anzeigen.
5. THE Update_Check_Setting SHALL persistiert werden und in v0.1 genau 0 ausgehende Requests auslösen.
6. THE Download_Engine SHALL eine Naht für Checksummen-Verifikation definieren und in v0.1 genau 0 Checksummen prüfen.
7. WHERE eine Funktion auf einen späteren Spec verschoben ist, THE Git_Repository SHALL die Verschiebung in `docs/implementation-status.md` benennen.

### Anforderung 24: Entwicklervorschau ohne Installation

**User Story:** Als Nutzer möchte ich die vollständige Oberfläche in einem gewöhnlichen Browser ansehen und bedienen können, ohne Rust zu bauen, ohne den Installer auszuführen und ohne das Erststart-Setup abzuschließen, damit ich Gestaltung, Abläufe und Zustände sofort beurteilen kann.

**Begründung:** Das ist heute unmöglich. `apps/desktop/src/main.tsx` ruft `invoke` aus `@tauri-apps/api/core` auf. Außerhalb einer Tauri-Webview existiert die injizierte Brücke nicht, der Aufruf wirft, und `pnpm dev` bricht bei der ersten Interaktion ab. Damit ist die Oberfläche nur nach einem vollständigen Rust-Build begutachtbar, was die Rückkopplung zwischen Entscheidung und Beobachtung unnötig auf Minuten streckt. Der Mock_Ipc schließt diese Lücke an der Adaptergrenze, nicht in den Komponenten, damit der Produktivpfad unberührt bleibt.

#### Acceptance Criteria

1. WHEN das Frontend außerhalb einer Tauri-Webview ausgeführt wird, THE Mock_Ipc SHALL jeden `invoke`- und `listen`-Aufruf gegen das Fake_Engine auflösen, sodass genau 0 Aufrufe eine Ausnahme werfen.
2. WHEN das Frontend innerhalb einer Tauri-Webview ausgeführt wird, THE Frontend SHALL ausschließlich den Tauri_Adapter verwenden und genau 0 Antworten aus dem Fake_Engine beziehen.
3. WHEN die Laufzeitumgebung bestimmt wird, THE Mock_Ipc SHALL ausschließlich die Anwesenheit des von Tauri injizierten globalen Objekts prüfen und genau 0 Auswertungen des User-Agent-Strings durchführen.
4. THE Frontend SHALL über genau einen in `docs/development.md` dokumentierten Befehl in einem gewöhnlichen Browser startbar sein.
5. WHEN das Frontend über diesen Befehl im Browser läuft, THE Frontend SHALL jeden Bildschirm der Anwendung erreichbar machen, einschließlich First_Run_Assistant, Downloadliste, Dialog zum Hinzufügen eines Downloads und Einstellungsoberfläche.
6. THE Fake_Engine SHALL den Lebenszyklus eines Transfers mit Fortschrittsticks, Pause, Fortsetzen, Fortsetzen nach Anwendungsneustart, Überlauf in die Warteschlange, Fehlschlag mit wiederholbarem Fehler und Fertigstellung nachbilden.
7. THE Fake_Engine SHALL jeden Zeitverlauf aus einer deterministischen, mit einem festen Startwert initialisierten Zeitquelle ableiten.
8. WHEN das Fake_Engine zweimal mit demselben Startwert ausgeführt wird, THE Fake_Engine SHALL dieselbe Folge von Zustandswechseln und Fortschrittswerten erzeugen.
9. THE Fake_Engine SHALL genau 0 Netzwerkverbindungen öffnen und genau 0 Dateien schreiben.
10. THE Mock_Ipc SHALL ausschließlich Antworten liefern, die dieselben `zod`-Schemata der Adaptergrenze aus Anforderung 15.3 erfüllen wie die Antworten des Tauri_Adapter.
11. THE Dev_Gallery SHALL über eine eigene Route erreichbar sein, die genau 0 Produktivbildschirme ersetzt oder überlagert.
12. THE Dev_Gallery SHALL Leerzustand, Ladeskelett, Fehlerzustand, alle neun Zustände der Zustandsmaschine, jeden Schritt des First_Run_Assistant sowie das helle und das dunkle Thema nebeneinander darstellen.
13. THE Freeloader_App SHALL Mock_Ipc, Fake_Engine und Dev_Gallery genau 0 Mal in das Produktions-Bundle aufnehmen.
14. WHEN die CI_Pipeline das Produktions-Bundle prüft, THE CI_Pipeline SHALL genau 0 Vorkommen der Kennungen von Mock_Ipc, Fake_Engine und Dev_Gallery im Build-Ergebnis feststellen und bei jedem gefundenen Vorkommen fehlschlagen.
15. WHERE der Vite-Entwicklungsserver für die Entwicklervorschau verwendet wird, THE Git_Repository SHALL ihn ausschließlich als Dev-Dependency führen; die Invariante „genau 0 lauschende Sockets" aus Anforderung 17.1 gilt ausschließlich für das ausgelieferte Binary und wird durch den Entwicklungsserver so wenig verletzt wie durch den Testserver aus Anforderung 19.2.
16. THE Fake_Engine SHALL genau 0 Bedienelemente freischalten, die in der Freeloader_App nicht vorhanden sind, damit die Vorschau keine Funktion suggeriert, die Anforderung 23 als verschoben ausweist.
17. WHEN der in der Dev_Gallery angebotene simulierte Anwendungsneustart ausgelöst wird, THE Fake_Engine SHALL alle Laufzeitobjekte verwerfen und über demselben speicherinternen Zustand neu aufbauen, sodass jeder unvollständige Transfer erneut mit Status `paused`, seinem zuletzt bestätigten Offset und seiner bekannten Gesamtgröße erscheint, entsprechend Anforderung 19.3.
18. IF das Frontend ein Kommando aufruft, für das das Fake_Engine keine Antwort besitzt, THEN THE Mock_Ipc SHALL ein strukturiertes Fehlerobjekt mit stabilem Code nach Anforderung 13.5 zurückgeben, den Zustand unverändert lassen und genau 0 Ausnahmen werfen.
19. WHEN die von `listen` zurückgegebene Abmeldefunktion aufgerufen wird, THE Mock_Ipc SHALL die Auslieferung an diesen Abonnenten einstellen und genau 0 weitere Ereignisse an ihn übergeben.

### Anforderung 25: UI-Qualität und Barrierefreiheit

**User Story:** Als Nutzer möchte ich eine Oberfläche, die konsistent gestaltet, in beiden Themen vollständig, mit der Tastatur bedienbar und mit einem Screenreader benutzbar ist, damit die Anwendung nicht nur funktioniert, sondern auch benutzbar ist.

**Begründung:** In einer früheren Klärungsrunde wurden Tailwind v4 und shadcn/ui gestrichen. Mit ihnen ist die gesamte Gestaltungs- und Barrierefreiheitssubstanz aus dem Spec gefallen; übrig blieb allein Kriterium 15.11, was für einen Anspruch auf gute Praxis deutlich zu dünn ist. Tailwind und shadcn/ui bleiben gestrichen — diese Entscheidung gilt weiter, weil sie eine Abhängigkeit und einen zusätzlichen Build-Schritt spart. Die Substanz kehrt stattdessen über tokenisiertes reines CSS zurück, das dasselbe Ergebnis ohne Abhängigkeit erreicht. Diese Abweichung ist bewusst getroffen und vom Nutzer freigegeben.

#### Acceptance Criteria

1. THE Token_Layer SHALL jede Farbe, jeden Radius, jede Abstandsstufe, jede Dauer und jede Beschleunigungskurve als semantisches Token über CSS Custom Properties innerhalb von `@layer` definieren, zum Beispiel `--color-surface-raised`, `--color-status-failed` und `--duration-fast`.
2. THE Frontend SHALL außerhalb der Token-Definitionen genau 0 rohe Hex-Farbwerte enthalten.
3. THE Frontend SHALL außerhalb der Token-Definitionen genau 0 fest verdrahtete Pixelwerte enthalten.
4. THE Token_Layer SHALL jede Abstandsstufe als Vielfaches von 4 px definieren.
5. THE Token_Layer SHALL ein vollständiges helles und ein vollständiges dunkles Thema als gleichrangige Themen bereitstellen, in denen jedes Token einen Wert besitzt.
6. WHEN kein manuell gewähltes Thema persistiert ist, THE Frontend SHALL das Thema aus `prefers-color-scheme` übernehmen.
7. WHEN der Nutzer ein Thema manuell wählt, THE Frontend SHALL die Auswahl persistieren und beim nächsten Start anwenden.
8. WHERE `forced-colors: active` gilt, THE Frontend SHALL die Systemfarben übernehmen und jede Statusinformation ohne eigene Farbwerte erkennbar halten.
9. WHEN die Verification_Suite läuft, THE Accessibility_Gate SHALL WCAG 2.2 Stufe AA mit `vitest-axe` maschinell prüfen und bei mindestens einer Verletzung fehlschlagen.
10. THE Accessibility_Gate SHALL das Hauptfenster, den Dialog zum Hinzufügen eines Downloads, den First_Run_Assistant und die Einstellungsoberfläche prüfen.
11. IF das Accessibility_Gate mindestens eine Verletzung meldet, THEN THE Commit_Gate SHALL das Zusammenführen des Pull Requests verhindern.
12. THE Verification_Suite SHALL für jede im Token_Layer vorgesehene Paarung aus Vordergrund- und Hintergrundtoken programmatisch aus den berechneten Werten nachweisen, dass das Kontrastverhältnis für Fließtext mindestens 4,5:1 beträgt.
13. THE Verification_Suite SHALL für großen Text und für die Begrenzungen von Bedienelementen programmatisch ein Kontrastverhältnis von mindestens 3:1 nachweisen.
14. THE Frontend SHALL zwischen der Füllung und der Spur der Fortschrittsanzeige ein Kontrastverhältnis von mindestens 3:1 einhalten.
15. THE Frontend SHALL jeden Status gleichzeitig über Farbe, Symbol und Textbezeichnung ausdrücken und genau 0 Status ausschließlich über Farbe unterscheiden.
16. THE Frontend SHALL jede Aktion allein mit der Tastatur erreichbar und auslösbar machen.
17. WHEN ein modaler Dialog geöffnet wird, THE Frontend SHALL den Fokus innerhalb des Dialogs halten.
18. WHEN ein modaler Dialog geschlossen wird, THE Frontend SHALL den Fokus auf das auslösende Bedienelement zurückstellen.
19. THE Frontend SHALL für jedes interaktive Element einen sichtbaren `:focus-visible`-Ring darstellen und genau 0 Verwendungen von `outline: none` ohne gleichwertigen sichtbaren Ersatz enthalten.
20. WHILE ein Element den Fokus besitzt, THE Frontend SHALL es weder durch die feststehende Werkzeugleiste noch durch die Statusleiste verdecken.
21. THE Frontend SHALL für jedes interaktive Ziel eine Trefffläche von mindestens 24 × 24 CSS-Pixeln bereitstellen oder einen Abstand einhalten, der die gleichwertige Ausnahme von WCAG 2.2 Erfolgskriterium 2.5.8 erfüllt.
22. THE Frontend SHALL `aria-live="polite"` ausschließlich für Fertigstellung und Fehlschlag verwenden und genau 0 Fortschrittsticks über eine Live-Region ankündigen.
23. THE Frontend SHALL für jedes Bedienelement eine korrekte Rolle und einen zugänglichen Namen vergeben.
24. THE Frontend SHALL Größen-, Geschwindigkeits- und Restzeitangaben mit Tabellenziffern über `font-variant-numeric: tabular-nums` darstellen, damit die Ziffern nicht springen.
25. WHERE `prefers-reduced-motion: reduce` gilt, THE Frontend SHALL jede nicht wesentliche Übergangs- und Bewegungsanimation abschalten.
26. WHEN die Verification_Suite läuft, THE Verification_Suite SHALL die Wirkung von `prefers-reduced-motion: reduce` automatisiert prüfen.
27. THE Frontend SHALL ausschließlich `transform` und `opacity` animieren und auf einem heißen Pfad genau 0 Animationen von `width`, `height`, `top`, `left`, `box-shadow` oder `filter` enthalten.
28. WHEN ein Fortschrittsereignis eintrifft, THE Frontend SHALL die Breite der Fortschrittsanzeige über eine CSS Custom Property setzen und genau 0 Neuzeichnungen der Listenzeile je Ereignis auslösen.
29. THE Frontend SHALL die Downloadliste entweder mit vollständiger Tabellensemantik einschließlich `columnheader` oder als Listenmuster auszeichnen und genau 0 Mischformen aus `role="table"` mit `role="row"` und `role="cell"` ohne `columnheader` verwenden; die derzeitige Auszeichnung in `apps/desktop/src/main.tsx` ist genau diese ungültige Mischform und wird ersetzt.
30. THE Git_Repository SHALL die bewusste Abweichung — Verzicht auf Tailwind v4 und shadcn/ui bei gleichzeitiger Rückkehr der Gestaltungssubstanz über den Token_Layer — in einer Architekturentscheidung unter `docs/adr/` begründen.
31. THE Token_Layer SHALL jede zulässige Paarung aus Vordergrund- und Hintergrundtoken maschinenlesbar deklarieren, sodass die Verification_Suite alle Paarungen ohne manuelle Auswahl aufzählen kann.
32. WHEN die CI_Pipeline läuft, THE CI_Pipeline SHALL die Stilquellen des Frontend maschinell auf rohe Farbwerte, auf Pixellängen außerhalb der Token-Definitionen, auf `outline: none` ohne sichtbaren Ersatz und auf Animationen von `width`, `height`, `top`, `left`, `box-shadow` und `filter` prüfen und bei jedem Treffer mit Datei und Zeile fehlschlagen.
33. WHEN `Escape` in einem offenen modalen Dialog gedrückt wird, THE Frontend SHALL den Dialog schließen, genau 0 seiner Eingaben übernehmen und den Fokus auf das auslösende Bedienelement zurückstellen.

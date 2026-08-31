//! German. Reviewed by a @maggo83: see
//! [`super::Lang::reviewed`].

use super::{Plurals, Strings};

pub static STRINGS: Strings = Strings {
    page_title: "Outofband: Übertragung direkt an einen Miner",
    lang_picker_label: "Sprache",
    ai_notice: "Diese Übersetzung stammt ursprünglich von einer KI und wurde inzwischen von einem \
        Muttersprachler geprüft. Wo sie vom Englischen abweicht, gilt das Englische.",
    ai_notice_action: "Read in English",

    disclosure_label: "Sicherheitshinweis",
    disclosure_link: "Coldcard-RNG-Schwachstelle, und warum du dieses Tool brauchen könntest",

    context_audience: "Dieses Tool ist nur für bestimmte Nutzer wichtig: vor allem für \
        *Liana*, *Miniscript* oder bestimmte *Multisig*-Wallets.",
    context_not_recommended: "Für akut gefährdete Wallets wird dieses Werkzeug !NICHT! empfohlen, \
        wie in [diesem Blogbeitrag ›](https://wizardsardine.com/blog/coldcard-rng-vulnerability/) \
        beschrieben",

    hero_line_one: "Lassen deine Transaktionen",
    hero_line_two: "direkt minen, ohne den öffentlichen Mempool.",

    fee_eyebrow: "Mindestens akzeptierte Fee rate",
    fee_sentence: "Alles unterhalb dieser Fee rate wird voraussichtlich nicht gemined. Sie zu \
        überschreiten bietet aber auch keine Garantie.",
    fee_stale: "Diese Fee rate ist möglicherweise veraltet.",
    fee_advice: "Dies ist eine *dynamische* Untergrenze. Sie bewegt sich mit der Nachfrage: Erzeuge deine \
        Transaktion daher mit einer *deutlich höheren* Fee rate als der Untergrenze, sonst \
        wird sie womöglich nicht mehr, inkludiert.",

    prepare_heading: "Transaktion vorbereiten",
    prepare_liana: "Liana-Nutzer bereiten ihre Transaktion wie gewohnt vor und signieren sie, \
        drücken aber !AUF KEINEN FALL! die Schaltfläche *Broadcast*. Klicke nach dem \
        Signieren stattdessen auf *Export*. Die so erstellte Datei lädst du im nächsten Schritt hoch.",
    prepare_drafts: "Falls du die PSBT-Datei bereits signiert aber nicht gespeichert hast, \
        findest du die Transaktion unter *Drafts and Approvals* wieder.",
    prepare_other_wallets: "Erstelle und signiere deine Transaktion wie gewohnt, aber \
        *mache keinen Broadcast ins Bitcoin-Netzwerk*. Nicht jede Software erlaubt das Signieren \
        ohne Broadcast; lies im Zweifel vor dem Signieren die Dokumentation der Software. Liana-Nutzer \
        lesen einfach die Karte oben.",
    guides_heading: "Anleitungen",
    guides_multilingual: "Mehrere Sprachen",

    load_heading: "Transaktionen laden",
    load_blurb: "Füge signierte PSBTs und rohe Transaktionen ein oder ziehe sie \
        hierher. Dein Browser sendet jede einzelne direkt an MARA Slipstream, wo sie gemined \
        werden, ohne den öffentlichen Mempool je zu berühren.",
    load_placeholder_or: "oder",
    load_placeholder_drop: "oder Datei hierher ziehen",
    load_accepts: "Dateien, Ordner oder Archive: .txt .psbt .txn .tar .tar.gz .zip",
    btn_add_to_queue: "Zur Warteschlange",
    btn_choose_files: "Dateien wählen",
    btn_clear_queue: "Warteschlange leeren",
    parse_error_title: "Nicht lesbar",
    detected_psbt: "ERKANNT · PSBT",
    detected_raw: "ERKANNT · ROHE TRANSAKTION",
    detected_unrecognised: "FORMAT NICHT ERKANNT",

    stat_in_queue: "In Warteschlange",
    stat_clear_floor: "Über Untergrenze",
    stat_below_floor: "Unter Untergrenze",
    stat_fee_unknown: "Gebühr unbekannt",
    btn_send: "Senden",
    btn_send_batch: "Stapel senden ({n})",
    btn_sending: "Wird gesendet…",

    col_source: "Quelle",
    col_format: "Format",
    col_vsize: "vsize",
    col_fee_rate: "Fee rate",
    col_status: "Status",
    btn_retry: "Erneut",
    row_remove_title: "Aus der Warteschlange entfernen",
    row_missing_value: "Die Eingangsbeträge dieser Transaktion sind nicht bekannt, daher lässt \
        sich die Gebühr lokal nicht berechnen. Gib den insgesamt ausgegebenen Betrag ein, \
        um sie vor dem Senden zu prüfen.",
    row_total_placeholder: "Gesamter Eingangsbetrag (sats)",

    status_invalid: "Ungültig",
    status_accepted: "Angenommen",
    status_sending: "Wird gesendet…",
    status_rejected: "Abgelehnt",
    status_rate_limited: "Gedrosselt",
    status_failed: "Fehlgeschlagen",
    status_fee_unknown: "Gebühr unbekannt",
    status_ready: "Bereit",
    status_below_floor: "Unter Untergrenze",

    modal_title: "PSBT kann nicht finalisiert werden",
    modal_instruction: "Korrigiere die PSBT und lade sie erneut.",
    btn_close: "Schließen",

    faq_eyebrow: "Fragen, die du vor Benutzung stellen solltest",
    faq_q_why: "Wofür ist dieses Tool gut?",
    faq_a_why: "Ein normaler Broadcast verteilt deine Transaktion an jeden Bitcoin-Knoten im \
        Netzwerk, bevor sie geschürft wird. Wenn jemand einen Schlüssel besitzt, der dieselben \
        Coins ausgeben kann, kann diese Person die Transaktion ersetzen und die Coins stehlen, \
        bevor sie gemined wird. Slipstream (welches dieses Werkzeug nutzt) überspringt diese \
        Verbreitung: Die Transaktion geht an einen einzigen Miner, ohne an den Rest des Netzwerks \
        gesendet zu werden. Das Schürfen dauert länger, dafür bist du vor dem Ersetzen der \
        Transaktion sicher.",
    faq_q_where: "Wohin geht das, was ich einfüge, tatsächlich?",
    faq_a_where: "Dekodieren, Finalisieren und Gebührenberechnung laufen alle in deinem Browser, \
        und diese Seite hat keinen Server dazwischen. Zwei Dinge verlassen diese Seite: die \
        finalisierte Transaktion als Hex, direkt von deinem Browser an MARA Slipstream, wenn du \
        auf Senden drückst, und ein anonymer Seitenaufruf, gezählt von Plausible. Plausible setzt \
        keine Cookies und erfasst nichts, was dich identifiziert, sieht aber wie MARA die \
        IP-Adresse, von der aus du dich verbindest. Nutzen Tor oder ein VPN, wenn dir das \
        wichtig ist. Was du einfügen, ist nie Teil von beidem.",
    faq_q_mara_learns: "Was erfährt MARA über mich?",
    faq_a_mara_learns: "Die Transaktionsdaten und deine IP-Adresse. Dein Browser spricht direkt mit \
        MARA, die Verbindung ist also deine, und MARA sieht die Adresse, von der aus du surfst. \
        MARA erfährt weder deine PSBT-Metadaten noch deine xpubs, deinen Descriptor oder welche \
        anderen Transaktionen du hier eingereiht hast. Nutze Tor oder ein VPN, wenn es \
        dir wichtig ist, dass MARA deine IP nicht sieht.",
    faq_q_guaranteed: "Wird meine Transaktion garantiert gemined?",
    faq_a_guaranteed: "Nein. Von Slipstream angenommen heißt nicht bestätigt. MARA mined nur \
        einen Teil der Blöcke, nicht alle, und kann deine Transaktion aus Gründen verwerfen, die \
        es nicht erklären muss. Betrachte das als bessere Chance, nicht als Zusage. Wenn sie nach \
        einigen Stunden nicht bestätigt ist, versuche es erneut.",
    faq_q_rate_differs: "Warum weicht die Fee rate von Mempool.space ab?",
    faq_a_rate_differs: "Slipstream wird über eine höhere Fee rate für den Dienst bezahlt. \
        Diese Website erhält davon keinen Anteil und keinerlei Vergütung.",
    faq_q_cpfp: "Kann eine zweite Transaktion die Gebühr für eine günstige bezahlen (CPFP)?",
    faq_a_cpfp: "Nein. Slipstream betrachtet jede Transaktion für sich, eine zu günstige wird \
        hier also nicht gemined, auch wenn du eine zweite mit höherer Gebühr hinterherschickst. \
        Du kannst weiterhin mehrere zusammenhängende Transaktionen senden, solange jede für sich \
        genug zahlt: Benenne die Dateien 01, 02, 03, dann gehen sie in dieser Reihenfolge \
        raus. Wenn eine Transaktion für eine andere zahlen muss, wende dich direkt an MARA.",
    faq_q_risks: "Welche Risiken hat dieser Dienst?",
    faq_a_risks: "MARA, dem Unternehmen hinter Slipstream, muss man vertrauen, den Angriff nicht \
        selbst auf deine Transaktion auszuführen. Das ist ein vertretbares Risiko verglichen \
        damit, sie öffentlich zu übertragen und JEDEM den Angriff zu ermöglichen.",
    faq_q_domain: "Warum liegt diese Seite auf der Wizardsardine-Domain?",
    faq_a_domain: "Wir (Wizardsardine) sind ein Sicherheitsunternehmen und betreuen die \
        Liana-Wallet ([lianawallet.com](https://lianawallet.com)). Liana-Nutzer hatten vor diesem \
        Werkzeug keinen einfachen Weg, an Slipstream zu senden; es ist also ein Dienst für sie, \
        offen für alle anderen.",

    footer_built_by: "Gebaut von [Wizardsardine](https://wizardsardine.com)",
    footer_source: "Quellcode",
    footer_disclosure: "Sicherheitshinweis",
    footer_slipstream_terms: "Slipstream-Bedingungen",

    msg_nothing_pasted: "Noch nichts eingefügt.",
    msg_unrecognised: "Keine gültigen base64-PSBT- oder Hex-Transaktionsdaten.",
    msg_too_large: "Transaktion hat {bytes} Bytes als Hex und überschreitet das Limit von \
        {limit} MiB.",
    msg_no_inputs: "Transaktion hat keine Eingänge.",
    msg_no_outputs: "Transaktion hat keine Ausgänge.",
    msg_rate_limited_note: "Slipstream drosselt diesen Browser. Warte einige Minuten und \
        sende erneut.",
    msg_retrying_now: "Gedrosselt, neuer Versuch jetzt…",
    msg_unreachable: "MARA Slipstream nicht erreichbar. Prüfe deine Verbindung, oder ob ein \
        VPN, ein Proxy oder eine Browser-Erweiterung slipstream.mara.com blockiert.",
    msg_unexpected_response: "Unerwartete Antwort von Slipstream (HTTP {status})",

    msg_missing_utxo: Plurals::two(
        "UTXO-Daten für {n} Eingang fehlen. Gebühr und finale Skripte konnten nicht geprüft \
         werden.",
        "UTXO-Daten für {n} Eingänge fehlen. Gebühr und finale Skripte konnten nicht geprüft \
         werden.",
    ),
    msg_retrying_in: Plurals::two(
        "Gedrosselt, neuer Versuch in {n} Sekunde…",
        "Gedrosselt, neuer Versuch in {n} Sekunden…",
    ),
    msg_duplicate: Plurals::two(
        "Diese Transaktion ist bereits in der Warteschlange.",
        "Diese Transaktionen sind bereits in der Warteschlange.",
    ),
};

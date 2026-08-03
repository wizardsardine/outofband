//! German. Drafted, not yet reviewed by a native speaker: see
//! [`super::Lang::reviewed`].

use super::{Plurals, Strings};

pub static STRINGS: Strings = Strings {
    page_title: "Outofband: Übertragung direkt an den Miner",
    lang_picker_label: "Sprache",

    disclosure_label: "Sicherheitshinweis",
    disclosure_link: "Coldcard-RNG-Schwachstelle, und warum Sie dieses Werkzeug brauchen könnten",

    context_audience: "Dieses Werkzeug ist nur für bestimmte Nutzer wichtig: vor allem für \
        *Liana*, *Miniscript* oder bestimmte *Multisig*-Wallets.",
    context_not_recommended: "Für akut gefährdete Wallets wird dieses Werkzeug !NICHT! empfohlen, \
        wie in [diesem Blogbeitrag ›](https://wizardsardine.com/blog/coldcard-rng-vulnerability/) \
        beschrieben",

    hero_line_one: "Lassen Sie Ihre Transaktionen",
    hero_line_two: "direkt schürfen, ohne den öffentlichen Mempool.",

    fee_eyebrow: "Mindestens akzeptierte Gebührenrate",
    fee_sentence: "Alles unterhalb dieser Rate wird voraussichtlich nicht geschürft. Sie zu \
        erreichen ist aber auch keine Garantie.",
    fee_stale: "Diese Rate ist möglicherweise veraltet.",
    fee_advice: "Diese Untergrenze ist *dynamisch*. Sie bewegt sich mit der Nachfrage: Bauen Sie \
        Ihre Transaktion daher mit einer *deutlich höheren* Rate als der Untergrenze, sonst \
        erreicht sie diese womöglich nicht mehr, bevor sie geschürft wird.",

    prepare_heading: "Transaktion vorbereiten",
    prepare_liana: "Liana-Nutzer bereiten ihre Transaktion wie gewohnt vor und signieren sie, \
        drücken aber !AUF KEINEN FALL! die Schaltfläche zum Senden. Klicken Sie nach dem \
        Signieren stattdessen auf *Export*. Diese Datei laden Sie im nächsten Schritt.",
    prepare_drafts: "Falls Sie bereits signiert, die PSBT-Datei aber nicht gespeichert haben, \
        finden Sie die Transaktion unter *Drafts and Approvals* wieder.",
    prepare_other_wallets: "Erstellen und signieren Sie Ihre Transaktion wie gewohnt, aber \
        *senden Sie sie nicht an das Bitcoin-Netzwerk*. Nicht jede Software erlaubt das Signieren \
        ohne Senden; lesen Sie im Zweifel vor dem Signieren deren Dokumentation. Liana-Nutzer \
        lesen einfach die Karte oben.",
    prepare_tutorials: "Sobald sie fertig sind, werden hier Links zu Anleitungen von Lehrenden \
        ergänzt. Wenn Sie unsicher sind, wie das mit Ihrer Wallet geht, können Sie warten und \
        später wieder hier nachsehen.",

    load_heading: "Transaktionen laden",
    load_blurb: "Fügen Sie signierte PSBTs und rohe Transaktionen ein oder ziehen Sie sie \
        hierher. Ihr Browser sendet jede einzelne direkt an MARA Slipstream, das sie schürft, \
        ohne den öffentlichen Mempool je zu berühren.",
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
    col_fee_rate: "Gebührenrate",
    col_status: "Status",
    btn_retry: "Erneut",
    row_remove_title: "Aus der Warteschlange entfernen",
    row_missing_value: "Die Eingangsbeträge dieser Transaktion sind nicht bekannt, daher lässt \
        sich die Gebühr lokal nicht berechnen. Geben Sie den insgesamt ausgegebenen Betrag ein, \
        um sie vor dem Senden zu prüfen.",
    row_total_placeholder: "Gesamter Eingangsbetrag (sats)",
    row_check_fee: "Gebühr prüfen",

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
    modal_instruction: "Korrigieren Sie die PSBT und laden Sie sie erneut.",
    btn_close: "Schließen",

    faq_eyebrow: "Fragen, die Sie vorher stellen sollten",
    faq_q_why: "Wofür ist dieses Werkzeug gut?",
    faq_a_why: "Eine normale Übertragung verteilt Ihre Transaktion an jeden Bitcoin-Knoten im \
        Netzwerk, bevor sie geschürft wird. Wenn jemand einen Schlüssel besitzt, der dieselben \
        Coins ausgeben kann, kann diese Person die Transaktion ersetzen und die Coins stehlen, \
        bevor sie geschürft wird. Slipstream (was dieses Werkzeug nutzt) überspringt diese \
        Verteilung: Die Transaktion geht an einen einzigen Miner, ohne an den Rest des Netzwerks \
        gesendet zu werden. Das Schürfen dauert länger, dafür sind Sie vor dem Ersetzen der \
        Transaktion sicher.",
    faq_q_where: "Wohin geht das, was ich einfüge, tatsächlich?",
    faq_a_where: "Dekodieren, Finalisieren und Gebührenberechnung laufen alle in Ihrem Browser, \
        und diese Seite hat keinen Server dazwischen. Das Einzige, was diese Seite (und Ihren \
        Rechner) je verlässt, ist die finalisierte Transaktion als Hex, direkt von Ihrem Browser \
        an MARA Slipstream, wenn Sie auf Senden drücken.",
    faq_q_mara_learns: "Was erfährt MARA über mich?",
    faq_a_mara_learns: "Die Transaktionsdaten und Ihre IP-Adresse. Ihr Browser spricht direkt mit \
        MARA, die Verbindung ist also Ihre, und MARA sieht die Adresse, von der aus Sie surfen. \
        MARA erfährt weder Ihre PSBT-Metadaten noch Ihre xpubs, Ihren Descriptor oder welche \
        anderen Transaktionen Sie hier eingereiht haben. Nutzen Sie Tor oder ein VPN, wenn es \
        Ihnen wichtig ist, dass MARA Ihre IP nicht sieht.",
    faq_q_guaranteed: "Wird meine Transaktion garantiert geschürft?",
    faq_a_guaranteed: "Nein. Von Slipstream angenommen heißt nicht bestätigt. MARA schürft nur \
        einen Teil der Blöcke, nicht alle, und kann Ihre Transaktion aus Gründen verwerfen, die \
        es nicht erklären muss. Sehen Sie das als bessere Chance, nicht als Zusage. Wenn sie nach \
        einigen Stunden nicht bestätigt ist, versuchen Sie es erneut.",
    faq_q_rate_differs: "Warum weicht die Gebührenrate von Mempool.space ab?",
    faq_a_rate_differs: "Slipstream wird über eine höhere Gebührenrate für den Dienst bezahlt. \
        Diese Website erhält davon keinen Anteil und keinerlei Vergütung.",
    faq_q_cpfp: "Kann eine zweite Transaktion die Gebühr für eine günstige bezahlen (CPFP)?",
    faq_a_cpfp: "Nein. Slipstream betrachtet jede Transaktion für sich, eine zu günstige wird \
        hier also nicht geschürft, auch wenn Sie eine zweite mit höherer Gebühr hinterherschicken. \
        Sie können weiterhin mehrere zusammenhängende Transaktionen senden, solange jede für sich \
        genug zahlt: Benennen Sie die Dateien 01, 02, 03, dann gehen sie in dieser Reihenfolge \
        raus. Wenn eine Transaktion für eine andere zahlen muss, wenden Sie sich direkt an MARA.",
    faq_q_risks: "Welche Risiken hat dieser Dienst?",
    faq_a_risks: "MARA, dem Unternehmen hinter Slipstream, muss man zutrauen, den Angriff nicht \
        selbst auf Ihre Transaktion auszuführen. Das ist ein vertretbares Risiko verglichen \
        damit, sie öffentlich zu übertragen und JEDEM den Angriff zu ermöglichen.",
    faq_q_domain: "Warum liegt diese Seite auf der Wizardsardine-Domain?",
    faq_a_domain: "Wir (Wizardsardine) sind ein Sicherheitsunternehmen und betreuen die \
        Liana-Wallet ([lianawallet.com](https://lianawallet.com)). Liana-Nutzer hatten vor diesem \
        Werkzeug keinen einfachen Weg, an Slipstream zu senden; es ist also ein Dienst für sie, \
        offen für alle anderen.",

    footer_built_by: "Gebaut von [Wizardsardine](https://wizardsardine.com)",
    footer_disclosure: "Sicherheitshinweis",
    footer_slipstream_terms: "Slipstream-Bedingungen",

    msg_nothing_pasted: "Noch nichts eingefügt.",
    msg_unrecognised: "Keine gültigen base64-PSBT- oder Hex-Transaktionsdaten.",
    msg_too_large: "Transaktion hat {bytes} Bytes als Hex und überschreitet das Limit von \
        {limit} MiB.",
    msg_no_inputs: "Transaktion hat keine Eingänge.",
    msg_no_outputs: "Transaktion hat keine Ausgänge.",
    msg_not_finalizable: "PSBT kann nicht finalisiert werden.",
    msg_rate_limited_note: "Slipstream drosselt diesen Browser. Warten Sie einige Minuten und \
        senden Sie erneut.",
    msg_retrying_now: "Gedrosselt, neuer Versuch jetzt…",
    msg_unreachable: "MARA Slipstream nicht erreichbar. Prüfen Sie Ihre Verbindung, oder ob ein \
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

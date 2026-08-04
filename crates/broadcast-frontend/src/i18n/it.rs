//! Italian. Drafted, not yet reviewed by a native speaker: see
//! [`super::Lang::reviewed`].

use super::{Plurals, Strings};

pub static STRINGS: Strings = Strings {
    page_title: "Outofband: Broadcast diretto al miner",
    lang_picker_label: "Lingua",
    ai_notice: "Questa traduzione è stata prodotta da un'IA e non è stata rivista da un \
        madrelingua. Dove differisce dall'inglese, vale l'inglese.",
    ai_notice_action: "Read in English",

    disclosure_label: "Avviso di sicurezza",
    disclosure_link: "Vulnerabilità dell'RNG Coldcard, e perché questo strumento potrebbe \
        servirti",

    context_audience: "Questo strumento serve solo ad alcuni profili: soprattutto i wallet \
        *Liana*, *Miniscript*, o certi tipi di *multisig*.",
    context_not_recommended: "Questo strumento !NON! è consigliato per wallet a rischio critico, \
        come spiega [questo articolo del blog \
        ›](https://wizardsardine.com/blog/coldcard-rng-vulnerability/)",

    hero_line_one: "Fai minare le tue transazioni",
    hero_line_two: "direttamente, senza passare dal mempool pubblico.",

    fee_eyebrow: "Fee rate minima accettata",
    fee_sentence: "Sotto questa Fee rate una transazione non ha motivo di essere minata. \
        Superarla non è comunque una garanzia.",
    fee_stale: "Questa Fee rate potrebbe non essere aggiornata.",
    fee_advice: "Questa soglia è *dinamica*. Segue la domanda, quindi costruisci la transazione \
        con una Fee rate *decisamente più alta* della soglia, altrimenti rischia di scendere sotto \
        prima di essere minata.",

    prepare_heading: "Prepara la transazione",
    prepare_liana: "Su Liana, prepara la transazione normalmente e firmala, ma !NON! premere il \
        pulsante *Broadcast*. Una volta firmata, clicca invece su *Export*. È quel file che \
        caricherai nel passo successivo.",
    prepare_drafts: "Se l'hai già firmata senza salvare il file PSBT, ritrovi la transazione in \
        *Drafts and Approvals*.",
    prepare_other_wallets: "Costruisci e firma la transazione normalmente, ma *non fare Broadcast \
        sulla rete Bitcoin*. Non tutti i software permettono di firmare senza fare Broadcast: nel \
        dubbio, consulta la loro documentazione prima di firmare. Chi usa Liana legga il riquadro \
        qui sopra.",
    prepare_tutorials: "Aggiungeremo qui i link a tutorial di divulgatori appena saranno pronti. \
        Se non sai come farlo con il tuo wallet, puoi aspettare e ricontrollare questa pagina più \
        avanti.",

    load_heading: "Carica le transazioni",
    load_blurb: "Incolla o trascina PSBT firmate e transazioni grezze. Il browser invia ciascuna \
        direttamente a MARA Slipstream, che la mina senza mai toccare il mempool pubblico.",
    load_placeholder_or: "oppure",
    load_placeholder_drop: "oppure trascina qui un file",
    load_accepts: "File, cartelle o archivi: .txt .psbt .txn .tar .tar.gz .zip",
    btn_add_to_queue: "Aggiungi alla coda",
    btn_choose_files: "Scegli i file",
    btn_clear_queue: "Svuota la coda",
    parse_error_title: "Lettura impossibile",
    detected_psbt: "RILEVATO · PSBT",
    detected_raw: "RILEVATO · TRANSAZIONE GREZZA",
    detected_unrecognised: "FORMATO NON RICONOSCIUTO",

    stat_in_queue: "In coda",
    stat_clear_floor: "Sopra la soglia",
    stat_below_floor: "Sotto la soglia",
    stat_fee_unknown: "Fee sconosciuta",
    btn_send: "Invia",
    btn_send_batch: "Invia il lotto ({n})",
    btn_sending: "Invio…",

    col_source: "Origine",
    col_format: "Formato",
    col_vsize: "vsize",
    col_fee_rate: "Fee rate",
    col_status: "Stato",
    btn_retry: "Riprova",
    row_remove_title: "Togli dalla coda",
    row_missing_value: "Gli importi in ingresso di questa transazione non sono noti, quindi la \
        fee non può essere calcolata localmente. Inserisci il valore totale speso per \
        verificarla prima di inviare.",
    row_total_placeholder: "Valore totale in ingresso (sats)",

    status_invalid: "Non valida",
    status_accepted: "Accettata",
    status_sending: "Invio…",
    status_rejected: "Rifiutata",
    status_rate_limited: "Limitata",
    status_failed: "Fallita",
    status_fee_unknown: "Fee sconosciuta",
    status_ready: "Pronta",
    status_below_floor: "Sotto la soglia",

    modal_title: "La PSBT non può essere finalizzata",
    modal_instruction: "Correggi la PSBT e ricaricala.",
    btn_close: "Chiudi",

    faq_eyebrow: "Domande da farsi prima di usare questo strumento",
    faq_q_why: "A cosa serve questo strumento?",
    faq_a_why: "Un Broadcast normale propaga la transazione a tutti i nodi Bitcoin della rete \
        prima che venga minata. Se qualcuno possiede una chiave in grado di spendere le stesse \
        monete, può sostituire la transazione e rubarle prima che venga minata. Slipstream (ciò \
        che usa questo strumento) evita quel Broadcast: la transazione va a un solo miner, senza \
        essere inviata al resto della rete. Sarà più lenta da minare, ma sarai al riparo dalla \
        sostituzione.",
    faq_q_where: "Dove finisce davvero quello che incollo?",
    faq_a_where: "Decodifica, finalizzazione e calcolo delle fee avvengono tutti nel browser, e \
        questo sito non ha alcun server nel mezzo. Due cose lasciano questa pagina: la \
        transazione finalizzata in hex, inviata direttamente dal browser a MARA Slipstream quando \
        premi Invia, e una visita anonima conteggiata da Plausible. Plausible non usa cookie e \
        non registra nulla che ti identifichi, ma, come MARA, vede l'indirizzo IP da cui ti \
        colleghi. Usa Tor o una VPN se per te è importante. Quello che incolli non fa mai parte \
        né dell'una né dell'altra.",
    faq_q_mara_learns: "Cosa viene a sapere MARA di me?",
    faq_a_mara_learns: "I dati della transazione e il tuo indirizzo IP. Il browser parla \
        direttamente con MARA, quindi la connessione è tua e MARA vede l'indirizzo da cui navighi. \
        MARA non conosce i metadati della tua PSBT, i tuoi xpub, il tuo descriptor, né quali altre \
        transazioni hai messo in coda qui. Usa Tor o una VPN se ti importa che MARA non veda il \
        tuo IP.",
    faq_q_guaranteed: "Ho la garanzia che la mia transazione venga minata?",
    faq_a_guaranteed: "No. Accettata da Slipstream non vuol dire confermata. MARA mina solo una \
        parte dei blocchi, non tutti, e può scartare la tua transazione per motivi che non è \
        tenuta a spiegare. Consideralo una possibilità migliore, non una promessa. Se dopo \
        qualche ora non è confermata, riprova.",
    faq_q_rate_differs: "Perché la Fee rate è diversa da quella di Mempool.space?",
    faq_a_rate_differs: "Slipstream viene pagata per il servizio con una Fee rate più alta. \
        Questo sito non prende alcuna parte di quel pagamento, né alcun compenso.",
    faq_q_cpfp: "Una seconda transazione può pagare la fee di una più economica (CPFP)?",
    faq_a_cpfp: "No. Slipstream guarda ogni transazione per conto suo, quindi una che non paga \
        abbastanza non verrà minata qui, anche se ne invii un'altra che paga di più per coprirla. \
        Puoi comunque inviare più transazioni collegate, purché ognuna paghi abbastanza da sola: \
        chiama i file 01, 02, 03 e partiranno in quell'ordine. Se serve che una transazione ne \
        paghi un'altra, rivolgiti direttamente a MARA.",
    faq_q_risks: "Quali sono i rischi di questo servizio?",
    faq_a_risks: "Bisogna fidarsi che MARA, l'azienda dietro Slipstream, non esegua essa stessa \
        l'attacco sulla tua transazione. È un rischio accettabile rispetto a diffonderla \
        pubblicamente e lasciare che CHIUNQUE esegua l'attacco.",
    faq_q_domain: "Perché questa pagina sta sul dominio di Wizardsardine?",
    faq_a_domain: "Noi (Wizardsardine) siamo un'azienda di sicurezza e manteniamo il wallet Liana \
        ([lianawallet.com](https://lianawallet.com)). Prima di questo strumento gli utenti Liana \
        non avevano un modo semplice di inviare a Slipstream, quindi è un servizio per loro, \
        aperto a tutti gli altri.",

    footer_built_by: "Realizzato da [Wizardsardine](https://wizardsardine.com)",
    footer_source: "Codice sorgente",
    footer_disclosure: "Avviso di sicurezza",
    footer_slipstream_terms: "Condizioni Slipstream",

    msg_nothing_pasted: "Non hai ancora incollato nulla.",
    msg_unrecognised: "Non sono dati validi di PSBT in base64 né di transazione in hex.",
    msg_too_large: "La transazione occupa {bytes} byte in hex e supera il limite di {limit} MiB.",
    msg_no_inputs: "La transazione non ha input.",
    msg_no_outputs: "La transazione non ha output.",
    msg_rate_limited_note: "Slipstream sta limitando questo browser. Aspetta qualche minuto e \
        invia di nuovo.",
    msg_retrying_now: "Limitata, nuovo tentativo adesso…",
    msg_unreachable: "Impossibile raggiungere MARA Slipstream. Controlla la connessione, o se una \
        VPN, un proxy o un'estensione del browser sta bloccando slipstream.mara.com.",
    msg_unexpected_response: "Risposta inattesa da Slipstream (HTTP {status})",

    msg_missing_utxo: Plurals::two(
        "Mancano i dati UTXO di {n} input. Fee e script finali non sono stati validati.",
        "Mancano i dati UTXO di {n} input. Fee e script finali non sono stati validati.",
    ),
    msg_retrying_in: Plurals::two(
        "Limitata, nuovo tentativo tra {n} secondo…",
        "Limitata, nuovo tentativo tra {n} secondi…",
    ),
    msg_duplicate: Plurals::two(
        "Quella transazione è già in coda.",
        "Quelle transazioni sono già in coda.",
    ),
};

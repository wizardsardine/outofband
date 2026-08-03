//! French. Drafted, not yet reviewed by a native speaker: see
//! [`super::Lang::reviewed`]. Wizardsardine is a French company, so this is
//! the locale most likely to be reviewed first.

use super::{Plurals, Strings};

pub static STRINGS: Strings = Strings {
    page_title: "Outofband : diffusion directe au mineur",
    lang_picker_label: "Langue",

    disclosure_label: "Avis de sécurité",
    disclosure_link: "Vulnérabilité du RNG Coldcard, et pourquoi cet outil peut vous être utile",

    context_audience: "Cet outil ne concerne que certains profils : surtout les portefeuilles \
        *Liana*, *Miniscript*, ou certains types de *multisig*.",
    context_not_recommended: "Cet outil n'est !PAS! recommandé pour les portefeuilles en danger \
        critique, comme l'explique [cet article de blog \
        ›](https://wizardsardine.com/blog/coldcard-rng-vulnerability/)",

    hero_line_one: "Faites miner vos transactions",
    hero_line_two: "directement, sans passer par le mempool public.",

    fee_eyebrow: "Taux de frais minimum accepté",
    fee_sentence: "En dessous de ce taux, une transaction n'a pas vocation à être minée. Le \
        dépasser n'est pas non plus une garantie.",
    fee_stale: "Ce taux n'est peut-être plus à jour.",
    fee_advice: "Ce seuil est *dynamique*. Il suit la demande : construisez donc votre \
        transaction à un taux *nettement supérieur* au seuil, sinon elle risque de repasser en \
        dessous avant d'être minée.",

    prepare_heading: "Préparez votre transaction",
    prepare_liana: "Sous Liana, préparez votre transaction normalement et signez-la, mais \
        n'appuyez !SURTOUT PAS! sur le bouton de diffusion. Une fois signée, cliquez plutôt sur \
        *Export*. C'est ce fichier que vous chargerez à l'étape suivante.",
    prepare_drafts: "Si vous l'avez déjà signée sans enregistrer le fichier PSBT, vous \
        retrouverez la transaction dans *Drafts and Approvals*.",
    prepare_other_wallets: "Construisez et signez votre transaction normalement, mais *ne la \
        diffusez pas sur le réseau Bitcoin*. Tous les logiciels ne permettent pas de signer sans \
        diffuser : en cas de doute, consultez leur documentation avant de signer. Utilisateurs de \
        Liana, lisez simplement l'encadré ci-dessus.",
    prepare_tutorials: "Des liens vers des tutoriels de vulgarisateurs seront ajoutés ici dès \
        qu'ils seront prêts. Si vous ne savez pas comment faire avec votre portefeuille, vous \
        pouvez attendre et revenir consulter cette page plus tard.",

    load_heading: "Chargez vos transactions",
    load_blurb: "Collez ou déposez des PSBT signées et des transactions brutes. Votre navigateur \
        envoie chacune directement à MARA Slipstream, qui la mine sans jamais toucher au mempool \
        public.",
    load_placeholder_or: "ou",
    load_placeholder_drop: "ou déposez un fichier ici",
    load_accepts: "Fichiers, dossiers ou archives : .txt .psbt .txn .tar .tar.gz .zip",
    btn_add_to_queue: "Ajouter à la file",
    btn_choose_files: "Choisir des fichiers",
    btn_clear_queue: "Vider la file",
    parse_error_title: "Lecture impossible",
    detected_psbt: "DÉTECTÉ · PSBT",
    detected_raw: "DÉTECTÉ · TRANSACTION BRUTE",
    detected_unrecognised: "FORMAT NON RECONNU",

    stat_in_queue: "Dans la file",
    stat_clear_floor: "Au-dessus du seuil",
    stat_below_floor: "Sous le seuil",
    stat_fee_unknown: "Frais inconnus",
    btn_send: "Envoyer",
    btn_send_batch: "Envoyer le lot ({n})",
    btn_sending: "Envoi…",

    col_source: "Source",
    col_format: "Format",
    col_vsize: "vsize",
    col_fee_rate: "Taux de frais",
    col_status: "Statut",
    btn_retry: "Réessayer",
    row_remove_title: "Retirer de la file",
    row_missing_value: "Les montants d'entrée de cette transaction sont inconnus, les frais ne \
        peuvent donc pas être calculés localement. Saisissez le montant total dépensé pour les \
        vérifier avant l'envoi.",
    row_total_placeholder: "Montant total en entrée (sats)",

    status_invalid: "Invalide",
    status_accepted: "Acceptée",
    status_sending: "Envoi…",
    status_rejected: "Rejetée",
    status_rate_limited: "Limitée",
    status_failed: "Échec",
    status_fee_unknown: "Frais inconnus",
    status_ready: "Prête",
    status_below_floor: "Sous le seuil",

    modal_title: "La PSBT ne peut pas être finalisée",
    modal_instruction: "Corrigez la PSBT, puis chargez-la de nouveau.",
    btn_close: "Fermer",

    faq_eyebrow: "Les questions à se poser avant d'utiliser cet outil",
    faq_q_why: "À quoi sert cet outil ?",
    faq_a_why: "Une diffusion normale propage votre transaction à tous les nœuds Bitcoin du \
        réseau avant qu'elle soit minée. Si quelqu'un détient une clé capable de dépenser les \
        mêmes pièces, il peut remplacer la transaction et voler ces pièces avant qu'elle soit \
        minée. Slipstream (ce qu'utilise cet outil) évite cette propagation : la transaction va à \
        un seul mineur, sans être envoyée au reste du réseau. Elle sera plus lente à miner, mais \
        vous serez à l'abri du remplacement.",
    faq_q_where: "Où va réellement ce que je colle ?",
    faq_a_where: "Le décodage, la finalisation et le calcul des frais se font tous dans votre \
        navigateur, et ce site n'a aucun serveur intermédiaire. La seule chose qui quitte cette \
        page (et votre ordinateur) est la transaction finalisée en hexadécimal, envoyée \
        directement de votre navigateur à MARA Slipstream quand vous appuyez sur Envoyer.",
    faq_q_mara_learns: "Qu'apprend MARA à mon sujet ?",
    faq_a_mara_learns: "Le contenu de la transaction, et votre adresse IP. Votre navigateur \
        parle directement à MARA : la connexion est la vôtre et MARA voit l'adresse depuis \
        laquelle vous naviguez. MARA n'apprend ni les métadonnées de votre PSBT, ni vos xpubs, ni \
        votre descripteur, ni quelles autres transactions vous avez mises en file ici. Utilisez \
        Tor ou un VPN si vous tenez à ce que MARA ne voie pas votre IP.",
    faq_q_guaranteed: "Suis-je certain que ma transaction sera minée ?",
    faq_a_guaranteed: "Non. Acceptée par Slipstream ne veut pas dire confirmée. MARA ne mine \
        qu'une partie des blocs, et peut écarter votre transaction pour des raisons qu'il n'a pas \
        à expliquer. Voyez cela comme une meilleure chance, pas comme une promesse. Si elle n'est \
        toujours pas confirmée au bout de quelques heures, réessayez.",
    faq_q_rate_differs: "Pourquoi le taux de frais diffère-t-il de Mempool.space ?",
    faq_a_rate_differs: "Slipstream est rémunéré pour ce service par un taux de frais plus \
        élevé. Ce site ne touche aucune part de ce paiement, ni aucune compensation.",
    faq_q_cpfp: "Une seconde transaction peut-elle payer les frais d'une première (CPFP) ?",
    faq_a_cpfp: "Non. Slipstream examine chaque transaction séparément : une transaction qui ne \
        paie pas assez ne sera pas minée ici, même si vous en envoyez une autre qui paie plus \
        pour la couvrir. Vous pouvez toujours envoyer plusieurs transactions liées, tant que \
        chacune paie assez à elle seule : nommez les fichiers 01, 02, 03 et elles partiront dans \
        cet ordre. S'il faut qu'une transaction en paie une autre, adressez-vous directement à \
        MARA.",
    faq_q_risks: "Quels sont les risques de ce service ?",
    faq_a_risks: "Il faut faire confiance à MARA, l'entreprise derrière Slipstream, pour ne pas \
        mener elle-même l'attaque sur votre transaction. C'est un risque acceptable comparé à une \
        diffusion publique, qui laisserait N'IMPORTE QUI mener cette attaque.",
    faq_q_domain: "Pourquoi cette page est-elle sur le domaine de Wizardsardine ?",
    faq_a_domain: "Nous (Wizardsardine) sommes une entreprise de sécurité et nous maintenons le \
        portefeuille Liana ([lianawallet.com](https://lianawallet.com)). Avant cet outil, les \
        utilisateurs de Liana n'avaient pas de moyen simple d'envoyer à Slipstream : c'est donc \
        un service pour eux, ouvert au reste de la communauté.",

    footer_built_by: "Réalisé par [Wizardsardine](https://wizardsardine.com)",
    footer_disclosure: "Avis de sécurité",
    footer_slipstream_terms: "Conditions Slipstream",

    msg_nothing_pasted: "Rien n'a encore été collé.",
    msg_unrecognised: "Ni une PSBT en base64 ni une transaction en hexadécimal valide.",
    msg_too_large: "La transaction fait {bytes} octets en hexadécimal et dépasse la limite de \
        {limit} Mio.",
    msg_no_inputs: "La transaction n'a aucune entrée.",
    msg_no_outputs: "La transaction n'a aucune sortie.",
    msg_rate_limited_note: "Slipstream limite ce navigateur. Attendez quelques minutes puis \
        renvoyez.",
    msg_retrying_now: "Limitée, nouvelle tentative maintenant…",
    msg_unreachable: "Impossible de joindre MARA Slipstream. Vérifiez votre connexion, ou si un \
        VPN, un proxy ou une extension de navigateur bloque slipstream.mara.com.",
    msg_unexpected_response: "Réponse inattendue de Slipstream (HTTP {status})",

    msg_missing_utxo: Plurals::two(
        "Données UTXO manquantes pour {n} entrée. Les frais et les scripts finaux n'ont pas pu \
         être validés.",
        "Données UTXO manquantes pour {n} entrées. Les frais et les scripts finaux n'ont pas pu \
         être validés.",
    ),
    msg_retrying_in: Plurals::two(
        "Limitée, nouvelle tentative dans {n} seconde…",
        "Limitée, nouvelle tentative dans {n} secondes…",
    ),
    msg_duplicate: Plurals::two(
        "Cette transaction est déjà dans la file.",
        "Ces transactions sont déjà dans la file.",
    ),
};

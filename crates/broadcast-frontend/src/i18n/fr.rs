//! French. Reviewed by ProfEduStream, so it carries no AI notice: see
//! [`super::Lang::reviewed`]. The first catalogue to be signed off, which
//! is what a French company should manage.
//!
//! The review kept `Fee rate` in English per the glossary and glossed it
//! once, in the FAQ answer about it, where there is room for a parenthesis
//! and a reader has already met the term twice.

use super::{Plurals, Strings};

pub static STRINGS: Strings = Strings {
    page_title: "Outofband : diffusion directe au mineur",
    lang_picker_label: "Langue",
    ai_notice: "Cette traduction a été produite par une IA et n'a pas été relue par un \
        locuteur natif. En cas de divergence avec l'anglais, c'est l'anglais qui fait foi.",
    ai_notice_action: "Read in English",

    disclosure_label: "Avis de sécurité",
    disclosure_link: "Vulnérabilité de l'entropie des portefeuilles Coldcard, détaillant \
        l'intérêt de cet outil",

    context_audience: "Cet outil n'a d'intérêt que pour certains set-ups particuliers, \
        notamment : les portefeuilles *Liana*, *Miniscript*, ainsi que certains types de \
        *multisig*.",
    context_not_recommended: "Cet outil n'est !PAS! recommandé pour les portefeuilles dont les \
        fonds sont à risque critique de vol, comme l'explique [cet article de blog \
        ›](https://wizardsardine.com/blog/coldcard-rng-vulnerability/)",

    hero_line_one: "Faites miner vos transactions",
    hero_line_two: "directement par un mineur, sans diffusion publique dans la mempool.",

    fee_eyebrow: "Fee rate minimal accepté",
    fee_sentence: "En dessous de ce Fee rate, une transaction n'a pas vocation à être minée. Le \
        dépasser n'est pas non plus une garantie qu'elle le soit, notamment dès le prochain \
        bloc.",
    fee_stale: "Ce taux n'est peut-être plus à jour.",
    fee_advice: "Ce seuil est *dynamique*. Il suit la demande : construisez donc votre \
        transaction avec un Fee rate *nettement supérieur* au seuil en cours, sans quoi elle \
        risque de repasser sous le seuil avant d'être minée.",

    prepare_heading: "Préparez votre transaction",
    prepare_liana: "Avec votre portefeuille Liana, préparez votre transaction normalement et \
        signez-la, mais n'appuyez !SURTOUT PAS! sur le bouton *Broadcast*. Une fois signée, \
        cliquez alors sur *Export*. C'est ce fichier PSBT que vous chargerez à l'étape suivante.",
    prepare_drafts: "Si vous l'avez déjà signée sans enregistrer le fichier PSBT, vous \
        retrouverez la transaction dans *Drafts and Approvals*.",
    prepare_other_wallets: "Construisez et signez votre transaction normalement, mais *ne la \
        diffusez SURTOUT PAS au réseau Bitcoin* (n'appuyez pas sur *Broadcast*). Tous les \
        portefeuilles logiciels ne permettent pas de signer sans diffuser : en cas de doute, \
        consultez la documentation du vôtre avant de signer. Utilisateurs de Liana, lisez \
        simplement l'encadré ci-dessus.",
    guides_heading: "Tutoriels",

    load_heading: "Chargez vos transactions",
    load_blurb: "Collez ou déposez ci-dessous vos PSBT signées et vos transactions brutes. \
        Votre navigateur envoie chacune d'elles directement à MARA Slipstream, qui la mine sans \
        qu'elle soit jamais diffusée dans la mempool publique.",
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
    col_fee_rate: "Fee rate",
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
    faq_a_why: "Habituellement, une transaction signée est diffusée à tous les nœuds Bitcoin du \
        réseau (le Broadcast), puis attend d'être minée. Tant qu'elle n'est pas dans un bloc, \
        quelqu'un qui détient une clé capable de dépenser les mêmes fonds peut la remplacer et \
        vous les voler. Slipstream, l'outil de l'entreprise de minage MARA, contourne cette \
        diffusion : la transaction part directement, et uniquement, vers un mineur, sans être \
        exposée au reste du réseau. Elle sera sans doute plus lente à être minée, mais elle sera \
        protégée d'une attaque par remplacement.",
    faq_q_where: "Où va réellement le texte que je colle ou le fichier PSBT que je dépose ?",
    faq_a_where: "Le décodage, la finalisation et le calcul des frais se font tous dans votre \
        navigateur, et ce site n'a aucun serveur intermédiaire. Deux choses quittent cette page : \
        la transaction finalisée en hexadécimal, envoyée directement de votre navigateur à MARA \
        Slipstream quand vous appuyez sur Envoyer, et une visite anonyme comptée par Plausible. \
        Plausible ne dépose aucun cookie et n'enregistre rien qui vous identifie, mais, comme \
        MARA, il voit l'adresse IP depuis laquelle vous vous connectez. Utilisez Tor ou un VPN si \
        cela compte pour vous. Ce que vous collez ne fait jamais partie ni de l'une ni de \
        l'autre.",
    faq_q_mara_learns: "Qu'apprend MARA à mon sujet ?",
    faq_a_mara_learns: "Le contenu de la transaction et votre adresse IP. Votre navigateur \
        parle directement à MARA : la connexion est la vôtre, et MARA reçoit depuis votre adresse \
        IP la transaction que vous souhaitez faire inscrire dans la chaîne de blocs. MARA \
        n'apprend en revanche ni les métadonnées de votre PSBT, ni vos clés publiques (xpubs), ni \
        votre descriptor, ni quelles autres transactions vous avez mises en file ici. Pour que \
        MARA ne voie pas votre adresse IP personnelle, utilisez Tor ou un VPN.",
    faq_q_guaranteed: "Suis-je certain que ma transaction sera minée ?",
    faq_a_guaranteed: "Non. Une transaction *Acceptée* par Slipstream ne veut pas dire qu'elle \
        est confirmée ni minée. MARA ne mine en effet qu'une partie des blocs, et ce mineur peut \
        aussi écarter votre transaction pour des raisons qu'il n'a pas à détailler. Voyez donc ce \
        service comme une meilleure chance, pas comme une promesse. Si votre transaction n'est \
        toujours pas confirmée au bout de quelques heures, n'hésitez pas à réessayer.",
    faq_q_rate_differs: "Pourquoi le Fee rate diffère-t-il de celui de Mempool.space ?",
    faq_a_rate_differs: "Slipstream se rémunère en demandant un Fee rate (taux de frais) plus \
        élevé que la normale, et donc que Mempool.space. Ce site ne touche aucune part de ce \
        paiement, ni aucune compensation.",
    faq_q_cpfp: "Une seconde transaction peut-elle payer les frais d'une première (CPFP) ?",
    faq_a_cpfp: "Non. Slipstream examine chaque transaction séparément : une transaction qui ne \
        paie pas assez de frais ne sera pas minée, même si vous en envoyez ensuite une autre qui \
        paie davantage pour la couvrir. Vous pouvez en revanche envoyer plusieurs transactions \
        liées, tant que chacune paie assez à elle seule. Si vous nommez les fichiers 01, 02, 03, \
        elles partiront dans cet ordre. Pour les cas plus particuliers, comme une transaction qui \
        doit nécessairement en payer une autre, adressez-vous directement à MARA.",
    faq_q_risks: "Quels sont les risques de ce service ?",
    faq_a_risks: "Il faut faire confiance à MARA, l'entreprise derrière Slipstream, pour ne pas \
        mener elle-même l'attaque par remplacement sur votre transaction. Ce risque reste \
        acceptable comparé à une diffusion publique dans la mempool, qui laisserait N'IMPORTE QUI \
        mener cette attaque.",
    faq_q_domain: "Pourquoi cette page est-elle sur le domaine de Wizardsardine ?",
    faq_a_domain: "Nous (Wizardsardine) sommes une entreprise de sécurité et nous maintenons le \
        portefeuille Liana ([lianawallet.com](https://lianawallet.com)). Avant cet outil, les \
        utilisateurs de Liana n'avaient pas de moyen simple d'envoyer une transaction à \
        Slipstream. Ce site est donc un service qui leur est dédié, et que nous ouvrons également \
        au reste de la communauté.",

    footer_built_by: "Réalisé par [Wizardsardine](https://wizardsardine.com)",
    footer_source: "Code source",
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

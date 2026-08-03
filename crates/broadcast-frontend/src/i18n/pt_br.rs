//! Portuguese (Brazil). Drafted, not yet reviewed by a native speaker: see
//! [`super::Lang::reviewed`].
//!
//! Written for Brazil, which is where most of the Bitcoin readership is, but
//! `from_tag` sends `pt-PT` here too: European Portuguese copy beats English
//! copy for a reader who asked for Portuguese.

use super::{Plurals, Strings};

pub static STRINGS: Strings = Strings {
    page_title: "Outofband: transmissão direta ao minerador",
    lang_picker_label: "Idioma",

    disclosure_label: "Aviso de segurança",
    disclosure_link: "Vulnerabilidade do RNG da Coldcard, e por que você pode precisar desta \
        ferramenta",

    context_audience: "Esta ferramenta só é importante para alguns perfis: principalmente \
        carteiras *Liana*, *Miniscript* ou alguns tipos de *multisig*.",
    context_not_recommended: "Esta ferramenta !NÃO! é recomendada para carteiras em risco \
        crítico, como explica [este artigo do blog \
        ›](https://wizardsardine.com/blog/coldcard-rng-vulnerability/)",

    hero_line_one: "Tenha suas transações",
    hero_line_two: "mineradas diretamente, sem usar a mempool pública.",

    fee_eyebrow: "Taxa mínima aceita",
    fee_sentence: "Nada abaixo desta taxa deve ser minerado. Ficar acima dela também não é \
        garantia.",
    fee_stale: "Esta taxa pode estar desatualizada.",
    fee_advice: "Este piso é *dinâmico*. Ele acompanha a demanda, então monte sua transação com \
        uma taxa *bem acima* do piso, ou ela pode deixar de superá-lo antes de ser minerada.",

    prepare_heading: "Prepare sua transação",
    prepare_liana: "No Liana, prepare sua transação normalmente e assine, mas !NÃO! aperte o \
        botão de transmitir. Depois de assinada, clique em *Export*. É esse arquivo que você vai \
        carregar na próxima etapa.",
    prepare_drafts: "Se você já assinou mas não salvou o arquivo PSBT, pode encontrar a \
        transação de novo em *Drafts and Approvals*.",
    prepare_other_wallets: "Monte e assine sua transação normalmente, mas *não a transmita para \
        a rede Bitcoin*. Nem todo programa permite assinar sem transmitir, então consulte a \
        documentação antes de assinar se estiver em dúvida. Quem usa Liana, é só ler o quadro \
        acima.",
    prepare_tutorials: "Vamos adicionar aqui links para tutoriais de educadores assim que \
        estiverem prontos. Se você não souber como fazer isso na sua carteira, pode esperar e \
        voltar a consultar esta página depois.",

    load_heading: "Carregue as transações",
    load_blurb: "Cole ou solte PSBTs assinadas e transações brutas. Seu navegador envia cada uma \
        direto para a MARA Slipstream, que a minera sem nunca tocar na mempool pública.",
    load_placeholder_or: "ou",
    load_placeholder_drop: "ou solte um arquivo aqui",
    load_accepts: "Arquivos, pastas ou compactados: .txt .psbt .txn .tar .tar.gz .zip",
    btn_add_to_queue: "Adicionar à fila",
    btn_choose_files: "Escolher arquivos",
    btn_clear_queue: "Limpar a fila",
    parse_error_title: "Não foi possível ler",
    detected_psbt: "DETECTADO · PSBT",
    detected_raw: "DETECTADO · TRANSAÇÃO BRUTA",
    detected_unrecognised: "FORMATO NÃO RECONHECIDO",

    stat_in_queue: "Na fila",
    stat_clear_floor: "Acima do piso",
    stat_below_floor: "Abaixo do piso",
    stat_fee_unknown: "Taxa desconhecida",
    btn_send: "Enviar",
    btn_send_batch: "Enviar lote ({n})",
    btn_sending: "Enviando…",

    col_source: "Origem",
    col_format: "Formato",
    col_vsize: "vsize",
    col_fee_rate: "Taxa",
    col_status: "Status",
    btn_retry: "Tentar de novo",
    row_remove_title: "Remover da fila",
    row_missing_value: "Os valores de entrada desta transação não são conhecidos, então a taxa \
        não pode ser calculada localmente. Informe o valor total gasto para conferir antes de \
        enviar.",
    row_total_placeholder: "Valor total de entrada (sats)",
    row_check_fee: "Conferir",

    status_invalid: "Inválida",
    status_accepted: "Aceita",
    status_sending: "Enviando…",
    status_rejected: "Rejeitada",
    status_rate_limited: "Limitada",
    status_failed: "Falhou",
    status_fee_unknown: "Taxa desconhecida",
    status_ready: "Pronta",
    status_below_floor: "Abaixo do piso",

    modal_title: "A PSBT não pode ser finalizada",
    modal_instruction: "Corrija a PSBT e carregue de novo.",
    btn_close: "Fechar",

    faq_eyebrow: "Perguntas que você deveria fazer antes de usar isto",
    faq_q_why: "Para que serve esta ferramenta?",
    faq_a_why: "Uma transmissão normal espalha sua transação para todos os nós Bitcoin da rede \
        antes de ela ser minerada. Se alguém tiver uma chave capaz de gastar essas mesmas moedas, \
        pode substituir a transação e roubá-las antes de ela ser minerada. A Slipstream (o que \
        esta ferramenta usa) pula esse espalhamento: a transação vai para um único minerador, sem \
        ser enviada ao resto da rede. Vai demorar mais para ser minerada, mas você fica protegido \
        da substituição.",
    faq_q_where: "Para onde vai de fato o que eu colo?",
    faq_a_where: "A decodificação, a finalização e o cálculo das taxas rodam todos no seu \
        navegador, e este site não tem servidor nenhum no meio. A única coisa que sai desta \
        página (e do seu computador) é a transação finalizada em hexadecimal, enviada direto do \
        seu navegador para a MARA Slipstream quando você aperta Enviar.",
    faq_q_mara_learns: "O que a MARA fica sabendo sobre mim?",
    faq_a_mara_learns: "Os dados da transação e o seu endereço IP. Seu navegador fala direto com \
        a MARA, então a conexão é sua e a MARA vê o endereço de onde você está navegando. A MARA \
        não fica sabendo os metadados da sua PSBT, suas xpubs, seu descriptor, nem quais outras \
        transações você colocou na fila aqui. Use Tor ou uma VPN se a MARA ver seu IP for um \
        problema para você.",
    faq_q_guaranteed: "Tenho garantia de que minha transação será minerada?",
    faq_a_guaranteed: "Não. Aceita pela Slipstream não é o mesmo que confirmada. A MARA minera \
        só uma parte dos blocos, não todos, e pode descartar sua transação por motivos que não \
        precisa explicar. Encare isso como uma chance melhor, não como uma promessa. Se não \
        confirmar depois de algumas horas, tente de novo.",
    faq_q_rate_differs: "Por que a taxa é diferente da do Mempool.space?",
    faq_a_rate_differs: "A Slipstream é paga pelo serviço através de uma taxa mais alta. Este \
        site não fica com nenhuma parte desse pagamento, nem com qualquer compensação.",
    faq_q_cpfp: "Uma segunda transação pode pagar a taxa de uma mais barata (CPFP)?",
    faq_a_cpfp: "Não. A Slipstream olha cada transação separadamente, então uma que não paga o \
        suficiente não será minerada aqui, mesmo que você envie outra pagando a mais para cobrir. \
        Você ainda pode enviar várias transações relacionadas, desde que cada uma pague o \
        suficiente sozinha: nomeie os arquivos 01, 02, 03 e eles saem nessa ordem. Se você \
        precisa que uma transação pague por outra, fale diretamente com a MARA.",
    faq_q_risks: "Quais são os riscos de usar este serviço?",
    faq_a_risks: "É preciso confiar que a MARA, a empresa por trás da Slipstream, não vai \
        executar ela mesma o ataque contra a sua transação. É um risco aceitável comparado a \
        transmitir publicamente e deixar QUALQUER UM executar o ataque.",
    faq_q_domain: "Por que esta página está no domínio da Wizardsardine?",
    faq_a_domain: "Nós (Wizardsardine) somos uma empresa de segurança e mantemos a carteira \
        Liana ([lianawallet.com](https://lianawallet.com)). Antes desta ferramenta, quem usava \
        Liana não tinha um jeito fácil de enviar para a Slipstream; então este é um serviço para \
        essas pessoas, aberto ao resto da comunidade.",

    footer_built_by: "Feito pela [Wizardsardine](https://wizardsardine.com)",
    footer_disclosure: "Aviso de segurança",
    footer_slipstream_terms: "Termos da Slipstream",

    msg_nothing_pasted: "Nada colado ainda.",
    msg_unrecognised: "Não são dados válidos de PSBT em base64 nem de transação em hexadecimal.",
    msg_too_large: "A transação tem {bytes} bytes em hexadecimal e passa do limite de {limit} MiB.",
    msg_no_inputs: "A transação não tem entradas.",
    msg_no_outputs: "A transação não tem saídas.",
    msg_not_finalizable: "A PSBT não pode ser finalizada.",
    msg_rate_limited_note: "A Slipstream está limitando este navegador. Espere alguns minutos e \
        envie de novo.",
    msg_retrying_now: "Limitada, tentando de novo agora…",
    msg_unreachable: "Não foi possível alcançar a MARA Slipstream. Verifique sua conexão, ou se \
        alguma VPN, proxy ou extensão do navegador está bloqueando slipstream.mara.com.",
    msg_unexpected_response: "Resposta inesperada da Slipstream (HTTP {status})",

    msg_missing_utxo: Plurals::two(
        "Faltam dados de UTXO de {n} entrada. A taxa e os scripts finais não puderam ser \
         validados.",
        "Faltam dados de UTXO de {n} entradas. A taxa e os scripts finais não puderam ser \
         validados.",
    ),
    msg_retrying_in: Plurals::two(
        "Limitada, tentando de novo em {n} segundo…",
        "Limitada, tentando de novo em {n} segundos…",
    ),
    msg_duplicate: Plurals::two(
        "Essa transação já está na fila.",
        "Essas transações já estão na fila.",
    ),
};

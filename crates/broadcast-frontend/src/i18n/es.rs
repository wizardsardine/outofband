//! Spanish. Drafted, not yet reviewed by a native speaker: see
//! [`super::Lang::reviewed`].

use super::{Plurals, Strings};

pub static STRINGS: Strings = Strings {
    page_title: "Outofband: difusión directa al minero",
    lang_picker_label: "Idioma",

    disclosure_label: "Aviso de seguridad",
    disclosure_link: "Vulnerabilidad del RNG de Coldcard, y por qué podrías necesitar esta \
        herramienta",

    context_audience: "Esta herramienta solo es importante para ciertos usuarios: sobre todo \
        carteras *Liana*, *Miniscript* o algunos tipos de *multisig*.",
    context_not_recommended: "Esta herramienta !NO! se recomienda para carteras en riesgo \
        crítico, como se explica en [esta entrada del blog \
        ›](https://wizardsardine.com/blog/coldcard-rng-vulnerability/)",

    hero_line_one: "Consigue que tus transacciones",
    hero_line_two: "se minen directamente, sin pasar por la mempool pública.",

    fee_eyebrow: "Fee rate mínima aceptada",
    fee_sentence: "No se espera que se mine nada por debajo de esta Fee rate. Superarla tampoco es \
        ninguna garantía.",
    fee_stale: "Esta tasa puede estar desactualizada.",
    fee_advice: "Este mínimo es *dinámico*. Se mueve con la demanda, así que construye tu \
        transacción con una Fee rate *bastante más alta* que el mínimo, o puede dejar de superarlo \
        antes de que se mine.",

    prepare_heading: "Prepara tu transacción",
    prepare_liana: "Si usas Liana, prepara tu transacción con normalidad y fírmala, pero !NO! \
        pulses el botón *Broadcast*. Una vez firmada, pulsa *Export* en su lugar. Ese archivo es \
        el que cargarás en el siguiente paso.",
    prepare_drafts: "Si ya la firmaste pero no guardaste el archivo PSBT, puedes encontrar la \
        transacción de nuevo en *Drafts and Approvals*.",
    prepare_other_wallets: "Construye y firma tu transacción con normalidad, pero *no hagas Broadcast a la red de Bitcoin*. No todos los programas permiten firmar sin hacer Broadcast, así \
        que consulta su documentación antes de firmar si tienes dudas. Si usas Liana, lee la \
        tarjeta de arriba.",
    prepare_tutorials: "Añadiremos aquí enlaces a tutoriales de divulgadores cuando estén \
        listos. Si no sabes cómo hacerlo con tu cartera, puedes esperar y volver a consultar esta \
        página más adelante.",

    load_heading: "Carga las transacciones",
    load_blurb: "Pega o suelta PSBT firmados y transacciones en crudo. Tu navegador envía cada \
        uno directamente a MARA Slipstream, que lo mina sin tocar nunca la mempool pública.",
    load_placeholder_or: "o",
    load_placeholder_drop: "o suelta un archivo aquí",
    load_accepts: "Archivos, carpetas o comprimidos: .txt .psbt .txn .tar .tar.gz .zip",
    btn_add_to_queue: "Añadir a la cola",
    btn_choose_files: "Elegir archivos",
    btn_clear_queue: "Vaciar la cola",
    parse_error_title: "No se pudo leer",
    detected_psbt: "DETECTADO · PSBT",
    detected_raw: "DETECTADO · TRANSACCIÓN EN CRUDO",
    detected_unrecognised: "FORMATO NO RECONOCIDO",

    stat_in_queue: "En cola",
    stat_clear_floor: "Superan el mínimo",
    stat_below_floor: "Bajo el mínimo",
    stat_fee_unknown: "Comisión desconocida",
    btn_send: "Enviar",
    btn_send_batch: "Enviar lote ({n})",
    btn_sending: "Enviando…",

    col_source: "Origen",
    col_format: "Formato",
    col_vsize: "vsize",
    col_fee_rate: "Fee rate",
    col_status: "Estado",
    btn_retry: "Reintentar",
    row_remove_title: "Quitar de la cola",
    row_missing_value: "No se conocen los importes de entrada de esta transacción, así que la \
        comisión no se puede calcular localmente. Introduce el valor total que se gasta para \
        comprobarla antes de enviar.",
    row_total_placeholder: "Valor total de entrada (sats)",

    status_invalid: "No válida",
    status_accepted: "Aceptada",
    status_sending: "Enviando…",
    status_rejected: "Rechazada",
    status_rate_limited: "Limitada",
    status_failed: "Fallida",
    status_fee_unknown: "Comisión desconocida",
    status_ready: "Lista",
    status_below_floor: "Bajo el mínimo",

    modal_title: "La PSBT no se puede finalizar",
    modal_instruction: "Corrige la PSBT y vuelve a cargarla.",
    btn_close: "Cerrar",

    faq_eyebrow: "Preguntas que deberías hacerte antes de usar esto",
    faq_q_why: "¿Para qué sirve esta herramienta?",
    faq_a_why: "Un Broadcast normal reparte tu transacción a todos los nodos de Bitcoin de la \
        red antes de que se mine. Si alguien tiene una clave capaz de gastar esas mismas monedas, \
        puede reemplazar la transacción y robarlas antes de que se mine. Slipstream (lo que usa \
        esta herramienta) se salta ese Broadcast: la transacción va a un solo minero, sin enviarse \
        al resto de la red. Tardará más en minarse, pero estarás a salvo del reemplazo.",
    faq_q_where: "¿Adónde va realmente lo que pego?",
    faq_a_where: "La descodificación, la finalización y el cálculo de comisiones ocurren en tu \
        navegador, y este sitio no tiene ningún servidor por medio. Lo único que sale de esta \
        página (y de tu ordenador) es la transacción finalizada en hexadecimal, enviada \
        directamente desde tu navegador a MARA Slipstream cuando pulsas Enviar.",
    faq_q_mara_learns: "¿Qué averigua MARA sobre mí?",
    faq_a_mara_learns: "Los datos de la transacción y tu dirección IP. Tu navegador habla \
        directamente con MARA, así que la conexión es tuya y MARA ve la dirección desde la que \
        navegas. MARA no conoce los metadatos de tu PSBT, tus xpubs, tu descriptor ni qué otras \
        transacciones has puesto en cola aquí. Usa Tor o una VPN si te importa que MARA vea tu IP.",
    faq_q_guaranteed: "¿Tengo garantizado que se mine mi transacción?",
    faq_a_guaranteed: "No. Que Slipstream la acepte no es que esté confirmada. MARA mina solo \
        una parte de los bloques, no todos, y puede descartar tu transacción por motivos que no \
        tiene que explicar. Tómatelo como una oportunidad mejor, no como una promesa. Si no se ha \
        confirmado al cabo de unas horas, vuelve a intentarlo.",
    faq_q_rate_differs: "¿Por qué la Fee rate es distinta de la de Mempool.space?",
    faq_a_rate_differs: "A Slipstream se le paga el servicio con una Fee rate más alta. Esta web no \
        se lleva ninguna parte de ese pago, ni ninguna compensación.",
    faq_q_cpfp: "¿Puede una segunda transacción pagar la comisión de otra más barata (CPFP)?",
    faq_a_cpfp: "No. Slipstream mira cada transacción por separado, así que una que no pague lo \
        suficiente no se minará aquí, aunque envíes otra pagando de más para cubrirla. Aun así \
        puedes enviar varias transacciones relacionadas, siempre que cada una pague lo suficiente \
        por sí sola: nombra los archivos 01, 02, 03 y saldrán en ese orden. Si necesitas que una \
        transacción pague por otra, contacta directamente con MARA.",
    faq_q_risks: "¿Qué riesgos tiene usar este servicio?",
    faq_a_risks: "Hay que confiar en que MARA, la empresa detrás de Slipstream, no ejecute ella \
        misma el ataque contra tu transacción. Es un riesgo aceptable comparado con difundirla \
        públicamente y dejar que CUALQUIERA ejecute el ataque.",
    faq_q_domain: "¿Por qué está esta página en el dominio de Wizardsardine?",
    faq_a_domain: "Nosotros (Wizardsardine) somos una empresa de seguridad y mantenemos la \
        cartera Liana ([lianawallet.com](https://lianawallet.com)). Antes de esta herramienta, \
        los usuarios de Liana no tenían una forma sencilla de enviar a Slipstream; es un servicio \
        para ellos, abierto al resto de la comunidad.",

    footer_built_by: "Hecho por [Wizardsardine](https://wizardsardine.com)",
    footer_disclosure: "Aviso de seguridad",
    footer_slipstream_terms: "Términos de Slipstream",

    msg_nothing_pasted: "Todavía no has pegado nada.",
    msg_unrecognised: "No son datos válidos de PSBT en base64 ni de transacción en hexadecimal.",
    msg_too_large: "La transacción ocupa {bytes} bytes en hexadecimal y supera el límite de \
        {limit} MiB.",
    msg_no_inputs: "La transacción no tiene entradas.",
    msg_no_outputs: "La transacción no tiene salidas.",
    msg_rate_limited_note: "Slipstream está limitando a este navegador. Espera unos minutos y \
        vuelve a enviar.",
    msg_retrying_now: "Limitada, reintentando ahora…",
    msg_unreachable: "No se pudo contactar con MARA Slipstream. Revisa tu conexión, o si una \
        VPN, un proxy o una extensión del navegador está bloqueando slipstream.mara.com.",
    msg_unexpected_response: "Respuesta inesperada de Slipstream (HTTP {status})",

    msg_missing_utxo: Plurals::two(
        "Faltan datos UTXO de {n} entrada. No se han podido validar la comisión ni los scripts \
         finales.",
        "Faltan datos UTXO de {n} entradas. No se han podido validar la comisión ni los scripts \
         finales.",
    ),
    msg_retrying_in: Plurals::two(
        "Limitada, reintentando en {n} segundo…",
        "Limitada, reintentando en {n} segundos…",
    ),
    msg_duplicate: Plurals::two(
        "Esa transacción ya está en la cola.",
        "Esas transacciones ya están en la cola.",
    ),
};

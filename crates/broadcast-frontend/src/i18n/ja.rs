//! Japanese. Drafted, not yet reviewed by a native speaker: see
//! [`super::Lang::reviewed`].
//!
//! No number inflection, so every counted message uses
//! [`super::Plurals::single`]. The CJK font stack is picked from
//! `<html lang>` (see `style.css`), which for `ja` puts Hiragino Sans and
//! Yu Gothic ahead of the Chinese faces: the two languages share Han
//! characters whose expected shapes differ, and a Chinese face rendering
//! Japanese text is immediately wrong to a Japanese reader.

use super::{Plurals, Strings};

pub static STRINGS: Strings = Strings {
    page_title: "Outofband: マイナーへ直接 Broadcast",
    lang_picker_label: "言語",
    ai_notice: "この翻訳は AI が作成したもので、ネイティブによる確認は済んでいません。\
        英語版と食い違う場合は英語版が正となります。",
    ai_notice_action: "Read in English",

    disclosure_label: "セキュリティ開示",
    disclosure_link: "Coldcard の乱数生成の脆弱性と、このツールが必要になる理由",

    context_audience: "このツールが重要なのは一部の利用者だけです。主に *Liana*、\
        *Miniscript*、および一部の *multisig* ウォレットです。",
    context_not_recommended: "深刻な危険にさらされているウォレットには、このツールは\
        !推奨されません!。理由は[このブログ記事 \
        ›](https://wizardsardine.com/blog/coldcard-rng-vulnerability/)をご覧ください",

    hero_line_one: "あなたのトランザクションを",
    hero_line_two: "公開 mempool を通さず直接マイニングへ。",

    fee_eyebrow: "受け付けられる最低 Fee rate",
    fee_sentence: "この Fee rate を下回るトランザクションはマイニングされない見込みです。\
        上回っても保証にはなりません。",
    fee_stale: "この Fee rate は古い可能性があります。",
    fee_advice: "この下限は*変動します*。需要に応じて動くため、下限より*十分に高い* Fee \
        rate でトランザクションを作成してください。そうしないと、マイニングされる前に\
        下限を下回るおそれがあります。",

    prepare_heading: "トランザクションを準備する",
    prepare_liana: "Liana をお使いの場合、いつもどおりトランザクションを作成して署名し、\
        *Broadcast* ボタンは!絶対に押さないでください!。署名したら代わりに *Export* を\
        クリックします。次のステップで読み込むのはそのファイルです。",
    prepare_drafts: "すでに署名したのに PSBT ファイルを保存していない場合は、\
        *Drafts and Approvals* からそのトランザクションを再び見つけられます。",
    prepare_other_wallets: "いつもどおりトランザクションを作成して署名し、ただし\
        *Bitcoin ネットワークへ Broadcast しないでください*。署名だけして Broadcast \
        しないことができないソフトウェアもあるので、不安な場合は署名する前にその\
        ドキュメントを確認してください。Liana をお使いの方は上のカードだけ読めば十分です。",
    guides_heading: "ガイド",
    guides_multilingual: "複数の言語",

    load_heading: "トランザクションを読み込む",
    load_blurb: "署名済みの PSBT や生トランザクションを貼り付けるか、ここにドロップして\
        ください。ブラウザがそれぞれを MARA Slipstream へ直接送り、公開 mempool に\
        一切触れずにマイニングされます。",
    load_placeholder_or: "または",
    load_placeholder_drop: "ここにファイルをドロップ",
    load_accepts: "ファイル・フォルダ・アーカイブ: .txt .psbt .txn .tar .tar.gz .zip",
    btn_add_to_queue: "キューに追加",
    btn_choose_files: "ファイルを選択",
    btn_clear_queue: "キューを空にする",
    parse_error_title: "解析できません",
    detected_psbt: "検出 · PSBT",
    detected_raw: "検出 · 生トランザクション",
    detected_unrecognised: "形式を認識できません",

    stat_in_queue: "キュー内",
    stat_clear_floor: "下限以上",
    stat_below_floor: "下限未満",
    stat_fee_unknown: "手数料不明",
    btn_send: "送信",
    btn_send_batch: "まとめて送信（{n}）",
    btn_sending: "送信中…",

    col_source: "取得元",
    col_format: "形式",
    col_vsize: "vsize",
    col_fee_rate: "Fee rate",
    col_status: "状態",
    btn_retry: "再試行",
    row_remove_title: "キューから削除",
    row_missing_value: "このトランザクションの入力金額が不明なため、手数料をローカルで\
        計算できません。送信前に確認できるよう、使用する合計額を入力してください。",
    row_total_placeholder: "入力の合計額（sats）",

    status_invalid: "無効",
    status_accepted: "受理",
    status_sending: "送信中…",
    status_rejected: "拒否",
    status_rate_limited: "レート制限",
    status_failed: "失敗",
    status_fee_unknown: "手数料不明",
    status_ready: "準備完了",
    status_below_floor: "下限未満",

    modal_title: "この PSBT はファイナライズできません",
    modal_instruction: "PSBT を修正してから読み込み直してください。",
    btn_close: "閉じる",

    faq_eyebrow: "使う前に確かめておきたいこと",
    faq_q_why: "このツールは何の役に立ちますか？",
    faq_a_why: "通常の Broadcast では、マイニングされる前にトランザクションが\
        ネットワーク上のすべての Bitcoin ノードへ広まります。同じコインを使える鍵を\
        誰かが持っていれば、マイニングされる前にトランザクションを置き換えてコインを\
        奪えます。Slipstream（このツールが使っているサービス）はその拡散を行いません。\
        トランザクションは 1 つのマイナーだけに渡り、ネットワークの他の部分には送られ\
        ません。マイニングは遅くなりますが、置き換えの心配はなくなります。",
    faq_q_where: "貼り付けたものは実際どこへ行きますか？",
    faq_a_where: "デコード、ファイナライズ、手数料の計算はすべてブラウザ内で行われ、\
        このサイトは間に一切サーバーを置いていません。このページから出ていくものは 2 \
        つです。送信を押したときにブラウザから MARA Slipstream へ直接送られる、\
        ファイナライズ済みトランザクションの hex。そして Plausible が数える匿名の\
        ページビューです。Plausible は cookie を使わず、あなたを特定する情報も記録\
        しませんが、MARA と同じく接続元の IP アドレスは見えます。気になる場合は Tor か \
        VPN を使ってください。貼り付けた内容がそのどちらかに含まれることはありません。",
    faq_q_mara_learns: "MARA は私について何を知りますか？",
    faq_a_mara_learns: "トランザクションの内容と、あなたの IP アドレスです。ブラウザが \
        MARA と直接やり取りするため、接続はあなたのものであり、MARA には接続元の\
        アドレスが見えます。MARA が知ることはありません: PSBT のメタデータ、xpub、\
        descriptor、ここでキューに入れた他のトランザクション。IP を見られたくない場合は \
        Tor か VPN を使ってください。",
    faq_q_guaranteed: "トランザクションは必ずマイニングされますか？",
    faq_a_guaranteed: "いいえ。Slipstream に受理されたことと承認されたことは別です。MARA \
        が採掘するのはブロックの一部であり全部ではありません。また、説明の義務なく\
        あなたのトランザクションを取り下げることもあります。約束ではなく、可能性が\
        高まる程度に考えてください。数時間たっても承認されない場合は、もう一度試して\
        ください。",
    faq_q_rate_differs: "なぜ Fee rate が Mempool.space と違うのですか？",
    faq_a_rate_differs: "Slipstream はこのサービスの対価を、より高い Fee rate という形で\
        受け取っています。このサイトはその支払いの取り分も、いかなる報酬も受け取って\
        いません。",
    faq_q_cpfp: "手数料の低いトランザクションの分を、2 つ目が肩代わりできますか（CPFP）？",
    faq_a_cpfp: "できません。Slipstream は各トランザクションを単独で見るため、十分に\
        払っていないものはここではマイニングされません。あとから多めに払うものを送っても\
        同じです。関連する複数のトランザクションを送ること自体は可能で、それぞれが単独で\
        十分に払っていれば問題ありません。ファイル名を 01、02、03 とすればその順に\
        送られます。片方がもう片方の分を払う必要がある場合は、MARA に直接お問い合わせ\
        ください。",
    faq_q_risks: "このサービスを使うリスクは何ですか？",
    faq_a_risks: "Slipstream を運営する MARA 自身が、あなたのトランザクションに対して\
        この攻撃を行わないと信頼する必要があります。公開の場に流して誰にでも攻撃を\
        許すことに比べれば、受け入れられるリスクです。",
    faq_q_domain: "なぜこのページは Wizardsardine のドメインにあるのですか？",
    faq_a_domain: "私たち（Wizardsardine）はセキュリティ企業で、Liana ウォレット\
        （[lianawallet.com](https://lianawallet.com)）を開発しています。このツールが\
        できるまで、Liana の利用者には Slipstream へ送る簡単な手段がありませんでした。\
        そのための仕組みであり、他のコミュニティにも開放しています。",

    footer_built_by: "制作: [Wizardsardine](https://wizardsardine.com)",
    footer_source: "ソースコード",
    footer_disclosure: "セキュリティ開示",
    footer_slipstream_terms: "Slipstream の利用条件",

    msg_nothing_pasted: "まだ何も貼り付けられていません。",
    msg_unrecognised: "有効な base64 の PSBT でも、hex のトランザクションデータでも\
        ありません。",
    msg_too_large: "このトランザクションは hex で {bytes} バイトあり、上限の {limit} MiB \
        を超えています。",
    msg_no_inputs: "このトランザクションには入力がありません。",
    msg_no_outputs: "このトランザクションには出力がありません。",
    msg_rate_limited_note: "Slipstream がこのブラウザをレート制限しています。数分待って\
        から送信し直してください。",
    msg_retrying_now: "レート制限中、すぐに再試行します…",
    msg_unreachable: "MARA Slipstream に接続できませんでした。接続状況と、VPN・プロキシ・\
        ブラウザ拡張が slipstream.mara.com を遮断していないかを確認してください。",
    msg_unexpected_response: "Slipstream から予期しない応答（HTTP {status}）",

    msg_missing_utxo: Plurals::single(
        "{n} 件の入力の UTXO データがありません。手数料と最終スクリプトを検証できません\
         でした。",
    ),
    msg_retrying_in: Plurals::single("レート制限中、{n} 秒後に再試行します…"),
    msg_duplicate: Plurals::single("そのトランザクションはすでにキューにあります。"),
};

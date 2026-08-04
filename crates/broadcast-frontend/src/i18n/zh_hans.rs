//! Simplified Chinese. Drafted, not yet reviewed by a native speaker: see
//! [`super::Lang::reviewed`].
//!
//! No number inflection, so every counted message uses
//! [`super::Plurals::single`]. No CJK font is downloaded either: the stack
//! in `tokens::FONT_SANS` falls through to a system face chosen by
//! `<html lang>` (see `style.css`), which for `zh` puts PingFang SC and
//! Microsoft YaHei ahead of the Japanese faces so Han characters get
//! Chinese shapes.

use super::{Plurals, Strings};

pub static STRINGS: Strings = Strings {
    page_title: "Outofband：直接发送给矿工的 Broadcast",
    lang_picker_label: "语言",
    ai_notice: "本页译文由 AI 生成，未经母语者校对。如与英文版本有出入，以英文为准。",
    ai_notice_action: "Read in English",

    disclosure_label: "安全披露",
    disclosure_link: "Coldcard 随机数漏洞，以及你为什么可能需要这个工具",

    context_audience: "这个工具只对部分用户重要：主要是 *Liana*、*Miniscript* \
        以及某些类型的 *multisig* 钱包。",
    context_not_recommended: "对于已处于严重风险中的钱包，!不建议! 使用本工具，原因见\
        [这篇博客文章 ›](https://wizardsardine.com/blog/coldcard-rng-vulnerability/)",

    hero_line_one: "让你的交易",
    hero_line_two: "被直接挖出，不经过公共 mempool。",

    fee_eyebrow: "最低可接受 Fee rate",
    fee_sentence: "低于这个 Fee rate 的交易预计不会被挖出。高于它也并不等于保证。",
    fee_stale: "这个 Fee rate 可能已经过时。",
    fee_advice: "这个下限是*动态*的，会随需求变化，所以请用*明显高于*下限的 Fee rate \
        构建交易，否则它可能在被挖出之前就低于下限了。",

    prepare_heading: "准备你的交易",
    prepare_liana: "Liana 用户请照常构建并签名交易，但!千万不要!按 *Broadcast* 按钮。\
        签名之后请点 *Export*，下一步要加载的就是这个文件。",
    prepare_drafts: "如果你已经签名但没有保存 PSBT 文件，可以在 *Drafts and Approvals* \
        里重新找到这笔交易。",
    prepare_other_wallets: "照常构建并签名你的交易，但*不要向 Bitcoin 网络做 Broadcast*。\
        并非所有软件都允许只签名而不 Broadcast，拿不准就先查它的文档再签名。Liana \
        用户看上面那张卡片就够了。",
    guides_heading: "教程",
    guides_multilingual: "多种语言",

    load_heading: "加载交易",
    load_blurb: "粘贴或拖入已签名的 PSBT 和原始交易。你的浏览器会把每一笔直接发给 MARA \
        Slipstream，由它挖出，全程不碰公共 mempool。",
    load_placeholder_or: "或者",
    load_placeholder_drop: "或者把文件拖到这里",
    load_accepts: "文件、文件夹或压缩包：.txt .psbt .txn .tar .tar.gz .zip",
    btn_add_to_queue: "加入队列",
    btn_choose_files: "选择文件",
    btn_clear_queue: "清空队列",
    parse_error_title: "无法解析",
    detected_psbt: "已识别 · PSBT",
    detected_raw: "已识别 · 原始交易",
    detected_unrecognised: "无法识别的格式",

    stat_in_queue: "队列中",
    stat_clear_floor: "高于下限",
    stat_below_floor: "低于下限",
    stat_fee_unknown: "手续费未知",
    btn_send: "发送",
    btn_send_batch: "批量发送（{n}）",
    btn_sending: "发送中…",

    col_source: "来源",
    col_format: "格式",
    col_vsize: "vsize",
    col_fee_rate: "Fee rate",
    col_status: "状态",
    btn_retry: "重试",
    row_remove_title: "从队列中移除",
    row_missing_value: "这笔交易的输入金额未知，因此无法在本地算出手续费。请输入花费的总额，\
        以便在提交前核对。",
    row_total_placeholder: "输入总额（sats）",

    status_invalid: "无效",
    status_accepted: "已接受",
    status_sending: "发送中…",
    status_rejected: "已拒绝",
    status_rate_limited: "被限流",
    status_failed: "失败",
    status_fee_unknown: "手续费未知",
    status_ready: "就绪",
    status_below_floor: "低于下限",

    modal_title: "PSBT 无法最终化",
    modal_instruction: "请修正 PSBT 后重新加载。",
    btn_close: "关闭",

    faq_eyebrow: "使用之前你应该问的问题",
    faq_q_why: "这个工具有什么用？",
    faq_a_why: "普通的 Broadcast 会在交易被挖出之前，把它扩散给网络上的每一个 Bitcoin \
        节点。如果有人持有同样能花掉这些币的密钥，他就能在交易被挖出之前替换它并把币偷走。\
        Slipstream（本工具所使用的服务）跳过了这一步扩散：交易只发给一个矿工，不会发往网络\
        的其余部分。被挖出会慢一些，但你不会被交易替换。",
    faq_q_where: "我粘贴的内容实际上去了哪里？",
    faq_a_where: "解码、最终化和手续费计算都在你的浏览器里完成，这个站点中间没有任何服务器。\
        离开本页面的有两样东西：最终化之后的交易 hex，在你按下发送时由浏览器直接发给 MARA \
        Slipstream；以及 Plausible 统计的一次匿名访问。Plausible 不使用 cookie，也不记录\
        任何能识别你的信息，但和 MARA 一样，它能看到你连接时使用的 IP 地址。如果你在意这一点，\
        请使用 Tor 或 VPN。你粘贴的内容永远不会包含在其中任何一项里。",
    faq_q_mara_learns: "MARA 能知道我的哪些信息？",
    faq_a_mara_learns: "交易内容，以及你的 IP 地址。你的浏览器直接与 MARA 通信，所以连接是\
        你自己的，MARA 会看到你上网的地址。MARA 不会知道你的 PSBT 元数据、你的 xpub、你的 \
        descriptor，也不知道你在这里还排了哪些交易。如果你介意 MARA 看到你的 IP，请使用 Tor \
        或 VPN。",
    faq_q_guaranteed: "我的交易一定会被挖出吗？",
    faq_a_guaranteed: "不一定。被 Slipstream 接受不等于已确认。MARA 只挖出一部分区块，而不是\
        全部，并且可以出于无需解释的理由丢弃你的交易。请把它当成更大的机会，而不是承诺。\
        如果过了几个小时仍未确认，就再试一次。",
    faq_q_rate_differs: "为什么这里的 Fee rate 和 Mempool.space 不一样？",
    faq_a_rate_differs: "Slipstream 通过更高的 Fee rate 获得服务报酬。本网站不从这笔费用中\
        抽取任何份额，也不收取任何报酬。",
    faq_q_cpfp: "可以用第二笔交易替低费用的那笔付手续费吗（CPFP）？",
    faq_a_cpfp: "不行。Slipstream 单独看待每一笔交易，所以付得不够的那笔在这里不会被挖出，\
        即使你再发一笔多付的来替它兜底也一样。你仍然可以发送多笔相关的交易，只要每一笔单独\
        都付得够：把文件命名为 01、02、03，它们就会按这个顺序发出。如果确实需要一笔交易替\
        另一笔付费，请直接联系 MARA。",
    faq_q_risks: "使用这项服务有什么风险？",
    faq_a_risks: "你必须信任 Slipstream 背后的公司 MARA 不会自己对你的交易发动这种攻击。\
        与公开广播、让任何人都能发动攻击相比，这是一个可以接受的风险。",
    faq_q_domain: "这个页面为什么放在 Wizardsardine 的域名下？",
    faq_a_domain: "我们（Wizardsardine）是一家安全公司，负责维护 Liana 钱包\
        （[lianawallet.com](https://lianawallet.com)）。在有这个工具之前，Liana \
        用户没有简单的办法把交易发给 Slipstream，所以这是为他们提供的服务，同时也向社区\
        其他人开放。",

    footer_built_by: "由 [Wizardsardine](https://wizardsardine.com) 打造",
    footer_source: "源代码",
    footer_disclosure: "安全披露",
    footer_slipstream_terms: "Slipstream 条款",

    msg_nothing_pasted: "还没有粘贴任何内容。",
    msg_unrecognised: "既不是有效的 base64 PSBT，也不是 hex 交易数据。",
    msg_too_large: "该交易的 hex 有 {bytes} 字节，超过了 {limit} MiB 的上限。",
    msg_no_inputs: "该交易没有输入。",
    msg_no_outputs: "该交易没有输出。",
    msg_rate_limited_note: "Slipstream 正在限制这个浏览器。请等几分钟后再发送。",
    msg_retrying_now: "被限流，正在重试…",
    msg_unreachable: "无法连接 MARA Slipstream。请检查你的网络，或者是否有 VPN、代理或浏览器\
        扩展挡住了 slipstream.mara.com。",
    msg_unexpected_response: "Slipstream 返回了意外的响应（HTTP {status}）",

    msg_missing_utxo: Plurals::single("缺少 {n} 个输入的 UTXO 数据，无法验证手续费和最终脚本。"),
    msg_retrying_in: Plurals::single("被限流，{n} 秒后重试…"),
    msg_duplicate: Plurals::single("该交易已经在队列中。"),
};

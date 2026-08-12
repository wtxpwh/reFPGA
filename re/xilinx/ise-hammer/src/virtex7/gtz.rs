use prjcombine_interconnect::db::{BelAttributeType, BelInputId};
use prjcombine_re_collector::diff::OcdMode;
use prjcombine_re_hammer::Session;
use prjcombine_virtex4::defs::{bcls::GTZ_OCTAL, bslots, virtex7::tcls};

use crate::{
    backend::{IseBackend, MultiValue},
    collector::CollectorCtx,
    generic::fbuild::{FuzzBuilderBase, FuzzCtx},
    virtex4::specials,
};

const GTZ_INVPINS: &[BelInputId] = &[
    GTZ_OCTAL::RXUSRCLK0,
    GTZ_OCTAL::RXUSRCLK1,
    GTZ_OCTAL::RXUSRCLK2,
    GTZ_OCTAL::RXUSRCLK3,
    GTZ_OCTAL::RXUSRCLK4,
    GTZ_OCTAL::RXUSRCLK5,
    GTZ_OCTAL::RXUSRCLK6,
    GTZ_OCTAL::RXUSRCLK7,
    GTZ_OCTAL::TXUSRCLK0,
    GTZ_OCTAL::TXUSRCLK1,
    GTZ_OCTAL::TXUSRCLK2,
    GTZ_OCTAL::TXUSRCLK3,
    GTZ_OCTAL::TXUSRCLK4,
    GTZ_OCTAL::TXUSRCLK5,
    GTZ_OCTAL::TXUSRCLK6,
    GTZ_OCTAL::TXUSRCLK7,
];

pub fn add_fuzzers<'a>(session: &mut Session<'a, IseBackend<'a>>, backend: &'a IseBackend<'a>) {
    for tcid in [tcls::GTZ_S, tcls::GTZ_N] {
        let Some(mut ctx) = FuzzCtx::try_new(session, backend, tcid) else {
            continue;
        };
        let mut bctx = ctx.bel(bslots::GTZ_OCTAL);
        let mode = "GTZE2_OCTAL";
        bctx.build()
            .null_bits()
            .test_bel_special(specials::PRESENT)
            .mode(mode)
            .commit();
        for &pin in GTZ_INVPINS {
            bctx.build().mode(mode).test_bel_input_inv_auto(pin);
        }

        for (aid, attr) in &backend.edev.db[GTZ_OCTAL].attributes {
            match attr.typ {
                BelAttributeType::Bool => {
                    bctx.mode(mode)
                        .test_bel_attr_bool_auto(aid, "FALSE", "TRUE");
                }
                BelAttributeType::Enum(_) => {
                    bctx.mode(mode).test_bel_attr_auto(aid);
                }
                BelAttributeType::BitVec(width) => {
                    if matches!(
                        aid,
                        GTZ_OCTAL::BLOCK_TRANSFER_REG
                            | GTZ_OCTAL::CLKOBS_CK_SEL
                            | GTZ_OCTAL::MULTI_LANE_MODE
                    ) || width < 4
                    {
                        bctx.mode(mode).test_bel_attr_multi(aid, MultiValue::Bin);
                    } else {
                        bctx.mode(mode).test_bel_attr_multi(aid, MultiValue::Hex(0));
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}

pub fn collect_fuzzers(ctx: &mut CollectorCtx) {
    for tcid in [tcls::GTZ_S, tcls::GTZ_N] {
        if !ctx.has_tcls(tcid) {
            continue;
        }
        let bslot = bslots::GTZ_OCTAL;
        for &pin in GTZ_INVPINS {
            ctx.collect_bel_input_inv_bi(tcid, bslot, pin);
        }

        for (aid, attr) in &ctx.edev.db[GTZ_OCTAL].attributes {
            match attr.typ {
                BelAttributeType::Bool => {
                    ctx.collect_bel_attr_bi(tcid, bslot, aid);
                }
                BelAttributeType::BitVec(_) => {
                    ctx.collect_bel_attr(tcid, bslot, aid);
                }
                BelAttributeType::Enum(_) => {
                    ctx.collect_bel_attr_ocd(tcid, bslot, aid, OcdMode::BitOrder);
                }
                _ => unreachable!(),
            }
        }
    }
}

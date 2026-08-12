use prjcombine_re_collector::diff::{Diff, xlat_bit};
use prjcombine_re_hammer::Session;
use prjcombine_virtex4::defs::{bcls, bslots, enums, virtex4::tcls};

use crate::{
    backend::{IseBackend, MultiValue},
    collector::CollectorCtx,
    generic::fbuild::{FuzzBuilderBase, FuzzCtx},
    virtex4::specials,
};

pub fn add_fuzzers<'a>(session: &mut Session<'a, IseBackend<'a>>, backend: &'a IseBackend<'a>) {
    let mut ctx = FuzzCtx::new(session, backend, tcls::CLB);
    let bk_l = "SLICEL";
    let bk_m = "SLICEM";
    for i in 0..4 {
        let mut bctx = ctx.bel(bslots::SLICE[i]);
        let is_m = matches!(i, 0 | 2);

        // inverters
        bctx.mode(bk_l)
            .attr("FFX", "#FF")
            .pin("XQ")
            .test_bel_input_inv_auto(bcls::SLICE::CE);
        bctx.mode(bk_l)
            .attr("FFX", "#FF")
            .pin("XQ")
            .test_bel_input_inv_auto(bcls::SLICE::CLK);
        bctx.mode(bk_l)
            .attr("FFX", "#FF")
            .attr("FFY", "#FF")
            .pin("XQ")
            .pin("YQ")
            .test_bel_input_inv_auto(bcls::SLICE::SR);
        bctx.mode(bk_l)
            .attr("FFX", "#FF")
            .attr("XUSED", "0")
            .attr("DXMUX", "BX")
            .pin("X")
            .pin("XQ")
            .test_bel_input_inv_auto(bcls::SLICE::BX);
        bctx.mode(bk_l)
            .attr("FFY", "#FF")
            .attr("YUSED", "0")
            .attr("DYMUX", "BY")
            .pin("Y")
            .pin("YQ")
            .test_bel_input_inv_auto(bcls::SLICE::BY);

        // LUT
        for attr in [bcls::SLICE::F, bcls::SLICE::G] {
            bctx.mode(bk_l).test_bel_attr_multi(attr, MultiValue::Lut);
        }

        // carry chain
        bctx.mode(bk_l)
            .attr("BXINV", "BX_B")
            .attr("F", "#LUT:0")
            .attr("G", "#LUT:0")
            .attr("COUTUSED", "0")
            .pin("CIN")
            .pin("BX")
            .pin("COUT")
            .test_bel_attr_auto(bcls::SLICE::CYINIT);
        bctx.mode(bk_l)
            .attr("CYINIT", "BX")
            .attr("BXINV", "BX_B")
            .attr("FXMUX", "FXOR")
            .attr("F", "#LUT:0")
            .attr("G", "#LUT:0")
            .attr("XMUXUSED", "0")
            .attr("COUTUSED", "0")
            .pin("F3")
            .pin("F2")
            .pin("BX")
            .pin("XMUX")
            .pin("COUT")
            .test_bel_attr_auto(bcls::SLICE::CY0F);
        bctx.mode(bk_l)
            .attr("CYINIT", "BX")
            .attr("BXINV", "BX_B")
            .attr("BYINV", "BY_B")
            .attr("GYMUX", "GXOR")
            .attr("F", "#LUT:0")
            .attr("G", "#LUT:0")
            .attr("YMUXUSED", "0")
            .attr("COUTUSED", "0")
            .pin("G3")
            .pin("G2")
            .pin("BX")
            .pin("BY")
            .pin("YMUX")
            .pin("COUT")
            .test_bel_attr_auto(bcls::SLICE::CY0G);

        // various muxes
        bctx.mode(bk_l)
            .attr("F", "#LUT:0")
            .attr("CYINIT", "BX")
            .attr("BXINV", "BX_B")
            .attr("XMUXUSED", "0")
            .pin("X")
            .pin("XMUX")
            .pin("BX")
            .test_bel_attr_auto(bcls::SLICE::FXMUX);
        bctx.mode(bk_l)
            .attr("F", "#LUT:0")
            .attr("G", "#LUT:0")
            .attr("CYINIT", "BX")
            .attr("BXINV", "BX_B")
            .attr("BYINV", "BY_B")
            .attr("YMUXUSED", "0")
            .pin("X")
            .pin("Y")
            .pin("FXINA")
            .pin("FXINB")
            .pin("YMUX")
            .pin("BX")
            .pin("BY")
            .test_bel_attr_auto(bcls::SLICE::GYMUX);
        for (vname, val_f5, val_fxor) in [
            ("BX", enums::SLICE_DXMUX::BX, enums::SLICE_DXMUX::BX),
            ("X", enums::SLICE_DXMUX::X, enums::SLICE_DXMUX::X),
            ("XB", enums::SLICE_DXMUX::XB, enums::SLICE_DXMUX::XB),
            ("XMUX", enums::SLICE_DXMUX::F5, enums::SLICE_DXMUX::FXOR),
        ] {
            bctx.mode(bk_l)
                .attr("F", "#LUT:0")
                .attr("FFX", "#FF")
                .attr("BXINV", "BX_B")
                .attr("FXMUX", "F5")
                .attr("XUSED", "0")
                .attr("XBUSED", "0")
                .attr("XMUXUSED", "0")
                .pin("BX")
                .pin("X")
                .pin("XB")
                .pin("XMUX")
                .pin("XQ")
                .test_bel_attr_val(bcls::SLICE::DXMUX, val_f5)
                .attr("DXMUX", vname)
                .commit();
            bctx.mode(bk_l)
                .attr("F", "#LUT:0")
                .attr("FFX", "#FF")
                .attr("BXINV", "BX_B")
                .attr("FXMUX", "FXOR")
                .attr("XUSED", "0")
                .attr("XBUSED", "0")
                .attr("XMUXUSED", "0")
                .pin("BX")
                .pin("X")
                .pin("XB")
                .pin("XMUX")
                .pin("XQ")
                .test_bel_attr_val(bcls::SLICE::DXMUX, val_fxor)
                .attr("DXMUX", vname)
                .commit();
        }
        for (vname, val_fx, val_gxor) in [
            ("BY", enums::SLICE_DYMUX::BY, enums::SLICE_DYMUX::BY),
            ("Y", enums::SLICE_DYMUX::Y, enums::SLICE_DYMUX::Y),
            ("YB", enums::SLICE_DYMUX::YB, enums::SLICE_DYMUX::YB),
            ("YMUX", enums::SLICE_DYMUX::FX, enums::SLICE_DYMUX::GXOR),
        ] {
            bctx.mode(bk_l)
                .attr("G", "#LUT:0")
                .attr("FFY", "#FF")
                .attr("BYINV", "BY_B")
                .attr("GYMUX", "FX")
                .attr("YUSED", "0")
                .attr("YBUSED", "0")
                .attr("YMUXUSED", "0")
                .pin("BY")
                .pin("Y")
                .pin("YB")
                .pin("YMUX")
                .pin("YQ")
                .test_bel_attr_val(bcls::SLICE::DYMUX, val_fx)
                .attr("DYMUX", vname)
                .commit();
            bctx.mode(bk_l)
                .attr("G", "#LUT:0")
                .attr("FFY", "#FF")
                .attr("BYINV", "BY_B")
                .attr("GYMUX", "GXOR")
                .attr("YUSED", "0")
                .attr("YBUSED", "0")
                .attr("YMUXUSED", "0")
                .pin("BY")
                .pin("Y")
                .pin("YB")
                .pin("YMUX")
                .pin("YQ")
                .test_bel_attr_val(bcls::SLICE::DYMUX, val_gxor)
                .attr("DYMUX", vname)
                .commit();
        }

        // LUT: memory mode
        if is_m {
            for (val, vname) in [
                (enums::SLICE_DIF_MUX::BX, "BX"),
                (enums::SLICE_DIF_MUX::ALT, "ALTDIF"),
                (enums::SLICE_DIF_MUX::ALT, "SHIFTIN"),
            ] {
                bctx.mode(bk_m)
                    .attr("F", "#RAM:0")
                    .attr("XUSED", "0")
                    .attr("BXINV", "BX_B")
                    .pin("X")
                    .pin("BX")
                    .pin("SHIFTIN")
                    .test_bel_attr_val(bcls::SLICE::DIF_MUX, val)
                    .attr("DIF_MUX", vname)
                    .commit();
            }
            for (val, vname) in [
                (enums::SLICE_DIG_MUX::BY, "BY"),
                (enums::SLICE_DIG_MUX::ALT, "ALTDIG"),
                (enums::SLICE_DIG_MUX::ALT, "SHIFTIN"),
            ] {
                bctx.mode(bk_m)
                    .attr("G", "#RAM:0")
                    .attr("YUSED", "0")
                    .attr("BYINV", "BY_B")
                    .pin("Y")
                    .pin("BY")
                    .pin("SHIFTIN")
                    .test_bel_attr_val(bcls::SLICE::DIG_MUX, val)
                    .attr("DIG_MUX", vname)
                    .commit();
            }
            for (val, vname) in [
                (enums::SLICE_XBMUX::FMC15, "0"),
                (enums::SLICE_XBMUX::FCY, "1"),
            ] {
                bctx.mode(bk_m)
                    .attr("F", "#RAM:0")
                    .pin("XB")
                    .test_bel_attr_val(bcls::SLICE::XBMUX, val)
                    .attr("XBMUX", vname)
                    .commit();
            }
            for (val, vname) in [
                (enums::SLICE_YBMUX::GMC15, "0"),
                (enums::SLICE_YBMUX::GCY, "1"),
            ] {
                bctx.mode(bk_m)
                    .attr("G", "#RAM:0")
                    .attr("YBUSED", "0")
                    .pin("YB")
                    .test_bel_attr_val(bcls::SLICE::YBMUX, val)
                    .attr("YBMUX", vname)
                    .commit();
            }
            bctx.mode(bk_m)
                .attr("XUSED", "0")
                .attr("G", "#LUT:0")
                .attr("F_ATTR", "DUAL_PORT")
                .pin("X")
                .test_bel_attr_bits(bcls::SLICE::F_RAM_ENABLE)
                .attr_diff("F", "#LUT:0", "#RAM:0")
                .commit();
            bctx.mode(bk_m)
                .attr("YUSED", "0")
                .attr("F", "#LUT:0")
                .attr("G_ATTR", "DUAL_PORT")
                .pin("Y")
                .test_bel_attr_bits(bcls::SLICE::G_RAM_ENABLE)
                .attr_diff("G", "#LUT:0", "#RAM:0")
                .commit();
            bctx.mode(bk_m)
                .attr("F", "#RAM:0")
                .attr("XUSED", "0")
                .pin("X")
                .test_bel_attr_bool_rename(
                    "F_ATTR",
                    bcls::SLICE::F_SHIFT_ENABLE,
                    "DUAL_PORT",
                    "SHIFT_REG",
                );
            bctx.mode(bk_m)
                .attr("G", "#RAM:0")
                .attr("YUSED", "0")
                .pin("Y")
                .test_bel_attr_bool_rename(
                    "G_ATTR",
                    bcls::SLICE::G_SHIFT_ENABLE,
                    "DUAL_PORT",
                    "SHIFT_REG",
                );
            for (pinused, attr_f, attr_g) in [
                (
                    "SLICEWE0USED",
                    bcls::SLICE::F_SLICEWE0USED,
                    bcls::SLICE::G_SLICEWE0USED,
                ),
                (
                    "SLICEWE1USED",
                    bcls::SLICE::F_SLICEWE1USED,
                    bcls::SLICE::G_SLICEWE1USED,
                ),
            ] {
                bctx.mode(bk_m)
                    .attr("F", "#RAM:0")
                    .attr("G", "")
                    .attr("XUSED", "0")
                    .attr("BXINV", "BX_B")
                    .pin("X")
                    .pin("BX")
                    .pin("SLICEWE1")
                    .test_bel_attr_bits(attr_f)
                    .attr(pinused, "0")
                    .commit();
                bctx.mode(bk_m)
                    .attr("F", "")
                    .attr("G", "#RAM:0")
                    .attr("YUSED", "0")
                    .attr("BXINV", "BX_B")
                    .pin("Y")
                    .pin("BX")
                    .pin("SLICEWE1")
                    .test_bel_attr_bits(attr_g)
                    .attr(pinused, "0")
                    .commit();
            }
            bctx.mode(bk_m)
                .null_bits()
                .attr("BYINV", "BY_B")
                .attr("BYINVOUTUSED", "")
                .pin("BY")
                .pin("BYOUT")
                .test_bel_special(specials::SLICE_BYOUTUSED)
                .attr("BYOUTUSED", "0")
                .commit();
            bctx.mode(bk_m)
                .null_bits()
                .attr("BYINV", "BY_B")
                .attr("BYOUTUSED", "")
                .pin("BY")
                .pin("BYOUT")
                .test_bel_special(specials::SLICE_BYINVOUTUSED)
                .attr("BYINVOUTUSED", "0")
                .commit();
        }

        // FF
        bctx.mode(bk_l)
            .pin("BX")
            .pin("XQ")
            .pin("CE")
            .attr("FFY", "")
            .attr("CEINV", "CE_B")
            .attr("FFX_INIT_ATTR", "INIT1")
            .test_bel_attr_bool_rename("FFX", bcls::SLICE::FF_LATCH, "#FF", "#LATCH");
        bctx.mode(bk_l)
            .pin("BY")
            .pin("YQ")
            .pin("CE")
            .attr("FFX", "")
            .attr("CEINV", "CE_B")
            .attr("FFY_INIT_ATTR", "INIT1")
            .test_bel_attr_bool_rename("FFY", bcls::SLICE::FF_LATCH, "#FF", "#LATCH");
        bctx.mode(bk_l)
            .pin("XQ")
            .attr("FFX", "#FF")
            .test_bel_attr_bool_rename("SYNC_ATTR", bcls::SLICE::FF_SR_SYNC, "ASYNC", "SYNC");
        bctx.mode(bk_l)
            .pin("XQ")
            .attr("FFX_INIT_ATTR", "INIT1")
            .attr("FFX", "#FF")
            .test_bel_attr_bool_rename("FFX_SR_ATTR", bcls::SLICE::FFX_SRVAL, "SRLOW", "SRHIGH");
        bctx.mode(bk_l)
            .pin("YQ")
            .attr("FFY_INIT_ATTR", "INIT1")
            .attr("FFY", "#FF")
            .test_bel_attr_bool_rename("FFY_SR_ATTR", bcls::SLICE::FFY_SRVAL, "SRLOW", "SRHIGH");
        bctx.mode(bk_l)
            .pin("XQ")
            .attr("FFX", "#FF")
            .test_bel_attr_bool_rename("FFX_INIT_ATTR", bcls::SLICE::FFX_INIT, "INIT0", "INIT1");
        bctx.mode(bk_l)
            .pin("YQ")
            .attr("FFY", "#FF")
            .test_bel_attr_bool_rename("FFY_INIT_ATTR", bcls::SLICE::FFY_INIT, "INIT0", "INIT1");
        bctx.mode(bk_l)
            .attr("FFX", "#FF")
            .attr("BYINV", "BY_B")
            .pin("XQ")
            .pin("BY")
            .test_bel_attr_bits(bcls::SLICE::FF_REV_ENABLE)
            .attr("REVUSED", "0")
            .commit();
    }
}

pub fn collect_fuzzers(ctx: &mut CollectorCtx) {
    let tcid = tcls::CLB;
    for (idx, bslot) in bslots::SLICE.into_iter().enumerate() {
        ctx.collect_bel_attr(tcid, bslot, bcls::SLICE::F);
        ctx.collect_bel_attr(tcid, bslot, bcls::SLICE::G);
        ctx.collect_bel_attr(tcid, bslot, bcls::SLICE::CYINIT);
        ctx.collect_bel_attr(tcid, bslot, bcls::SLICE::CY0F);
        ctx.collect_bel_attr(tcid, bslot, bcls::SLICE::CY0G);

        // LUT RAM
        let is_m = matches!(idx, 0 | 2);
        if is_m {
            ctx.get_diff_attr_bool_bi(tcid, bslot, bcls::SLICE::F_SHIFT_ENABLE, false)
                .assert_empty();
            ctx.get_diff_attr_bool_bi(tcid, bslot, bcls::SLICE::G_SHIFT_ENABLE, false)
                .assert_empty();
            let f_ram = ctx.get_diff_attr_bit(tcid, bslot, bcls::SLICE::F_RAM_ENABLE, 0);
            let g_ram = ctx.get_diff_attr_bit(tcid, bslot, bcls::SLICE::G_RAM_ENABLE, 0);
            let (f_ram, g_ram, ram) = Diff::split(f_ram, g_ram);
            ctx.insert_bel_attr_bool(tcid, bslot, bcls::SLICE::FF_SR_ENABLE, xlat_bit(!ram));
            let f_shift_d =
                ctx.get_diff_attr_bool_bi(tcid, bslot, bcls::SLICE::F_SHIFT_ENABLE, true);
            let g_shift_d =
                ctx.get_diff_attr_bool_bi(tcid, bslot, bcls::SLICE::G_SHIFT_ENABLE, true);
            let f_shift = f_ram.combine(&f_shift_d);
            let g_shift = g_ram.combine(&g_shift_d);
            ctx.insert_bel_attr_bool(tcid, bslot, bcls::SLICE::F_RAM_ENABLE, xlat_bit(f_ram));
            ctx.insert_bel_attr_bool(tcid, bslot, bcls::SLICE::G_RAM_ENABLE, xlat_bit(g_ram));
            ctx.insert_bel_attr_bool(tcid, bslot, bcls::SLICE::F_SHIFT_ENABLE, xlat_bit(f_shift));
            ctx.insert_bel_attr_bool(tcid, bslot, bcls::SLICE::G_SHIFT_ENABLE, xlat_bit(g_shift));

            ctx.collect_bel_attr(tcid, bslot, bcls::SLICE::DIF_MUX);
            ctx.collect_bel_attr(tcid, bslot, bcls::SLICE::DIG_MUX);
            ctx.collect_bel_attr(tcid, bslot, bcls::SLICE::F_SLICEWE0USED);
            ctx.collect_bel_attr(tcid, bslot, bcls::SLICE::F_SLICEWE1USED);
            ctx.collect_bel_attr(tcid, bslot, bcls::SLICE::G_SLICEWE0USED);
            ctx.collect_bel_attr(tcid, bslot, bcls::SLICE::G_SLICEWE1USED);
        }

        // muxes
        ctx.collect_bel_attr(tcid, bslot, bcls::SLICE::FXMUX);
        ctx.collect_bel_attr(tcid, bslot, bcls::SLICE::GYMUX);
        ctx.collect_bel_attr(tcid, bslot, bcls::SLICE::DXMUX);
        ctx.collect_bel_attr(tcid, bslot, bcls::SLICE::DYMUX);
        if is_m {
            ctx.collect_bel_attr(tcid, bslot, bcls::SLICE::XBMUX);
            ctx.collect_bel_attr(tcid, bslot, bcls::SLICE::YBMUX);
        }

        // FFs
        ctx.collect_bel_attr_bi(tcid, bslot, bcls::SLICE::FF_SR_SYNC);
        ctx.collect_bel_attr_bi(tcid, bslot, bcls::SLICE::FF_LATCH);
        ctx.collect_bel_attr(tcid, bslot, bcls::SLICE::FF_REV_ENABLE);
        ctx.collect_bel_attr_bi(tcid, bslot, bcls::SLICE::FFX_SRVAL);
        ctx.collect_bel_attr_bi(tcid, bslot, bcls::SLICE::FFY_SRVAL);
        ctx.collect_bel_attr_bi(tcid, bslot, bcls::SLICE::FFX_INIT);
        ctx.collect_bel_attr_bi(tcid, bslot, bcls::SLICE::FFY_INIT);

        // inverts
        let int = tcls::INT;
        ctx.collect_bel_input_inv_int_bi(&[int], tcid, bslot, bcls::SLICE::CLK);
        ctx.collect_bel_input_inv_int_bi(&[int], tcid, bslot, bcls::SLICE::SR);
        ctx.collect_bel_input_inv_int_bi(&[int], tcid, bslot, bcls::SLICE::CE);
        ctx.collect_bel_input_inv_bi(tcid, bslot, bcls::SLICE::BX);
        ctx.collect_bel_input_inv_bi(tcid, bslot, bcls::SLICE::BY);
    }
}

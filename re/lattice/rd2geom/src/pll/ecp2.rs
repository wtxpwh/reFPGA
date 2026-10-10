use prjcombine_ecp::{
    chip::{ChipKind, PllPad, SpecialIoKey},
    defs::{bcls, bslots, tcls, wires},
};
use prjcombine_interconnect::{
    db::WireSlotIdExt,
    dir::{DirH, DirV},
};

use crate::ChipContext;

impl ChipContext<'_> {
    pub(super) fn prep_pll_ecp2(&mut self) {
        for (loc, bcrd) in self.chip.plls() {
            if loc.quad.v == DirV::S && loc.idx == 0 {
                let cell_pll = match loc.quad.h {
                    DirH::W => bcrd.cell.delta(2, 0),
                    DirH::E => bcrd.cell.delta(-2, 0),
                };

                self.hybrid_out_rc_cell(bcrd, cell_pll, "JCLKOP_PLL", wires::OUT_PLL_CLKOP.cell(0));
                self.hybrid_out_rc_cell(bcrd, cell_pll, "JCLKOS_PLL", wires::OUT_PLL_CLKOS.cell(0));
                self.hybrid_out_rc_cell(bcrd, cell_pll, "JCLKOK_PLL", wires::OUT_PLL_CLKOK.cell(0));

                let bcrd = bcrd.bel(bslots::DLL[0]);
                self.hybrid_out_rc(bcrd, "JCLKOP_DLL", wires::OUT_DLL_CLKOP.cell(0));
                self.hybrid_out_rc(bcrd, "JCLKOS_DLL", wires::OUT_DLL_CLKOS.cell(0));

                let bcrd = bcrd.bel(bslots::CLKDIV[0]);
                self.hybrid_out_rc(bcrd, "JCDIV1_CLKDIV", wires::OUT_CLKDIV_CDIV1.cell(0));
                self.hybrid_out_rc(bcrd, "JCDIV2_CLKDIV", wires::OUT_CLKDIV_CDIV2.cell(0));
                self.hybrid_out_rc(bcrd, "JCDIV4_CLKDIV", wires::OUT_CLKDIV_CDIV4.cell(0));
                self.hybrid_out_rc(bcrd, "JCDIV8_CLKDIV", wires::OUT_CLKDIV_CDIV8.cell(0));

                let cell = bcrd.cell;
                let eclk0_out = self.rc_wire(cell, "JFRC0");
                let eclk1_out = self.rc_wire(cell, "JFRC1");
                self.bind_wire(eclk0_out, cell.wire(wires::PLL_TO_ECLK[0]));
                self.bind_wire(eclk1_out, cell.wire(wires::PLL_TO_ECLK[1]));
            } else {
                self.hybrid_out_rc(bcrd, "JCLKOP_SPLL", wires::OUT_PLL_CLKOP.cell(0));
                self.hybrid_out_rc(bcrd, "JCLKOS_SPLL", wires::OUT_PLL_CLKOS.cell(0));
                self.hybrid_out_rc(bcrd, "JCLKOK_SPLL", wires::OUT_PLL_CLKOK.cell(0));
            }
        }

        if self.chip.kind == ChipKind::Ecp2M {
            for tcid in [tcls::SERDES_S, tcls::SERDES_N] {
                for &tcrd in &self.edev.tile_index[tcid] {
                    let bcrd = tcrd.bel(bslots::SERDES);
                    let cell = bcrd.cell.delta(
                        if bcrd.col < self.chip.col_clk { 12 } else { 13 },
                        if bcrd.row < self.chip.row_clk { -1 } else { 1 },
                    );
                    self.hybrid_out_rc_cell(
                        bcrd,
                        cell,
                        "JFF_TX_F_CLK_PCS",
                        wires::OUT_SERDES_FF_TX_F_CLK.cell(0),
                    );
                    self.hybrid_out_rc_cell(
                        bcrd,
                        cell,
                        "JFF_TX_H_CLK_PCS",
                        wires::OUT_SERDES_FF_TX_H_CLK.cell(0),
                    );
                }
            }
        }
    }

    pub(super) fn process_pll_ecp2(&mut self) {
        for (loc, bcrd) in self.chip.plls() {
            let cell = bcrd.cell;
            if loc.quad.v == DirV::S && loc.idx == 0 {
                // GPLL + DLL + CLKDIV + DLLDEL
                let cell_pll = match loc.quad.h {
                    DirH::W => cell.delta(2, 0),
                    DirH::E => cell.delta(-2, 0),
                };

                // switchbox

                self.extract_switchbox(cell.bel(bslots::BEL_INT))
                    // PLL.CLKI
                    .cell(cell_pll)
                    .output_bind_rc("CLKI", wires::IMUX_PLL_CLKI.cell(0))
                    .input_xlat_rc("JPLLCLKI0")
                    .input_xlat_rc("JPLLCLKI1")
                    .input_xlat_rc("JPLLCLKI2")
                    .input_io_rc("JPLLCLKI3", wires::OUT_IO_PADDI[0].cell(4))
                    // PLL.CLKFB
                    .output_bind_rc("CLKFB", wires::IMUX_PLL_CLKFB.cell(0))
                    .input_xlat_rc("JPLLCLKFB0")
                    .input_xlat_rc("JPLLCLKFB1")
                    .input_bind_rc("PLLCLKINTFB", wires::OUT_PLL_CLKINTFB.cell(0))
                    .input_xlat_rc("JPLLCLKFB3")
                    // DLL.CLKI, DLLDEL.CLKI
                    .cell(cell)
                    .output_bind_rc("DLLCLKI", wires::IMUX_DLL_CLKI.cell(0))
                    .output_bind_rc("DLLDELCLKI", wires::IMUX_DLLDEL_CLKI.cell(0))
                    .input_xlat_rc("JDLLCLKI0")
                    .input_xlat_rc("JDLLCLKI1")
                    .input_xlat_rc("JDLLCLKI2")
                    .input_io_rc("JDLLCLKI3", wires::OUT_IO_PADDI[0].cell(4))
                    // DLL.CLKFB
                    .output_bind_rc("DLLCLKFB", wires::IMUX_DLL_CLKFB.cell(0))
                    .input_xlat_rc("JDLLCLKFB0")
                    .input_xlat_rc("JDLLCLKFB1")
                    .input_xlat_rc("JDLLCLKFB2")
                    .input_xlat_rc("JDLLCLKFB3")
                    // PLL_TO_ECLK
                    .output_xlat_rc("JFRC0")
                    .output_xlat_rc("JFRC1")
                    .input_io_rc("JPIO0", wires::OUT_IO_PADDI[0].cell(4))
                    .input_io_rc("JPIO1", wires::OUT_IO_PADDI[0].cell(4))
                    .input_xlat_rc("JCIBCLK0")
                    .input_xlat_rc("JCIBCLK1")
                    .input_bind_rc("DLLDEL", wires::OUT_DLLDEL_CLKO.cell(0))
                    .input_bind_rc("BYPASS", wires::OUT_DLLDEL_BYPASS.cell(0))
                    .input_xlat_rc("JPLLCLKOP")
                    .input_xlat_rc("JPLLCLKOS")
                    .input_xlat_rc("DLLCLKOP")
                    .input_xlat_rc("DLLCLKOS")
                    .output_bind_rc("CLKDIVCLKI", wires::IMUX_CLKDIV_CLKI.cell(0))
                    .input_xlat_rc("JFRC0")
                    .input_xlat_rc("JFRC1");

                // PLL

                self.extract_bel_cell(cell.bel(bslots::PLL[0]), cell_pll)
                    .name_rc("PLL")
                    .suffix("PLL")
                    .input_xlat_rc(bcls::PLL::CLKI, "CLKI_PLL")
                    .input_xlat_rc(bcls::PLL::CLKFB, "CLKFB_PLL")
                    .output_xlat_rc(bcls::PLL::CLKINTFB, "CLKINTFB_PLL");

                // DLL

                self.extract_bel(cell.bel(bslots::DLL[0]))
                    .name_rc("DLL")
                    .suffix("DLL")
                    .input_xlat_rc(bcls::DLL::CLKI, "CLKI_DLL")
                    .input_xlat_rc(bcls::DLL::CLKFB, "CLKFB_DLL")
                    // ???
                    .add_wire_rc("CLKIDUTY", "JCLKIDUTY_DLL");

                // DLLDEL

                {
                    let mut bel = self.extract_bel(cell.bel(bslots::DLLDEL[0]));
                    bel.name_rc("DLLDEL")
                        .suffix("DLLDEL")
                        .input_xlat_rc(bcls::DLLDEL::CLKI, "CLKI_DLLDEL")
                        .output_xlat_rc(bcls::DLLDEL::BYPASS, "BYPASS_DLLDEL");

                    for i in 0..9 {
                        // ???
                        let wire_dll = bel.ctx.rc_wire(cell, &format!("JDCNTL{i}_DLL"));
                        let wire_dlldel = bel.ctx.rc_wire(cell, &format!("JDCNTL{i}_DLLDEL"));
                        bel.add_wire(format!("DCNTL{i}"), wire_dlldel);
                        bel.ctx.claim_pip(wire_dll, wire_dlldel);
                    }
                }

                // CLKDIV

                self.extract_bel(cell.bel(bslots::CLKDIV[0]))
                    .name_rc("CLKDIV")
                    .suffix("CLKDIV")
                    .input_xlat_rc(bcls::CLKDIV::CLKI, "CLKI_CLKDIV");

                // IO check
                let io_pll_in0 =
                    self.get_special_io_wire_in(SpecialIoKey::Pll(PllPad::PllIn0, loc));
                let io_pll_in1 =
                    self.get_special_io_wire_in(SpecialIoKey::Pll(PllPad::PllIn1, loc));
                let io_dll_in0 =
                    self.get_special_io_wire_in(SpecialIoKey::Pll(PllPad::DllIn0, loc));
                let io_dll_in1 =
                    self.get_special_io_wire_in(SpecialIoKey::Pll(PllPad::DllIn1, loc));
                assert_eq!(io_pll_in0, io_dll_in1);
                assert_eq!(io_pll_in1, io_dll_in0);
            } else {
                // SPLL

                self.extract_switchbox(cell.bel(bslots::BEL_INT))
                    .output_bind_rc("CLKI", wires::IMUX_PLL_CLKI.cell(0))
                    .input_xlat_rc("JSPLLCLKI0")
                    .input_xlat_rc("JSPLLCLKI1")
                    .input_xlat_rc("JSPLLCLKI2")
                    .input_xlat_rc("JSPLLCLKI3")
                    .output_bind_rc("CLKFB", wires::IMUX_PLL_CLKFB.cell(0))
                    .input_xlat_rc("JSPLLCLKFB0")
                    .input_io_rc("JSPLLCLKFB1", wires::OUT_IO_PADDI[0].cell(4))
                    .input_bind_rc("SPLLCLKINTFB", wires::OUT_PLL_CLKINTFB.cell(0));

                self.extract_bel(cell.bel(bslots::PLL[0]))
                    .name_rc("SPLL")
                    .suffix("SPLL")
                    .input_xlat_rc(bcls::PLL::CLKI, "CLKI_SPLL")
                    .input_xlat_rc(bcls::PLL::CLKFB, "CLKFB_SPLL")
                    .output_xlat_rc(bcls::PLL::CLKINTFB, "CLKINTFB_SPLL");
            }
        }
    }
}

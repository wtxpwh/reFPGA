use prjcombine_ecp::{
    chip::{PllPad, SpecialIoKey, SpecialLocKey},
    defs::{bcls, bslots, wires},
};
use prjcombine_interconnect::{
    db::WireSlotIdExt,
    dir::{Dir, DirH, DirV},
};

use crate::ChipContext;

impl ChipContext<'_> {
    pub(super) fn process_pll_ecp2(&mut self) {
        for (&loc, &cell) in &self.chip.special_loc {
            let SpecialLocKey::Pll(loc) = loc else {
                continue;
            };
            if loc.quad.v == DirV::S && loc.idx == 0 {
                // GPLL + DLL + CLKDIV + DLLDEL
                let cell_pll = match loc.quad.h {
                    DirH::W => cell.delta(2, 0),
                    DirH::E => cell.delta(-2, 0),
                };

                // PLL

                let clkintfb_pll = self.rc_wire(cell_pll, "CLKINTFB_PLL");
                let clkintfb = self.rc_wire(cell_pll, "PLLCLKINTFB");
                self.claim_pip(clkintfb, clkintfb_pll);

                let clki_pll = self.rc_wire(cell_pll, "CLKI_PLL");
                let clki = self.rc_wire(cell_pll, "CLKI");
                self.claim_pip(clki_pll, clki);

                let clkfb_pll = self.rc_wire(cell_pll, "CLKFB_PLL");
                let clkfb = self.rc_wire(cell_pll, "CLKFB");
                self.claim_pip(clkfb_pll, clkfb);

                let clkop_pll = self.rc_wire(cell_pll, "JCLKOP_PLL");
                let pll_clkop = self.rc_wire(cell, "JPLLCLKOP");
                let clkos_pll = self.rc_wire(cell_pll, "JCLKOS_PLL");
                let pll_clkos = self.rc_wire(cell, "JPLLCLKOS");
                self.claim_pip(pll_clkop, clkop_pll);
                self.claim_pip(pll_clkos, clkos_pll);

                let wire_clki = wires::IMUX_PLL_CLKI.cell(0);
                let wire_clkfb = wires::IMUX_PLL_CLKFB.cell(0);
                let wire_clkintfb = wires::OUT_PLL_CLKINTFB.cell(0);
                let wire_clkop = wires::OUT_PLL_CLKOP.cell(0);
                let wire_clkos = wires::OUT_PLL_CLKOS.cell(0);
                let wire_clkok = wires::OUT_PLL_CLKOK.cell(0);

                self.extract_bel(cell.bel(bslots::PLL[0]), cell_pll)
                    .name_rc("PLL")
                    .suffix("PLL")
                    .input(bcls::PLL::CLKI, wire_clki)
                    .input(bcls::PLL::CLKFB, wire_clkfb)
                    .output(bcls::PLL::CLKINTFB, wire_clkintfb)
                    .output_extra(bcls::PLL::CLKOP, wire_clkop)
                    .output_extra(bcls::PLL::CLKOS, wire_clkos)
                    .output_extra(bcls::PLL::CLKOK, wire_clkok)
                    .add_wire("CLKINTFB_PLL", clkintfb_pll)
                    .add_wire("CLKI_PLL", clki_pll)
                    .add_wire("CLKFB_PLL", clkfb_pll);

                // DLL

                let clki_dll = self.rc_wire(cell, "CLKI_DLL");
                let dll_clki = self.rc_wire(cell, "DLLCLKI");
                self.claim_pip(clki_dll, dll_clki);

                let clkfb_dll = self.rc_wire(cell, "CLKFB_DLL");
                let dll_clkfb = self.rc_wire(cell, "DLLCLKFB");
                self.claim_pip(clkfb_dll, dll_clkfb);

                let clkop_dll = self.rc_wire(cell, "JCLKOP_DLL");
                let dll_clkop = self.rc_wire(cell, "DLLCLKOP");
                let clkos_dll = self.rc_wire(cell, "JCLKOS_DLL");
                let dll_clkos = self.rc_wire(cell, "DLLCLKOS");
                self.claim_pip(dll_clkop, clkop_dll);
                self.claim_pip(dll_clkos, clkos_dll);

                // ???
                let clkiduty = self.rc_wire(cell, "JCLKIDUTY_DLL");

                let wire_dll_clki = wires::IMUX_DLL_CLKI.cell(0);
                let wire_dll_clkfb = wires::IMUX_DLL_CLKFB.cell(0);
                let wire_dll_clkop = wires::OUT_DLL_CLKOP.cell(0);
                let wire_dll_clkos = wires::OUT_DLL_CLKOS.cell(0);

                self.extract_bel(cell.bel(bslots::DLL[0]), cell)
                    .name_rc("DLL")
                    .suffix("DLL")
                    .input(bcls::DLL::CLKI, wire_dll_clki)
                    .input(bcls::DLL::CLKFB, wire_dll_clkfb)
                    .output_extra(bcls::DLL::CLKOP, wire_dll_clkop)
                    .output_extra(bcls::DLL::CLKOS, wire_dll_clkos)
                    .add_wire("CLKI_DLL", clki_dll)
                    .add_wire("CLKFB_DLL", clkfb_dll)
                    .add_wire("CLKIDUTY", clkiduty);

                // DLLDEL

                let clki_dlldel = self.rc_wire(cell, "CLKI_DLLDEL");
                let dlldel_clki = self.rc_wire(cell, "DLLDELCLKI");
                self.claim_pip(clki_dlldel, dlldel_clki);

                let bypass_dlldel = self.rc_wire(cell, "BYPASS_DLLDEL");
                let dlldel_bypass = self.rc_wire(cell, "BYPASS");
                self.claim_pip(dlldel_bypass, bypass_dlldel);

                let clko_dlldel = self.rc_wire(cell, "JCLKO_DLLDEL");
                let dlldel_clko = self.rc_wire(cell, "DLLDEL");
                self.claim_pip(dlldel_clko, clko_dlldel);

                let wire_dlldel_clki = wires::IMUX_DLLDEL_CLKI.cell(0);
                let wire_dlldel_clko = wires::OUT_DLLDEL_CLKO.cell(0);
                let wire_dlldel_bypass = wires::OUT_DLLDEL_BYPASS.cell(0);

                {
                    let mut bel = self.extract_bel(cell.bel(bslots::DLLDEL[0]), cell);
                    bel.name_rc("DLLDEL")
                        .suffix("DLLDEL")
                        .input(bcls::DLLDEL::CLKI, wire_dlldel_clki)
                        .add_wire("CLKI_DLL", clki_dlldel)
                        .output(bcls::DLLDEL::CLKO, wire_dlldel_clko)
                        .output(bcls::DLLDEL::BYPASS, wire_dlldel_bypass)
                        .add_wire("CLKO_DLLDEL", clko_dlldel)
                        .add_wire("BYPASS_DLLDEL", bypass_dlldel);

                    for i in 0..9 {
                        // ???
                        let wire_dll = bel.ctx.rc_wire(cell, &format!("JDCNTL{i}_DLL"));
                        let wire_dlldel = bel.ctx.rc_wire(cell, &format!("JDCNTL{i}_DLLDEL"));
                        bel.add_wire(format!("DCNTL{i}"), wire_dlldel);
                        bel.ctx.claim_pip(wire_dll, wire_dlldel);
                    }
                }

                // CLKDIV

                let clki_clkdiv = self.rc_wire(cell, "CLKI_CLKDIV");
                let clki_clkdiv_in = self.rc_wire(cell, "CLKDIVCLKI");
                self.claim_pip(clki_clkdiv, clki_clkdiv_in);

                let wire_clkdiv_clki = wires::IMUX_CLKDIV_CLKI.cell(0);
                let wire_clkdiv_cdiv1 = wires::OUT_CLKDIV_CDIV1.cell(0);
                let wire_clkdiv_cdiv2 = wires::OUT_CLKDIV_CDIV2.cell(0);
                let wire_clkdiv_cdiv4 = wires::OUT_CLKDIV_CDIV4.cell(0);
                let wire_clkdiv_cdiv8 = wires::OUT_CLKDIV_CDIV8.cell(0);

                self.extract_bel(cell.bel(bslots::CLKDIV[0]), cell)
                    .name_rc("CLKDIV")
                    .suffix("CLKDIV")
                    .input(bcls::CLKDIV::CLKI, wire_clkdiv_clki)
                    .add_wire("CLKI", clki_clkdiv)
                    .output_extra(bcls::CLKDIV::CDIV1, wire_clkdiv_cdiv1)
                    .output_extra(bcls::CLKDIV::CDIV2, wire_clkdiv_cdiv2)
                    .output_extra(bcls::CLKDIV::CDIV4, wire_clkdiv_cdiv4)
                    .output_extra(bcls::CLKDIV::CDIV8, wire_clkdiv_cdiv8);

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

                // switchbox
                let bel_eclk_root = self.chip.bel_eclk_root(Dir::H(loc.quad.h));
                let eclk0_in = self.naming.bel_wire(bel_eclk_root, "ECLK0_IN");

                let clki0 = self.rc_wire(cell_pll, "JPLLCLKI0");
                let clki1 = self.rc_wire(cell_pll, "JPLLCLKI1");
                let clki2 = self.rc_wire(cell_pll, "JPLLCLKI2");
                let clki3 = self.rc_wire(cell_pll, "JPLLCLKI3");
                let clkfb0 = self.rc_wire(cell_pll, "JPLLCLKFB0");
                let clkfb1 = self.rc_wire(cell_pll, "JPLLCLKFB1");
                let clkfb3 = self.rc_wire(cell_pll, "JPLLCLKFB3");
                self.claim_pip(clkfb3, eclk0_in);

                let dll_clki0 = self.rc_wire(cell, "JDLLCLKI0");
                let dll_clki1 = self.rc_wire(cell, "JDLLCLKI1");
                let dll_clki2 = self.rc_wire(cell, "JDLLCLKI2");
                let dll_clki3 = self.rc_wire(cell, "JDLLCLKI3");
                let dll_clkfb0 = self.rc_wire(cell, "JDLLCLKFB0");
                let dll_clkfb1 = self.rc_wire(cell, "JDLLCLKFB1");
                let dll_clkfb2 = self.rc_wire(cell, "JDLLCLKFB2");
                let dll_clkfb3 = self.rc_wire(cell, "JDLLCLKFB3");
                self.claim_pip(dll_clkfb0, eclk0_in);
                self.claim_pip(dll_clkfb2, clkop_dll);

                let eclk0_out = self.rc_wire(cell, "JFRC0");
                let eclk1_out = self.rc_wire(cell, "JFRC1");
                let eclk0_io = self.rc_wire(cell, "JPIO0");
                let eclk1_io = self.rc_wire(cell, "JPIO1");
                let eclk0_int = self.rc_wire(cell, "JCIBCLK0");
                let eclk1_int = self.rc_wire(cell, "JCIBCLK1");

                self.extract_switchbox(cell.bel(bslots::BEL_INT))
                    // PLL.CLKI
                    .output(clki, wire_clki)
                    .input_io(clki0, wires::OUT_IO_PADDI[0].cell(5))
                    .input_xlat(clki1)
                    .input_xlat(clki2)
                    .input_io(clki3, wires::OUT_IO_PADDI[0].cell(4))
                    // PLL.CLKFB
                    .output(clkfb, wire_clkfb)
                    .input_xlat(clkfb0)
                    .input_io(clkfb1, wires::OUT_IO_PADDI[0].cell(6))
                    .input(clkintfb, wire_clkintfb)
                    .input(clkfb3, wires::ECLK[0].cell(0))
                    // DLL.CLKI, DLLDEL.CLKI
                    .output(dll_clki, wire_dll_clki)
                    .output(dlldel_clki, wire_dlldel_clki)
                    .input_io(dll_clki0, wires::OUT_IO_PADDI[0].cell(5))
                    .input_xlat(dll_clki1)
                    .input_xlat(dll_clki2)
                    .input_io(dll_clki3, wires::OUT_IO_PADDI[0].cell(4))
                    // DLL.CLKFB
                    .output(dll_clkfb, wire_dll_clkfb)
                    .input(dll_clkfb0, wires::ECLK[0].cell(0))
                    .input_io(dll_clkfb1, wires::OUT_IO_PADDI[0].cell(7))
                    .input(dll_clkfb2, wire_dll_clkop)
                    .input_xlat(dll_clkfb3)
                    // PLL_TO_ECLK
                    .output(eclk0_out, wires::PLL_TO_ECLK[0].cell(0))
                    .output(eclk1_out, wires::PLL_TO_ECLK[1].cell(0))
                    .input_io(eclk0_io, wires::OUT_IO_PADDI[0].cell(4))
                    .input_io(eclk1_io, wires::OUT_IO_PADDI[0].cell(4))
                    .input_xlat(eclk0_int)
                    .input_xlat(eclk1_int)
                    .input(dlldel_clko, wire_dlldel_clko)
                    .input(dlldel_bypass, wire_dlldel_bypass)
                    .input(pll_clkop, wire_clkop)
                    .input(pll_clkos, wire_clkos)
                    .input(dll_clkop, wire_dll_clkop)
                    .input(dll_clkos, wire_dll_clkos)
                    .output(clki_clkdiv_in, wire_clkdiv_clki);
            } else {
                // SPLL

                let clkintfb_pll = self.rc_wire(cell, "CLKINTFB_SPLL");
                let clkintfb = self.rc_wire(cell, "SPLLCLKINTFB");
                self.claim_pip(clkintfb, clkintfb_pll);

                let clki_pll = self.rc_wire(cell, "CLKI_SPLL");
                let clki = self.rc_wire(cell, "CLKI");
                self.claim_pip(clki_pll, clki);

                let clkfb_pll = self.rc_wire(cell, "CLKFB_SPLL");
                let clkfb = self.rc_wire(cell, "CLKFB");
                self.claim_pip(clkfb_pll, clkfb);

                let wire_clki = wires::IMUX_PLL_CLKI.cell(0);
                let wire_clkfb = wires::IMUX_PLL_CLKFB.cell(0);
                let wire_clkintfb = wires::OUT_PLL_CLKINTFB.cell(0);
                let wire_clkop = wires::OUT_PLL_CLKOP.cell(0);
                let wire_clkos = wires::OUT_PLL_CLKOS.cell(0);
                let wire_clkok = wires::OUT_PLL_CLKOK.cell(0);

                self.extract_bel(cell.bel(bslots::PLL[0]), cell)
                    .name_rc("SPLL")
                    .suffix("SPLL")
                    .input(bcls::PLL::CLKI, wire_clki)
                    .input(bcls::PLL::CLKFB, wire_clkfb)
                    .output(bcls::PLL::CLKINTFB, wire_clkintfb)
                    .output_extra(bcls::PLL::CLKOP, wire_clkop)
                    .output_extra(bcls::PLL::CLKOS, wire_clkos)
                    .output_extra(bcls::PLL::CLKOK, wire_clkok)
                    .add_wire("CLKINTFB_PLL", clkintfb_pll)
                    .add_wire("CLKI_PLL", clki_pll)
                    .add_wire("CLKFB_PLL", clkfb_pll);

                let clki0 = self.rc_wire(cell, "JSPLLCLKI0");
                let clki1 = self.rc_wire(cell, "JSPLLCLKI1");
                let clki2 = self.rc_wire(cell, "JSPLLCLKI2");
                let clki3 = self.rc_wire(cell, "JSPLLCLKI3");
                let clkfb0 = self.rc_wire(cell, "JSPLLCLKFB0");
                let clkfb1 = self.rc_wire(cell, "JSPLLCLKFB1");

                self.extract_switchbox(cell.bel(bslots::BEL_INT))
                    .output(clki, wire_clki)
                    .input_io(clki0, wires::OUT_IO_PADDI[0].cell(3))
                    .input_xlat(clki1)
                    .input_xlat(clki2)
                    .input_io(clki3, wires::OUT_IO_PADDI[0].cell(2))
                    .output(clkfb, wire_clkfb)
                    .input_xlat(clkfb0)
                    .input_io(clkfb1, wires::OUT_IO_PADDI[0].cell(4))
                    .input(clkintfb, wire_clkintfb);
            }
        }
    }
}

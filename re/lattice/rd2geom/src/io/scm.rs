use prjcombine_ecp::{
    chip::{Chip, IoGroupKind, SpecialLocKey},
    defs::{bcls, bslots, tcls, wires},
};
use prjcombine_interconnect::{
    db::WireSlotIdExt,
    dir::{Dir, DirH, DirHV, DirV},
    grid::{BelCoord, CellCoord},
};

use crate::{ChipContext, RcMode};

impl ChipContext<'_> {
    pub fn xlat_io_loc_scm(&self, bcrd: BelCoord) -> (CellCoord, &'static str) {
        let idx = bslots::IO.index_of(bcrd.slot).unwrap();
        let abcd = ["A", "B", "C", "D"][idx % 4];
        let cidx = idx / 4;
        match self.chip.get_io_edge(bcrd) {
            Dir::H(edge) => {
                let kind = match edge {
                    DirH::W => self.chip.rows[bcrd.row].io_w,
                    DirH::E => self.chip.rows[bcrd.row].io_e,
                };
                let dy = match kind {
                    IoGroupKind::Quad => 0,
                    IoGroupKind::Dozen => 2 - cidx,
                    _ => unreachable!(),
                };
                (bcrd.cell.delta(0, dy as i32), abcd)
            }
            Dir::V(_) => (bcrd.cell.delta(cidx as i32, 0), abcd),
        }
    }

    pub(super) fn prep_io_scm(&mut self) {
        for cell in self.edev.die_cells(Chip::DIE) {
            for iob in 0..12 {
                let bcrd = cell.bel(bslots::IO[iob]);
                if self.edev.has_bel(bcrd) {
                    let (cell_io, abcd) = self.xlat_io_loc_scm(bcrd);
                    self.hybrid_out_rc_io_cell(
                        bcrd,
                        cell_io,
                        format!("JINDDCK{abcd}"),
                        wires::OUT_IO_INCK[iob].cell(0),
                    );
                }
            }
        }
        for hv in DirHV::DIRS {
            let cell_tile = self.chip.bel_pll(hv, 0).cell;
            let cell = cell_tile.with_row(self.chip.row_edge(hv.v));
            for i in 0..2 {
                let bcrd = cell_tile.bel(bslots::PLL[i]);
                let abcd = ['A', 'B'][i];
                self.hybrid_out(
                    bcrd,
                    self.rc_corner_wire(cell, &format!("JCLKOP{abcd}_PLL")),
                    wires::OUT_PLL_CLKOP[i].cell(0),
                );
                self.hybrid_out(
                    bcrd,
                    self.rc_corner_wire(cell, &format!("JCLKOS{abcd}_PLL")),
                    wires::OUT_PLL_CLKOS[i].cell(0),
                );
            }
            let num_dll = match hv.v {
                DirV::S => 4,
                DirV::N => 2,
            };
            for i in 0..num_dll {
                let bcrd = cell_tile.bel(bslots::DLL[i]);
                let abcd = ['C', 'D', 'E', 'F'][i];
                self.hybrid_out(
                    bcrd,
                    self.rc_corner_wire(cell, &format!("JCLKOP{abcd}_DLL")),
                    wires::OUT_DLL_CLKOP[i].cell(0),
                );
                self.hybrid_out(
                    bcrd,
                    self.rc_corner_wire(cell, &format!("JCLKOS{abcd}_DLL")),
                    wires::OUT_DLL_CLKOS[i].cell(0),
                );
            }
        }
    }

    pub(super) fn process_eclk_scm(&mut self) {
        for (bank, edge) in [
            (1, Dir::N),
            (2, Dir::E),
            (4, Dir::S),
            (5, Dir::S),
            (7, Dir::W),
        ] {
            let cell_tile = self.chip.special_loc[&SpecialLocKey::Bc(bank)];
            let cell = if bank == 1 {
                cell_tile.delta(-1, 0)
            } else {
                cell_tile
            };
            {
                let mut ex = self.extract_switchbox(cell_tile.bel(bslots::CLK_INT));
                ex.cell(cell)
                    .output_bind_rc_io("CLKIA", wires::IMUX_CLKDIV_CLKI[0].cell(1))
                    .output_bind_rc_io("CLKIB", wires::IMUX_CLKDIV_CLKI[1].cell(1))
                    .output_bind_rc_io("CLKIC", wires::IMUX_CLKDIV_CLKI[2].cell(1))
                    .output_bind_rc_io("CLKID", wires::IMUX_CLKDIV_CLKI[3].cell(1))
                    .input_bind_rc_io("ELSRA", wires::OUT_CLKDIV_ELSR[0].cell(1))
                    .input_bind_rc_io("ELSRB", wires::OUT_CLKDIV_ELSR[1].cell(1))
                    .input_bind_rc_io("ELSRC", wires::OUT_CLKDIV_ELSR[2].cell(1))
                    .input_bind_rc_io("ELSRD", wires::OUT_CLKDIV_ELSR[3].cell(1))
                    .input_xlat_rc_io("JCIB_0")
                    .input_xlat_rc_io("JCIB_1");
                if edge != Dir::N {
                    ex.input_xlat_rc_io("JXXCMCLKA")
                        .input_xlat_rc_io("JXXCMCLKB")
                        .input_xlat_rc_io("JXXCMCLKC")
                        .input_xlat_rc_io("JXXCMCLKD")
                        .input_xlat_rc_io("JXXXMCLKE")
                        .input_xlat_rc_io("JXXXMCLKF")
                        .input_xlat_rc_io("JXXCNCLKA")
                        .input_xlat_rc_io("JXXCNCLKB")
                        .input_xlat_rc_io("JXXCNCLKC")
                        .input_xlat_rc_io("JXXCNCLKD")
                        .input_xlat_rc_io("JXXXNCLKE")
                        .input_xlat_rc_io("JXXXNCLKF");
                }
                if matches!(edge, Dir::H(_)) {
                    ex.input_xlat_rc_io("JXXXMCLKG")
                        .input_xlat_rc_io("JXXXMCLKH")
                        .input_xlat_rc_io("JXXXNCLKG")
                        .input_xlat_rc_io("JXXXNCLKH");
                }
                for i in 0..8 {
                    ex.output_int(wires::ECLK[i].cell(1))
                        .input_io_rc_io(format!("JPIO_{i}"), wires::IO_GCLK[i].cell(1));
                }
            }
            for i in 0..4 {
                let bcrd = cell_tile.bel(bslots::CLKDIV[i]);
                let abcd = ['A', 'B', 'C', 'D'][i];

                self.extract_bel_cell(bcrd, cell)
                    .name(format!("CLKDIV{bank}{abcd}"))
                    .input_xlat_rc_io(bcls::CLKDIV::LSR, format!("JLSR{abcd}_CLKDIV"))
                    .input_xlat_rc_io(bcls::CLKDIV::CLKI, format!("CLKI{abcd}_CLKDIV"))
                    .output_bind_rc_io(
                        bcls::CLKDIV::CLKO,
                        format!("JCLKO{abcd}_CLKDIV"),
                        wires::OUT_CLKDIV_CLKO[i].cell(1),
                    )
                    .output_xlat_rc_io(bcls::CLKDIV::ELSR, format!("ELSR{abcd}_CLKDIV"))
                    .claim_pip("CLKO", "CLKI");
            }
        }
    }

    pub(super) fn process_pll_scm(&mut self) {
        let chip = self.chip;
        for hv in DirHV::DIRS {
            let corner = match hv {
                DirHV::SW => "LL",
                DirHV::SE => "LR",
                DirHV::NW => "UL",
                DirHV::NE => "UR",
            };
            let cell_tile = chip.bel_pll(hv, 0).cell;
            let cell = cell_tile.with_row(chip.row_edge(hv.v));

            {
                let bcrd = cell_tile.bel(bslots::PLL_SMI);
                let mut bel = self.extract_bel_cell(bcrd, cell);
                bel.name_null()
                    .input_xlat_rc_corner(bcls::PLL_SMI::SMICLK, "JSMICLK")
                    .input_xlat_rc_corner(bcls::PLL_SMI::SMIRSTN, "JSMIRSTN")
                    .input_xlat_rc_corner(bcls::PLL_SMI::SMIRD, "JSMIRD")
                    .input_xlat_rc_corner(bcls::PLL_SMI::SMIWR, "JSMIWR")
                    .input_xlat_rc_corner(bcls::PLL_SMI::SMIWDATA, "JSMIWDATA");
                for i in 0..10 {
                    bel.input_xlat_rc_corner(bcls::PLL_SMI::SMIADDR[i], format!("JSMIADDR{i}"));
                }
            }

            {
                let bcrd = cell_tile.bel(bslots::BEL_INT);
                let mut ex = self.extract_switchbox(bcrd);
                ex.cell(cell);
                for i in 0..6 {
                    if hv.v == DirV::N && i >= 4 {
                        continue;
                    }
                    let abcd = ['A', 'B', 'C', 'D', 'E', 'F'][i];
                    if i < 2 {
                        ex.input_bind_rc_corner(
                            format!("CLKINTFB{abcd}"),
                            wires::OUT_PLL_CLKINTFB[i].cell(0),
                        )
                        .output_bind_rc_corner(
                            format!("CLKI{abcd}"),
                            wires::IMUX_PLL_CLKI[i].cell(0),
                        )
                        .output_bind_rc_corner(
                            format!("CLKFB{abcd}"),
                            wires::IMUX_PLL_CLKFB[i].cell(0),
                        );
                    } else {
                        ex.output_bind_rc_corner(
                            format!("CLKI{abcd}"),
                            wires::IMUX_DLL_CLKI[i - 2].cell(0),
                        )
                        .output_bind_rc_corner(
                            format!("CLKFB{abcd}"),
                            wires::IMUX_DLL_CLKFB[i - 2].cell(0),
                        )
                        .output_bind_rc_corner(
                            format!("CLKIDUTY{abcd}"),
                            wires::IMUX_DLL_CLKIDUTY[i - 2].cell(0),
                        );
                    }
                    ex.input_xlat_rc_corner(format!("JSC_IN_{abcd}"))
                        .input_xlat_rc_corner(format!("JSC_FB_{abcd}"))
                        .input_xlat_rc_corner(format!("CLKOP{abcd}"))
                        .input_xlat_rc_corner(format!("CLKOS{abcd}"))
                        .input_io_rc_corner(format!("JPIO_IN_{abcd}"), wires::IO_PLL[i].cell(0))
                        .input_io_rc_corner(
                            format!("JPIO_FB_{abcd}"),
                            wires::IO_PLL[i ^ 1].cell(0),
                        );
                }
                for i in 0..8 {
                    let lr = match hv.h {
                        DirH::W => 'L',
                        DirH::E => 'R',
                    };
                    let wire = chip
                        .clk_root()
                        .with_col(chip.col_edge(hv.h))
                        .wire(wires::ECLK[i]);
                    let wname = ex.ctx.rc_corner_wire(cell, &format!("JVECX{i:02}00"));
                    let wname_in = ex.ctx.rc_corner_wire(cell, &format!("JVEC{lr}{i:02}00"));
                    ex.add_wire(format!("ECLK_WE{i}_IN"), wname_in);
                    ex.ctx.claim_pip(wname, wname_in);
                    ex.ctx
                        .claim_pip_bi(wname_in, ex.ctx.naming.interconnect[&wire]);
                    ex.ctx.bind_wire(wname, wire);
                    ex.ctx.claim_node(wname);
                    ex.input_xlat(wname);
                }

                if hv.v == DirV::S {
                    let cell_eclk = chip.special_loc[&SpecialLocKey::Bc(match hv.h {
                        DirH::W => 5,
                        DirH::E => 4,
                    })];
                    for i in 0..8 {
                        let wire = cell_eclk.wire(wires::ECLK[i]);
                        let wname = ex.ctx.rc_corner_wire(cell, &format!("JHECX{i:02}00"));
                        let wname_in = ex.ctx.rc_corner_wire(cell, &format!("JHECB{i:02}00"));
                        ex.add_wire(format!("ECLK_S{i}_IN"), wname_in);
                        ex.ctx.claim_pip(wname, wname_in);
                        ex.ctx
                            .claim_pip_bi(wname_in, ex.ctx.naming.interconnect[&wire]);
                        ex.ctx.bind_wire(wname, wire);
                        ex.ctx.claim_node(wname);
                        ex.input_xlat(wname);
                    }
                }
            }

            for idx in 0..6 {
                let bcrd = cell_tile.bel(if idx < 2 {
                    bslots::PLL[idx]
                } else {
                    bslots::DLL[idx - 2]
                });
                if !self.edev.has_bel(bcrd) {
                    continue;
                }
                let abcd = ['A', 'B', 'C', 'D', 'E', 'F'][idx];
                let kind = if idx < 2 { "PLL" } else { "DLL" };

                let mut bel = self.extract_bel_cell(bcrd, cell);
                bel.name(format!("{kind}_{corner}C{abcd}"));

                for pin in [
                    "SMIRD", "SMIWR", "SMICLK", "SMIRSTN", "SMIWDATA", "SMIADDR0", "SMIADDR1",
                    "SMIADDR2", "SMIADDR3", "SMIADDR4", "SMIADDR5", "SMIADDR6", "SMIADDR7",
                    "SMIADDR8", "SMIADDR9",
                ] {
                    let wire_common = bel.ctx.rc_corner_wire(cell, &format!("J{pin}"));
                    bel.add_wire_rc_corner(pin, format!("{pin}{abcd}_{kind}"))
                        .add_wire_rc_corner(format!("{pin}_IN"), format!("{pin}{abcd}"))
                        .claim_pip(pin, format!("{pin}_IN"))
                        .claim_pip_from(format!("{pin}_IN"), wire_common);
                }

                let pins = if idx < 2 {
                    [(bcls::PLL::RST, "RST")].as_slice()
                } else {
                    const {
                        [
                            (bcls::DLL::RST, "RST"),
                            (bcls::DLL::DTCCST.index_const(0), "DTCCST0"),
                            (bcls::DLL::DTCCST.index_const(1), "DTCCST1"),
                            (bcls::DLL::UDDCNTL, "UDDCNTL"),
                        ]
                    }
                    .as_slice()
                };
                for &(pin, pname) in pins {
                    let pname_alt0 = if pname == "RST" && kind == "DLL" {
                        "RSTN"
                    } else {
                        pname
                    };
                    let pname_alt1 = if pname == "RST" { "RSTN" } else { pname };
                    bel.add_wire_rc_corner(pname, format!("{pname_alt0}{abcd}_{kind}"))
                        .add_wire_rc_corner(format!("{pname}_IN"), format!("{pname_alt0}{abcd}"))
                        .claim_pip(pname, format!("{pname}_IN"))
                        .input_xlat_rename_rc_mode(
                            pin,
                            format!("{pname}_CIB"),
                            format!("JCIB_{pname_alt1}{abcd}"),
                            RcMode::Corner,
                        )
                        .claim_pip(format!("{pname}_IN"), format!("{pname}_CIB"));
                }

                let pins = if idx < 2 {
                    [
                        (bcls::PLL::LOCK, "LOCK"),
                        (bcls::PLL::SMIRDATA, "SMIRDATA"),
                        (bcls::PLL::TPREF, "TPREF"),
                        (bcls::PLL::TPFB, "TPFB"),
                    ]
                    .as_slice()
                } else {
                    [(bcls::DLL::LOCK, "LOCK"), (bcls::DLL::SMIRDATA, "SMIRDATA")].as_slice()
                };

                for &(pin, pname) in pins {
                    bel.add_wire_rc_corner(pname, format!("{pname}{abcd}_{kind}"))
                        .add_wire_rc_corner(format!("{pname}_OUT"), format!("{pname}{abcd}"))
                        .claim_pip(format!("{pname}_OUT"), pname)
                        .output_xlat_rename_rc_corner(
                            pin,
                            format!("{pname}_CIB"),
                            format!("JCIB_{pname}{abcd}"),
                        )
                        .claim_pip(format!("{pname}_CIB"), format!("{pname}_OUT"));
                }

                if kind == "PLL" {
                    bel.input_xlat_rc_corner(bcls::PLL::CLKI, format!("CLKI{abcd}_PLL"))
                        .input_xlat_rc_corner(bcls::PLL::CLKFB, format!("CLKFB{abcd}_PLL"))
                        .output_xlat_rc_corner(bcls::PLL::CLKINTFB, format!("CLKINTFB{abcd}_PLL"))
                        .output_xlat_rc_corner(bcls::PLL::CLKOP, format!("JCLKOP{abcd}_PLL"))
                        .output_xlat_rc_corner(bcls::PLL::CLKOS, format!("JCLKOS{abcd}_PLL"));
                } else {
                    for j in 0..9 {
                        bel.add_wire_rc_corner(
                            format!("DCNTL{j}"),
                            format!("DCNTL{j}{abcd}_{kind}"),
                        )
                        .add_wire_rc_corner(format!("DCNTL{j}_OUT"), format!("DCNTL{j}{abcd}"))
                        .claim_pip(format!("DCNTL{j}_OUT"), format!("DCNTL{j}"));
                    }

                    bel.input_xlat_rc_corner(bcls::DLL::CLKI, format!("CLKI{abcd}_DLL"))
                        .input_xlat_rc_corner(bcls::DLL::CLKFB, format!("CLKFB{abcd}_DLL"))
                        .input_xlat_rc_corner(bcls::DLL::CLKIDUTY, format!("CLKIDUTY{abcd}_DLL"))
                        .output_xlat_rc_corner(bcls::DLL::CLKOP, format!("JCLKOP{abcd}_DLL"))
                        .output_xlat_rc_corner(bcls::DLL::CLKOS, format!("JCLKOS{abcd}_DLL"));
                }
            }
            for i in 0..2 {
                let bcrd = cell_tile.bel(bslots::DLL_DCNTL[i]);
                let mut bel = self.extract_bel_cell(bcrd, cell);
                bel.name_null();

                for j in 0..9 {
                    let wire_in = bel.ctx.rc_corner_wire(cell, &format!("JDCNTL{i}_IN_{j}"));
                    let wire_out = bel.ctx.rc_corner_wire(cell, &format!("JDCNTL{i}_OUT_{j}"));
                    bel.input_xlat(bcls::DLL_DCNTL::DCNTL_IN[j], wire_in)
                        .output_xlat(bcls::DLL_DCNTL::DCNTL_OUT[j], wire_out);

                    let abcd = ['C', 'D'][i];
                    let wire_dll = bel.ctx.rc_corner_wire(cell, &format!("DCNTL{j}{abcd}"));
                    bel.ctx.claim_pip(wire_out, wire_dll);
                    if hv.v == DirV::S {
                        let abcd = ['E', 'F'][i];
                        let wire_dll = bel.ctx.rc_corner_wire(cell, &format!("DCNTL{j}{abcd}"));
                        bel.ctx.claim_pip(wire_out, wire_dll);
                    }

                    for edge in ['H', 'V'] {
                        if edge == 'H' && hv == DirHV::NW {
                            continue;
                        }
                        let wire_edge = bel
                            .ctx
                            .rc_corner_wire(cell, &format!("JDCNTL{i}_{edge}_{j}"));
                        bel.ctx
                            .add_bel_wire(bcrd, format!("DCNTL_{edge}{j}"), wire_edge);
                        bel.ctx.claim_pip(wire_edge, wire_in);

                        let abcd = ['C', 'D'][i];
                        let wire_dll = bel.ctx.rc_corner_wire(cell, &format!("DCNTL{j}{abcd}"));
                        bel.ctx.claim_pip(wire_edge, wire_dll);
                        if hv.v == DirV::S {
                            let abcd = ['E', 'F'][i];
                            let wire_dll = bel.ctx.rc_corner_wire(cell, &format!("DCNTL{j}{abcd}"));
                            bel.ctx.claim_pip(wire_edge, wire_dll);
                        }

                        let wire_edge_mid = bel.ctx.claim_single_out(wire_edge);
                        bel.ctx
                            .add_bel_wire(bcrd, format!("DCNTL_{edge}{j}_MID"), wire_edge_mid);
                        let wire_edge_out = bel.ctx.claim_single_out(wire_edge_mid);
                        bel.ctx
                            .add_bel_wire(bcrd, format!("DCNTL_{edge}{j}_OUT"), wire_edge_out);
                    }
                }
            }
            if hv == DirHV::SW {
                self.extract_bel_cell(cell_tile.bel(bslots::RNET), cell)
                    .name("RNET")
                    .suffix("RNET");
            }
            if hv == DirHV::NW {
                for (slot, name, suffix) in [
                    (bslots::M0, "M0PAD", "M0"),
                    (bslots::M1, "M1PAD", "M1"),
                    (bslots::M2, "M2PAD", "M2"),
                    (bslots::M3, "M3PAD", "M3"),
                ] {
                    self.extract_bel_cell(cell_tile.bel(slot), cell)
                        .name(name)
                        .output_xlat_rc_corner(bcls::IPAD::PADDI, format!("J{suffix}_{suffix}"));
                }
                {
                    self.extract_bel_cell(cell_tile.bel(bslots::RESETN), cell)
                        .name("RESETN")
                        .output_xlat_rc_corner(bcls::IPAD::PADDI, "JRSTN_RSTN");
                }
                {
                    let cell_rdbk = chip.special_loc[&SpecialLocKey::Config];
                    let tsalln = self.rc_wire(cell_rdbk, "JTSALLN_RDBK");
                    self.extract_bel_cell(cell_tile.bel(bslots::RDCFGN), cell)
                        .name("RDCFGN")
                        .add_wire_rc_corner("RDCFGN", "RDCFGN_RDCFGN")
                        .output_xlat_rename_rc_corner(bcls::RDCFGN::RDCFGN, "RDCFGN_OUT", "JRDCFGN")
                        .add_wire_rc_corner("TSALLN_IO", "JTSALLN")
                        .input_xlat(bcls::RDCFGN::TSALLN, tsalln)
                        .claim_pip("RDCFGN_OUT", "RDCFGN")
                        .claim_pip("TSALLN_IO", "RDCFGN")
                        .claim_pip("TSALLN", "TSALLN_IO");
                }
            }
            if hv == DirHV::NE {
                for (slot, name) in [
                    (bslots::TCK, "TCK"),
                    (bslots::TMS, "TMS"),
                    (bslots::TDI, "TDI"),
                    (bslots::CCLK, "CCLK"),
                ] {
                    self.extract_bel_cell(cell_tile.bel(slot), cell)
                        .name(name)
                        .output_xlat_rc_corner_quiet(bcls::IPAD::PADDI, format!("J{name}_{name}"));
                }
            }
            if let Some(name) = match hv {
                DirHV::SW => Some("PROMON2V"),
                DirHV::SE => Some("PROMON1V"),
                _ => None,
            } {
                self.extract_bel_cell(cell_tile.bel(bslots::PROMON), cell)
                    .name(name)
                    .suffix(name);
            }
        }
    }

    fn process_single_io_scm(&mut self, bcrd: BelCoord) {
        let edge = self.chip.get_io_edge(bcrd);
        let bank = self.chip.get_io_bank(bcrd);
        let idx = bslots::IO.index_of(bcrd.slot).unwrap();
        let (cell, abcd) = self.xlat_io_loc_scm(bcrd);
        let (r, c) = self.rc(cell);
        let names = match edge {
            Dir::W => [format!("PL{r}{abcd}"), format!("IOLOGICL{r}{abcd}")],
            Dir::E => [format!("PR{r}{abcd}"), format!("IOLOGICR{r}{abcd}")],
            Dir::S => [format!("PB{c}{abcd}"), format!("IOLOGICB{c}{abcd}")],
            Dir::N => [format!("PT{c}{abcd}"), format!("IOLOGICT{c}{abcd}")],
        };
        let iol = if idx.is_multiple_of(2) && edge != Dir::N {
            "IOLOGICE"
        } else {
            "IOLOGIC"
        };

        {
            let mut ex = self.extract_switchbox(bcrd.bel(bslots::IO_INT));
            ex.cell(cell)
                .output_bind_rc_io(format!("JECLK{abcd}"), wires::IMUX_IO_ECLK[idx].cell(0))
                .output_bind_rc_io(format!("JELSR{abcd}"), wires::IMUX_IO_ELSR[idx].cell(0));
            for i in 0..8 {
                ex.input_int(wires::ECLK[i].cell(0));
            }
            if edge != Dir::N {
                ex.output_bind_rc_io(format!("JTD_{abcd}_INT"), wires::IMUX_IO_TD[idx].cell(0))
                    .input_xlat_rc_io(format!("JTD_{abcd}"))
                    .input_io_int(wires::IO_T_W.cell(0))
                    .input_io_int(wires::IO_T_E.cell(0));
            }
        }

        let mut bel = self.extract_bel_cell(bcrd, cell);
        bel.names(names)
            .input_xlat_rc_io(bcls::IO::CLK, format!("JCLK{abcd}_{iol}"))
            .input_xlat_rc_io(bcls::IO::LSR, format!("JLSR{abcd}_{iol}"))
            .input_xlat_rc_io(bcls::IO::ECLK, format!("JECLK{abcd}_{iol}"))
            .input_xlat_rc_io(bcls::IO::ELSR, format!("JELSR{abcd}_{iol}"))
            .input_xlat_rc_io(bcls::IO::CE, format!("JCE{abcd}_{iol}"))
            .output_xlat_rc_io(bcls::IO::UP, format!("JUP{abcd}_{iol}"))
            .output_xlat_rc_io(bcls::IO::INFF, format!("JINFF{abcd}_{iol}"));
        for i in 0..4 {
            bel.input_xlat_rc_io(bcls::IO::OPOS[i], format!("JOPOS{i}{abcd}_{iol}"))
                .input_xlat_rc_io(bcls::IO::ONEG[i], format!("JONEG{i}{abcd}_{iol}"))
                .output_xlat_rc_io(bcls::IO::IPOS[i], format!("JIPOS{i}{abcd}_{iol}"))
                .output_xlat_rc_io(bcls::IO::INEG[i], format!("JINEG{i}{abcd}_{iol}"));
        }
        if iol == "IOLOGICE" {
            bel.output_xlat_rc_io(bcls::IO::LOCK, format!("JLOCK{abcd}_{iol}"))
                .input_xlat_rc_io(bcls::IO::RUNAIL, format!("JRUNAIL{abcd}_{iol}"));
        }

        bel.add_wire_rc_io("PADDI_PIO", format!("JPADDI{abcd}_PIO"))
            .add_wire_rc_io("PADDI", format!("JPADDI{abcd}"))
            .claim_pip("PADDI", "PADDI_PIO")
            .add_wire_rc_io("DI_IOLOGIC", format!("DI{abcd}_{iol}"))
            .claim_pip("DI_IOLOGIC", "PADDI_PIO")
            .add_wire_rc_io("INDD_IOLOGIC", format!("JINDD{abcd}_{iol}"))
            .add_wire_rc_io("INDD", format!("JINDD{abcd}"))
            .claim_pip("INDD", "INDD_IOLOGIC")
            .output_xlat_rc_io(bcls::IO::INDDCK, format!("JINDDCK{abcd}"))
            .claim_pip("INDDCK", "PADDI")
            .claim_pip("INDDCK", "INDD")
            .input_xlat_rename_rc_io(bcls::IO::OPOS[0], "PADDO", format!("JPADDO{abcd}_PIO"))
            .add_wire_rc_io("PADDT", format!("JPADDT{abcd}_PIO"))
            .add_wire_rc_io("TD_IOLOGIC", format!("JTD{abcd}_{iol}"));

        if edge == Dir::N {
            bel.add_wire_rc_io("TD_INT", format!("JTD_{abcd}_INT"))
                .input_xlat_rc_io(bcls::IO::TD, format!("JTD_{abcd}"))
                .claim_pip("TD_INT", "TD");
        } else {
            bel.input_xlat_rename_rc_io(bcls::IO::TD, "TD_INT", format!("JTD_{abcd}_INT"));
        }
        bel.claim_pip("PADDT", "TD_INT")
            .claim_pip("TD_IOLOGIC", "TD_INT");

        bel.add_wire_rc_io("IOLDO_PIO", format!("IOLDO{abcd}_PIO"))
            .add_wire_rc_io("IOLDO_IOLOGIC", format!("IOLDO{abcd}_{iol}"))
            .claim_pip("IOLDO_PIO", "IOLDO_IOLOGIC")
            .add_wire_rc_io("IOLTO_PIO", format!("IOLTO{abcd}_PIO"))
            .add_wire_rc_io("IOLTO_IOLOGIC", format!("IOLTO{abcd}_{iol}"))
            .claim_pip("IOLTO_PIO", "IOLTO_IOLOGIC");

        let (hv, edge) = match bank {
            1 => (DirHV::NE, 'H'),
            2 => (DirHV::NE, 'V'),
            3 => (DirHV::SE, 'V'),
            4 => (DirHV::SE, 'H'),
            5 => (DirHV::SW, 'H'),
            6 => (DirHV::SW, 'V'),
            7 => (DirHV::NW, 'V'),
            _ => unreachable!(),
        };
        let cell_dll_dcntl = bel.ctx.chip.bel_pll(hv, 0).cell;
        for i in 0..9 {
            bel.add_wire_rc_io(format!("DCNTL{i}"), format!("JDCNTL{i}{abcd}_{iol}"))
                .add_wire_rc_io(format!("DCNTL{i}_IN"), format!("JDCNTL{i}{abcd}"))
                .claim_pip(format!("DCNTL{i}"), format!("DCNTL{i}_IN"));
            for j in 0..2 {
                let bel_dll_dcntl = cell_dll_dcntl.bel(bslots::DLL_DCNTL[j]);
                let wire_dll = bel
                    .ctx
                    .naming
                    .bel_wire(bel_dll_dcntl, &format!("DCNTL_{edge}{i}_OUT"));
                bel.claim_pip_from(format!("DCNTL{i}_IN"), wire_dll);
            }
        }
    }

    fn process_pictest_scm(&mut self, bcrd: BelCoord) {
        let idx = bslots::PICTEST
            .iter()
            .position(|&slot| slot == bcrd.slot)
            .unwrap();
        let edge = if bcrd.col == self.chip.col_w() {
            Dir::W
        } else if bcrd.col == self.chip.col_e() {
            Dir::E
        } else if bcrd.row == self.chip.row_s() {
            Dir::S
        } else if bcrd.row == self.chip.row_n() {
            Dir::N
        } else {
            unreachable!()
        };
        let cell = match edge {
            Dir::H(edge) => {
                let io_kind = match edge {
                    DirH::W => self.chip.rows[bcrd.row].io_w,
                    DirH::E => self.chip.rows[bcrd.row].io_e,
                };
                match io_kind {
                    IoGroupKind::Quad => bcrd.cell,
                    IoGroupKind::Dozen => bcrd.cell.delta(0, 2 - idx as i32),
                    _ => unreachable!(),
                }
            }
            Dir::V(_) => bcrd.cell.delta(idx as i32, 0),
        };
        let (r, c) = self.rc(cell);
        let name = match edge {
            Dir::W => format!("LPICTEST{r}"),
            Dir::E => format!("RPICTEST{r}"),
            Dir::S => format!("BPICTEST{c}"),
            Dir::N => format!("TPICTEST{c}"),
        };
        self.extract_bel_cell(bcrd, cell)
            .name(name)
            .suffix("PICTEST");
    }

    pub(super) fn process_io_scm(&mut self) {
        for (tcid, num_io) in [
            (tcls::IO_W4, 4),
            (tcls::IO_W12, 12),
            (tcls::IO_E4, 4),
            (tcls::IO_E12, 12),
            (tcls::IO_S4, 4),
            (tcls::IO_S12, 12),
            (tcls::IO_N4, 4),
            (tcls::IO_N8, 8),
            (tcls::IO_N12, 12),
        ] {
            for &tcrd in &self.edev.tile_index[tcid] {
                for i in 0..num_io {
                    let bcrd = tcrd.bel(bslots::IO[i]);
                    self.process_single_io_scm(bcrd);
                }
                for i in 0..num_io / 4 {
                    let bcrd = tcrd.bel(bslots::PICTEST[i]);
                    self.process_pictest_scm(bcrd);
                }
            }
        }
    }
}

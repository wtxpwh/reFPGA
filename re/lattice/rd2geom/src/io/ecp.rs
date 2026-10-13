use std::collections::BTreeMap;

use prjcombine_ecp::{
    chip::{Chip, ChipKind, IoGroupKind, RowKind},
    defs::{bcls, bslots, tcls, wires},
};
use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    db::WireSlotIdExt,
    dir::{Dir, DirH, DirV},
    grid::{BelCoord, CellCoord},
};

use crate::ChipContext;

impl ChipContext<'_> {
    fn process_single_io_ecp(&mut self, bcrd: BelCoord, mut dummy: bool, is_dqs: bool) {
        let is_ecp2 = !matches!(self.chip.kind, ChipKind::Ecp | ChipKind::Xp);
        let idx = bslots::IO.index_of(bcrd.slot).unwrap();
        let cell = bcrd.cell;
        let ab = ['A', 'B'][idx];
        let (r, c) = self.rc(cell);
        let names = match self.chip.get_io_edge(bcrd) {
            Dir::W => [format!("PL{r}{ab}"), format!("IOL_L{r}{ab}")],
            Dir::E => [format!("PR{r}{ab}"), format!("IOL_R{r}{ab}")],
            Dir::S => [format!("PB{c}{ab}"), format!("IOL_B{c}{ab}")],
            Dir::N => [format!("PT{c}{ab}"), format!("IOL_T{c}{ab}")],
        };

        if self.chip.kind == ChipKind::Xp
            && self.chip.rows.len() == 36
            && cell.col == self.chip.col_e() - 1
        {
            dummy = false;
        }

        if self.chip.kind == ChipKind::Xp
            && self.chip.rows.len() == 48
            && (cell.col == self.chip.col_w() + 1
                || cell.col == self.chip.col_w() + 2
                || cell.col == self.chip.col_e() - 1
                || cell.col == self.chip.col_e() - 2)
        {
            dummy = false;
        }

        let very_dummy = dummy
            && self.chip.rows.len() == 44
            && (cell.col == self.chip.col_w() + 1 || cell.col == self.chip.col_e() - 1);

        let mut bel = self.extract_bel(bcrd);
        bel.names(names);

        let di_iologic = bel.add_wire_rc_io_get("DI_IOLOGIC", format!("DI{ab}_IOLOGIC"));
        let indd_iologic = bel.add_wire_rc_io_get("INDD_IOLOGIC", format!("JINDD{ab}_IOLOGIC"));
        let dqs_iologic = bel.add_wire_rc_io_get("DQS_IOLOGIC", format!("JDQS{ab}_IOLOGIC"));
        let ddrclkpol_iologic =
            bel.add_wire_rc_io_get("DDRCLKPOLA_IOLOGIC", format!("JDDRCLKPOL{ab}_IOLOGIC"));
        let paddo_pio = bel.add_wire_rc_io_get("PADDO_PIO", format!("JPADDO{ab}_PIO"));
        let paddi_pio = bel.add_wire_rc_io_get("PADDI_PIO", format!("JPADDI{ab}_PIO"));
        let iolto_pio = bel.add_wire_rc_io_get("IOLTO_PIO", format!("IOLTO{ab}_PIO"));
        let ioldo_pio = bel.add_wire_rc_io_get("IOLDO_PIO", format!("IOLDO{ab}_PIO"));
        let iolto_iologic = bel.add_wire_rc_io_get("IOLTO_IOLOGIC", format!("IOLTO{ab}_IOLOGIC"));
        let ioldo_iologic = bel.add_wire_rc_io_get("IOLDO_IOLOGIC", format!("IOLDO{ab}_IOLOGIC"));
        let td_iologic = bel.add_wire_rc_io_get("TD_IOLOGIC", format!("JTD{ab}_IOLOGIC"));
        let inff_iologic = bel.add_wire_rc_io_get("INFF", format!("JINFF{ab}_IOLOGIC"));

        if is_ecp2 {
            bel.add_wire_rc_io("DQSXFER_IOLOGIC", format!("JDQSXFER{ab}_IOLOGIC"));
            bel.add_wire_rc_io("XCLK", format!("JXCLK{ab}_IOLOGIC"));
            bel.input_xlat_rc_io(bcls::IO::ECLKI, format!("JECLKI{ab}_IOLOGIC"));
            bel.input_xlat_rc_io(bcls::IO::ECLKO, format!("JECLKO{ab}_IOLOGIC"));
        }

        if very_dummy {
            bel.add_wire_rc_io("CLK", format!("JCLK{ab}_IOLOGIC"));
            bel.add_wire_rc_io("LSR", format!("JLSR{ab}_IOLOGIC"));
            bel.add_wire_rc_io("CE", format!("JCE{ab}_IOLOGIC"));
            bel.add_wire_rc_io("OPOS0", format!("JOPOS0{ab}_IOLOGIC"));
            bel.add_wire_rc_io("OPOS1", format!("JOPOS1{ab}_IOLOGIC"));
            bel.add_wire_rc_io("ONEG0", format!("JONEG0{ab}_IOLOGIC"));
            bel.add_wire_rc_io("ONEG1", format!("JONEG1{ab}_IOLOGIC"));
            bel.add_wire_rc_io("IPOS0", format!("JIPOS0{ab}_IOLOGIC"));
            bel.add_wire_rc_io("IPOS1", format!("JIPOS1{ab}_IOLOGIC"));
            bel.add_wire_rc_io("PADDT", format!("JPADDT{ab}_PIO"));
        } else {
            bel.input_xlat_rc_io(bcls::IO::CLK, format!("JCLK{ab}_IOLOGIC"));
            bel.input_xlat_rc_io(bcls::IO::LSR, format!("JLSR{ab}_IOLOGIC"));
            bel.input_xlat_rc_io(bcls::IO::CE, format!("JCE{ab}_IOLOGIC"));
            bel.input_xlat_rc_io(bcls::IO::OPOS[0], format!("JOPOS0{ab}_IOLOGIC"));
            bel.input_xlat_rc_io(bcls::IO::OPOS[1], format!("JOPOS1{ab}_IOLOGIC"));
            bel.input_xlat_rc_io(bcls::IO::ONEG[0], format!("JONEG0{ab}_IOLOGIC"));
            bel.input_xlat_rc_io(bcls::IO::ONEG[1], format!("JONEG1{ab}_IOLOGIC"));
            bel.output_xlat_rc_io(bcls::IO::IPOS[0], format!("JIPOS0{ab}_IOLOGIC"));
            bel.output_xlat_rc_io(bcls::IO::IPOS[1], format!("JIPOS1{ab}_IOLOGIC"));
            bel.input_xlat_rc_io(bcls::IO::PADDT, format!("JPADDT{ab}_PIO"));

            let bpin_oneg1_alt = bel.ctx.xlat_input(bcrd, td_iologic);
            assert_eq!(bel.bel.inputs[bcls::IO::ONEG[1]], bpin_oneg1_alt);
            let bpin_oneg0_alt = bel.ctx.xlat_input(bcrd, paddo_pio);
            assert_eq!(bel.bel.inputs[bcls::IO::ONEG[0]], bpin_oneg0_alt);
            let bpin_inff = bel.ctx.xlat_output(bcrd, inff_iologic);
            assert_eq!(bel.bel.outputs[bcls::IO::IPOS[0]], bpin_inff);

            let di = bel.add_wire_rc_io_get("DI", format!("JDI{ab}"));
            bel.output_xlat_no_name(bcls::IO::DI, di);

            if is_ecp2 {
                for i in 0..4 {
                    bel.input_xlat_rc_io(bcls::IO::DEL[i], format!("JDEL{i}{ab}_IOLOGIC"));
                }
                for i in 0..2 {
                    bel.output_xlat_rc_io(bcls::IO::QPOS[i], format!("JQPOS{i}{ab}_IOLOGIC"));
                    if idx == 1 {
                        let wire = bel.ctx.rc_io_wire(cell, &format!("JQNEG{i}A_IOLOGIC"));
                        assert_eq!(
                            bel.bel.outputs[bcls::IO::QPOS[i]],
                            bel.ctx.xlat_output(bcrd, wire)
                        );
                    }
                }
                if idx == 1 {
                    for (pin, pname) in [(bcls::IO::OPOS[0], "OPOS2"), (bcls::IO::ONEG[0], "ONEG2")]
                    {
                        let wire = bel.ctx.rc_io_wire(cell, &format!("J{pname}A_IOLOGIC"));
                        let bpin = bel.ctx.xlat_input(bcrd, wire);
                        assert_eq!(bel.bel.inputs[pin], bpin);
                    }
                }
                for pin in ["QNEG0", "QNEG1", "OPOS2", "ONEG2"] {
                    bel.add_wire_rc_io(pin, format!("J{pin}{ab}_IOLOGIC"));
                }
            }

            if !dummy {
                let indd = bel.add_wire_rc_io_get("INDD", format!("JINDD{ab}"));
                let paddi = bel.add_wire_rc_io_get("PADDI", format!("JPADDI{ab}"));
                bel.ctx.claim_pip(indd, indd_iologic);
                bel.ctx.claim_pip(paddi, paddi_pio);
                bel.ctx.claim_pip(di, indd);
                bel.ctx.claim_pip(di, paddi);
            }

            if bel.ctx.edev.bel_io_dqs(bcrd).is_some()
                || bel.ctx.chip.kind == ChipKind::Ecp
                || bel.ctx.chip.kind == ChipKind::Xp2
                || (matches!(bel.ctx.chip.kind, ChipKind::Ecp2 | ChipKind::Ecp2M)
                    && bel.ctx.chip.get_io_edge(bcrd) != Dir::N)
                || (bel.ctx.chip.kind == ChipKind::Xp
                    && bel.ctx.chip.rows.len() == 48
                    && matches!(cell.row.to_idx(), 22..26))
            {
                let dqs = bel.ctx.rc_io_wire(cell, "JDQS");
                let ddrclkpol = bel.ctx.rc_io_wire(cell, "JDDRCLKPOL");
                if idx == 0 {
                    bel.add_wire("DQS", dqs);
                    bel.add_wire("DDRCLKPOL", ddrclkpol);
                    if let Some(bcrd_dqs) = bel.ctx.edev.bel_io_dqs(bcrd) {
                        let dqs_tree = bel.ctx.naming.bel_wire(bcrd_dqs, "DQSO_TREE");
                        let ddrclkpol_tree = bel.ctx.naming.bel_wire(bcrd_dqs, "DDRCLKPOL_TREE");
                        bel.ctx.claim_pip(dqs, dqs_tree);
                        bel.ctx.claim_pip(ddrclkpol, ddrclkpol_tree);
                    }
                } else {
                    bel.add_wire_no_claim("DQS", dqs);
                    bel.add_wire_no_claim("DDRCLKPOL", ddrclkpol);
                }
                if !dummy {
                    let skip_self_dqs = match bel.ctx.chip.kind {
                        ChipKind::Ecp2 | ChipKind::Xp2 => true,
                        ChipKind::Ecp2M => bel.ctx.chip.rows.len() != 55,
                        _ => false,
                    };
                    if !(skip_self_dqs && is_dqs) {
                        bel.ctx.claim_pip(dqs_iologic, dqs);
                    }
                    bel.ctx.claim_pip(ddrclkpol_iologic, ddrclkpol);
                }
                if is_ecp2 {
                    let dqsxfer_iologic =
                        bel.ctx.rc_io_wire(cell, &format!("JDQSXFER{ab}_IOLOGIC"));
                    let dqsxfer = bel.ctx.rc_io_wire(cell, "JDQSXFER");
                    if idx == 0 {
                        bel.add_wire("DQSXFER", dqsxfer);
                        if let Some(bcrd_dqs) = bel.ctx.edev.bel_io_dqs(bcrd) {
                            let dqsxfer_tree = bel.ctx.naming.bel_wire(bcrd_dqs, "DQSXFER_TREE");
                            bel.ctx.claim_pip(dqsxfer, dqsxfer_tree);
                        }
                    } else {
                        bel.add_wire_no_claim("DQSXFER", dqsxfer);
                    }
                    bel.ctx.claim_pip(dqsxfer_iologic, dqsxfer);
                }
            }
            bel.output(bcls::IO::PADDI, wires::OUT_IO_PADDI[idx].cell(0));
        }

        bel.ctx.claim_pip(di_iologic, paddi_pio);
        bel.ctx.claim_pip(iolto_pio, iolto_iologic);
        bel.ctx.claim_pip(ioldo_pio, ioldo_iologic);
        bel.bind_wire(paddi_pio, wires::OUT_IO_PADDI[idx].cell(0));

        if very_dummy {
            bel.discard();
        }
    }

    fn process_dqs_ecp(&mut self, bcrd: BelCoord) {
        let is_ecp2 = !matches!(self.chip.kind, ChipKind::Ecp | ChipKind::Xp);
        let cell = bcrd.cell;
        let bcrd_io = bcrd.bel(bslots::IO[0]);
        let edge = self.chip.get_io_edge(bcrd_io);
        let (r, c) = self.rc(cell);
        let name = match edge {
            Dir::W => format!("LDQS{r}"),
            Dir::E => format!("RDQS{r}"),
            Dir::S => format!("BDQS{c}"),
            Dir::N => format!("TDQS{c}"),
        };
        let ddrclkpol = self.rc_io_wire(cell, "JDDRCLKPOL_DQS");
        let ddrclkpol_out = self.rc_io_wire(cell, if is_ecp2 { "CLKPOL2PIC" } else { "DDRCLKPOL" });
        let dqsdel = self.rc_io_wire(cell, "JDQSDEL_DQS");
        let dqsc = self.rc_io_wire(cell, "JDQSC_DQS");
        let dqsi = self.rc_io_wire(cell, "JDQSI_DQS");
        let dqso = self.rc_io_wire(cell, "DQSO_DQS");
        let dqso_out = self.rc_io_wire(cell, if is_ecp2 { "DQSO2PIC" } else { "DQSO" });
        let indqsa = self.rc_io_wire(cell, "JINDQSA");
        let io0_di = self.rc_io_wire(cell, "JDIA");
        let dqso_tree = self.find_single_out(dqso_out);
        let ddrclkpol_tree = self.pips_fwd[&ddrclkpol_out]
            .iter()
            .copied()
            .find(|w| !self.int_wires.contains_key(w))
            .unwrap();
        let mut bel = self.extract_bel(bcrd);
        bel.name(name).suffix("DQS");
        if edge == Dir::S || (is_ecp2 && edge == Dir::W) {
            // fuck this shit.
            bel.output_xlat_no_name(bcls::DQS::DDRCLKPOL, ddrclkpol_out);
            bel.add_wire("DDRCLKPOL", ddrclkpol);
        }
        bel.add_wire("DDRCLKPOL_OUT", ddrclkpol_out);
        bel.add_wire("DQSDEL", dqsdel);
        bel.add_wire("DQSC", dqsc);
        bel.add_wire("DQSI", dqsi);
        bel.add_wire("DQSO", dqso);
        bel.add_wire("DQSO_OUT", dqso_out);
        bel.add_wire("INDQSA", indqsa);
        if bel.ctx.chip.kind == ChipKind::Xp
            && bel.ctx.chip.rows.len() == 27
            && (matches!(cell.col.to_idx(), 13 | 29))
        {
            // ??? idk I guess the database is fucked
            bel.add_wire_no_claim("DQSO_TREE", dqso_tree);
            bel.add_wire_no_claim("DDRCLKPOL_TREE", ddrclkpol_tree);
        } else {
            bel.add_wire("DQSO_TREE", dqso_tree);
            bel.add_wire("DDRCLKPOL_TREE", ddrclkpol_tree);
        }
        let wire_io = bel.ctx.get_io_wire_in(bcrd_io);
        bel.ctx.claim_pip(dqsi, wire_io);
        bel.ctx.claim_pip(indqsa, dqsc);
        bel.ctx.claim_pip(io0_di, indqsa);
        bel.ctx.claim_pip(dqso_out, dqso);
        bel.ctx.claim_pip(dqso_tree, dqso_out);
        bel.ctx.claim_pip(ddrclkpol_out, ddrclkpol);
        bel.ctx.claim_pip(ddrclkpol_tree, ddrclkpol_out);

        if is_ecp2 {
            let dqsxfer = bel.ctx.rc_io_wire(cell, "DQSXFER_DQS");
            let dqsxfer_out = bel.ctx.rc_io_wire(cell, "DQSXFER2PIC");
            let dqsxfer_tree = bel.ctx.pips_fwd[&dqsxfer_out]
                .iter()
                .copied()
                .find(|w| !bel.ctx.int_wires.contains_key(w))
                .unwrap();
            bel.add_wire("DQSXFER", dqsxfer);
            bel.add_wire("DQSXFER_OUT", dqsxfer_out);
            bel.add_wire("DQSXFER_TREE", dqsxfer_tree);
            bel.ctx.claim_pip(dqsxfer_out, dqsxfer);
            bel.ctx.claim_pip(dqsxfer_tree, dqsxfer_out);

            let xclk = bel.ctx.rc_io_wire(cell, "JXCLK_DQS");
            let xclk_iologic = bel.ctx.rc_io_wire(cell, "JXCLKA_IOLOGIC");
            bel.add_wire("XCLK", xclk);
            bel.ctx.claim_pip(xclk, xclk_iologic);
        }

        let bcrd_dqsdll = bel.ctx.chip.bel_dqsdll(cell);
        let dqsdel_dqsdll = bel.ctx.naming.bel_wire(bcrd_dqsdll, "DQSDEL");
        bel.ctx.claim_pip(dqsdel, dqsdel_dqsdll);
    }

    pub(super) fn process_dqsdll_ecp(&mut self) {
        for edge in [DirV::S, DirV::N] {
            let bcrd = self.chip.bel_dqsdll_ecp(edge);
            self.extract_bel_cell(bcrd, bcrd.with_col(self.chip.col_clk - 1))
                .name(match edge {
                    DirV::S => "BDLL",
                    DirV::N => "TDLL",
                })
                .suffix("DQSDLL")
                .add_wire_rc_io("DQSDEL", "JDQSDEL_DQSDLL");
        }
    }

    pub(super) fn process_dqsdll_ecp2(&mut self) {
        for edge in [DirH::W, DirH::E] {
            let bcrd = self.chip.bel_dqsdll_ecp2(edge);
            self.extract_bel(bcrd)
                .name(match edge {
                    DirH::W => "LDQSDLL",
                    DirH::E => "RDQSDLL",
                })
                .suffix("DQSDLL")
                .add_wire_rc_io("DQSDEL", "JDQSDEL_DQSDLL");
        }
    }

    fn process_io_cell_ecp(&mut self, cell: CellCoord, kind: IoGroupKind) {
        let is_ecp2 = !matches!(self.chip.kind, ChipKind::Ecp | ChipKind::Xp);
        if is_ecp2 && matches!(kind, IoGroupKind::None | IoGroupKind::Serdes) {
            return;
        }
        if is_ecp2 {
            let edge = if cell.col == self.chip.col_w() {
                Dir::W
            } else if cell.col == self.chip.col_e() {
                Dir::E
            } else if cell.row == self.chip.row_s() {
                Dir::S
            } else if cell.row == self.chip.row_n() {
                Dir::N
            } else {
                unreachable!()
            };
            let bel_eclk = self.chip.bel_eclk_root(edge);
            let eclk0 = self.naming.bel_wire(bel_eclk, "ECLK0");
            let eclk1 = self.naming.bel_wire(bel_eclk, "ECLK1");
            self.extract_switchbox(cell.bel(bslots::IO_INT))
                .output_bind_rc_io("JECLKIA", wires::IMUX_IO_ECLKI[0].cell(0))
                .output_bind_rc_io("JECLKIB", wires::IMUX_IO_ECLKI[1].cell(0))
                .output_bind_rc_io("JECLKOA", wires::IMUX_IO_ECLKO[0].cell(0))
                .output_bind_rc_io("JECLKOB", wires::IMUX_IO_ECLKO[1].cell(0))
                .input_bind_no_claim(eclk0, wires::ECLK[0].cell(0))
                .input_bind_no_claim(eclk1, wires::ECLK[1].cell(0));
        }
        self.process_single_io_ecp(
            cell.bel(bslots::IO[0]),
            matches!(kind, IoGroupKind::DoubleB | IoGroupKind::None),
            kind == IoGroupKind::DoubleDqs,
        );
        self.process_single_io_ecp(
            cell.bel(bslots::IO[1]),
            matches!(kind, IoGroupKind::DoubleA | IoGroupKind::None),
            false,
        );
    }

    fn prep_io_cell_ecp(&mut self, cell: CellCoord, kind: IoGroupKind) {
        let is_ecp2 = !matches!(self.chip.kind, ChipKind::Ecp | ChipKind::Xp);
        if is_ecp2 && matches!(kind, IoGroupKind::None | IoGroupKind::Serdes) {
            return;
        }
        let paddi_pio = self.rc_io_wire(cell, "JPADDIA_PIO");
        self.bind_wire(paddi_pio, cell.wire(wires::OUT_IO_PADDI[0]));
        let paddi_pio = self.rc_io_wire(cell, "JPADDIB_PIO");
        self.bind_wire(paddi_pio, cell.wire(wires::OUT_IO_PADDI[1]));
    }

    pub(super) fn prep_io_ecp(&mut self) {
        let die = Chip::DIE;
        for cell in self.edev.column(die, self.chip.col_w()) {
            let rd = &self.chip.rows[cell.row];
            if !matches!(rd.kind, RowKind::Plc | RowKind::Fplc) {
                continue;
            }
            self.prep_io_cell_ecp(cell, rd.io_w);
        }
        for cell in self.edev.column(die, self.chip.col_e()) {
            let rd = &self.chip.rows[cell.row];
            if !matches!(rd.kind, RowKind::Plc | RowKind::Fplc) {
                continue;
            }
            self.prep_io_cell_ecp(cell, rd.io_e);
        }
        for cell in self.edev.row(die, self.chip.row_s()) {
            if cell.col == self.chip.col_w() || cell.col == self.chip.col_e() {
                continue;
            }
            let cd = &self.chip.columns[cell.col];
            self.prep_io_cell_ecp(cell, cd.io_s);
        }
        for cell in self.edev.row(die, self.chip.row_n()) {
            if cell.col == self.chip.col_w() || cell.col == self.chip.col_e() {
                continue;
            }
            let cd = &self.chip.columns[cell.col];
            self.prep_io_cell_ecp(cell, cd.io_n);
        }
    }

    pub(super) fn process_io_ecp(&mut self) {
        let die = Chip::DIE;
        for cell in self.edev.column(die, self.chip.col_w()) {
            if self.chip.rows[cell.row].io_w == IoGroupKind::DoubleDqs {
                self.process_dqs_ecp(cell.bel(bslots::DQS[0]));
            }
        }
        for cell in self.edev.column(die, self.chip.col_e()) {
            if self.chip.rows[cell.row].io_e == IoGroupKind::DoubleDqs {
                self.process_dqs_ecp(cell.bel(bslots::DQS[0]));
            }
        }
        for cell in self.edev.row(die, self.chip.row_s()) {
            if self.chip.columns[cell.col].io_s == IoGroupKind::DoubleDqs {
                self.process_dqs_ecp(cell.bel(bslots::DQS[0]));
            }
        }
        for cell in self.edev.row(die, self.chip.row_n()) {
            if self.chip.columns[cell.col].io_n == IoGroupKind::DoubleDqs {
                self.process_dqs_ecp(cell.bel(bslots::DQS[0]));
            }
        }
        for cell in self.edev.column(die, self.chip.col_w()) {
            let rd = &self.chip.rows[cell.row];
            if !matches!(rd.kind, RowKind::Plc | RowKind::Fplc) {
                continue;
            }
            self.process_io_cell_ecp(cell, rd.io_w);
        }
        for cell in self.edev.column(die, self.chip.col_e()) {
            let rd = &self.chip.rows[cell.row];
            if !matches!(rd.kind, RowKind::Plc | RowKind::Fplc) {
                continue;
            }
            self.process_io_cell_ecp(cell, rd.io_e);
        }
        for cell in self.edev.row(die, self.chip.row_s()) {
            if cell.col == self.chip.col_w() || cell.col == self.chip.col_e() {
                continue;
            }
            let cd = &self.chip.columns[cell.col];
            self.process_io_cell_ecp(cell, cd.io_s);
        }
        for cell in self.edev.row(die, self.chip.row_n()) {
            if cell.col == self.chip.col_w() || cell.col == self.chip.col_e() {
                continue;
            }
            let cd = &self.chip.columns[cell.col];
            self.process_io_cell_ecp(cell, cd.io_n);
        }
    }

    pub(super) fn process_eclk_ecp2(&mut self) {
        for (edge, num_outs, io) in [
            (
                Dir::W,
                1,
                self.edev
                    .column(Chip::DIE, self.chip.col_w())
                    .find(|cell| self.chip.rows[cell.row].io_w == IoGroupKind::Double)
                    .unwrap(),
            ),
            (
                Dir::E,
                1,
                self.edev
                    .column(Chip::DIE, self.chip.col_e())
                    .find(|cell| self.chip.rows[cell.row].io_e == IoGroupKind::Double)
                    .unwrap(),
            ),
            (
                Dir::S,
                2,
                self.edev
                    .row(Chip::DIE, self.chip.row_s())
                    .find(|cell| self.chip.columns[cell.col].io_s == IoGroupKind::Double)
                    .unwrap(),
            ),
            (
                Dir::N,
                2,
                self.edev
                    .row(Chip::DIE, self.chip.row_n())
                    .find(|cell| self.chip.columns[cell.col].io_n == IoGroupKind::Double)
                    .unwrap(),
            ),
        ] {
            let eclki = self.rc_io_wire(io, "JECLKIA");
            let eclks = self.pips_bwd[&eclki].clone();
            let bcrd = self.chip.bel_eclk_root(edge);

            let mut ex = self.extract_switchbox(bcrd);

            for i in 0..2 {
                let eclk = eclks
                    .iter()
                    .copied()
                    .find(|wn| ex.ctx.naming.strings[wn.suffix].ends_with(['0', '1'][i]))
                    .unwrap();

                let eclk_in = if matches!(edge, Dir::H(_)) {
                    ex.add_wire(format!("ECLK{i}"), eclk);
                    let eclk_in = ex.ctx.claim_single_in(eclk);
                    ex.add_wire_no_claim(format!("ECLK{i}_IN"), eclk_in);
                    eclk_in
                } else {
                    ex.add_wire_no_claim(format!("ECLK{i}"), eclk);
                    eclk
                };

                ex.output_bind(eclk_in, wires::ECLK[i].cell(0));

                let mut inps = BTreeMap::new();
                for &wn in &ex.ctx.pips_bwd[&eclk_in] {
                    inps.insert(ex.ctx.naming.strings[wn.suffix].clone(), wn);
                }

                let eclk_io = inps.remove(&format!("JPIO{i}")).unwrap();
                let wire_io = wires::OUT_IO_PADDI[0].cell(num_outs + i);
                ex.input_io(eclk_io, wire_io);
                for ci in 0..num_outs {
                    ex.permabuf_int_io(wires::OUT_F[6 + i].cell(ci), wire_io);
                }

                let eclk_int = inps.remove(&format!("JCIBCLK{i}")).unwrap();
                ex.input_xlat(eclk_int);

                if let Dir::H(_) = edge {
                    let eclk_pll = inps.remove(&format!("JFRC{i}")).unwrap();
                    ex.input_xlat(eclk_pll);
                }

                assert!(inps.is_empty());
            }
        }
    }

    pub(super) fn process_eclk_xp2(&mut self) {
        for (lrbt, edge, num_outs) in [
            ('L', Dir::W, 1),
            ('R', Dir::E, 1),
            ('B', Dir::S, 2),
            ('T', Dir::N, 2),
        ] {
            let bcrd = self.chip.bel_eclk_root(edge);
            let cell = match edge {
                Dir::H(_) => bcrd.cell,
                Dir::V(_) => bcrd.cell.delta(-1, 0),
            };

            let mut ex = self.extract_switchbox(bcrd);
            ex.cell(cell);

            if let Dir::H(h) = edge {
                for v in DirV::DIRS {
                    if ex.ctx.chip.try_bel_pll(h & v, 0).is_none() {
                        continue;
                    }
                    let idx = match (h, v) {
                        (DirH::W, DirV::S) => 1,
                        (DirH::W, DirV::N) => 0,
                        (DirH::E, DirV::S) => 0,
                        (DirH::E, DirV::N) => 1,
                    };

                    ex.input_xlat_rc(format!("JPLLPIO{idx}"));
                    ex.input_xlat_rc(format!("JPLLCLKOP{idx}"));
                    ex.input_xlat_rc(format!("JPLLCLKOS{idx}"));
                }
            }

            for i in 0..2 {
                let eclk_in = ex.ctx.rc_wire(cell, &format!("J{lrbt}FRC{i}"));
                ex.add_wire_no_claim(format!("ECLK{i}_IN"), eclk_in);

                let eclk = ex.ctx.pips_fwd[&eclk_in]
                    .iter()
                    .copied()
                    .find(|wn| {
                        ex.ctx.naming.strings[wn.suffix].contains("FRC")
                            && !ex.ctx.naming.strings[wn.suffix].contains("JFRC")
                    })
                    .unwrap();
                ex.add_wire(format!("ECLK{i}"), eclk);
                ex.ctx.claim_pip(eclk, eclk_in);

                let wire_eclk = wires::ECLK[i].cell(0);
                ex.output_bind(eclk_in, wire_eclk);

                let wire_io = wires::OUT_IO_PADDI[0].cell(num_outs + i);
                ex.input_io_rc(format!("JPIO{i}"), wire_io);
                for ci in 0..num_outs {
                    ex.permabuf_int_io(wires::OUT_F[6 + i].cell(ci), wire_io);
                }

                ex.input_xlat_rc(format!("JCIBCLK{i}"));

                if let Dir::H(h) = edge {
                    for (v, ci, ci_io) in [(DirV::S, 3, 5), (DirV::N, 4, 6)] {
                        if ex.ctx.chip.try_bel_pll(h & v, 0).is_none() {
                            ex.edge(wire_eclk, wires::OUT_IO_PADDI[0].cell(ci_io));
                            if i == 0 {
                                ex.edge(wire_eclk, wires::OUT_PLL_CLKOP[0].cell(ci));
                            } else {
                                ex.edge(wire_eclk, wires::OUT_PLL_CLKOS[0].cell(ci));
                            }
                        }
                    }
                }
            }
        }
    }

    pub(super) fn process_eclk_tap_ecp2(&mut self) {
        for &tcrd in &self.edev.tile_index[tcls::ECLK_TAP] {
            let edge = if tcrd.row == self.chip.row_s() {
                Dir::S
            } else if tcrd.row == self.chip.row_n() {
                Dir::N
            } else if tcrd.col == self.chip.col_w() {
                Dir::W
            } else if tcrd.col == self.chip.col_e() {
                Dir::E
            } else {
                unreachable!()
            };
            let bel_eclk = self.chip.bel_eclk_root(edge);
            let eclk0 = self.naming.bel_wire(bel_eclk, "ECLK0");
            let eclk1 = self.naming.bel_wire(bel_eclk, "ECLK1");
            self.claim_pip_int_out(tcrd.wire(wires::OUT_F[6]), eclk0);
            self.claim_pip_int_out(tcrd.wire(wires::OUT_F[7]), eclk1);
        }
    }
}

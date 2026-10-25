use prjcombine_ecp::{
    chip::{Chip, IoGroupKind},
    defs::{bcls, bslots, cslots, wires},
};
use prjcombine_interconnect::{
    db::{BelInput, Mux, SwitchBox, SwitchBoxItem, WireSlotIdExt},
    dir::{Dir, DirH, DirV},
    grid::{BelCoord, CellCoord, DieIdExt},
};

use crate::{ChipContext, chip::ChipExt};

impl ChipContext<'_> {
    fn process_single_io_machxo(&mut self, bcrd: BelCoord, sb: &mut SwitchBox) {
        let idx = bslots::IO.index_of(bcrd.slot).unwrap();
        let cell = bcrd.cell;
        let bcrd_int = bcrd.bel(bslots::IO_INT);
        let abcd = ['A', 'B', 'C', 'D', 'E', 'F'][idx];
        let (r, c) = self.rc(cell);
        let name = match self.chip.get_io_edge(cell) {
            Dir::W => format!("PL{r}{abcd}"),
            Dir::E => format!("PR{r}{abcd}"),
            Dir::S => format!("PB{c}{abcd}"),
            Dir::N => format!("PT{c}{abcd}"),
        };

        let ddtd0 = self.rc_io_wire(cell, &format!("JDDTD0{abcd}"));
        let ddtd1 = self.rc_io_wire(cell, &format!("JDDTD1{abcd}"));
        let wire_ddtd0 = self.xlat_input(bcrd_int, ddtd0).wire();
        let wire_ddtd1 = self.xlat_input(bcrd_int, ddtd1).wire();
        self.add_sb_wire(bcrd_int, wire_ddtd0, ddtd0);
        self.add_sb_wire(bcrd_int, wire_ddtd1, ddtd1);

        let (src_cell, plc_idx) = self.chip.io_direct_plc[&bcrd];
        let plc_cell = self.edev[src_cell].conns[cslots::IO_FROM_PLC]
            .target
            .unwrap();
        let slice = plc_cell.bel(bslots::SLICE[plc_idx / 2]);
        let plcf = self.naming.bel_wire(slice, &format!("F{}_IO", plc_idx % 2));
        let plcq = self.naming.bel_wire(slice, &format!("Q{}_IO", plc_idx % 2));

        let wire_o_plc = wires::IMUX_IO_O_PLC[idx].cell(0);
        let wire_o = wires::IMUX_IO_O[idx].cell(0);
        let wire_t = wires::IMUX_IO_T[idx].cell(0);

        let dd2 = self.rc_io_wire(cell, &format!("JDD2{abcd}"));

        let dd = self.rc_wire(self.chip.xlat_rc_wire(plcf), &format!("JDD{plc_idx}"));
        self.claim_pip(dd, plcf);
        self.claim_pip(dd, plcq);
        self.claim_pip(dd2, dd);

        let paddo = self.rc_io_wire(cell, &format!("JPADDO{abcd}"));
        let paddt = self.rc_io_wire(cell, &format!("JPADDT{abcd}"));
        self.add_sb_wire(bcrd_int, wire_o_plc, dd2);
        self.add_sb_wire(bcrd_int, wire_o, paddo);
        self.add_sb_wire(bcrd_int, wire_t, paddt);
        self.claim_pip(paddo, dd2);
        self.claim_pip(paddo, ddtd0);
        self.claim_pip(paddo, ddtd1);
        self.claim_pip(paddt, ddtd0);
        self.claim_pip(paddt, ddtd1);

        sb.items.push(SwitchBoxItem::Mux(Mux {
            dst: wire_o,
            bits: vec![],
            src: [wire_ddtd0, wire_ddtd1, wire_o_plc]
                .into_iter()
                .map(|k| (k.pos(), Default::default()))
                .collect(),
            bits_off: None,
        }));
        sb.items.push(SwitchBoxItem::Mux(Mux {
            dst: wire_t,
            bits: vec![],
            src: [wire_ddtd0, wire_ddtd1]
                .into_iter()
                .map(|k| (k.pos(), Default::default()))
                .collect(),
            bits_off: None,
        }));

        let paddo_pio = self.rc_io_wire(cell, &format!("JPADDO{abcd}_PIO"));
        let paddt_pio = self.rc_io_wire(cell, &format!("JPADDT{abcd}_PIO"));
        self.claim_pip(paddo_pio, paddo);
        self.claim_pip(paddt_pio, paddt);

        self.extract_bel(bcrd)
            .name(name)
            .add_wire("DD", dd)
            .add_wire("PADDO_PIO", paddo_pio)
            .add_wire("PADDT_PIO", paddt_pio)
            .input_raw(bcls::IO::PADDO, BelInput::Fixed(wire_o.pos()))
            .input_raw(bcls::IO::PADDT, BelInput::Fixed(wire_t.pos()))
            .output_xlat_rc_io(bcls::IO::PADDI, format!("JPADDI{abcd}_PIO"))
            .output(bcls::IO::INCK, wires::OUT_IO_INCK[idx].cell(0));
    }

    fn process_pictest(&mut self, cell: CellCoord) {
        let (r, c) = self.rc(cell);
        let lr = if c == 1 { 'L' } else { 'R' };
        self.extract_bel(cell.bel(bslots::PICTEST[0]))
            .name(format!("{lr}PICTEST{r}"))
            .suffix("PICTEST");
    }

    pub(super) fn process_io_machxo(&mut self) {
        for (row, rd) in &self.chip.rows {
            let num_io = match rd.io[DirH::W] {
                IoGroupKind::None => 0,
                IoGroupKind::Double => 2,
                IoGroupKind::Quad | IoGroupKind::QuadReverse => 4,
                _ => unreachable!(),
            };
            if num_io != 0 {
                let cell = Chip::DIE.cell(self.chip.col_w(), row);
                self.name_bel_null(cell.bel(bslots::IO_INT));
                let mut sb = SwitchBox::default();
                for i in 0..num_io {
                    self.process_single_io_machxo(cell.bel(bslots::IO[i]), &mut sb);
                }
                self.insert_switchbox(cell.bel(bslots::IO_INT), sb);
                if num_io == 2 {
                    self.process_pictest(cell);
                }
            }
            let num_io = match rd.io[DirH::E] {
                IoGroupKind::None => 0,
                IoGroupKind::Double => 2,
                IoGroupKind::Quad | IoGroupKind::QuadReverse => 4,
                _ => unreachable!(),
            };
            if num_io != 0 {
                let cell = Chip::DIE.cell(self.chip.col_e(), row);
                self.name_bel_null(cell.bel(bslots::IO_INT));
                let mut sb = SwitchBox::default();
                for i in 0..num_io {
                    self.process_single_io_machxo(cell.bel(bslots::IO[i]), &mut sb);
                }
                self.insert_switchbox(cell.bel(bslots::IO_INT), sb);
                if num_io == 2 {
                    self.process_pictest(cell);
                }
            }
        }
        for (col, cd) in &self.chip.columns {
            let num_io = match cd.io[DirV::S] {
                IoGroupKind::None => 0,
                IoGroupKind::Quad | IoGroupKind::QuadReverse => 4,
                IoGroupKind::Hex | IoGroupKind::HexReverse => 6,
                _ => unreachable!(),
            };
            if num_io != 0 {
                let cell = Chip::DIE.cell(col, self.chip.row_s());
                self.name_bel_null(cell.bel(bslots::IO_INT));
                let mut sb = SwitchBox::default();
                for i in 0..num_io {
                    self.process_single_io_machxo(cell.bel(bslots::IO[i]), &mut sb);
                }
                self.insert_switchbox(cell.bel(bslots::IO_INT), sb);
            }
            let num_io = match cd.io[DirV::N] {
                IoGroupKind::None => 0,
                IoGroupKind::Quad | IoGroupKind::QuadReverse => 4,
                IoGroupKind::Hex | IoGroupKind::HexReverse => 6,
                _ => unreachable!(),
            };
            if num_io != 0 {
                let cell = Chip::DIE.cell(col, self.chip.row_n());
                self.name_bel_null(cell.bel(bslots::IO_INT));
                let mut sb = SwitchBox::default();
                for i in 0..num_io {
                    self.process_single_io_machxo(cell.bel(bslots::IO[i]), &mut sb);
                }
                self.insert_switchbox(cell.bel(bslots::IO_INT), sb);
            }
        }
    }
}

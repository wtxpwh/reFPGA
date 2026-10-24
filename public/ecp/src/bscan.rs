use std::collections::{BTreeMap, BTreeSet};

use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    dir::DirV,
    grid::{CellCoord, DieIdExt, EdgeIoCoord, TileIobId},
};
use prjcombine_types::bscan::{BScanBuilder, BScanPad};

use crate::{
    bond::{BondPad, CfgPad, SerdesPad},
    chip::{Chip, ChipKind, IoGroupKind, NexusKind, RowKind, SpecialIoKey, SpecialLocKey},
    defs::bslots,
};

#[derive(Debug)]
pub struct BScan {
    pub bits: usize,
    pub pads: BTreeMap<BondPad, BScanPad>,
}

impl Chip {
    pub fn get_bscan(&self) -> BScan {
        let mut builder = BScanBuilder::new();
        let mut pads = BTreeMap::new();
        match self.kind {
            ChipKind::Scm => {
                for (col, cd) in self.columns.iter().rev() {
                    if col >= self.col_clk {
                        continue;
                    }
                    let num_io = match cd.io_n {
                        IoGroupKind::None => 0,
                        IoGroupKind::Quad => 4,
                        IoGroupKind::Octal => 8,
                        IoGroupKind::Dozen => 12,
                        IoGroupKind::Serdes => {
                            for (pad, val) in [
                                (SerdesPad::InP(3), builder.get_i()),
                                (SerdesPad::OutP(3), builder.get_o()),
                                (SerdesPad::OutP(2), builder.get_o()),
                                (SerdesPad::InP(2), builder.get_i()),
                                (SerdesPad::InP(1), builder.get_i()),
                                (SerdesPad::OutP(1), builder.get_o()),
                                (SerdesPad::OutP(0), builder.get_o()),
                                (SerdesPad::InP(0), builder.get_i()),
                                (SerdesPad::ClkP, builder.get_i()),
                                (SerdesPad::RxClkP, builder.get_i()),
                            ] {
                                pads.insert(BondPad::Serdes(DirV::N, col, pad), val);
                            }
                            continue;
                        }
                        _ => unreachable!(),
                    };
                    for iob in (0..num_io).rev() {
                        let crd = EdgeIoCoord::N(col, TileIobId::from_idx(iob));
                        pads.insert(BondPad::Io(crd), builder.get_tb());
                    }
                }
                pads.insert(BondPad::Cfg(CfgPad::ResetB), builder.get_i());
                pads.insert(BondPad::Cfg(CfgPad::RdCfgB), builder.get_i());
                pads.insert(BondPad::Cfg(CfgPad::Done), builder.get_tb());
                pads.insert(BondPad::Cfg(CfgPad::InitB), builder.get_tb());
                pads.insert(BondPad::Cfg(CfgPad::M0), builder.get_i());
                pads.insert(BondPad::Cfg(CfgPad::M1), builder.get_i());
                pads.insert(BondPad::Cfg(CfgPad::M2), builder.get_i());
                pads.insert(BondPad::Cfg(CfgPad::M3), builder.get_i());
                for (row, rd) in self.rows.iter().rev() {
                    let num_io = match rd.io_w {
                        IoGroupKind::None => 0,
                        IoGroupKind::Quad => 4,
                        IoGroupKind::Octal => 8,
                        IoGroupKind::Dozen => 12,
                        _ => unreachable!(),
                    };
                    for iob in 0..num_io {
                        let crd = EdgeIoCoord::W(row, TileIobId::from_idx(iob));
                        pads.insert(BondPad::Io(crd), builder.get_tb());
                    }
                }
                for (col, cd) in self.columns.iter() {
                    let num_io = match cd.io_s {
                        IoGroupKind::None => 0,
                        IoGroupKind::Quad => 4,
                        IoGroupKind::Octal => 8,
                        IoGroupKind::Dozen => 12,
                        _ => unreachable!(),
                    };
                    for iob in 0..num_io {
                        let crd = EdgeIoCoord::S(col, TileIobId::from_idx(iob));
                        pads.insert(BondPad::Io(crd), builder.get_tb());
                    }
                }
                for (row, rd) in self.rows.iter() {
                    let num_io = match rd.io_e {
                        IoGroupKind::None => 0,
                        IoGroupKind::Quad => 4,
                        IoGroupKind::Octal => 8,
                        IoGroupKind::Dozen => 12,
                        _ => unreachable!(),
                    };
                    for iob in (0..num_io).rev() {
                        let crd = EdgeIoCoord::E(row, TileIobId::from_idx(iob));
                        pads.insert(BondPad::Io(crd), builder.get_tb());
                    }
                }
                pads.insert(BondPad::Cfg(CfgPad::ProgB), builder.get_i());
                pads.insert(BondPad::Cfg(CfgPad::MpiIrqB), builder.get_to());
                pads.insert(BondPad::Cfg(CfgPad::Cclk), builder.get_tb());
                for (col, cd) in self.columns.iter().rev() {
                    if col < self.col_clk {
                        continue;
                    }
                    let num_io = match cd.io_n {
                        IoGroupKind::None => 0,
                        IoGroupKind::Quad => 4,
                        IoGroupKind::Octal => 8,
                        IoGroupKind::Dozen => 12,
                        IoGroupKind::Serdes => {
                            for (pad, val) in [
                                (SerdesPad::RxClkP, builder.get_i()),
                                (SerdesPad::ClkP, builder.get_i()),
                                (SerdesPad::InP(0), builder.get_i()),
                                (SerdesPad::OutP(0), builder.get_o()),
                                (SerdesPad::OutP(1), builder.get_o()),
                                (SerdesPad::InP(1), builder.get_i()),
                                (SerdesPad::InP(2), builder.get_i()),
                                (SerdesPad::OutP(2), builder.get_o()),
                                (SerdesPad::OutP(3), builder.get_o()),
                                (SerdesPad::InP(3), builder.get_i()),
                            ] {
                                pads.insert(BondPad::Serdes(DirV::N, col, pad), val);
                            }
                            continue;
                        }
                        _ => unreachable!(),
                    };
                    for iob in (0..num_io).rev() {
                        let crd = EdgeIoCoord::N(col, TileIobId::from_idx(iob));
                        pads.insert(BondPad::Io(crd), builder.get_tb());
                    }
                }
            }
            ChipKind::Ecp | ChipKind::Ecp2 | ChipKind::Ecp2M => {
                for (col, cd) in &self.columns {
                    if col < self.col_clk {
                        continue;
                    }
                    if cd.io_s == IoGroupKind::Serdes {
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::InP(3)),
                            builder.get_i(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::OutP(3)),
                            builder.get_o(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::OutP(2)),
                            builder.get_o(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::InP(2)),
                            builder.get_i(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::ClkP),
                            builder.get_i(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::InP(1)),
                            builder.get_i(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::OutP(1)),
                            builder.get_o(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::OutP(0)),
                            builder.get_o(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::InP(0)),
                            builder.get_i(),
                        );
                    } else if cd.io_s != IoGroupKind::None {
                        for iob in [0, 1] {
                            let crd = EdgeIoCoord::S(col, TileIobId::from_idx(iob));
                            pads.insert(BondPad::Io(crd), builder.get_tb());
                        }
                    }
                }
                let mut got_cfg = false;
                if matches!(self.kind, ChipKind::Ecp2 | ChipKind::Ecp2M) {
                    pads.insert(BondPad::Cfg(CfgPad::M2), builder.get_i());
                    pads.insert(BondPad::Cfg(CfgPad::M1), builder.get_i());
                    pads.insert(BondPad::Cfg(CfgPad::M0), builder.get_i());
                    pads.insert(BondPad::Cfg(CfgPad::ProgB), builder.get_i());
                    pads.insert(BondPad::Cfg(CfgPad::Cclk), builder.get_tb());
                    pads.insert(BondPad::Cfg(CfgPad::InitB), builder.get_tb());
                    pads.insert(BondPad::Cfg(CfgPad::Done), builder.get_tb());
                    got_cfg = true;
                    if self.rows[self.row_s() + 2].io_e == IoGroupKind::None {
                        for pad in [
                            CfgPad::WriteN,
                            CfgPad::Cs1N,
                            CfgPad::CsN,
                            CfgPad::D(0),
                            CfgPad::D(1),
                            CfgPad::D(2),
                            CfgPad::D(3),
                            CfgPad::D(4),
                            CfgPad::D(5),
                            CfgPad::D(6),
                            CfgPad::D(7),
                            CfgPad::Di,
                            CfgPad::Dout,
                            CfgPad::Busy,
                        ] {
                            pads.insert(BondPad::Cfg(pad), builder.get_tb());
                        }
                    }
                }
                for (row, rd) in &self.rows {
                    if rd.kind == RowKind::Ebr && !got_cfg {
                        pads.insert(BondPad::Cfg(CfgPad::M2), builder.get_i());
                        pads.insert(BondPad::Cfg(CfgPad::M1), builder.get_i());
                        pads.insert(BondPad::Cfg(CfgPad::M0), builder.get_i());
                        pads.insert(BondPad::Cfg(CfgPad::ProgB), builder.get_i());
                        pads.insert(BondPad::Cfg(CfgPad::Cclk), builder.get_tb());
                        pads.insert(BondPad::Cfg(CfgPad::InitB), builder.get_tb());
                        pads.insert(BondPad::Cfg(CfgPad::Done), builder.get_tb());
                        got_cfg = true;
                    }
                    if rd.io_e != IoGroupKind::None {
                        for iob in [1, 0] {
                            let crd = EdgeIoCoord::E(row, TileIobId::from_idx(iob));
                            pads.insert(BondPad::Io(crd), builder.get_tb());
                        }
                    }
                }
                for (col, cd) in self.columns.iter().rev() {
                    if cd.io_n == IoGroupKind::Serdes {
                        pads.insert(
                            BondPad::Serdes(DirV::N, col, SerdesPad::InP(0)),
                            builder.get_i(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::N, col, SerdesPad::OutP(0)),
                            builder.get_o(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::N, col, SerdesPad::OutP(1)),
                            builder.get_o(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::N, col, SerdesPad::InP(1)),
                            builder.get_i(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::N, col, SerdesPad::ClkP),
                            builder.get_i(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::N, col, SerdesPad::InP(2)),
                            builder.get_i(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::N, col, SerdesPad::OutP(2)),
                            builder.get_o(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::N, col, SerdesPad::OutP(3)),
                            builder.get_o(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::N, col, SerdesPad::InP(3)),
                            builder.get_i(),
                        );
                    } else if cd.io_n != IoGroupKind::None {
                        for iob in [1, 0] {
                            let crd = EdgeIoCoord::N(col, TileIobId::from_idx(iob));
                            pads.insert(BondPad::Io(crd), builder.get_tb());
                        }
                    }
                }
                for (row, rd) in self.rows.iter().rev() {
                    if rd.io_w != IoGroupKind::None {
                        for iob in [0, 1] {
                            let crd = EdgeIoCoord::W(row, TileIobId::from_idx(iob));
                            pads.insert(BondPad::Io(crd), builder.get_tb());
                        }
                    }
                }
                if matches!(self.kind, ChipKind::Ecp2 | ChipKind::Ecp2M) {
                    pads.insert(BondPad::Cfg(CfgPad::Hfp), builder.get_i());
                }
                for (col, cd) in &self.columns {
                    if col >= self.col_clk {
                        continue;
                    }
                    if cd.io_s == IoGroupKind::Serdes {
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::InP(3)),
                            builder.get_i(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::OutP(3)),
                            builder.get_o(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::OutP(2)),
                            builder.get_o(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::InP(2)),
                            builder.get_i(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::ClkP),
                            builder.get_i(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::InP(1)),
                            builder.get_i(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::OutP(1)),
                            builder.get_o(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::OutP(0)),
                            builder.get_o(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::InP(0)),
                            builder.get_i(),
                        );
                    } else if cd.io_s != IoGroupKind::None {
                        for iob in [0, 1] {
                            let crd = EdgeIoCoord::S(col, TileIobId::from_idx(iob));
                            pads.insert(BondPad::Io(crd), builder.get_tb());
                        }
                    }
                }
            }
            ChipKind::Xp => {
                for (col, cd) in &self.columns {
                    if col < self.col_clk {
                        continue;
                    }
                    let iobs = match cd.io_n {
                        IoGroupKind::None => [].as_slice(),
                        IoGroupKind::Double | IoGroupKind::DoubleDqs => [0, 1].as_slice(),
                        IoGroupKind::DoubleA => [0].as_slice(),
                        IoGroupKind::DoubleB => [1].as_slice(),
                        _ => unreachable!(),
                    };
                    for &iob in iobs {
                        let crd = EdgeIoCoord::N(col, TileIobId::from_idx(iob));
                        pads.insert(BondPad::Io(crd), builder.get_tb());
                    }
                }
                for (row, rd) in self.rows.iter().rev() {
                    let iobs = match rd.io_e {
                        IoGroupKind::None => [].as_slice(),
                        IoGroupKind::Double | IoGroupKind::DoubleDqs => [0, 1].as_slice(),
                        IoGroupKind::DoubleA => [0].as_slice(),
                        IoGroupKind::DoubleB => [1].as_slice(),
                        _ => unreachable!(),
                    };
                    for &iob in iobs {
                        let crd = EdgeIoCoord::E(row, TileIobId::from_idx(iob));
                        pads.insert(BondPad::Io(crd), builder.get_tb());
                    }
                }
                for (col, cd) in self.columns.iter().rev() {
                    let iobs = match cd.io_s {
                        IoGroupKind::None => [].as_slice(),
                        IoGroupKind::Double | IoGroupKind::DoubleDqs => [1, 0].as_slice(),
                        IoGroupKind::DoubleA => [0].as_slice(),
                        IoGroupKind::DoubleB => [1].as_slice(),
                        _ => unreachable!(),
                    };
                    for &iob in iobs {
                        let crd = EdgeIoCoord::S(col, TileIobId::from_idx(iob));
                        pads.insert(BondPad::Io(crd), builder.get_tb());
                    }
                }
                pads.insert(BondPad::Cfg(CfgPad::InitB), builder.get_tb());
                pads.insert(BondPad::Cfg(CfgPad::SleepB), builder.get_i());
                for (row, rd) in &self.rows {
                    let iobs = match rd.io_w {
                        IoGroupKind::None => [].as_slice(),
                        IoGroupKind::Double | IoGroupKind::DoubleDqs => [1, 0].as_slice(),
                        IoGroupKind::DoubleA => [0].as_slice(),
                        IoGroupKind::DoubleB => [1].as_slice(),
                        _ => unreachable!(),
                    };
                    for &iob in iobs {
                        let crd = EdgeIoCoord::W(row, TileIobId::from_idx(iob));
                        pads.insert(BondPad::Io(crd), builder.get_tb());
                    }
                }
                pads.insert(BondPad::Cfg(CfgPad::Cclk), builder.get_tb());
                pads.insert(BondPad::Cfg(CfgPad::ProgB), builder.get_i());
                pads.insert(BondPad::Cfg(CfgPad::Done), builder.get_tb());
                pads.insert(BondPad::Cfg(CfgPad::M1), builder.get_i());
                pads.insert(BondPad::Cfg(CfgPad::M0), builder.get_i());
                for (col, cd) in &self.columns {
                    if col >= self.col_clk {
                        continue;
                    }
                    let iobs = match cd.io_n {
                        IoGroupKind::None => [].as_slice(),
                        IoGroupKind::Double | IoGroupKind::DoubleDqs => [0, 1].as_slice(),
                        IoGroupKind::DoubleA => [0].as_slice(),
                        IoGroupKind::DoubleB => [1].as_slice(),
                        _ => unreachable!(),
                    };
                    for &iob in iobs {
                        let crd = EdgeIoCoord::N(col, TileIobId::from_idx(iob));
                        pads.insert(BondPad::Io(crd), builder.get_tb());
                    }
                }
            }
            ChipKind::MachXo => {
                for (row, rd) in &self.rows {
                    let num_io = match rd.io_w {
                        IoGroupKind::None => 0,
                        IoGroupKind::Double => 2,
                        IoGroupKind::Quad => 4,
                        IoGroupKind::QuadReverse => {
                            for i in 0..4 {
                                let crd = EdgeIoCoord::W(row, TileIobId::from_idx(i));
                                pads.insert(BondPad::Io(crd), builder.get_tb());
                            }
                            continue;
                        }
                        _ => unreachable!(),
                    };
                    for i in (0..num_io).rev() {
                        let crd = EdgeIoCoord::W(row, TileIobId::from_idx(i));
                        pads.insert(BondPad::Io(crd), builder.get_tb());
                    }
                }
                for (col, cd) in &self.columns {
                    let num_io = match cd.io_n {
                        IoGroupKind::None => 0,
                        IoGroupKind::Quad => 4,
                        IoGroupKind::Hex => 6,
                        IoGroupKind::HexReverse => {
                            for i in (0..6).rev() {
                                let crd = EdgeIoCoord::N(col, TileIobId::from_idx(i));
                                pads.insert(BondPad::Io(crd), builder.get_tb());
                            }
                            continue;
                        }
                        _ => unreachable!(),
                    };
                    for i in 0..num_io {
                        let crd = EdgeIoCoord::N(col, TileIobId::from_idx(i));
                        pads.insert(BondPad::Io(crd), builder.get_tb());
                    }
                }
                for (row, rd) in self.rows.iter().rev() {
                    let num_io = match rd.io_e {
                        IoGroupKind::None => 0,
                        IoGroupKind::Double => 2,
                        IoGroupKind::Quad => 4,
                        _ => unreachable!(),
                    };
                    for i in 0..num_io {
                        let crd = EdgeIoCoord::E(row, TileIobId::from_idx(i));
                        pads.insert(BondPad::Io(crd), builder.get_tb());
                    }
                }
                for (col, cd) in self.columns.iter().rev() {
                    let num_io = match cd.io_s {
                        IoGroupKind::None => 0,
                        IoGroupKind::Quad => 4,
                        IoGroupKind::Hex => 6,
                        IoGroupKind::HexReverse => {
                            for i in 0..6 {
                                let crd = EdgeIoCoord::S(col, TileIobId::from_idx(i));
                                pads.insert(BondPad::Io(crd), builder.get_tb());
                            }
                            continue;
                        }
                        _ => unreachable!(),
                    };
                    for i in (0..num_io).rev() {
                        let crd = EdgeIoCoord::S(col, TileIobId::from_idx(i));
                        pads.insert(BondPad::Io(crd), builder.get_tb());
                    }
                }
            }
            ChipKind::Xp2 => {
                for (row, rd) in &self.rows {
                    if row < self.row_clk {
                        continue;
                    }
                    if rd.io_e != IoGroupKind::None {
                        for iob in [1, 0] {
                            let crd = EdgeIoCoord::E(row, TileIobId::from_idx(iob));
                            pads.insert(BondPad::Io(crd), builder.get_tb());
                        }
                    }
                }
                for (col, cd) in self.columns.iter().rev() {
                    if cd.io_n != IoGroupKind::None {
                        for iob in [1, 0] {
                            let crd = EdgeIoCoord::N(col, TileIobId::from_idx(iob));
                            pads.insert(BondPad::Io(crd), builder.get_tb());
                        }
                    }
                }
                for (row, rd) in self.rows.iter().rev() {
                    if row == self.row_clk {
                        pads.insert(BondPad::Cfg(CfgPad::Toe), builder.get_i());
                        pads.insert(BondPad::Cfg(CfgPad::M0), builder.get_i());
                    }
                    if rd.io_w != IoGroupKind::None {
                        for iob in [0, 1] {
                            let crd = EdgeIoCoord::W(row, TileIobId::from_idx(iob));
                            pads.insert(BondPad::Io(crd), builder.get_tb());
                        }
                    }
                }
                for (col, cd) in &self.columns {
                    if cd.io_s != IoGroupKind::None {
                        for iob in [0, 1] {
                            let crd = EdgeIoCoord::S(col, TileIobId::from_idx(iob));
                            pads.insert(BondPad::Io(crd), builder.get_tb());
                        }
                    }
                }
                for (row, rd) in &self.rows {
                    if row >= self.row_clk {
                        continue;
                    }
                    if rd.io_e != IoGroupKind::None {
                        for iob in [1, 0] {
                            let crd = EdgeIoCoord::E(row, TileIobId::from_idx(iob));
                            pads.insert(BondPad::Io(crd), builder.get_tb());
                        }
                    }
                }
            }
            ChipKind::Ecp3(_) => {
                let row_cfg = self.special_loc[&SpecialLocKey::Config].row;
                let mut pll_rows = BTreeSet::new();
                for (_, bcrd) in self.plls() {
                    pll_rows.insert(bcrd.row);
                }
                for (row, rd) in &self.rows {
                    if row < row_cfg {
                        continue;
                    }
                    if rd.io_e != IoGroupKind::None {
                        for iob in [3, 2, 1, 0] {
                            let crd = EdgeIoCoord::E(row, TileIobId::from_idx(iob));
                            pads.insert(BondPad::Io(crd), builder.get_tb());
                        }
                    }
                }
                pads.insert(BondPad::Cfg(CfgPad::M2), builder.get_i());
                pads.insert(BondPad::Cfg(CfgPad::M1), builder.get_i());
                pads.insert(BondPad::Cfg(CfgPad::M0), builder.get_i());
                pads.insert(BondPad::Cfg(CfgPad::ProgB), builder.get_i());
                pads.insert(BondPad::Cfg(CfgPad::Cclk), builder.get_tb());
                pads.insert(BondPad::Cfg(CfgPad::InitB), builder.get_tb());
                pads.insert(BondPad::Cfg(CfgPad::Done), builder.get_tb());
                for (col, cd) in self.columns.iter().rev() {
                    if cd.io_n != IoGroupKind::None {
                        for iob in [3, 2, 1, 0] {
                            let crd = EdgeIoCoord::N(col, TileIobId::from_idx(iob));
                            pads.insert(BondPad::Io(crd), builder.get_tb());
                        }
                    }
                }
                for (row, rd) in self.rows.iter().rev() {
                    if rd.io_w != IoGroupKind::None {
                        for iob in [0, 1, 2, 3] {
                            let crd = EdgeIoCoord::W(row, TileIobId::from_idx(iob));
                            pads.insert(BondPad::Io(crd), builder.get_tb());
                        }
                    }
                    if pll_rows.contains(&row) {
                        for iob in [4, 5, 6, 7] {
                            let crd = EdgeIoCoord::W(row, TileIobId::from_idx(iob));
                            pads.insert(BondPad::Io(crd), builder.get_tb());
                        }
                    }
                }
                for (col, cd) in &self.columns {
                    if cd.io_s == IoGroupKind::Serdes {
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::InP(3)),
                            builder.get_i(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::OutP(3)),
                            builder.get_o(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::OutP(2)),
                            builder.get_o(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::InP(2)),
                            builder.get_i(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::ClkP),
                            builder.get_i(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::InP(1)),
                            builder.get_i(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::OutP(1)),
                            builder.get_o(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::OutP(0)),
                            builder.get_o(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::InP(0)),
                            builder.get_i(),
                        );
                    } else if cd.io_s != IoGroupKind::None {
                        for iob in [0, 1, 2, 3] {
                            let crd = EdgeIoCoord::S(col, TileIobId::from_idx(iob));
                            pads.insert(BondPad::Io(crd), builder.get_tb());
                        }
                    }
                }
                for (row, rd) in &self.rows {
                    if row >= row_cfg {
                        continue;
                    }
                    if pll_rows.contains(&row) {
                        for iob in [7, 6, 5, 4] {
                            let crd = EdgeIoCoord::E(row, TileIobId::from_idx(iob));
                            pads.insert(BondPad::Io(crd), builder.get_tb());
                        }
                    }
                    if rd.io_e != IoGroupKind::None {
                        for iob in [3, 2, 1, 0] {
                            let crd = EdgeIoCoord::E(row, TileIobId::from_idx(iob));
                            pads.insert(BondPad::Io(crd), builder.get_tb());
                        }
                    }
                }
            }
            ChipKind::MachXo2(_) => {
                let skip_io = [
                    self.get_io_crd(self.special_io[&SpecialIoKey::Tck]),
                    self.get_io_crd(self.special_io[&SpecialIoKey::Tms]),
                    self.get_io_crd(self.special_io[&SpecialIoKey::Tdi]),
                    self.get_io_crd(self.special_io[&SpecialIoKey::Tdo]),
                ];
                for (row, rd) in self.rows.iter().rev() {
                    let num_io = match rd.io_w {
                        IoGroupKind::None => 0,
                        IoGroupKind::Double => 2,
                        IoGroupKind::Quad | IoGroupKind::QuadI3c => 4,
                        _ => unreachable!(),
                    };
                    for i in 0..num_io {
                        let crd = EdgeIoCoord::W(row, TileIobId::from_idx(i));
                        pads.insert(BondPad::Io(crd), builder.get_tb());
                    }
                }
                for (col, cd) in &self.columns {
                    let iobs = match cd.io_s {
                        IoGroupKind::None => [].as_slice(),
                        IoGroupKind::Double => [0, 1].as_slice(),
                        IoGroupKind::Quad => [0, 1, 2, 3].as_slice(),
                        IoGroupKind::QuadReverse => [2, 3, 0, 1].as_slice(),
                        _ => unreachable!(),
                    };
                    for &i in iobs {
                        let crd = EdgeIoCoord::S(col, TileIobId::from_idx(i));
                        pads.insert(BondPad::Io(crd), builder.get_tb());
                    }
                }
                for (row, rd) in &self.rows {
                    let num_io = match rd.io_e {
                        IoGroupKind::None => 0,
                        IoGroupKind::Double => 2,
                        IoGroupKind::Quad => 4,
                        _ => unreachable!(),
                    };
                    for i in (0..num_io).rev() {
                        let crd = EdgeIoCoord::E(row, TileIobId::from_idx(i));
                        pads.insert(BondPad::Io(crd), builder.get_tb());
                    }
                }
                for (col, cd) in self.columns.iter().rev() {
                    let iobs = match cd.io_n {
                        IoGroupKind::None | IoGroupKind::Ebr => [].as_slice(),
                        IoGroupKind::Double => [1, 0].as_slice(),
                        IoGroupKind::Quad => [3, 2, 1, 0].as_slice(),
                        IoGroupKind::QuadReverse => [1, 0, 3, 2].as_slice(),
                        _ => unreachable!(),
                    };
                    for &i in iobs {
                        let crd = EdgeIoCoord::N(col, TileIobId::from_idx(i));
                        if skip_io.contains(&crd) {
                            continue;
                        }
                        pads.insert(BondPad::Io(crd), builder.get_tb());
                    }
                }
            }
            ChipKind::Ecp4 => {
                let (num_serdes, col_serdes) =
                    if let Some(cell) = self.special_loc.get(&SpecialLocKey::SerdesSingle) {
                        (1, cell.col)
                    } else if let Some(cell) = self.special_loc.get(&SpecialLocKey::SerdesDouble) {
                        (2, cell.col)
                    } else if let Some(cell) = self.special_loc.get(&SpecialLocKey::SerdesTriple) {
                        (3, cell.col)
                    } else {
                        unreachable!()
                    };
                for (idx, pad) in [
                    (0, SerdesPad::InP(3)),
                    (0, SerdesPad::InP(2)),
                    (0, SerdesPad::InP(1)),
                    (0, SerdesPad::InP(0)),
                    (0, SerdesPad::OutP(3)),
                    (0, SerdesPad::OutP(2)),
                    (0, SerdesPad::ClkP),
                    (0, SerdesPad::OutP(1)),
                    (0, SerdesPad::OutP(0)),
                    (1, SerdesPad::OutP(0)),
                    (1, SerdesPad::OutP(1)),
                    (1, SerdesPad::ClkP),
                    (1, SerdesPad::OutP(2)),
                    (1, SerdesPad::OutP(3)),
                    (1, SerdesPad::InP(0)),
                    (1, SerdesPad::InP(1)),
                    (1, SerdesPad::InP(2)),
                    (1, SerdesPad::InP(3)),
                    (2, SerdesPad::InP(3)),
                    (2, SerdesPad::InP(2)),
                    (2, SerdesPad::InP(1)),
                    (2, SerdesPad::InP(0)),
                    (2, SerdesPad::OutP(3)),
                    (2, SerdesPad::OutP(2)),
                    (2, SerdesPad::ClkP),
                    (2, SerdesPad::OutP(1)),
                    (2, SerdesPad::OutP(0)),
                ] {
                    if idx >= num_serdes {
                        continue;
                    }
                    pads.insert(
                        BondPad::Serdes(DirV::S, col_serdes + idx, pad),
                        if matches!(pad, SerdesPad::OutP(_)) {
                            builder.get_o()
                        } else {
                            builder.get_i()
                        },
                    );
                }
                pads.insert(BondPad::SerdesCorner(SerdesPad::ClkP), builder.get_i());
                for (row, rd) in &self.rows {
                    if rd.kind == RowKind::Ebr {
                        for iob in (4..8).rev() {
                            let crd = EdgeIoCoord::E(row, TileIobId::from_idx(iob));
                            pads.insert(BondPad::Io(crd), builder.get_tb());
                        }
                    }
                    if matches!(
                        rd.io_e,
                        IoGroupKind::Quad | IoGroupKind::QuadDqs | IoGroupKind::QuadEbrDqs
                    ) {
                        for iob in (0..4).rev() {
                            let crd = EdgeIoCoord::E(row, TileIobId::from_idx(iob));
                            pads.insert(BondPad::Io(crd), builder.get_tb());
                        }
                    }
                }
                for (col, cd) in self.columns.iter().rev() {
                    if matches!(
                        cd.io_n,
                        IoGroupKind::Quad | IoGroupKind::QuadDqs | IoGroupKind::QuadEbrDqs
                    ) {
                        for iob in (0..4).rev() {
                            let crd = EdgeIoCoord::N(col, TileIobId::from_idx(iob));
                            pads.insert(BondPad::Io(crd), builder.get_tb());
                        }
                    }
                }
                for (row, rd) in self.rows.iter().rev() {
                    if matches!(
                        rd.io_w,
                        IoGroupKind::Quad | IoGroupKind::QuadDqs | IoGroupKind::QuadEbrDqs
                    ) {
                        for iob in 0..4 {
                            let crd = EdgeIoCoord::W(row, TileIobId::from_idx(iob));
                            pads.insert(BondPad::Io(crd), builder.get_tb());
                        }
                    }
                    if rd.kind == RowKind::Ebr {
                        for iob in 4..8 {
                            let crd = EdgeIoCoord::W(row, TileIobId::from_idx(iob));
                            pads.insert(BondPad::Io(crd), builder.get_tb());
                        }
                    }
                }
                pads.insert(BondPad::Cfg(CfgPad::M2), builder.get_i());
                pads.insert(BondPad::Cfg(CfgPad::M1), builder.get_i());
                pads.insert(BondPad::Cfg(CfgPad::M0), builder.get_i());
                pads.insert(BondPad::Cfg(CfgPad::ProgB), builder.get_i());
                pads.insert(BondPad::Cfg(CfgPad::Done), builder.get_tb());
                pads.insert(BondPad::Cfg(CfgPad::Cclk), builder.get_tb());
                pads.insert(BondPad::Cfg(CfgPad::InitB), builder.get_tb());
            }
            ChipKind::Ecp5 => {
                for (col, cd) in &self.columns {
                    if cd.io_s == IoGroupKind::Serdes {
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::OutP(0)),
                            builder.get_o(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::InP(0)),
                            builder.get_i(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::InP(1)),
                            builder.get_i(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::OutP(1)),
                            builder.get_o(),
                        );
                        pads.insert(
                            BondPad::Serdes(DirV::S, col, SerdesPad::ClkP),
                            builder.get_i(),
                        );
                    }
                    if cd.io_s == IoGroupKind::Double && col >= self.col_clk {
                        for iob in 0..2 {
                            let crd = EdgeIoCoord::S(col, TileIobId::from_idx(iob));
                            pads.insert(BondPad::Io(crd), builder.get_tb());
                        }
                    }
                }
                for (row, rd) in &self.rows {
                    if matches!(rd.io_e, IoGroupKind::Quad | IoGroupKind::QuadDqs) {
                        for iob in (0..4).rev() {
                            let crd = EdgeIoCoord::E(row, TileIobId::from_idx(iob));
                            pads.insert(BondPad::Io(crd), builder.get_tb());
                        }
                    }
                }
                for (col, cd) in self.columns.iter().rev() {
                    if matches!(cd.io_n, IoGroupKind::Double) {
                        for iob in (0..2).rev() {
                            let crd = EdgeIoCoord::N(col, TileIobId::from_idx(iob));
                            pads.insert(BondPad::Io(crd), builder.get_tb());
                        }
                    }
                }
                for (row, rd) in self.rows.iter().rev() {
                    if matches!(rd.io_w, IoGroupKind::Quad | IoGroupKind::QuadDqs) {
                        for iob in 0..4 {
                            let crd = EdgeIoCoord::W(row, TileIobId::from_idx(iob));
                            pads.insert(BondPad::Io(crd), builder.get_tb());
                        }
                    }
                }
                for (col, cd) in &self.columns {
                    if col >= self.col_clk {
                        break;
                    }
                    match cd.io_s {
                        IoGroupKind::Double => {
                            for iob in 0..2 {
                                let crd = EdgeIoCoord::S(col, TileIobId::from_idx(iob));
                                pads.insert(BondPad::Io(crd), builder.get_tb());
                            }
                        }
                        IoGroupKind::Single => {
                            let crd = EdgeIoCoord::S(col, TileIobId::from_idx(0));
                            pads.insert(BondPad::Io(crd), builder.get_tb());
                        }
                        _ => (),
                    }
                }

                pads.insert(BondPad::Cfg(CfgPad::Cclk), builder.get_tb());
                pads.insert(BondPad::Cfg(CfgPad::InitB), builder.get_tb());
                pads.insert(BondPad::Cfg(CfgPad::ProgB), builder.get_i());
                pads.insert(BondPad::Cfg(CfgPad::Done), builder.get_tb());
                pads.insert(BondPad::Cfg(CfgPad::M2), builder.get_i());
                pads.insert(BondPad::Cfg(CfgPad::M1), builder.get_i());
                pads.insert(BondPad::Cfg(CfgPad::M0), builder.get_i());
            }
            ChipKind::Crosslink => {
                // no JTAG
            }
            ChipKind::Nexus(kind) => {
                let mut handle_cell = |kind, cell: CellCoord, rev: bool| {
                    if matches!(kind, IoGroupKind::None | IoGroupKind::Mipi) {
                        return;
                    }
                    let iobs = if rev { [1, 0] } else { [0, 1] };
                    for iob in iobs {
                        let bcrd = cell.bel(bslots::IO[iob]);
                        if bcrd == self.special_io[&SpecialIoKey::Cclk]
                            || bcrd == self.special_io[&SpecialIoKey::SpiPCsB]
                            || bcrd == self.special_io[&SpecialIoKey::D(0)]
                            || bcrd == self.special_io[&SpecialIoKey::D(1)]
                            || bcrd == self.special_io[&SpecialIoKey::JtagEn]
                        {
                            continue;
                        }
                        let io = self.get_io_crd(bcrd);
                        pads.insert(BondPad::Io(io), builder.get_tb());
                    }
                };
                for (col, cd) in self.columns.iter().rev() {
                    if let Some(bank) = cd.bank_n {
                        if kind == NexusKind::SmallHalfIo && bank == 1 {
                            continue;
                        }
                        handle_cell(cd.io_n, Chip::DIE.cell(col, self.row_n()), true);
                    }
                }
                for (row, rd) in self.rows.iter().rev() {
                    if rd.bank_w.is_some() {
                        handle_cell(rd.io_w, Chip::DIE.cell(self.col_w(), row), true);
                    }
                }
                for (col, cd) in self.columns.iter() {
                    if cd.bank_s.is_some() {
                        handle_cell(cd.io_s, Chip::DIE.cell(col, self.row_s()), false);
                    }
                }
                for (row, rd) in self.rows.iter() {
                    if rd.bank_e.is_some() {
                        handle_cell(rd.io_e, Chip::DIE.cell(self.col_e(), row), false);
                    }
                }
                if kind == NexusKind::SmallHalfIo {
                    for (col, cd) in self.columns.iter().rev() {
                        if cd.bank_n == Some(1) {
                            handle_cell(cd.io_n, Chip::DIE.cell(col, self.row_n()), true);
                        }
                    }
                }
            }
        }
        BScan {
            bits: builder.bits,
            pads,
        }
    }
}

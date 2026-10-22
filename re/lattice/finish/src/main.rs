use std::{
    collections::{BTreeMap, btree_map},
    path::PathBuf,
};

use clap::Parser;
use prjcombine_ecp::db::Database;
use prjcombine_entity::{EntityPartVec, EntityVec};
use prjcombine_types::db::DumpFlags;

#[derive(Debug, Parser)]
struct Args {
    db: PathBuf,
    txt: PathBuf,
    geom: PathBuf,
    tiledb: Vec<PathBuf>,
}

fn main() {
    let args = Args::parse();
    let geom = prjcombine_re_lattice_naming::Database::from_file(&args.geom).unwrap();

    let mut chips = EntityVec::new();
    let mut chip_xlat = EntityVec::new();

    for (chip, _) in geom.chips.into_values() {
        let cid = 'cid: {
            for (cid, cur_chip) in &chips {
                if *cur_chip == chip {
                    break 'cid cid;
                }
            }
            chips.push(chip)
        };
        chip_xlat.push(cid);
    }
    let mut dev_xlat = BTreeMap::new();
    let mut devices = vec![];
    for mut dev in geom.devices {
        dev.chip = chip_xlat[dev.chip];
        match dev_xlat.entry((dev.name.clone(), dev.chip)) {
            btree_map::Entry::Vacant(e) => {
                let idx = devices.len();
                devices.push(dev);
                e.insert(idx);
            }
            btree_map::Entry::Occupied(e) => {
                let idx = *e.get();
                let cur_dev = &mut devices[idx];
                assert_eq!(cur_dev.speeds, dev.speeds);
                let mut devbond_xlat = EntityPartVec::new();
                for (dbid, bname, bid) in dev.bonds {
                    assert!(!cur_dev.bonds.contains_key(&bname));
                    let ndbid = cur_dev.bonds.insert(bname, bid).0;
                    devbond_xlat.insert(dbid, ndbid);
                }
                for mut combo in dev.combos {
                    combo.devbond = devbond_xlat[combo.devbond];
                    cur_dev.combos.push(combo);
                }
            }
        }
    }

    let db = Database {
        chips,
        bonds: geom.bonds,
        devices,
        int: geom.int,
    };
    db.to_file(&args.db).unwrap();
    db.dump(
        &mut std::fs::File::create(&args.txt).unwrap(),
        DumpFlags::all(),
    )
    .unwrap();
}

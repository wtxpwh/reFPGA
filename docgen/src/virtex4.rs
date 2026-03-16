use indexmap::IndexMap;
use prjcombine_virtex4::defs::devdata;

use crate::{
    DocgenContext,
    interconnect::{gen_devdata, gen_intdb},
};

pub fn gen_virtex4(ctx: &mut DocgenContext) {
    for kind in ["virtex4", "virtex5", "virtex6", "virtex7"] {
        let db = prjcombine_virtex4::db::Database::from_file(
            ctx.ctx.root.join(format!("../databases/{kind}.zstd")),
        )
        .unwrap();
        gen_intdb(ctx, kind, &db.int);
        let mut devdata = IndexMap::new();
        for device in &db.devices {
            devdata.insert(device.name.as_str(), &device.data);
        }

        match kind {
            "virtex4" => {}
            "virtex5" => {
                gen_devdata(
                    ctx,
                    "virtex5",
                    &db.int,
                    "iodelay-default",
                    &devdata,
                    &[devdata::IODELAY_V5_IDELAY_DEFAULT],
                );
                gen_devdata(
                    ctx,
                    "virtex5",
                    &db.int,
                    "ppc-clock-delay",
                    &devdata,
                    &[devdata::PPC440_CLOCK_DELAY],
                );
                gen_devdata(
                    ctx,
                    "virtex5",
                    &db.int,
                    "pll-in-dly-set",
                    &devdata,
                    &[devdata::PLL_V5_IN_DLY_SET],
                );
            }
            "virtex6" => {
                gen_devdata(
                    ctx,
                    "virtex6",
                    &db.int,
                    "iodelay-default",
                    &devdata,
                    &[devdata::IODELAY_V6_IDELAY_DEFAULT],
                );
                gen_devdata(
                    ctx,
                    "virtex6",
                    &db.int,
                    "pll-in-dly-set",
                    &devdata,
                    &[devdata::PLL_V6_IN_DLY_SET],
                );
            }
            "virtex7" => {}
            _ => unreachable!(),
        }
    }
}

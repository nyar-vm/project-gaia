#![deny(missing_debug_implementations, missing_copy_implementations)]
#![warn(missing_docs, rustdoc::missing_crate_level_docs)]
#![doc = include_str!("../readme.md")]
#![doc(html_logo_url = "https://raw.githubusercontent.com/oovm/shape-rs/dev/projects/images/Trapezohedron.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/oovm/shape-rs/dev/projects/images/Trapezohedron.svg")]

mod errors;

pub use crate::errors::{Error, Result};


use exe::*;

fn main() {
    let pe = VecPE::from_disk_file("../../test/cff_explorer.exe").unwrap();
    let rsrc = ResourceDirectory::parse(&pe).unwrap();
    let icons = rsrc.icon_groups(&pe).unwrap();

    for (id, dir) in &icons {
        let filename = match id {
            ResolvedDirectoryID::ID(val) => format!("{}.ico", val),
            ResolvedDirectoryID::Name(name) => format!("{}.ico", name),
        };

        println!("Writing {}", filename);

        let icon_file = dir.to_icon_buffer(&pe).unwrap();
        icon_file.save(filename).unwrap();
    }

    println!("Icons dumped from executable");
}
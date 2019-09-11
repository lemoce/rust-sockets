extern crate cmake;
use cmake::Config;

fn main()
{
    let _dst = Config::new("csocks").build();       

    // println!("cargo:rustc-link-search=native={}", dst.display());
    // println!("cargo:rustc-link-lib=static=foo");    
}

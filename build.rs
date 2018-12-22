extern crate lalrpop;

fn main() {
    use std::env;
    use std::path::PathBuf;
    use std::os::unix::fs::PermissionsExt;
    use std::fs::Permissions;

    lalrpop::process_root().unwrap();

    // let out = env::var("OUT_DIR").unwrap();
    // eprintln!("out_dir='{}'", out);
    // let cmd = format!("chmod -r 744 {}", out);
    // ::std::process::Command::new(cmd).output().unwrap();
    // let path = PathBuf::from(out).join("interpreter");
    // let new_perms = Permissions::from_mode(0x744);
    // ::std::fs::set_permissions(&path, new_perms).unwrap();
}

use std::env;
use std::ffi::{CStr, CString};

fn print_usage() {
    eprintln!("Uso: groups [USUÁRIO]");
    eprintln!("Exibe a filiação de grupo para o USUÁRIO, ou para o processo atual se nenhum for especificado.");
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("groups", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = env::args().skip(1).collect();

    for arg in &args {
        if arg == "--help" || arg == "-h" {
            print_usage();
            return;
        }
        if arg == "--version" {
            println!("groups (JeffUtils) 1.0");
            return;
        }
    }

    if args.len() > 1 {
        print_usage();
        std::process::exit(1);
    }

    #[cfg(unix)]
    {
        unsafe {
            if args.is_empty() {
                // Get groups for the current process
                let ngroups = libc::getgroups(0, std::ptr::null_mut());
                if ngroups >= 0 {
                    let mut groups: Vec<libc::gid_t> = vec![0; ngroups as usize];
                    if libc::getgroups(ngroups, groups.as_mut_ptr()) >= 0 {
                        let mut group_names = Vec::new();
                        for g in groups {
                            let gr = libc::getgrgid(g as libc::gid_t);
                            if !gr.is_null() {
                                let name = CStr::from_ptr((*gr).gr_name).to_string_lossy().into_owned();
                                group_names.push(name);
                            } else {
                                group_names.push(g.to_string());
                            }
                        }
                        println!("{}", group_names.join(" "));
                        return;
                    }
                }
            } else {
                // Get groups for the specified user
                let username = &args[0];
                let username_c = match CString::new(username.clone()) {
                    Ok(s) => s,
                    Err(_) => {
                        eprintln!("groups: usuário inválido");
                        std::process::exit(1);
                    }
                };
                let pwd = libc::getpwnam(username_c.as_ptr());
                if pwd.is_null() {
                    eprintln!("groups: '{}': no such user", username);
                    std::process::exit(1);
                }
                let primary_gid = (*pwd).pw_gid;

                let mut ngroups = 100;
                #[cfg(any(target_os = "macos", target_os = "ios"))]
                type GroupListT = i32;
                #[cfg(not(any(target_os = "macos", target_os = "ios")))]
                type GroupListT = libc::gid_t;

                let mut groups: Vec<GroupListT> = vec![0; ngroups as usize];
                let res = libc::getgrouplist(username_c.as_ptr(), primary_gid as GroupListT, groups.as_mut_ptr(), &mut ngroups);
                if res < 0 {
                    groups.resize(ngroups as usize, 0);
                    libc::getgrouplist(username_c.as_ptr(), primary_gid as GroupListT, groups.as_mut_ptr(), &mut ngroups);
                }

                groups.truncate(ngroups as usize);
                let mut group_names = Vec::new();
                for g in groups {
                    let gr = libc::getgrgid(g as libc::gid_t);
                    if !gr.is_null() {
                        let name = CStr::from_ptr((*gr).gr_name).to_string_lossy().into_owned();
                        group_names.push(name);
                    } else {
                        group_names.push(g.to_string());
                    }
                }
                println!("{}", group_names.join(" "));
                return;
            }
        }
    }

    eprintln!("groups: failed to get groups");
    std::process::exit(1);
}